use std::sync::Mutex;

use crate::prelude::*;
use leptos::prelude::*;

pub fn run_game<
    GS: GameState,
    Stores: EntityStoreCombination<GS>,
    Receivers: CommandReceiver<GS>,
>() {
}

pub struct GameMachine<
    GS: GameState,
    Stores: EntityStoreCombination<GS>,
    Receivers: CommandReceiver<GS>,
> {
    state: GS,
    stores: Stores,
    receivers: Receivers,
    ms_until_transition: Option<f64>,
}

impl<GS: GameState, S: EntityStoreCombination<GS>> EntityStoreCombination<GS> for ArcRwSignal<S> {
    fn gather_entities(&mut self, state: &GS) -> bool {
        self.try_maybe_update(|x| {
            let changed = x.gather_entities(state);
            (changed, changed)
        })
        .unwrap_or_default()
    }

    fn step_animations(&mut self, delta_ms: f64) {
        self.update_untracked(|x| {
            x.step_animations(delta_ms);
        })
    }
}

impl<GS: GameState, Stores: EntityStoreCombination<GS>, Receivers: CommandReceiver<GS>>
    GameMachine<GS, Stores, Receivers>
{
    pub fn new(state: GS, mut stores: Stores, receivers: Receivers) -> Self {
        stores.gather_entities(&state);

        Self {
            state,
            stores,
            receivers,
            ms_until_transition: Some(0.0),
        }
    }

    pub fn run_game(self) {
        let mutex = Mutex::new(self);
        leptos_use::use_raf_fn(move |args| match mutex.lock() {
            Ok(mut machine) => {
                machine.step_game(args.delta);
            }
            Err(err) => panic!("{err}"),
        });
    }

    pub fn step_game(&mut self, delta_ms: f64) {
        let mut remaining_ms = delta_ms;

        while let Some(transition_ms) = self.ms_until_transition
            && transition_ms <= remaining_ms
        {
            //run animations up to the transition
            self.stores.step_animations(transition_ms);

            remaining_ms -= transition_ms;

            let mr = self.state.maybe_transition();

            self.ms_until_transition = mr.transition_callback_in_ms;

            if mr.changed {
                self.stores.gather_entities(&self.state);
            }
        }

        match &mut self.ms_until_transition {
            Some(x) => *x -= remaining_ms,
            None => {}
        };

        //run animations
        if remaining_ms > 0.0 {
            self.stores.step_animations(remaining_ms);
        }

        let mut entities_changed_by_commands = false;

        while let Some(mr) = self.receivers.try_apply_command(&mut self.state) {
            entities_changed_by_commands |= mr.changed;

            match (self.ms_until_transition, mr.transition_callback_in_ms) {
                (_, None) => {}
                (None, Some(_)) => self.ms_until_transition = mr.transition_callback_in_ms,
                (Some(a), Some(b)) => self.ms_until_transition = Some(a.min(b)),
            }
        }

        if entities_changed_by_commands {
            self.stores.gather_entities(&self.state);
        }
    }
}
