use leptos::prelude::{ArcRwSignal, Update};

use crate::prelude::*;

pub type LeptosEntityStore<T : GameEntity> = EntityStore<T, ArcRwSignal<T::Artifact>>;

impl<T: Sized + 'static> ValueSignal for ArcRwSignal<T> {
    type Value = T;

    fn new(value: Self::Value) -> Self {
        Self::new(value)
    }

    fn try_maybe_update<U>(&self, fun: impl FnOnce(&mut Self::Value) -> (bool, U)) -> Option<U> {
        <Self as Update>::try_maybe_update(self, fun)
    }
}
