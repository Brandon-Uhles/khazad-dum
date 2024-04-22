use rltk::RGB;
use specs::prelude::*;

use super::components::{Monster, Name, Player, Position, Renderable, Viewshed};
use crate::map;

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
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .build();
}

fn create_goblin(world: &mut World, name: String, x: i32, y: i32) {
    world
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('g'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Monster {})
        .with(Name { name })
        .build();
}

fn create_ogre(world: &mut World, name: String, x: i32, y: i32) {
    world
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('o'),
            fg: RGB::named(rltk::DARKGREEN),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Monster {})
        .with(Name { name })
        .build();
}

pub fn gen_mob_per_room(world: &mut World, map: &map::Map) {
    let mut rng = rltk::RandomNumberGenerator::new();
    let mut g = 1;
    let mut o = 1;

    for room in map.rooms.iter().skip(1) {
        let (x, y) = room.center();

        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => {
                create_goblin(world, format!("Goblin #{}", &g), x, y);
                g += 1;
            }
            _ => {
                create_ogre(world, format!("Ogre #{}", &o), x, y);
                o += 1;
            }
        }
    }
}
