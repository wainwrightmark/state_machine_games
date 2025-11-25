use crate::prelude::MutationResult;

pub trait AnyGameCommand: Send + Sync + 'static + std::fmt::Debug + Clone {}

pub trait GameCommand<GS> : AnyGameCommand{
    fn apply_command(&self, games_state: &mut GS) -> MutationResult;
}


impl AnyGameCommand for () {}//todo remove this?