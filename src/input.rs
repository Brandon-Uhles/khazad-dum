use crate::ecs::systems::try_move_player;
use crate::{RunState, State};
use rltk::{Rltk, VirtualKeyCode};

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    //player movement
    match ctx.key {
        None => return RunState::Paused,
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            _ => return RunState::Paused,
        },
    }
    RunState::Running
}
