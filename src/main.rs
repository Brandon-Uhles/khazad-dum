mod ecs;
mod input;


use input::input::player_input;
use rltk::{GameState, Rltk};
use specs::prelude::*;

use ecs::components::{LeftMover, Player, Position, Renderable};
use ecs::systems::LeftWalker;
use ecs::entities::create_player;

pub struct State {
    ecs: World
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        player_input(self, ctx);
        self.run_systems();

        // Clear screen, print test message
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        // loops through an iterator of all entities that have positon AND renderable components. 
        for(pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut lw = LeftWalker{};
        lw.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    // generates 80x50 terminal screen w/ title "Roguelike Test"
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Test")
        .build()?;
    let mut gs = State {
        ecs: World::new()
    };

    // tells Specs to generate storage systems for registered components
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Player>();

    create_player(&mut gs.ecs, 40, 25);

    rltk::main_loop(context, gs)
}
