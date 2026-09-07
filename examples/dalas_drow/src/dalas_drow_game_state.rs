use state_machine_games::define_lens;
use ws_core::prelude::{Grid, WordTrait};

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct DalasDrowGameState {
    pub puzzle: Puzzle,
    pub selected_tile: Option<Tile4x4>,
    pub grid: Grid<4, 16>,
    pub start_timestamp: f64,
    pub finish_seconds: Option<u32>,
}

impl DalasDrowGameState {
    pub fn new(puzzle: Puzzle, start_timestamp: f64) -> Self {
        Self {
            grid: puzzle.grid.clone(),
            puzzle,
            start_timestamp,
            finish_seconds: None,
            selected_tile: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        for w in self.puzzle.words.iter() {
            if w.find_solution(self.grid).is_none() {
                return false;
            }
        }
        return true;
    }
}

impl GameState for DalasDrowGameState {}

define_lens!(StartTimeLens, DalasDrowGameState, f64, start_timestamp);
define_lens!(
    FinishTimeLens,
    DalasDrowGameState,
    Option<u32>,
    finish_seconds
);

impl DalasDrowGameState {
    pub fn background_target_color(&self) -> Srgba {
        if self.finish_seconds.is_some() {
            CLASSIC_COLOR_SCHEME.background_complete
        } else {
            CLASSIC_COLOR_SCHEME.background_incomplete
        }
    }
}
