use std::marker::PhantomData;

use leptos::prelude::{RwSignal, Update};

use crate::{
    lens::GetValueLens,
    prelude::{ChangeWatcher, GameState, InitFromGameState},
};

#[derive(Debug)]
pub struct ValueWatcher<L: GetValueLens>
where
    L::Object: GameState,
    L::Value: Send + Sync + 'static,
{
    pub value_signal: RwSignal<L::Value>,
    phantom: PhantomData<L>,
}

impl<L: GetValueLens> InitFromGameState<L::Object> for ValueWatcher<L>
where
    L::Object: GameState,
    L::Value: PartialEq + Send + Sync + 'static,
{
    fn init(state: &L::Object) -> Self {
        let value_signal = RwSignal::new(L::get_value(state));
        Self {
            value_signal,
            phantom: PhantomData,
        }
    }
}

impl<L: GetValueLens> ChangeWatcher<L::Object> for ValueWatcher<L>
where
    L::Object: GameState,
    L::Value: PartialEq + Send + Sync + 'static,
{
    fn on_state_change(
        &mut self,
        state: &L::Object,
        _reason: &crate::prelude::StateChangeReason<L::Object>,
    ) -> bool {
        self.value_signal
            .try_update(|v| {
                let new_v: L::Value = L::get_value(state);
                if new_v.eq(v) {
                    false
                } else {
                    *v = new_v;
                    true
                }
            })
            .unwrap_or_default()
    }

    fn step_animations(&mut self, _delta_ms: f64) {}
}
