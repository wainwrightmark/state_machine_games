use std::sync::mpsc;

use impl_trait_for_tuples::impl_for_tuples;

use crate::prelude::*;

pub trait CommandReceiver<GS: GameState>: 'static {
    fn try_apply_command(&self, gs: &mut GS) -> Option<MutationResult>;
}

impl<GS: GameState> CommandReceiver<GS> for () {
    fn try_apply_command(&self, _gs: &mut GS) -> Option<MutationResult> {
        None
    }
}

impl<GS: GameState, C: GameCommand<GS>> CommandReceiver<GS> for mpsc::Receiver<C> {
    fn try_apply_command(&self, gs: &mut GS) -> Option<MutationResult> {
        let cmd = self.try_recv().ok()?;

        Some(cmd.apply_command(gs))
    }
}

#[impl_for_tuples(1, 8)]
impl<GS: GameState> CommandReceiver<GS> for Tuple {
    for_tuples!( where #( Tuple: CommandReceiver<GS> )* );

    fn try_apply_command(&self, gs: &mut GS) -> Option<MutationResult> {
        for_tuples!( #(if let Some(r) = self.Tuple.try_apply_command(gs){return Some(r)};) *);

        return None;
    }
}
