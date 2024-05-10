use crate::components::{CombatStats, Player, Position, Viewshed, WantsToMelee};
use crate::{Map, Monster, RunState};
use bracket_lib::prelude::*;
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let mut viewshed = ecs.write_storage::<Viewshed>();
    let entities = ecs.entities();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.fetch::<Map>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();

    for (entity, _player, pos, viewshed) in
        (&entities, &players, &mut positions, &mut viewshed).join()
    {
        // checks to see if player is about to step out-of bounds
        // TODO: currently does not work as expected, fix
        if pos.x + delta_x < 1
            || pos.x + delta_x > map.width - 2
            || pos.y + delta_y < 1
            || pos.y + delta_x > map.height - 2
        {
            console::log(&format!("Unable to step forward"));
            return;
        }
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        // if there is a target in front of the player, attempt to melee.
        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *potential_target,
                        },
                    )
                    .expect("Failed to add target");
                return;
            }
        }

        // if location isn't blocked, move to location and update viewshed.
        if !map.blocked[destination_idx] {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));

            viewshed.dirty = true;
            let mut player_pos = ecs.write_resource::<Point>();
            player_pos.x = pos.x;
            player_pos.y = pos.y;
        }
    }
}

pub fn skip_turn(world: &mut World) -> RunState {
    let player_entity = world.fetch::<Entity>();
    let viewshed_components = world.read_storage::<Viewshed>();
    let mobs = world.read_storage::<Monster>();

    let worldmap_resource = world.fetch::<Map>();

    let mut can_heal = true;
    let viewshed = viewshed_components.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = worldmap_resource.xy_idx(tile.x, tile.y);
        for entity_id in worldmap_resource.tile_content[idx].iter() {
            let mob = mobs.get(*entity_id);
            match mob {
                None => {}
                Some(_) => {
                    can_heal = false;
                }
            }
        }
    }

    if can_heal {
        let mut health_components = world.write_storage::<CombatStats>();
        let player_hp = health_components.get_mut(*player_entity).unwrap();
        player_hp.hp = i32::min(player_hp.hp + 1, player_hp.max_hp);
    }
    return RunState::PlayerTurn;
}
