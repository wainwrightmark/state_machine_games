pub trait GameCommand: Send + 'static {}

// pub trait GameCommand<GS> : Send + 'static {
//     fn apply_command(&self, game_state: &mut GS) -> MutationResult;
// }

// impl<GS: GameState> GameCommand<GS> for () {
//     fn apply_command(&self, _game_state: &mut GS) -> MutationResult {
//         MutationResult::NO_CHANGE
//     }
// }
