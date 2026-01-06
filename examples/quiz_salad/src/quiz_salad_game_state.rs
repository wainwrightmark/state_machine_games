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

    fn apply_command(&mut self, command: &Self::Command) -> MutationResult {
        match command {
            QuizSaladCommand::TileClicked(clicked_tile) => {
                let clicked_tile = *clicked_tile;
                if self.found_words.unneeded_tiles.get_bit(&clicked_tile) {
                    self.chosen_state = ChosenState::default();
                    return MutationResult::CHANGED_NO_TRANSITION;
                }

                let current_solution = self.chosen_state.current_solution();

                let Some(last_tile) = current_solution.last().copied() else {
                    self.chosen_state = ChosenState {
                        solution: ArrayVec::from_iter([clicked_tile]),
                    };
                    return MutationResult::CHANGED_NO_TRANSITION;
                };

                if clicked_tile == last_tile {
                    let mut new_solution = current_solution.clone();
                    new_solution.pop();
                    self.chosen_state = ChosenState {
                        solution: new_solution,
                    };
                    return MutationResult::CHANGED_NO_TRANSITION;
                }

                if let Some(position) = current_solution.iter().position(|&x| x == clicked_tile) {
                    let mut new_solution = current_solution.clone();
                    new_solution.truncate(position + 1);
                    self.chosen_state = ChosenState {
                        solution: new_solution,
                    };

                    return MutationResult::CHANGED_NO_TRANSITION;
                }

                if clicked_tile.is_adjacent_to(&last_tile) {
                    let mut new_solution = current_solution.clone();
                    new_solution.push(clicked_tile);
                    self.chosen_state = ChosenState {
                        solution: new_solution,
                    };
                    if let Some(solution_index) =
                        self.puzzle.check_solution(&self.chosen_state.solution)
                    {
                        let c_index = self
                            .found_words
                            .word_completions
                            .iter()
                            .filter(|x| x.is_complete())
                            .count() as u8;

                        if let Some(completion) =
                            self.found_words.word_completions.get_mut(solution_index)
                        {
                            if !completion.is_complete() {
                                *completion =
                                    found_words_state::Completion::Complete { index: c_index };
                                self.word_just_found = true;
                                return MutationResult::changed_with_transition(500.0);
                            }
                        }
                    }

                    return MutationResult::CHANGED_NO_TRANSITION;
                } else {
                    self.chosen_state = ChosenState::default();

                    return MutationResult::CHANGED_NO_TRANSITION;
                }
            }
            QuizSaladCommand::LozengeClicked(index) => {
                self.current_clue = *index;
                return MutationResult::CHANGED_NO_TRANSITION;
            }
        }
    }

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