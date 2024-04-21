mod ecs;
mod input;
mod map;

use rltk::{GameState, Rltk};
use specs::prelude::*;

use ecs::components::{ Player, Position, Renderable};
use ecs::entities::create_player;
use ecs::systems::FoVSystem;
use input::player_input;
use map::{Map, draw_map};

use crate::ecs::components::Viewshed;

pub struct State {
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut fov = FoVSystem{};
        fov.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        player_input(self, ctx);
        self.run_systems();

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        // loops through an iterator of all entities that have positon AND renderable components.
        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}


fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    // generates 80x50 terminal screen w/ title "Roguelike Test"
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Test")
        .build()?;
    let mut gs = State { ecs: World::new() };

    // tells Specs to generate storage systems for registered components
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();

    let map : Map = Map::new_map_room_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    gs.ecs.insert(map);
    create_player(&mut gs.ecs, player_x, player_y);

    //initial loop for game
    rltk::main_loop(context, gs)
}
