use std::fmt::Debug;

use crate::{delayed_effect::MutationResult, prelude::GameState};

pub trait GameCommand<GS: GameState>: Send + Debug + 'static {
    fn apply_command(&self, game_state: &mut GS) -> MutationResult<GS>;
}

impl<GS: GameState> GameCommand<GS> for () {
    fn apply_command(&self, _game_state: &mut GS) -> MutationResult<GS> {
        MutationResult::NoChange
    }
}
