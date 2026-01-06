use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum QuizSaladCommand {
    TileClicked(TileClickedCommand),
    LozengeClicked(LozengeClickedCommand),
}

impl GameCommand<QuizSaladGameState> for QuizSaladCommand {
    fn apply_command(&self, game_state: &mut QuizSaladGameState) -> MutationResult {
        match self {
            QuizSaladCommand::TileClicked(tile_clicked_command) => {
                tile_clicked_command.apply_command(game_state)
            }
            QuizSaladCommand::LozengeClicked(lozenge_clicked_command) => {
                lozenge_clicked_command.apply_command(game_state)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TileClickedCommand(pub Tile4x4);

impl GameCommand<QuizSaladGameState> for TileClickedCommand {
    fn apply_command(&self, game_state: &mut QuizSaladGameState) -> MutationResult {
        let clicked_tile = self.0;
        if game_state.found_words.unneeded_tiles.get_bit(&clicked_tile) {
            game_state.chosen_state = ChosenState::default();
            return MutationResult::CHANGED_NO_TRANSITION;
        }

        let current_solution = game_state.chosen_state.current_solution();

        let Some(last_tile) = current_solution.last().copied() else {
            game_state.chosen_state = ChosenState {
                solution: ArrayVec::from_iter([clicked_tile]),
            };
            return MutationResult::CHANGED_NO_TRANSITION;
        };

        if clicked_tile == last_tile {
            let mut new_solution = current_solution.clone();
            new_solution.pop();
            game_state.chosen_state = ChosenState {
                solution: new_solution,
            };
            return MutationResult::CHANGED_NO_TRANSITION;
        }

        if let Some(position) = current_solution.iter().position(|&x| x == clicked_tile) {
            let mut new_solution = current_solution.clone();
            new_solution.truncate(position + 1);
            game_state.chosen_state = ChosenState {
                solution: new_solution,
            };

            return MutationResult::CHANGED_NO_TRANSITION;
        }

        if clicked_tile.is_adjacent_to(&last_tile) {
            let mut new_solution = current_solution.clone();
            new_solution.push(clicked_tile);
            game_state.chosen_state = ChosenState {
                solution: new_solution,
            };
            if let Some(solution_index) = game_state
                .puzzle
                .check_solution(&game_state.chosen_state.solution)
            {
                let c_index = game_state
                    .found_words
                    .word_completions
                    .iter()
                    .filter(|x| x.is_complete())
                    .count() as u8;

                if let Some(completion) = game_state
                    .found_words
                    .word_completions
                    .get_mut(solution_index)
                {
                    if !completion.is_complete() {
                        *completion = found_words_state::Completion::Complete { index: c_index };
                        game_state.word_just_found = true;
                        return MutationResult::changed_with_transition(500.0);
                    }
                }
            }

            return MutationResult::CHANGED_NO_TRANSITION;
        } else {
            game_state.chosen_state = ChosenState::default();

            return MutationResult::CHANGED_NO_TRANSITION;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct LozengeClickedCommand(pub usize);

impl GameCommand<QuizSaladGameState> for LozengeClickedCommand {
    fn apply_command(&self, game_state: &mut QuizSaladGameState) -> MutationResult {
        game_state.current_clue = self.0;
        return MutationResult::CHANGED_NO_TRANSITION;
    }
}
