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
    saveload::{SimpleMarker, SimpleMarkerAllocator}, storage::GenericReadStorage
};

use components::*;
use entities::{create_player, spawn_room};
use gui::{draw_ui, drop_item_menu, ranged_target, ItemMenuResult, MainMenuResult, MainMenuSelection};
use input::player_input;
use map::{draw_map, Map};
use systems::{
    damage::{self, DamageSystem}, inventory::{ItemCollectionSystem, ItemDropSystem, ItemUseSystem}, map_indexing::MapIndexingSystem, melee_combat::MeleeCombatSystem, monster_ai::MonsterAI, player, saveload, visibility::FoVSystem
};
use gamelog::GameLog;
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
    SaveGame,
    NextLevel,
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

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let player_entity = self.ecs.fetch::<Entity>();

        let mut to_delete : Vec<Entity> = Vec::new();
        for entity in entities.join() {
            let mut should_delete = true;

            let p = player.get(entity);
            if let Some(_p) = p {
                should_delete = false;
            }

            let bp = backpack.get(entity);
            if let Some(bp) = bp {
                if bp.owner == *player_entity {
                    should_delete = false;
                }
            }

            if should_delete {
                to_delete.push(entity);
            }            
        }
        to_delete
    }

    fn goto_next_level(&mut self) {
        // delete all non-player & player-item entities
        let to_delete = self.entities_to_remove_on_level_change();
        for entity in to_delete {
            self.ecs.delete_entity(entity).expect("Unable to delete entity");
        }

        // build new map, place player
        let worldmap;
        {
            let mut worldmap_resource = self.ecs.write_resource::<Map>();
            let current_depth = worldmap_resource.depth;
            *worldmap_resource = Map::new_map_room_and_corridors(current_depth + 1);
            worldmap = worldmap_resource.clone();
        }

        for room in worldmap.rooms.iter().skip(1) {
            entities::spawn_room(&mut self.ecs, room);
        }

        let (player_x, player_y) = worldmap.rooms[0].center();
        let mut player_pos = self.ecs.write_resource::<Point>();
        *player_pos = Point::new(player_x, player_y);
        let mut position_components = self.ecs.write_storage::<Position>();
        let player_entity = self.ecs.fetch::<Entity>();
        let player_pos_comp = position_components.get_mut(*player_entity);
        if let Some(player_pos_comp) = player_pos_comp {
            player_pos_comp.x = player_x;
            player_pos_comp.y = player_y;
        }

        // set viewshed to dirty 
        let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
        let vs = viewshed_components.get_mut(*player_entity);
        if let Some(vs) = vs {
            vs.dirty = true;
        }

        //Notify player, give small health bump
        let mut gamelog = self.ecs.fetch_mut::<GameLog>();
        gamelog.entries.push("You descend further into the mountain. Take a moment to rest.".to_string());
        let mut player_health_store = self.ecs.write_storage::<CombatStats>();
        let player_health = player_health_store.get_mut(*player_entity);
        if let Some(player_health) = player_health {
            player_health.hp = i32::max(player_health.hp, player_health.max_hp / 2)
        }

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
                            MainMenuSelection::NewGame => {
                                newrunstate = RunState::PreRun;
                                saveload::delete_save();
                            }
                            MainMenuSelection::LoadGame => {
                                saveload::load_game(&mut self.ecs);
                                newrunstate = RunState::AwaitingInput;
                                saveload::delete_save();
                            }
                            MainMenuSelection::Quit => {std::process::exit(0);}
                        }
                    }
                }
            }
            RunState::SaveGame => {
                saveload::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu { menu_selection: MainMenuSelection::LoadGame }
            }

            RunState::NextLevel => {
                self.goto_next_level();
                newrunstate = RunState::PreRun;
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

    let map: Map = Map::new_map_room_and_corridors(1);
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
