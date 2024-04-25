use crate::components::{Player, Position, Viewshed};
use crate::Map;
use rltk::{field_of_view, Point};
use specs::prelude::*;

pub struct FoVSystem {}

impl<'a> System<'a> for FoVSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewshed, position, _player) = data;

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
                    }
                }
            }
        }
    }
}
