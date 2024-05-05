pub mod components;
pub mod entities;
pub mod gamelog;
pub mod gui;
pub mod input;
pub mod map;
pub mod systems;
pub mod menu;

extern crate serde;
extern crate bracket_lib;

use bracket_lib::prelude::*;

use specs::{
    prelude::*,
    saveload::{SimpleMarker, SimpleMarkerAllocator}
};

use components::*;
use entities::{create_player, spawn_room};
use gui::{draw_ui, drop_item_menu, ranged_target, ItemMenuResult, MainMenuResult, MainMenuSelection};
use input::player_input;
use map::{draw_map, Map};
use systems::{
    damage::{self, DamageSystem},
    inventory::{ItemCollectionSystem, ItemDropSystem, ItemUseSystem},
    map_indexing::MapIndexingSystem,
    melee_combat::MeleeCombatSystem,
    monster_ai::MonsterAI,
    visibility::FoVSystem,
    saveload,
};
use menu::main_menu;

#[derive(Copy, Clone, PartialEq)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range: i32, item: Entity },
    MainMenu { menu_selection: MainMenuSelection },
    //PreRun,
    SaveGame,
}
pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut fov = FoVSystem {};
        fov.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem {};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);
        let mut item_collection = ItemCollectionSystem {};
        item_collection.run_now(&self.ecs);
        let mut items = ItemUseSystem {};
        items.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem {};
        drop_items.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        ctx.cls();
        match newrunstate {
            RunState::MainMenu { .. } => {},
            _ => {
                draw_map(&self.ecs, ctx);

                {
                    let positions = self.ecs.read_storage::<Position>();
                    let renderables = self.ecs.read_storage::<Renderable>();
                    let map = self.ecs.fetch::<Map>();
        
                    let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
                    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
        
                    for (pos, render) in data.iter() {
                        let idx = map.xy_idx(pos.x, pos.y);
                        if map.visible_tiles[idx] {
                            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph)
                        }
                    }
                    draw_ui(&self.ecs, ctx);
                }
            }
        }


        match newrunstate {
            RunState::AwaitingInput => newrunstate = player_input(self, ctx),
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::ShowDropItem => {
                let result = drop_item_menu(self, ctx);
                match result.0 {
                    ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    ItemMenuResult::NoResponse => {}
                    ItemMenuResult::Selected => {
                        let item = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(*self.ecs.fetch::<Entity>(), WantsToDropItem { item: item })
                            .expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item = result.1.unwrap();
                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        let is_item_ranged = is_ranged.get(item);
                        if let Some(is_item_ranged) = is_item_ranged {
                            newrunstate = RunState::ShowTargeting {
                                range: (is_item_ranged.range),
                                item: (item),
                            }
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent
                                .insert(
                                    *self.ecs.fetch::<Entity>(),
                                    WantsToUseItem {
                                        item: item,
                                        target: None,
                                    },
                                )
                                .expect("Unable to insert intent");
                            newrunstate = RunState::PlayerTurn;
                        }
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let target = ranged_target(self, ctx, range);
                match target.0 {
                    ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    ItemMenuResult::NoResponse => {}
                    ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToUseItem {
                                    item,
                                    target: target.1,
                                },
                            )
                            .expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }

            RunState::MainMenu { .. } => {
                let result = main_menu(self, ctx);
                match result {
                    MainMenuResult::NoSelection { selected } => newrunstate = RunState::MainMenu { menu_selection: selected },
                    MainMenuResult::Selected { selected } =>{
                        match selected {
                            MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
                            MainMenuSelection::LoadGame => newrunstate = RunState::PreRun,
                            MainMenuSelection::Quit => {std::process::exit(0);}
                        }
                    }
                }
            }
            RunState::SaveGame => {
                saveload::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu { menu_selection: MainMenuSelection::LoadGame }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        damage::delete_dead(&mut self.ecs);
    }
}

fn main() -> BError {
    // generates 80x50 terminal screen w/ title "Roguelike Test"
    let mut context = BTermBuilder::simple80x50()
        .with_title("Roguelike Test")
        .build()?;
    // Adds fun scanlines and screen burn, very retro
    // TODO: Reconsider enemy colors, make scanlines & screenburn toggleable.
    context.with_post_scanlines(true);
    let mut gs = State { ecs: World::new() };

    // tells Specs to generate storage systems for registered components
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    let map: Map = Map::new_map_room_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    let player = create_player(&mut gs.ecs, player_x, player_y);

    gs.ecs.insert(RandomNumberGenerator::new());
    for room in map.rooms.iter().skip(1) {
        spawn_room(&mut gs.ecs, room)
    }

    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(player);
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["Welcome to Stinky Roguelike!".to_string()],
    });
 

    // initial loop for game
    main_loop(context, gs)
}
