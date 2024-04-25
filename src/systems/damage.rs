use specs::prelude::*;
use crate::components::{CombatStats, SufferDamage, Player};
use rltk::console;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data : Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }

        damage.clear();
    }
}

/// delete dead entities
pub fn delete_dead(ecs: &mut World) {
    let mut dead : Vec<Entity> = Vec::new();

    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let entities = ecs.entities();

        // gathers all entities with combat_stats and loops through each
        // if entity.hp < 1, check to see if the entity is the player.
        // if it is, tell the player they died.
        // if not, push them into the dead vector
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                let player = players.get(entity);
                match player {
                    None => dead.push(entity),
                    Some(_) => console::log("You died")
                }
            }
        }
    }
    // deletes all entities in the dead vector
    for victim in dead {
        ecs.delete_entity(victim).expect("unable to delete");
    }
}