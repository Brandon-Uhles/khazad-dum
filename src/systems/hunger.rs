use specs::prelude::*;
use crate::{
    HungerClock, HungerState, RunState, SufferDamage, gamelog::GameLog
};

pub struct HungerSystem {}

impl<'a> System<'a> for HungerSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, HungerClock>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, runstate, entities, mut hunger_clock, mut suffer_damage, mut log) = data;

        for(entity, mut clock) in (&entities, &mut hunger_clock).join() {
            let mut proceed = false;

            match *runstate {
                RunState::PlayerTurn => {
                    if entity == *player_entity {
                        proceed = true;
                    }
                }
                RunState::MonsterTurn => {
                    if entity != *player_entity {
                        proceed = true;
                    }
                }
                _ => proceed = false
            }

            if proceed {
                clock.duration -= 1;
                if clock.duration < 1 {
                    match clock.state {
                        HungerState::WellFed => {
                            clock.state = HungerState::Normal;
                            clock.duration = 50;
                            if entity == *player_entity {
                                log.entries.push("You are no longer well fed.".to_string());
                            }
                        }
                        HungerState::Normal => {
                            clock.state = HungerState::Hungry;
                            clock.duration = 50;
                            if entity == *player_entity {
                                log.entries.push("You are hungry.".to_string());
                            }
                        }
                        HungerState::Hungry => {
                            clock.state = HungerState::Starving;
                            clock.duration = 50;
                            
                            if entity == *player_entity {
                                log.entries.push("You are starving!".to_string());
                            }
                        }
                        HungerState::Starving => {
                            if entity == *player_entity {
                                log.entries.push("You need to eat!".to_string());
                            }
                            SufferDamage::new_damage(&mut suffer_damage, entity, 1);
                        }
                    }
                }
            }
        }
    }
}