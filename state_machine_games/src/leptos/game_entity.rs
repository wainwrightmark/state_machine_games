use crate::{
    game_entity::GameEntity, game_state::GameState, leptos::command_sender::CommandSender,
    prelude::GameArtifact,
};
use leptos::prelude::*;

pub trait LeptosArtifact<GS: GameState>: GameArtifact {
    fn render(&self, sender: CommandSender<GS>) -> AnyView;
}

pub trait LeptosGameState:
    GameState<Entity: GameEntity<GameState = Self, Artifact: LeptosArtifact<Self>>>
{
}
