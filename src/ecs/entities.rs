use rltk::RGB;
use specs::prelude::*;

use super::components::{Player, Position, Renderable};

pub fn create_player(world: &mut World, x: i32, y: i32) {
    world
    .create_entity()
    .with(Player {})
    .with(Position { x, y })
    .with(Renderable {
        glyph: rltk::to_cp437('@'),
        fg: RGB::named(rltk::YELLOW),
        bg: RGB::named(rltk::BLACK),
    })
    .build();
}