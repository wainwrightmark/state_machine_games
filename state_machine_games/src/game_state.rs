use crate::{
    delayed_effect::{DelayedEffectStore, MutationResult},
    prelude::*,
};
use leptos::prelude::Update;
use std::sync::{Arc, Mutex, mpsc};

pub trait GameState: Send + Sync + 'static + Sized {}

pub fn run_game<GS: GameState>(
    state: impl Clone + Update<Value = GS> + 'static,
    initial_mutations: impl Iterator<Item = DelayedMutation<GS>>
) -> mpsc::Sender<Box<dyn GameCommand<GS>>> {
    let (sender, receiver) = mpsc::channel::<Box<dyn GameCommand<GS>>>();

    let mut delayed_effect_store = DelayedEffectStore::new();
    for dm in initial_mutations{
        delayed_effect_store.push_delayed_mutation(dm);
    }

    let delayed_effect_store: Arc<Mutex<DelayedEffectStore<GS>>> =
        Arc::new(Mutex::new(delayed_effect_store));

    //let ms_until_transition: Arc<Mutex<Option<f64>>> = Arc::new(Mutex::new(Some(0.0)));
    let receiver = Arc::new(receiver);

    leptos_use::use_raf_fn(move |args| {
        step_game(
            state.clone(),
            receiver.clone(),
            delayed_effect_store.clone(),
            args.delta,
        )
    });

    sender
}

fn step_game<GS: GameState>(
    gs: impl Update<Value = GS>,
    receiver: Arc<mpsc::Receiver<Box<dyn GameCommand<GS>>>>,
    delayed_effect_store: Arc<Mutex<DelayedEffectStore<GS>>>,

    delta_ms: f64,
) {
    let mut delayed_effect_store = delayed_effect_store
        .lock()
        .expect("Could not get delayed effect store");

    gs.maybe_update(|gs| delayed_effect_store.tick(gs, delta_ms));

    while let Some(cmd) = receiver.try_recv().ok() {
        //leptos::logging::log!("Found command {cmd:?}");
        gs.maybe_update(|gs| match cmd.apply_command(gs) {
            MutationResult::NoChange => false,
            MutationResult::Changed(maybe_delayed_effect) => {
                if let Some(dm) = maybe_delayed_effect {
                    delayed_effect_store.push_delayed_mutation(dm);
                }

                true
            }
        });
    }
}

// #[cfg(test)]
// mod tests {
//     use std::{ops::Deref, sync::RwLock};

//     use crate::prelude::*;

//     #[derive(Debug, PartialEq, Clone)]
//     struct MyGameState(Vec<u32>);

//     impl GameState for MyGameState {
//         type Command = MyCommand;
//         fn maybe_transition(&mut self) -> crate::prelude::MutationResult {
//             MutationResult::NO_CHANGE
//         }
//     }

//     #[derive(Debug, PartialEq)]
//     struct MyEntity(u32, String);

//     #[derive(Debug, Clone)]
//     struct MyArtifact(String);

//     #[derive(Debug, Clone)]
//     struct MyCommand(Vec<u32>);

//     impl GameCommand<MyGameState> for MyCommand {
//         fn apply_command(&self, state: &mut MyGameState) -> MutationResult {
//             state.0 = self.0.clone();
//             MutationResult::CHANGED_NO_TRANSITION
//         }
//     }

//     impl GameArtifact for MyArtifact {}

//     impl GameEntity for MyEntity {
//         type Artifact = MyArtifact;
//         type Key = u32;
//         type State = MyGameState;
//         fn key(&self) -> Self::Key {
//             self.0
//         }

//         fn get_entities(game_state: &Self::State) -> impl Iterator<Item = Self> {
//             game_state.0.iter().copied().map(|x| Self(x, x.to_string()))
//         }

//         fn on_new(
//             &self,
//         ) -> (
//             Self::Artifact,
//             crate::prelude::AnimationList<Self::Artifact>,
//         ) {
//             (MyArtifact(self.1.clone()), AnimationList::EMPTY)
//         }

//         fn on_update(
//             &self,
//             artifact: &mut Self::Artifact,
//             _former_entity_state: crate::prelude::EntityState,
//             _previous_animations: AnimationList<Self::Artifact>,
//         ) -> crate::prelude::AnimationList<Self::Artifact> {
//             artifact.0 = self.1.clone();
//             AnimationList::EMPTY
//         }
//     }

//     #[test]
//     pub fn test_game_machine() {
//         let state = MyGameState(vec![1, 2, 3]);
//         let store: std::sync::Arc<RwLock<SingleTypeEntityStore<MyEntity>>> =
//             InitFromGameState::init(&state);

//         let mut machine = GameMachine::new(state);
//         machine.add_change_watcher(store.clone());

//         let sender = machine.sender.clone();

//         fn assert_entities(store: &SingleTypeEntityStore<MyEntity>, expected: &str) {
//             let mut actual = String::new();
//             for (index, x) in store
//                 .entities
//                 .iter()
//                 .map(|x| x.artifact.0.as_str())
//                 .enumerate()
//             {
//                 if index > 0 {
//                     actual.push(',');
//                 }
//                 actual += x;
//             }

//             assert_eq!(actual, expected)
//         }

//         assert_entities(store.read().unwrap().deref(), "1,2,3");

//         machine.step_game(1000.0);
//         assert_entities(store.read().unwrap().deref(), "1,2,3");

//         sender.send(MyCommand(vec![2, 3, 4])).unwrap();
//         assert_entities(store.read().unwrap().deref(), "1,2,3");

//         machine.step_game(1000.0);
//         assert_entities(store.read().unwrap().deref(), "2,3,4");

//         sender.send(MyCommand(vec![4, 2, 5])).unwrap();

//         machine.step_game(1000.0);
//         assert_entities(store.read().unwrap().deref(), "2,4,5");
//     }
// }
