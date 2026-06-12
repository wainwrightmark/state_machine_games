use leptos::logging;
use state_machine_games::prelude::{GameCommand, MutationResult};
use web_sys::js_sys;
use ws_core::prelude::Tile4x4;

use crate::{dalas_drow_game_state::DalasDrowGameState, puzzle::Puzzle};

#[derive(Debug, Clone, PartialEq)]
pub enum DalasDrowCommand {
    ClickTile(Tile4x4),
    ChangeLevel { puzzle: Puzzle },
    ResetLevel,
}

impl GameCommand<DalasDrowGameState> for DalasDrowCommand {
    fn apply_command(&self, game_state: &mut DalasDrowGameState) -> MutationResult {
        match self {
            DalasDrowCommand::ResetLevel => {
                let start_timestamp = js_sys::Date::now();
                *game_state = DalasDrowGameState::new(game_state.puzzle.clone(), start_timestamp);
                MutationResult::CHANGED_NO_TRANSITION
            }

            DalasDrowCommand::ChangeLevel { puzzle } => {
                *game_state = DalasDrowGameState::new(puzzle.clone(), js_sys::Date::now());
                MutationResult::CHANGED_NO_TRANSITION
            }
            DalasDrowCommand::ClickTile(tile) => match game_state.selected_tile {
                Some(selected_tile) => {
                    if selected_tile == *tile {
                        game_state.selected_tile = None;
                        return MutationResult::CHANGED_NO_TRANSITION;
                    } else {
                        //logging::log!("Swapping tiles {tile} & {selected_tile}");
                        
                        game_state.grid.swap(*tile, selected_tile);
                        //logging::log!("New Grid: {}", game_state.grid);
                        game_state.selected_tile = None;
                        if game_state.finish_seconds.is_none() && game_state.is_complete() {
                            game_state.finish_seconds = Some(
                                ((js_sys::Date::now() - game_state.start_timestamp) / 1000.0) as u32,
                            );
                        }
                        return MutationResult::CHANGED_NO_TRANSITION;
                    }
                }
                None => {
                    game_state.selected_tile = Some(*tile);
                    return MutationResult::CHANGED_NO_TRANSITION;
                }
            },
        }
    }
}
