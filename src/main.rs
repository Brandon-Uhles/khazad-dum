mod ecs;
mod input;
mod map;

use rltk::{GameState, Point, Rltk};
use specs::prelude::*;

use ecs::components::{Monster, Name, Player, Position, Renderable};
use ecs::entities::{create_player, gen_mob_per_room};
use ecs::systems;
use input::player_input;
use map::{draw_map, Map};

use crate::ecs::components::Viewshed;

#[derive(Copy, Clone, PartialEq)]
pub enum RunState {
    Paused,
    Running,
}
pub struct State {
    pub ecs: World,
    pub runstate: RunState,
}

impl State {
    fn run_systems(&mut self) {
        let mut fov = systems::FoVSystem {};
        fov.run_now(&self.ecs);
        let mut mob = systems::MonsterAI {};
        mob.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        if self.runstate == RunState::Running {
            self.run_systems();
            self.runstate = RunState::Paused;
        } else {
            self.runstate = player_input(self, ctx);
        }

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        // loops through an iterator of all entities that have positon AND renderable components.
        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    // generates 80x50 terminal screen w/ title "Roguelike Test"
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Test")
        .build()?;
    let mut gs = State {
        ecs: World::new(),
        runstate: RunState::Running,
    };

    // tells Specs to generate storage systems for registered components
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Name>();

    let map: Map = Map::new_map_room_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    create_player(&mut gs.ecs, player_x, player_y);
    gen_mob_per_room(&mut gs.ecs, &map);

    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_x, player_y));

    // initial loop for game
    rltk::main_loop(context, gs)
}
