use bracket_lib::prelude::*;
use specs::{
    prelude::*,
    saveload::{MarkedBuilder, SimpleMarker},
};

use crate::{
    AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, DefenseBonus, EntryTrigger,
    EquipmentSlot, Equippable, Hidden, HungerClock, HungerState::*, InflictsDamage, Item,
    MagicMapper, MeleePowerBonus, Monster, Name, Player, Position, ProvidesFood, ProvidesHealing,
    Ranged, Renderable, SerializeMe, Viewshed,
};

pub const MAX_MOBS: i32 = 4;
pub const MAX_ITEMS: i32 = 2;

pub fn create_player(world: &mut World, x: i32, y: i32) -> Entity {
    world
        .create_entity()
        .with(Player {})
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('@'),
            fg: RGB::named(YELLOW),
            bg: RGB::named(BLACK),
            render_order: 0,
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
        .with(HungerClock {
            state: WellFed,
            duration: 20,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

pub fn create_ogre(world: &mut World, x: i32, y: i32) {
    mob_gen(world, x, y, to_cp437('o'), "Ogre");
}
pub fn create_goblin(world: &mut World, x: i32, y: i32) {
    mob_gen(world, x, y, to_cp437('g'), "Goblin");
}

fn mob_gen<S: ToString>(world: &mut World, x: i32, y: i32, glyph: FontCharType, name: S) {
    world
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: glyph,
            fg: RGB::named(RED),
            bg: RGB::named(BLACK),
            render_order: 1,
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
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn health_potion(world: &mut World, x: i32, y: i32) {
    world
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('i'),
            fg: RGB::named(MAGENTA),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Health Potion".to_string(),
        })
        .with(Item {})
        .with(ProvidesHealing { restore_hp: 8 })
        .with(Consumable {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn magic_missile_scroll(world: &mut World, x: i32, y: i32) {
    world
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437(')'),
            fg: RGB::named(CYAN),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Scroll of Magic Missile".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn fireball_scroll(world: &mut World, x: i32, y: i32) {
    world
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437(')'),
            fg: RGB::named(ORANGE),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Scroll of Fireball".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 20 })
        .with(AreaOfEffect { radius: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn confusion_scroll(world: &mut World, x: i32, y: i32) {
    world
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437(')'),
            fg: RGB::named(GREEN),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Scroll of Confusion".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(Confusion { turns: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
pub fn magic_mapping_scroll(world: &mut World, x: i32, y: i32) {
    world
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437(')'),
            fg: RGB::named(CYAN3),
            bg: RGB::named(MAGENTA),
            render_order: 2,
        })
        .with(Name {
            name: "Scroll of Magic Mapping".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(MagicMapper {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn dagger(world: &mut World, x: i32, y: i32) {
    world
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('/'),
            fg: RGB::named(CYAN),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Dagger".to_string(),
        })
        .with(Item {})
        .with(Ranged { range: 6 })
        .with(Equippable {
            slot: EquipmentSlot::Melee,
        })
        .with(MeleePowerBonus { power: 2 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn shield(world: &mut World, x: i32, y: i32) {
    world
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('('),
            fg: RGB::named(CYAN),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Shield".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Shield,
        })
        .with(DefenseBonus { defense: 1 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn ration(world: &mut World, x: i32, y: i32) {
    world
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('%'),
            fg: RGB::named(GREEN),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Ration".to_string(),
        })
        .with(Item {})
        .with(ProvidesFood {})
        .with(Consumable {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn bear_trap(world: &mut World, x: i32, y: i32) {
    world
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437('^'),
            fg: RGB::named(RED),
            bg: RGB::named(BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Bear Trap".to_string(),
        })
        .with(Hidden {})
        .with(EntryTrigger {})
        .with(InflictsDamage { damage: 6 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
