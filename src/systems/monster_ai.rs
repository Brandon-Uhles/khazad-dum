use crate::components::*;
use crate::{ Map, RunState};
use rltk::Point;
use specs::prelude::*;

pub struct MonsterAI {}
impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, Confusion>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            player_entity,
            runstate,
            entities,
            mut viewshed,
            monster,
            mut position,
            mut wants_to_melee,
            mut confused,
        ) = data;

        // break out of monster_ai if it is not the monster's turn
        if *runstate != RunState::MonsterTurn {
            return;
        }

        for (entity, mut viewshed, _monster, mut position) in
            (&entities, &mut viewshed, &monster, &mut position).join()
        {
            let mut can_act = true;

            let is_confused = confused.get_mut(entity);
            if let Some(is_confused) = is_confused {
                is_confused.turns -= 1;
                if is_confused.turns < 1 {
                    confused.remove(entity);
                }
                can_act = false;
            }

            if can_act {
                // grabs distance from monster to player via pythagorean distance algorithm
                // TODO: evaluate potential for differnet algos
                let distance = rltk::DistanceAlg::Pythagoras
                .distance2d(Point::new(position.x, position.y), *player_pos);
                
                // in melee range? melee
                if distance < 1.5 {
                    wants_to_melee
                        .insert(
                            entity,
                            WantsToMelee {
                                target: *player_entity,
                            },
                        )
                        .expect("Unable to insert attack");
                }

                    // if out of melee range but visible, move towards player
                else if viewshed.visible_tiles.contains(&*player_pos) {
                    // Path to player
                    // TODO: research a_star_search, see if pathing algo is ideal
                    let path = rltk::a_star_search(
                        map.xy_idx(position.x, position.y),
                        map.xy_idx(player_pos.x, player_pos.y),
                        &mut *map,
                    );
                        // if the pathing algo is successful and the monster isn't in front of the player
                        // unblock old spot, step forward, block new spot, trigger viewshed redraw
                    if path.success && path.steps.len() > 1 {
                        let mut idx = map.xy_idx(position.x, position.y);
                        map.blocked[idx] = false;
                        position.x = path.steps[1] as i32 % map.width;
                        position.y = path.steps[1] as i32 / map.width;
                        idx = map.xy_idx(position.x, position.y);
                        map.blocked[idx] = true;
                        viewshed.dirty = true;
                    }
                }   
            } 
        }
    }
}
