use crate::{components::*, gamelog::GameLog, Map, Point};
use specs::prelude::*;
use bracket_lib::prelude::*;
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

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ProvidesHealing>,
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, InflictsDamage>,
        ReadExpect<'a, Map>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, Confusion>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_use,
            names,
            healing,
            mut combat_stats,
            consumables,
            inflict_damage,
            map,
            mut suffer_damage,
            aoe,
            mut confuse,
        ) = data;

        for (entity, useitem) in (&entities, &wants_use).join() {
            let mut used_item = true;

            //targeting
            let mut targets: Vec<Entity> = Vec::new();
            match useitem.target {
                None => targets.push(*player_entity),
                Some(target) => {
                    let area_of_effect = aoe.get(useitem.item);
                    match area_of_effect {
                        None => {
                            //Single target
                            let idx = map.xy_idx(target.x, target.y);
                            for mob in map.tile_content[idx].iter() {
                                targets.push(*mob);
                            }
                        }
                        Some(area_of_effect) => {
                            // AoE
                            let mut affected_tiles =
                                    field_of_view(target, area_of_effect.radius, &*map);
                            affected_tiles.retain(|p| p.x < map.width - 1 && p.y < map.height - 1);
                            for tile_idx in affected_tiles.iter() {
                                let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                                for mob in map.tile_content[idx].iter() {
                                    targets.push(*mob);
                                }
                            }
                        }
                    }
                }
            }

            // if item heals, apply healing
            let item_heals = healing.get(useitem.item);
            match item_heals {
                None => {}
                Some(healer) => {
                    for target in targets.iter() {
                        let stats = combat_stats.get_mut(*target);
                        if let Some(stats) = stats {
                            stats.hp = i32::min(stats.max_hp, stats.hp + healer.restore_hp);
                            if entity == *player_entity {
                                gamelog.entries.push(format!(
                                    "You used the {}, healing {} hit points.",
                                    names.get(useitem.item).unwrap().name,
                                    healer.restore_hp
                                ));
                            }
                        }
                    }
                }
            }
            let mut add_confusion = Vec::new();
            {
                let item_confuses = confuse.get(useitem.item);
                match item_confuses {
                    None => {}
                    Some(confusion) => {
                        used_item = false;
                        for mob in targets.iter() {
                            add_confusion.push((*mob, confusion.turns));
                            if entity == *player_entity {
                                let mob_name = &names.get(*mob).unwrap().name;
                                let item_name = &names.get(useitem.item).unwrap().name;
                                gamelog.entries.push(format!(
                                    "You use {} on {}, confusing them.",
                                    item_name, mob_name
                                ))
                            }
                        }
                    }
                }
            }
            for mob in add_confusion.iter() {
                confuse
                    .insert(mob.0, Confusion { turns: mob.1 })
                    .expect("Unable to insert status");
            }

            //if damaging, apply damage to target.
            let item_damages = inflict_damage.get(useitem.item);
            match item_damages {
                None => {}
                Some(damage) => {
                    used_item = false;
                    for mob in targets.iter() {
                        SufferDamage::new_damage(&mut suffer_damage, *mob, damage.damage);
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(useitem.item).unwrap();
                            gamelog.entries.push(format!(
                                "You use the {} on {}, inflicting {} damage.",
                                item_name.name, mob_name.name, damage.damage
                            ))
                        }
                        used_item = true;
                    }
                }
            }
            //if consumable, consume item
            if used_item {
                let consumable = consumables.get(useitem.item);
                match consumable {
                    None => {}
                    Some(_) => {
                        entities.delete(useitem.item).expect("Delete failed");
                    }
                }
            }
        }
        wants_use.clear();
    }
}

pub struct ItemDropSystem {}
impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_drop,
            names,
            mut positions,
            mut backpack,
        ) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_position = Position { x: 0, y: 0 };
            {
                let dropped_position = positions.get(entity).unwrap();
                dropper_position.x = dropped_position.x;
                dropper_position.y = dropped_position.y;
            }
            positions
                .insert(
                    to_drop.item,
                    Position {
                        x: dropper_position.x,
                        y: dropper_position.y,
                    },
                )
                .expect("Unable to insert position.");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(format!(
                    "You drop the {}",
                    names.get(to_drop.item).unwrap().name
                ));
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
