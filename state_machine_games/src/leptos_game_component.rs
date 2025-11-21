use crate::prelude::*;
use leptos::{logging::log, prelude::*};

pub fn leptos_game_component<GS: GameState>(
    viewbox_width: f32,
    viewbox_height: f32,
    initial_state: GS,
) -> impl IntoView {
    let game_machine = GameMachine::new(initial_state);
    let store = game_machine.store.clone();
    let command_sender = game_machine.command_sender.clone();

    leptos_use::use_raf_fn(move |args| {
        game_machine.game_loop(args.delta);
    });

    // game_machine.store.read_untracked().

    view! {
        <svg viewBox=format!("0 0 {viewbox_width} {viewbox_height}")  style="max-width: 800px;  margin-inline: auto; ">
        {move || EntityStore::render(store.clone(), command_sender.clone())}
        </svg>
    }
}

struct GameMachine<GS: GameState> {
    state: ArcRwSignal<GS>,
    store: ArcRwSignal<EntityStore<GS::Command, GS::Key>>,
    command_sender: BasicCommandSender<GS::Command>,
    ms_until_transition: ArcRwSignal<Option<f64>>,
}

impl<GS: GameState> GameMachine<GS> {
    pub fn new(state: GS) -> Self {
        let mut store = EntityStore::new();

        let mut receiver = GeneralEntityReceiver::new(&mut store);

        state.entities(&mut receiver);

        receiver.finish();

        Self {
            state: ArcRwSignal::new(state),
            store: ArcRwSignal::new(store),
            command_sender: BasicCommandSender::new(),
            ms_until_transition: ArcRwSignal::new(Some(0.0)),
        }
    }

    fn update_entities(&self) {
        self.store.update_untracked(|mut store| {
            let mut receiver = GeneralEntityReceiver::new(&mut store);
            let gs = self.state.read_untracked();
            gs.entities(&mut receiver);
            receiver.finish();
        });
    }

    fn step_animations(&self, delta_ms: f64) {
        self.store.update_untracked(|store| {
            store.animate_step(delta_ms);
        })
    }

    pub fn game_loop(&self, delta_ms: f64) {
        let mut remaining_ms = delta_ms;

        while let Some(transition_ms) = self.ms_until_transition.get()
            && transition_ms <= remaining_ms
        {
            //run animations up to the transition
            self.step_animations(transition_ms);

            remaining_ms -= transition_ms;
            let changed: bool;
            let new_ms: Option<f64>;

            match self.state.try_update(|s| s.maybe_transition()) {
                Some(mutation_result) => {
                    changed = mutation_result.changed;
                    new_ms = mutation_result.transition_callback_in_ms;
                }
                None => {
                    changed = false;
                    new_ms = None;
                }
            }
            self.ms_until_transition.set(new_ms);
            
            if changed {
                self.update_entities();
            }
        }

        self.ms_until_transition.update(|x|{
            match x.as_mut(){
                Some( x) => *x -= remaining_ms,
                None => {},
            }
        });

        //run animations
        if remaining_ms > 0.0 {
            self.step_animations(remaining_ms);
        }

        let mut entities_changed_by_commands = false;
        //run commands
        match self.command_sender.queue.lock() {
            Ok(mut queue) => {
                while let Some(command) = queue.pop_front() {
                    //log!("Running Command {command:?}");
                    match self.state.try_update(|state| state.apply_command(command)) {
                        Some(r) => {
                            entities_changed_by_commands |= r.changed;

                            self.ms_until_transition.set(
                                [self.ms_until_transition.get(), r.transition_callback_in_ms]
                                    .into_iter()
                                    .flatten()
                                    .min_by(|a, b| a.total_cmp(b)),
                            );
                        }
                        None => {}
                    }
                }
            }
            Err(err) => panic!("{err}"),
        }

        if entities_changed_by_commands {            
            self.update_entities();
        }

        //update entities
    }
}
