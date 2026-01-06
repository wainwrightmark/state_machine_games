use crate::{quiz_salad_command::QuizSaladCommand, *};

#[derive(Debug, Clone, PartialEq)]
pub struct QuizSaladGameState {
    pub puzzle: Puzzle,
    pub current_clue: usize,
    pub found_words: FoundWordsState,
    pub chosen_state: ChosenState,
    pub word_just_found: bool,
}

impl QuizSaladGameState {
    pub fn new(puzzle: Puzzle) -> Self {
        let found_words = FoundWordsState::new_from_level(&puzzle);

        Self {
            current_clue: Default::default(),
            puzzle,
            found_words,
            chosen_state: Default::default(),
            word_just_found: false,
        }
    }
}

impl GameState for QuizSaladGameState {
    type Command = QuizSaladCommand;

    fn maybe_transition(&mut self) -> MutationResult {
        if self.word_just_found {
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
                let new_current_clue = (0..self.puzzle.words.len())
                    .cycle()
                    .skip(self.current_clue)
                    .take(self.puzzle.words.len())
                    .filter(|&index| !self.found_words.get_completion(index).is_complete())
                    .next();

                self.current_clue = new_current_clue.unwrap_or_default();
            }

            MutationResult::CHANGED_NO_TRANSITION
        } else {
            MutationResult::NO_CHANGE
        }
    }
}

impl HasSegment<FoundWordsState> for QuizSaladGameState {
    fn get_segment(&self) -> FoundWordsState {
        self.found_words.clone()
    }

    fn segment_eq(&self, s: &FoundWordsState) -> bool {
        self.found_words.eq(s)
    }
}
