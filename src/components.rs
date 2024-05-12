// module for all components
use bracket_lib::prelude::*;
use serde::{Deserialize, Serialize};
use specs::{prelude::*, saveload::ConvertSaveload, saveload::Marker};
use specs_derive::*;
use std::convert::Infallible as NoError;


#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct ParticleLifetime {
    pub lifetime_ms : f32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MagicMapper {}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ProvidesFood {}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum HungerState {
    WellFed,
    Normal,
    Hungry,
    Starving,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct HungerClock {
    pub state: HungerState,
    pub duration: i32,
}   

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Melee,
    Shield,
}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct BlocksTile {}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Confusion {
    pub turns: i32,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Consumable {}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Item {}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct ProvidesHealing {
    pub restore_hp: i32,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Monster {}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Player {}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Renderable {
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

pub struct SerializeMe;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map: crate::map::Map,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<Point>,
}
#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToDropItem {
    pub item: Entity,
}

// subject.wantstomelee.target
#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToPickupItem {
    pub acquired_by: Entity,
    pub item: Entity,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone )]
pub struct MeleePowerBonus {
    pub power : i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone )]
pub struct DefenseBonus {
    pub defense: i32,
}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct SufferDamage {
    pub amount: Vec<i32>,
}

impl SufferDamage {
    /// if the entity has suffered damage this turn, push new damage into amount vector.
    /// if the entity has not suffered damage, generate new damage vector, apply damage, and store vector in writestorage
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage {
                amount: vec![amount],
            };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}

// Viewshed stores all tiles visible to the owner before pushing them to the owner
// dirty determines whether the viewshed has been changed and needs to be reapplied
#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    pub dirty: bool,
}
