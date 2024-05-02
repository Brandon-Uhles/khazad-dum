use crate::components::BlocksTile;
use crate::{Map, Position};
use specs::prelude::*;
pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, position, blockers, entities) = data;
        map.populate_blocked();
        map.clear_content_index();

        for (entity, position) in (&entities, &position).join() {
            let idx = map.xy_idx(position.x, position.y);

            // If entity blocks, update  blocked list
            // poor naming, blockers is a list of entities that block
            // map.blocked is what is actually read to determine whether an entity can be moved to that point
            let _p: Option<&BlocksTile> = blockers.get(entity);
            if let Some(_p) = _p {
                map.blocked[idx] = true;
            }

            // Push entity to index slot inside map
            // entity implements Copy, so we don't need to clone it.
            map.tile_content[idx].push(entity);
        }
    }
}
