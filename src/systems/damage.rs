use crate::{
    components::{CombatStats, Name, Player, SufferDamage}, entities, gamelog::GameLog, Map, Position
};
use specs::prelude::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, Map>,
        Entities<'a>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage, positions, mut map, entities) = data;

        for (entity, mut stats, damage) in (&entities, &mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
            let pos = positions.get(entity);
            if let Some(pos) = pos {
                let idx = map.xy_idx(pos.x, pos.y);
                map.bloodstains.insert(idx);
            }
        }

        damage.clear();
    }
}

/// delete dead entities
pub fn delete_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();

    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let entities = ecs.entities();
        let names = ecs.read_storage::<Name>();
        let mut log = ecs.write_resource::<GameLog>();

        // gathers all entities with combat_stats and loops through each
        // if entity.hp < 1, check to see if the entity is the player.
        // if it is, tell the player they died.
        // if not, push them into the dead vector
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                let player = players.get(entity);
                match player {
                    None => {
                        let victim_name = names.get(entity);
                        if let Some(victim_name) = victim_name {
                            log.entries.push(format!("{} has died.", victim_name.name));
                        }
                        dead.push(entity)
                    }
                    Some(_) => {
                        log.entries.push(format!("You died"));
                    }
                }
            }
        }
    }
    // deletes all entities in the dead vector
    for victim in dead {
        ecs.delete_entity(victim).expect("unable to delete");
    }
}
