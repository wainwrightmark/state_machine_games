use crate::{
    entity_store::StoredEntityMeta, game_entity::GameEntity, game_state::GameState,
    leptos::{command_sender::CommandSender, prelude::Timestamp},
};
use leptos::prelude::*;

/// An entity which can be rendered in Leptos
pub trait LeptosGameEntity: GameEntity + PartialEq + Sized {
    fn render(
        &self,
        meta: &StoredEntityMeta<Self>,
        sender: CommandSender<Self::GameState>,
        start_time: f64,
        current_time: Signal<f64>,
    ) -> AnyView;
}

pub trait LeptosGameState: GameState<Entity: LeptosGameEntity<GameState = Self>> {}
