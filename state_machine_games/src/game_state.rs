use crate::prelude::*;

pub trait GameState: Send + Sync + 'static {    
    fn maybe_transition(&mut self) -> MutationResult;
}
