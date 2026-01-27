use indexed_db_futures::{Build, database::Database};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use state_machine_games::prelude::ChangeWatcher;
use web_sys::js_sys;
use ws_core::{BasicWordTrait, LevelTrait, TileSet32};

use crate::{
    database_handler::{self, *},
    found_words_state::FoundWordsState,
    puzzle::Puzzle,
    quiz_salad_command::QuizSaladCommand,
    quiz_salad_game_state::QuizSaladGameState,
};

const STORE_NAME: &'static str = "quiz_salad_saves";

pub struct FoundWordsStateTracker {
    current_puzzle: Puzzle,
    current_found_words: FoundWordsState,
    sender: async_channel::Sender<QSDatabaseCommand>,
}

impl FoundWordsStateTracker {
    pub fn new(
        current_puzzle: Puzzle,
        current_found_words: FoundWordsState,
        sender: async_channel::Sender<QSDatabaseCommand>,
    ) -> Self {
        Self {
            current_puzzle,
            current_found_words,
            sender,
        }
    }
}

impl ChangeWatcher<QuizSaladGameState> for FoundWordsStateTracker {
    fn on_state_change(
        &mut self,
        state: &QuizSaladGameState,
        reason: &state_machine_games::prelude::StateChangeReason<QuizSaladGameState>,
    ) -> bool {
        match reason {
            state_machine_games::prelude::StateChangeReason::InitialState => {
                return false;
            }
            state_machine_games::prelude::StateChangeReason::Transition => {
                return false;
            }
            state_machine_games::prelude::StateChangeReason::Command(_) => {}
        }

        if self.current_puzzle != state.puzzle {
            self.current_puzzle = state.puzzle.clone();
            self.current_found_words = state.found_words.clone();
            //update the puzzle and found words but don't submit a save
            return true;
        }

        if self.current_found_words != state.found_words {
            self.current_found_words = state.found_words.clone();
            let now = js_sys::Date::now();
            let ms_used = (now - state.start_timestamp).max(0.0);

            let sls =
                SavedLevelState::new(&self.current_puzzle, &self.current_found_words, ms_used);
            leptos::logging::log!("Send Save Level Command");
            self.sender
                .try_send(QSDatabaseCommand::SaveLevel(sls))
                .unwrap();
            return true;
        }
        return false;
    }

    fn step_animations(&mut self, _delta_ms: f64) {}
}

pub async fn set_up_db() -> indexed_db_futures::OpenDbResult<Database> {
    let db = Database::open("quiz_salad")
        .with_version(1u8)
        .with_on_upgrade_needed(|_event, db| {
            let _ = db
                .create_object_store(STORE_NAME)
                .with_key_path(indexed_db_futures::KeyPath::One("key"))
                .build()?;

            Ok(())
        })
        .await?;

    Ok(db)
}

pub async fn handle_db_messages(
    output_sender: std::sync::mpsc::Sender<QuizSaladCommand>,
    command_receiver: async_channel::Receiver<QSDatabaseCommand>,
) {
    leptos::logging::log!("Opening DB");
    let db = set_up_db().await.expect("Could not open database");
    leptos::logging::log!("DB Opened");
    let mut handler = DatabaseHandler {
        db,
        output_sender,
        command_receiver,
    };

    handler.run().await.unwrap()
}

pub enum QSDatabaseCommand {
    SaveLevel(SavedLevelState),
    LoadLevel(Puzzle),
}

impl DatabaseCommand for QSDatabaseCommand {
    type Output = QuizSaladCommand;

    async fn run_command(
        &self,
        db: &indexed_db_futures::database::Database,
    ) -> Result<Option<Self::Output>, anyhow::Error> {
        match self {
            QSDatabaseCommand::SaveLevel(saved_level_state) => {
                crate::database_handler::try_write_object(saved_level_state, db, STORE_NAME)
                    .await?;
                return Ok(None);
            }
            QSDatabaseCommand::LoadLevel(puzzle) => {
                let key = get_level_key(puzzle);

                match database_handler::try_read_object::<SavedLevelState>(db, STORE_NAME, &key)
                    .await?
                {
                    Some(saved_level_state) => {
                        let found_words = saved_level_state.to_found_words_state(puzzle);
                        let elapsed_ms = saved_level_state.elapsed_ms;
                        let command = QuizSaladCommand::ChangeLevel {
                            puzzle: puzzle.clone(),
                            found_words,
                            elapsed_ms
                        };
                        return Ok(Some(command));
                    }
                    None => return Ok(None),
                }
            }
        }
    }
}

pub fn get_level_key(level: &impl LevelTrait<4, 16>) -> String {
    let grid = level.grid().iter().join("");
    let words = level.words().iter().map(|x| x.text()).join("\t");
    let unencoded = format!("{grid}\t\t{words}");
    use base64::Engine;
    let encoded = base64::engine::general_purpose::URL_SAFE.encode(unencoded);
    encoded
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedLevelState {
    pub key: String,
    pub word_completions: Vec<crate::Completion>,
    pub hints_used: usize,
    pub elapsed_ms: f64,
}

impl SavedLevelState {
    pub fn new(
        level: &impl LevelTrait<4, 16>,
        found_words: &FoundWordsState,
        ms_used: f64,
    ) -> Self {
        let key = get_level_key(level);
        Self {
            key,
            word_completions: found_words.word_completions.clone(),
            hints_used: found_words.hints_used,
            elapsed_ms: ms_used,
        }
    }

    pub fn to_found_words_state(&self, level: &impl LevelTrait<4, 16>) -> FoundWordsState {
        let mut fws = FoundWordsState {
            unneeded_tiles: TileSet32::EMPTY, //we will recalculate this
            word_completions: self.word_completions.clone(),
            hints_used: self.hints_used,
        };
        fws.recalculate_unneeded_tiles(level);

        fws
    }
}
