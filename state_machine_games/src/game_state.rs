use std::{fmt::Debug, hash::Hash, time::Duration};

use crate::game_entity::GameEntity;

pub trait GameState: Send + Sync + Sized + 'static {
    type Settings: GameSettings;
    type Assets: GameAssets;
    type Storage: GameStorage;
    type Command: GameCommand;
    type InputState: GameInputState;
    type Entity: GameEntity;    

    fn get_entities(
        &self,
        settings: &Self::Settings,
        assets: &Self::Assets,
        storage: &Self::Storage,
    ) -> impl Iterator<Item = Self::Entity>;

    fn apply_command(
        &mut self,
        command: Self::Command,
        settings: &Self::Settings,
        assets: &Self::Assets,
        storage: &Self::Storage,
    ) -> MutationResult;

    fn maybe_transition(
        &mut self,
        settings: &Self::Settings,
        assets: &Self::Assets,
        storage: &Self::Storage,
    ) -> MutationResult;

    ///Runs all queued transitions. Returns whether anything changed
    fn fast_forward_transitions(
        &mut self,
        settings: &Self::Settings,
        assets: &Self::Assets,
        storage: &Self::Storage,
    ) -> bool {
        let mut r = self.maybe_transition(settings, assets, storage);
        let mut changed = r.changed;
        while r.transition_callback_in.is_some() {
            r = self.maybe_transition(settings, assets, storage);
            changed |= r.changed;
        }

        changed
    }
}

pub trait GameAssets: Send + Sync + Sized + 'static {}

impl GameAssets for () {}

pub trait GameStorage: Send + Sync + Sized + 'static {}

impl GameStorage for () {}

pub trait GameSettings: Send + Sync + Sized + 'static {}

impl GameSettings for () {}

pub trait GameInputState: Send + Sync + Sized + Default + 'static {}
impl GameInputState for () {}

pub trait GameCommand: Send + Sync + Sized + 'static {}

pub trait GameEntityKey:
    Send + Sync + 'static + PartialEq + Clone + Copy + PartialOrd + Debug + Eq + Ord + Hash
{
}

impl GameEntityKey for u8 {}
impl GameEntityKey for u16 {}
impl GameEntityKey for u32 {}
impl GameEntityKey for u64 {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MutationResult {
    /// Whether anything changed
    pub changed: bool,
    ///How long to wait before calling back for another transition
    pub transition_callback_in: Option<Duration>,
}

impl MutationResult {
    pub const NO_CHANGE: Self = Self {
        changed: false,
        transition_callback_in: None,
    };

    pub const CHANGED_NO_TRANSITION: Self = Self {
        changed: true,
        transition_callback_in: None,
    };
}
