use crate::prelude::{GameState, MutationResult};

pub trait AnyGameCommand: Send + Sync + 'static + std::fmt::Debug + Clone {}

pub trait GameCommand<GS> : AnyGameCommand{
    fn apply_command(&self, game_state: &mut GS) -> MutationResult;
}


impl AnyGameCommand for () {}//todo remove this?

impl<GS :GameState> GameCommand<GS> for (){
    fn apply_command(&self, _game_state: &mut GS) -> MutationResult {
        MutationResult::NO_CHANGE
    }
}