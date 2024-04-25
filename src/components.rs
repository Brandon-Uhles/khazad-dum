// module for all components
use rltk::RGB;
use specs::prelude::*;
use specs_derive::*;


#[derive(Component, Debug)]
pub struct BlocksTile{}

#[derive(Component, Debug)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}
#[derive(Component, Debug)]
pub struct Monster {}

#[derive(Component, Debug)]
pub struct Name {
    pub name: String,
}

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

// subject.wantstomelee.target
#[derive(Component, Debug, Clone)]
pub struct WantsToMelee {
    pub target : Entity
}

#[derive(Component, Debug)]
pub struct SufferDamage {
    pub amount : Vec<i32>
}

impl SufferDamage {
    /// if the entity has suffered damage this turn, push new damage into amount vector.
    /// if the entity has not suffered damage, generate new damage vector, apply damage, and store vector in writestorage
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage { amount : vec![amount]};
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}


// Viewshed stores all tiles visible to the owner before pushing them to the owner
// dirty determines whether the viewshed has been changed and needs to be reapplied
#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}
