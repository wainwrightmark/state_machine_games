use std::time::Duration;

use leptos::prelude::*;

use crate::game_state::GameState;

#[derive(Debug)]
pub struct CommandSender<T: GameState> {
    inner: WriteSignal<T>,
    settings: ReadSignal<T::Settings>,
    assets: ReadSignal<T::Assets>,
    storage: ReadSignal<T::Storage>,
    transition_callback_at: ArcRwSignal<Option<web_time::Instant>>,
}

//impl<T: GameState> Copy for CommandSender<T> {}

impl<T: GameState> Clone for CommandSender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            settings: self.settings.clone(),
            assets: self.assets.clone(),
            storage: self.storage.clone(),
            transition_callback_at: self.transition_callback_at.clone(),
        }
    }
}

impl<T: GameState> CommandSender<T> {
    pub fn new(
        inner: WriteSignal<T>,
        settings: ReadSignal<T::Settings>,
        assets: ReadSignal<T::Assets>,
        storage: ReadSignal<T::Storage>,
        first_transition_callback: Option<Duration>,
    ) -> Self {
        let (duration, transition_callback_at) = match first_transition_callback {
            Some(duration) => {
                let now = web_time::Instant::now();

                (Some(duration), ArcRwSignal::new(Some(now + duration)))
            }
            None => (None, ArcRwSignal::new(None)),
        };

        let result = Self {
            inner,
            settings,
            assets,
            storage,
            transition_callback_at,
        };
        if let Some(duration) = duration {
            result.schedule_transition(duration);
        }

        result
    }

    fn maybe_transition_state(&self) {
        let settings = self.settings.read_untracked();
        let assets = self.assets.read_untracked();
        let storage = self.storage.read_untracked();

        let now = web_time::Instant::now();

        match self.transition_callback_at.get() {
            Some(callback_instant) => {
                if now < callback_instant {
                    return;
                }
            }
            None => {
                return;
            }
        }

        match self.inner.try_maybe_update(|x| {
            let transition_result = x.maybe_transition(&settings, &assets, &storage);

            (
                transition_result.changed,
                transition_result.transition_callback_in,
            )
        }) {
            Some(Some(dur)) => {
                let callback_at = now + dur;

                self.transition_callback_at.set(Some(callback_at));
                self.schedule_transition(dur);
            }
            Some(None) => self.transition_callback_at.set(None),
            None => self.transition_callback_at.set(None),
        }
    }

    fn schedule_transition(&self, duration: Duration) {        
        let s = self.clone();
        set_timeout(
            move || {                
                s.maybe_transition_state();
            },
            duration,
        );
    }

    pub fn send_command(&self, command: T::Command) {
        //log::info!("Action Sent");

        let settings = self.settings.read_untracked();
        let assets = self.assets.read_untracked();
        let storage = self.storage.read_untracked();

        match self.inner.try_maybe_update(|x| {
            let transition_result = x.apply_command(command, &settings, &assets, &storage);
            (
                transition_result.changed,
                transition_result.transition_callback_in,
            )
        }) {
            Some(Some(dur)) => {
                let now = web_time::Instant::now();
                let callback_at = now + dur;

                self.transition_callback_at.set(Some(callback_at));
                self.schedule_transition(dur);
            }
            Some(None) => self.transition_callback_at.set(None),
            None => self.transition_callback_at.set(None),
        }
    }
}
