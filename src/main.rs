pub mod components;
pub mod entities;
pub mod gamelog;
pub mod gui;
pub mod input;
pub mod map;
pub mod systems;

use rltk::{GameState, Point, Rltk};
use specs::prelude::*;

use components::{
    BlocksTile, CombatStats, Monster, Name, Player, Position, Renderable, SufferDamage, Viewshed,
    WantsToMelee,
};
use entities::{create_player, spawn_room};
use gui::draw_ui;
use input::player_input;
use map::{draw_map, Map};
use systems::{
    damage::{self, DamageSystem},
    map_indexing::MapIndexingSystem,
    melee_combat::MeleeCombatSystem,
    monster_ai::MonsterAI,
    visibility::FoVSystem,
};

#[derive(Copy, Clone, PartialEq)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
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
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::AwaitingInput => newrunstate = player_input(self, ctx),
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        damage::delete_dead(&mut self.ecs);
        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        // loops through an iterator of all entities that have positon AND renderable components.
        // TODO: abstract away into rendering function within visibility.rs
        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }

        draw_ui(&self.ecs, ctx);
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    // generates 80x50 terminal screen w/ title "Roguelike Test"
    let mut context = RltkBuilder::simple80x50()
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

    let map: Map = Map::new_map_room_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    let player = create_player(&mut gs.ecs, player_x, player_y);

    gs.ecs.insert(rltk::RandomNumberGenerator::new());
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
    rltk::main_loop(context, gs)
}
