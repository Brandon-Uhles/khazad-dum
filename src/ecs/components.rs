// module for all components
use specs::prelude::*;
use specs_derive::Component;
use rltk::RGB;

// example random movement component
#[derive(Component)]
pub struct LeftMover {}

#[derive(Component, Debug)]
pub struct Player {}

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}


