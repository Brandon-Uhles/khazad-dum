// module for all systems
use specs::prelude::*;
use super::components::{LeftMover, Player, Position};
use std::cmp::{max, min};

pub struct LeftWalker{}

impl<'a> System<'a> for LeftWalker {
    // requests read and write access for the specified components from the system data
    type SystemData = (
        ReadStorage<'a, LeftMover>,
        WriteStorage<'a, Position>
    );

    fn run(&mut self, (lefty, mut pos) : Self::SystemData) {
        for(_lefty,pos) in (&lefty, &mut pos).join() {
            pos.x -= 1;
            if pos.x < 0 { pos.x = 79; }
        }
    }
}

 pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        pos.x = min(79 , max(0, pos.x + delta_x));
        pos.y = min(49, max(0, pos.y + delta_y));
    }
}