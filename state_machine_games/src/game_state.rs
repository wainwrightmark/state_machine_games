use crate::prelude::*;

pub trait GameState: Send + Sync + 'static {
    type Command: GameCommand;

    fn apply_command(&mut self, command: &Self::Command) -> MutationResult;

    fn maybe_transition(&mut self) -> MutationResult;
}

pub trait HasSegment<T>: GameState {
    fn get_segment(&self) -> T;
    fn segment_eq(&self, s: &T) -> bool;
}

impl<T: GameState + PartialEq + Clone> HasSegment<T> for T {
    fn get_segment(&self) -> T {
        self.clone()
    }

    fn segment_eq(&self, s: &T) -> bool {
        self.eq(s)
    }
}

impl<T: GameState> HasSegment<()> for T {
    fn get_segment(&self) -> () {
        ()
    }

    fn segment_eq(&self, _: &()) -> bool {
        true
    }
}
