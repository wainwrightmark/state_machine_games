// use std::sync::mpsc;

// use impl_trait_for_tuples::impl_for_tuples;

// use crate::prelude::*;

// pub trait CommandReceiver<GS: GameState>: 'static {
//     fn try_apply_command(&self, gs: &mut GS) -> Option<MutationResult>;
// }

// impl<GS: GameState> CommandReceiver<GS> for () {
//     fn try_apply_command(&self, _gs: &mut GS) -> Option<MutationResult> {
//         None
//     }
// }

// impl<GS: GameState> CommandReceiver<GS> for mpsc::Receiver<GS::Command> {
//     fn try_apply_command(&self, gs: &mut GS) -> Option<MutationResult> {
//         let cmd = self.try_recv().ok()?;

//         Some(gs.apply_command(cmd))
//     }
// }