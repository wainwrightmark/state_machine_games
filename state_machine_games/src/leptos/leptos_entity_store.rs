use leptos::prelude::{ArcRwSignal, Update};

use crate::entity_store::{EntityStore, StoredEntityMeta, ValueSignal};

pub type LeptosEntityStore<T> = EntityStore<T, ArcRwSignal<(T, StoredEntityMeta<T>)>>;

impl<T: Sized + 'static> ValueSignal for ArcRwSignal<T> {
    type Value = T;

    fn new(value: Self::Value) -> Self {
        Self::new(value)
    }

    fn try_maybe_update<U>(&self, fun: impl FnOnce(&mut Self::Value) -> (bool, U)) -> Option<U> {
        <Self as Update>::try_maybe_update(self, fun)
    }
}
