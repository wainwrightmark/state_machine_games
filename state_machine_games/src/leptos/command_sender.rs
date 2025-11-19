use std::time::Duration;

use leptos::prelude::*;

use crate::game_state::GameState;

#[derive(Debug)]
pub struct CommandSender<T: GameState> {
    game_state_read: ReadSignal<T>,
    game_state_write: WriteSignal<T>,
    input_state: ArcRwSignal<T::InputState>,
    settings: ReadSignal<T::Settings>,
    assets: ReadSignal<T::Assets>,
    storage: ReadSignal<T::Storage>,
    transition_callback_at: ArcRwSignal<Option<web_time::Instant>>,
}

impl<T: GameState> Clone for CommandSender<T> {
    fn clone(&self) -> Self {
        Self {
            game_state_read: self.game_state_read.clone(),
            game_state_write: self.game_state_write.clone(),
            input_state: self.input_state.clone(),
            settings: self.settings.clone(),
            assets: self.assets.clone(),
            storage: self.storage.clone(),
            transition_callback_at: self.transition_callback_at.clone(),
        }
    }
}

impl<T: GameState> CommandSender<T> {
    pub fn new(
        game_state_read: ReadSignal<T>,
        game_state_write: WriteSignal<T>,
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
            game_state_read,
            game_state_write,
            input_state: Default::default(),
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

        match self.game_state_write.try_maybe_update(|x| {
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

    // pub fn handle_event(&self, t: PointerEventType, event: PointerEvent, node_ref: NodeRef<Svg>){

    //     if let Some(event) = PointerInputEvent::new(t, event, node_ref){
    //         //log!("Handle Event {event:?}");

    //         Self::handle_input(&self, event);
    //     }
    // }

    // fn handle_input(&self, event: PointerInputEvent) {

    //     let mut command: Option<T::Command> = None;

    //     self.input_state.update(|input_state| {
    //         let state = self.game_state_read.read_untracked();
    //         let settings = self.settings.read_untracked();
    //         let assets = self.assets.read_untracked();
    //         let storage = self.storage.read_untracked();

    //         command = state.handle_pointer_event(input_state, event, &settings, &assets, &storage)
    //     });

    //     if let Some(command) = command {
    //         self.send_command(command);
    //     }
    // }

    pub fn handle_game_input_event(&self, event: impl GameInputEvent<T>) {
        let mut command: Option<T::Command> = None;
        self.input_state.update(|input_state| {
            let game_state: &T = &self.game_state_read.read_untracked();
            let settings: &T::Settings = &self.settings.read_untracked();
            let assets: &T::Assets = &self.assets.read_untracked();
            let storage: &T::Storage = &self.storage.read_untracked();

            command = event.handle_event(input_state, game_state, settings, assets, storage);
        });

        if let Some(command) = command {
            self.send_command(command);
        }
    }

    pub fn send_command(&self, command: T::Command) {
        //log::info!("Action Sent");

        let settings = self.settings.read_untracked();
        let assets = self.assets.read_untracked();
        let storage = self.storage.read_untracked();

        match self.game_state_write.try_maybe_update(|x| {
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

pub trait GameInputEvent<T: GameState> {
    fn handle_event(
        &self,
        input_state: &mut T::InputState,
        game_state: &T,
        settings: &T::Settings,
        assets: &T::Assets,
        storage: &T::Storage,
    ) -> Option<T::Command>;
}
