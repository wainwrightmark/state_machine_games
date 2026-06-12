use state_machine_games::define_lens;

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct QuizSaladGameState {
    pub puzzle: Puzzle,
    pub current_clue: usize,
    pub found_words: FoundWordsState,
    pub chosen_state: ChosenState,
    pub input_state: GridInputState,
    pub start_timestamp: f64,
    pub finish_seconds: Option<u32>,
}

impl QuizSaladGameState {
    pub fn new(puzzle: Puzzle, start_timestamp: f64) -> Self {
        let found_words = FoundWordsState::new_from_level(&puzzle);

        Self {
            current_clue: Default::default(),
            puzzle,
            found_words,
            chosen_state: Default::default(),
            input_state: Default::default(),
            start_timestamp,
            finish_seconds: None,
        }
    }

    pub fn is_close_to_solution(&self) -> bool {
        self.chosen_state
            .is_close_to_a_solution(&self.puzzle, &self.found_words)
    }
}

impl GameState for QuizSaladGameState {
    fn maybe_transition(&mut self) -> MutationResult {
        if self.chosen_state.word_just_found {
            self.chosen_state.solution = ArrayVec::new();

            let new_unneeded_tiles = self
                .puzzle
                .calculate_unneeded_tiles(self.found_words.unneeded_tiles, |x| {
                    self.found_words.get_completion(x).is_complete()
                });
            self.found_words.unneeded_tiles = new_unneeded_tiles;

            if self
                .found_words
                .get_completion(self.current_clue)
                .is_complete()
            {
                if self.found_words.is_level_complete() {
                } else {
                    let new_current_clue = (0..self.puzzle.words.len())
                        .cycle()
                        .skip(self.current_clue)
                        .take(self.puzzle.words.len())
                        .filter(|&index| !self.found_words.get_completion(index).is_complete())
                        .next();

                    self.current_clue = new_current_clue.unwrap_or_default();
                }
            }
            self.chosen_state.word_just_found = false;

            MutationResult::CHANGED_NO_TRANSITION
        } else {
            MutationResult::NO_CHANGE
        }
    }
}

define_lens!(StartTimeLens, QuizSaladGameState, f64, start_timestamp);
define_lens!(
    FinishTimeLens,
    QuizSaladGameState,
    Option<u32>,
    finish_seconds
);

impl QuizSaladGameState {
    pub fn background_target_color(&self) -> Srgba {
        if self.found_words.is_level_complete() {
            CLASSIC_COLOR_SCHEME.background_complete
        } else {
            CLASSIC_COLOR_SCHEME.background_incomplete
        }
    }
}
