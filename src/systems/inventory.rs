use specs::prelude::*;
use crate::{
    components::{Item, InBackpack, Name, Position, WantsToPickupItem}, 
    gamelog::GameLog,
    Point,
};

pub struct ItemCollectionSystem {}

impl<'a> System <'a> for ItemCollectionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack {owner: pickup.acquired_by }).expect("Unable to add to backpack");
            
            if pickup.acquired_by == *player_entity {
                gamelog.entries.push(format!("You pick up the {}.", names.get(pickup.item).unwrap().name));
            };
        }
    }
}

pub fn get_item(world: &mut World) {
    let player_pos = world.fetch::<Point>();
    let player_entity = world.fetch::<Entity>();
    let entities = world.entities();
    let items = world.read_storage::<Item>();
    let positions = world.read_storage::<Position>();
    let mut gamelog = world.fetch_mut::<GameLog>();

    let mut target : Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target = Some(item_entity);
        }

        match target {
            None => gamelog.entries.push("There is nothing here to pick up".to_string()),
            Some(item) => {
                let mut pickup = world.write_storage::<WantsToPickupItem>();
                pickup.insert(*player_entity, WantsToPickupItem{ acquired_by: *player_entity, item}).expect("Unable to insert item");

            }
        }
    }
}