use crate::{grid_input::GridInputCommand, *};

#[derive(Debug, Clone, PartialEq)]
pub enum QuizSaladCommand {
    BoardPointerEvent(GridInputCommand),
    LozengeClicked(LozengeClickedCommand),
    ChangeLevel {
        puzzle: Puzzle,
        found_words: FoundWordsState,
        elapsed_ms: f64,
    },
    ResetLevel
}

impl GameCommand<QuizSaladGameState> for QuizSaladCommand {
    fn apply_command(&self, game_state: &mut QuizSaladGameState) -> MutationResult {
        match self {
            QuizSaladCommand::BoardPointerEvent(gic) => {
                //leptos::logging::log!("GIC: {gic:?}");
                gic.apply_command(game_state)
            }
            QuizSaladCommand::LozengeClicked(lozenge_clicked_command) => {
                lozenge_clicked_command.apply_command(game_state)
            }

            QuizSaladCommand::ResetLevel =>{
                let start_timestamp = js_sys::Date::now();
                *game_state = QuizSaladGameState::new(game_state.puzzle.clone(), start_timestamp);
                MutationResult::CHANGED_NO_TRANSITION
            }

            QuizSaladCommand::ChangeLevel {
                puzzle,
                found_words,
                elapsed_ms,
            } => {
                let current_clue = found_words
                    .word_completions
                    .iter()
                    .enumerate()
                    .filter(|(_index, completion)| !completion.is_complete())
                    .map(|x| x.0)
                    .next()
                    .unwrap_or_default();

                let start_timestamp = (js_sys::Date::now() - elapsed_ms).max(0.0);

                let finish_seconds = if found_words.is_level_complete() {
                    Some((elapsed_ms / 1000.0).floor().max(0.0) as u32)
                } else {
                    None
                };

                *game_state = QuizSaladGameState {
                    puzzle: puzzle.clone(),
                    current_clue,
                    found_words: found_words.clone(),
                    chosen_state: ChosenState::default(),
                    input_state: GridInputState::default(),
                    start_timestamp,
                    finish_seconds,
                };
                MutationResult::CHANGED_NO_TRANSITION
            }
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
