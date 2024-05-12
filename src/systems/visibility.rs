use crate::components::{Hidden, Name, Player, Position, Viewshed};
use crate::{gamelog::GameLog, Map};
use bracket_lib::prelude::*;
use specs::prelude::*;

pub struct FoVSystem {}

impl<'a> System<'a> for FoVSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Hidden>,
        WriteExpect<'a, RandomNumberGenerator>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            entities,
            mut viewshed,
            position,
            _player,
            mut hidden,
            mut rng,
            mut log,
            names,
        ) = data;

        for (_entity, viewshed, position) in (&entities, &mut viewshed, &position).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles =
                    field_of_view(Point::new(position.x, position.y), viewshed.range, &*map);
                viewshed
                    .visible_tiles
                    .retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

                // reveal player sight
                let _p: Option<&Player> = _player.get(_entity);
                if let Some(_p) = _p {
                    for t in map.visible_tiles.iter_mut() {
                        *t = false
                    }
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.xy_idx(vis.x, vis.y);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true;

                        for e in map.tile_content[idx].iter() {
                            let maybe_hidden = hidden.get(*e);
                            if let Some(_maybe_hidden) = maybe_hidden {
                                if rng.roll_dice(1, 24) == 1 {
                                    let name = names.get(*e);
                                    if let Some(name) = name {
                                        log.entries.push(format!("You spotted a {}.", &name.name));
                                    }
                                    hidden.remove(*e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
