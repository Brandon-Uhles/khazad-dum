use crate::components::Item;
use crate::gui::ItemMenuResult;
use crate::systems::{
    inventory::get_item,
    player::try_move_player
};
use crate::{RunState, State};
use specs::prelude::*;
use rltk::{Rltk, VirtualKeyCode};

/// tracks player input. TODO: add controller support
pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    //player movement
    match ctx.key {
        None => return RunState::AwaitingInput,
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                try_move_player(-1, 0, &mut gs.ecs)
            }

            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                try_move_player(1, 0, &mut gs.ecs)
            }

            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                try_move_player(0, -1, &mut gs.ecs)
            }

            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                try_move_player(0, 1, &mut gs.ecs)
            }

            // Diagonals
            VirtualKeyCode::Numpad9 | VirtualKeyCode::Y => try_move_player(1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad7 | VirtualKeyCode::U => try_move_player(-1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad3 | VirtualKeyCode::N => try_move_player(1, 1, &mut gs.ecs),

            VirtualKeyCode::Numpad1 | VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.ecs),

            VirtualKeyCode::G  => get_item(&mut gs.ecs),

            VirtualKeyCode::I => return RunState::ShowInventory,

            _ => return RunState::AwaitingInput,
        },
    }
    RunState::PlayerTurn
}
 
 pub fn menu_input(ctx: &mut Rltk, count: usize, equippable: &mut Vec<Entity>) -> (ItemMenuResult, Option<Entity>) {
    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => {(ItemMenuResult::Cancel, None)} 
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (ItemMenuResult::Selected, Some(equippable[selection as usize]))
                    }
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
 }