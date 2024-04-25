use core::num;

use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;

use super::components::{
    BlocksTile, CombatStats, Item, Monster, Name, Player, Position, Potion, Renderable, Viewshed,
};
use crate::map::{Rect, MAP_WIDTH};

const MAX_MOBS: i32 = 4;
const MAX_ITEMS: i32 = 2;

pub fn create_player(world: &mut World, x: i32, y: i32) -> Entity {
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
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .build()
}

fn create_ogre(world: &mut World, x: i32, y: i32) {
    mob_gen(world, x, y, rltk::to_cp437('o'), "Ogre");
}
fn create_goblin(world: &mut World, x: i32, y: i32) {
    mob_gen(world, x, y, rltk::to_cp437('g'), "Goblin");
}

fn mob_gen<S: ToString>(world: &mut World, x: i32, y: i32, glyph: rltk::FontCharType, name: S) {
    world
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Monster {})
        .with(Name {
            name: name.to_string(),
        })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .build();
}

/// Mob generator, currently only two options
/// TODO: Add more variety
pub fn rng_mob(world: &mut World, x: i32, y: i32) {
    let result: i32;
    {
        let mut rng = world.write_resource::<RandomNumberGenerator>();
        result = rng.roll_dice(1, 2);
    }
    match result {
        1 => create_ogre(world, x, y),
        _ => create_goblin(world, x, y),
    }
}

/// Takes a room, randomly generates mobs
pub fn spawn_room(world: &mut World, room: &Rect) {
    let mut monster_spawn_points: Vec<usize> = Vec::new();
    let mut item_spawn_points: Vec<usize> = Vec::new();
    {
        let mut rng = world.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MOBS + 2) - 3;
        let num_items = rng.roll_dice(1, MAX_ITEMS + 2) - 3;

        for _i in 0..num_monsters {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAP_WIDTH) + x;
                if !monster_spawn_points.contains(&idx) {
                    monster_spawn_points.push(idx);
                    added = true;
                }
            }
        }

        for _i in 0..num_items {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAP_WIDTH) + x;
                if !item_spawn_points.contains(&idx) {
                    item_spawn_points.push(idx);
                    added = true;
                }
            }
        }
    }
    for idx in monster_spawn_points.iter() {
        let x = *idx % MAP_WIDTH;
        let y = *idx / MAP_WIDTH;
        rng_mob(world, x as i32, y as i32);
    }

    for idx in item_spawn_points.iter() {
        let x = *idx % MAP_WIDTH;
        let y = *idx / MAP_WIDTH;
        health_potion(world, x as i32, y as i32)
    }
}

fn health_potion(world: &mut World, x: i32, y: i32) {
    world
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('i'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Name {
            name: "Health Potion".to_string(),
        })
        .with(Item {})
        .with(Potion { restore_hp: 8 })
        .build();
}
