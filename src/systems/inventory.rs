use crate::{
    components::{
        CombatStats, InBackpack, Item, Name, Position, Potion, WantsToDrinkPotion, WantsToDropItem, WantsToPickupItem
    },
    gamelog::GameLog,
    Point,
};
use specs::prelude::*;

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) =
            data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.acquired_by,
                    },
                )
                .expect("Unable to add to backpack");

            if pickup.acquired_by == *player_entity {
                gamelog.entries.push(format!(
                    "You pick up the {}.",
                    names.get(pickup.item).unwrap().name
                ));
            };
        }
        wants_pickup.clear();
    }
}

pub struct PotionUseSystem {}

impl<'a> System<'a> for PotionUseSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDrinkPotion>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Potion>,
        WriteStorage<'a, CombatStats>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_drink,
            names,
            potions,
            mut combat_stats,
        ) = data;

        for (entity, drink, stats) in (&entities, &wants_drink, &mut combat_stats).join() {
            let potion = potions.get(drink.potion);
            match potion {
                None => {}
                Some(potion) => {
                    stats.hp = i32::min(stats.max_hp, stats.hp + potion.restore_hp);
                    if entity == *player_entity {
                        gamelog.entries.push(format!(
                            "You drink the {}, healing {} hp.",
                            names.get(drink.potion).unwrap().name,
                            potion.restore_hp
                        ));
                    }
                    entities.delete(drink.potion).expect("Delete failed");
                }
            }
        }
        wants_drink.clear();
    }
}

pub struct ItemDropSystem {}
impl<'a> System <'a> for ItemDropSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

        fn run(&mut self, data : Self::SystemData) {
            let (player_entity, mut gamelog, entities, mut wants_drop, names, mut positions, mut backpack) = data;

            for(entity, to_drop) in (&entities, &wants_drop).join() {
                let mut dropper_position = Position{x: 0, y: 0};
                {
                    let dropped_position = positions.get(entity).unwrap();
                    dropper_position.x = dropped_position.x;
                    dropper_position.y = dropped_position.y;
                }
                positions.insert(to_drop.item, Position{x: dropper_position.x, y: dropper_position.y}).expect("Unable to insert position.");
                backpack.remove(to_drop.item);

                if entity == *player_entity {
                    gamelog.entries.push(format!("You drop the {}", names.get(to_drop.item).unwrap().name));
                }
            }
            wants_drop.clear();
        }
}

pub fn get_item(world: &mut World) {
    let player_pos = world.fetch::<Point>();
    let player_entity = world.fetch::<Entity>();
    let entities = world.entities();
    let items = world.read_storage::<Item>();
    let positions = world.read_storage::<Position>();
    let mut gamelog = world.fetch_mut::<GameLog>();

    let mut target: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target = Some(item_entity);
        }

        match target {
            None => gamelog
                .entries
                .push("There is nothing here to pick up".to_string()),
            Some(item) => {
                let mut pickup = world.write_storage::<WantsToPickupItem>();
                pickup
                    .insert(
                        *player_entity,
                        WantsToPickupItem {
                            acquired_by: *player_entity,
                            item,
                        },
                    )
                    .expect("Unable to insert item");
            }
        }
    }
}
