use std::time::Duration;

use crate::game_state::GameState;

pub trait GameEntity: PartialEq + Send + Sync + 'static {
    type GameState: GameState<Entity = Self>;

    fn key(&self) -> <<Self as GameEntity>::GameState as GameState>::EntityKey;

    fn death_duration(&self) -> Option<Duration>;
}
