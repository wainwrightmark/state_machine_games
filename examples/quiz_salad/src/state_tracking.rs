use std::sync::{Arc, Mutex};

use indexed_db_futures::{Build, database::Database, prelude::*};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use state_machine_games::prelude::ChangeWatcher;
use ws_core::{BasicWordTrait, LevelTrait, TileSet32};

use crate::{found_words_state::FoundWordsState, puzzle::Puzzle, quiz_salad_game_state::QuizSaladGameState};

const STORE_NAME: &'static str = "quiz_salad_saves";


pub struct FoundWordsStateTracker{
    //pub db: Arc<Mutex<Database> >,
    pub current_puzzle: Puzzle,
    pub current_found_words: FoundWordsState,
}

// impl ChangeWatcher<QuizSaladGameState> for FoundWordsStateTracker{
//     fn on_state_change(&mut self, state: &QuizSaladGameState, reason: &state_machine_games::prelude::StateChangeReason<QuizSaladGameState>) -> bool {
//         panic!()   
//     }

//     fn step_animations(&mut self, delta_ms: f64) {
//         //do nothing
//     }

//     // fn new(state: &QuizSaladGameState) -> Self {
        
//     //     Self { db: () }
//     // }
// }


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

pub async fn try_save_progress(
    level: &impl LevelTrait<4, 16>,
    found_words: &FoundWordsState,
    db: &Database,
) -> Result<(), anyhow::Error> {
    let transaction = db
        .transaction(STORE_NAME)
        .with_mode(web_sys::IdbTransactionMode::Readwrite)
        .build()
        .map_err(|err| anyhow::anyhow!("{err}"))?;

    let store = transaction
        .object_store(STORE_NAME)
        .map_err(|err| anyhow::anyhow!("{err}"))?;

    let saved_level_state = SavedLevelState::new(level, found_words);

    let _ = store.put(saved_level_state);

    transaction
        .commit()
        .await
        .map_err(|err| anyhow::anyhow!("{err}"))?;

    Ok(())
}

pub async fn try_load_progress(
    level: &impl LevelTrait<4, 16>,
    db: &Database,
) -> Result<Option<FoundWordsState>, anyhow::Error> {
    let transaction = db
        .transaction(STORE_NAME)
        .with_mode(web_sys::IdbTransactionMode::Readonly)
        .build()
        .map_err(|err| anyhow::anyhow!("{err}"))?;

    let store = transaction
        .object_store(STORE_NAME)
        .map_err(|err| anyhow::anyhow!("{err}"))?;
    let key = get_level_key(level);

    let get_result: Option<String> = store
        .get(key)
        .await
        .map_err(|err| anyhow::anyhow!("{err}"))?;

    transaction
        .commit()
        .await
        .map_err(|err| anyhow::anyhow!("{err}"))?;

    let found_result = match get_result {
        Some(str) => match serde_json::from_str::<SavedLevelState>(&str) {
            Ok(saved_level_state) => Some(saved_level_state.to_found_words_state(level)),
            Err(err) => {
                anyhow::bail!("{err}");
            }
        },
        None => None,
    };

    Ok(found_result)
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
}

impl SavedLevelState {
    pub fn new(level: &impl LevelTrait<4, 16>, found_words: &FoundWordsState) -> Self {
        let key = get_level_key(level);
        Self {
            key,
            word_completions: found_words.word_completions.clone(),
            hints_used: found_words.hints_used,
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
