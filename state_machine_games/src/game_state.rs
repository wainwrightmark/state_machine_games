use crate::prelude::*;

pub trait GameState: Send + Sync + 'static {
    type Command: GameCommand;
    type Key: GameEntityKey;
    fn entities(&self, receiver: &mut impl EntityReceiver<Self::Command, Self::Key>);

    fn apply_command(&mut self, command: Self::Command) -> MutationResult;

    fn maybe_transition(&mut self) -> MutationResult;
}
