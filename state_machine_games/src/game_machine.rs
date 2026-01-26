use std::sync::mpsc;

use crate::prelude::*;

pub fn run_game<GS: GameState, Stores: ChangeWatcher<GS>>() {}

pub struct GameMachine<GS: GameState> {
    state: GS,
    change_watchers: Vec<Box<dyn ChangeWatcher<GS>>>,
    ms_until_transition: Option<f64>,
    sender: mpsc::Sender<GS::Command>,
    receiver: mpsc::Receiver<GS::Command>,
}

#[cfg(feature = "leptos")]
impl<GS: GameState, S: ChangeWatcher<GS>> ChangeWatcher<GS> for leptos::prelude::ArcRwSignal<S> {
    fn on_state_change(&mut self, state: &GS, reason: &StateChangeReason<GS>) -> bool {
        use leptos::prelude::Update;

        self.try_maybe_update(|x| {
            let changed = x.on_state_change(state, reason);
            (changed, changed)
        })
        .unwrap_or_default()
    }

    fn step_animations(&mut self, delta_ms: f64) {
        use leptos::prelude::UpdateUntracked;

        self.update_untracked(|x| {
            x.step_animations(delta_ms);
        })
    }
}

#[cfg(feature = "leptos")]
impl<GS: GameState, S: InitFromGameState<GS>> InitFromGameState<GS>
    for leptos::prelude::ArcRwSignal<S>
{
    fn init(state: &GS) -> Self {
        use leptos::prelude::ArcRwSignal;

        ArcRwSignal::new(S::init(state))
    }
}

impl<GS: GameState> GameMachine<GS> {
    pub fn new(state: GS) -> Self {
        let (sender, receiver) = mpsc::channel();

        Self {
            state,
            change_watchers: vec![],
            ms_until_transition: Some(0.0),
            sender,
            receiver,
        }
    }

    pub fn init_change_watcher<S: ChangeWatcher<GS> + InitFromGameState<GS>>(&mut self) {
        let s = S::init(&self.state);
        self.add_change_watcher(s);
    }

    pub fn add_change_watcher<S: ChangeWatcher<GS>>(&mut self, mut s: S) {
        s.on_state_change(&self.state, &StateChangeReason::InitialState);
        self.change_watchers.push(Box::new(s));
    }

    #[cfg(feature = "leptos")]
    pub fn run_game(self) {
        let mutex = std::sync::Mutex::new(self);
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
            for x in self.change_watchers.iter_mut(){
                x.step_animations(transition_ms);
            }            

            remaining_ms -= transition_ms;

            let mr = self.state.maybe_transition();

            self.ms_until_transition = mr.transition_callback_in_ms;

            if mr.changed {
                for x in self.change_watchers.iter_mut(){
                    x.on_state_change(&self.state, &StateChangeReason::Transition);
                }
                    
            }
        }

        match &mut self.ms_until_transition {
            Some(x) => *x -= remaining_ms,
            None => {}
        };

        //run animations
        if remaining_ms > 0.0 {
            for x in self.change_watchers.iter_mut(){
                x.step_animations(remaining_ms);
            }            
        }

        while let Some((cmd, mr)) = self.receiver.try_recv().ok().map(|cmd| {
            let mr = cmd.apply_command(&mut self.state);
            (cmd, mr)
        }) {
            if mr.changed {
                let reason = StateChangeReason::Command(cmd);
                for x in self.change_watchers.iter_mut(){
                    x.on_state_change(&self.state, &reason);
                }                    
            }

            match (self.ms_until_transition, mr.transition_callback_in_ms) {
                (_, None) => {}
                (None, Some(_)) => self.ms_until_transition = mr.transition_callback_in_ms,
                (Some(a), Some(b)) => self.ms_until_transition = Some(a.min(b)),
            }
        }
    }

    pub fn command_sender(&self) -> mpsc::Sender<GS::Command> {
        self.sender.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::{ops::Deref, sync::RwLock};

    use crate::prelude::*;

    #[derive(Debug, PartialEq, Clone)]
    struct MyGameState(Vec<u32>);

    impl GameState for MyGameState {
        type Command = MyCommand;
        fn maybe_transition(&mut self) -> crate::prelude::MutationResult {
            MutationResult::NO_CHANGE
        }
    }

    #[derive(Debug, PartialEq)]
    struct MyEntity(u32, String);

    #[derive(Debug, Clone)]
    struct MyArtifact(String);

    #[derive(Debug, Clone)]
    struct MyCommand(Vec<u32>);

    impl GameCommand<MyGameState> for MyCommand {
        fn apply_command(&self, state: &mut MyGameState) -> MutationResult {
            state.0 = self.0.clone();
            MutationResult::CHANGED_NO_TRANSITION
        }
    }

    impl GameArtifact for MyArtifact {}

    impl GameEntity for MyEntity {
        type Artifact = MyArtifact;
        type Key = u32;
        type StateSegment = MyGameState;
        fn key(&self) -> Self::Key {
            self.0
        }

        fn get_entities(game_state: &Self::StateSegment) -> impl Iterator<Item = Self> {
            game_state.0.iter().copied().map(|x| Self(x, x.to_string()))
        }

        fn on_new(
            &self,
        ) -> (
            Self::Artifact,
            crate::prelude::AnimationList<Self::Artifact>,
        ) {
            (MyArtifact(self.1.clone()), AnimationList::EMPTY)
        }

        fn on_update(
            &self,
            artifact: &mut Self::Artifact,
            _former_entity_state: crate::prelude::EntityState,
            _previous_animations: AnimationList<Self::Artifact>,
        ) -> crate::prelude::AnimationList<Self::Artifact> {
            artifact.0 = self.1.clone();
            AnimationList::EMPTY
        }
    }

    #[test]
    pub fn test_game_machine() {
        let state = MyGameState(vec![1, 2, 3]);
        let store: std::sync::Arc<RwLock<SingleTypeEntityStore<MyEntity>>> = InitFromGameState::init(&state);
        

        let mut machine = GameMachine::new(state);
        machine.add_change_watcher(store.clone());

        let sender = machine.sender.clone();

        fn assert_entities(store: &SingleTypeEntityStore<MyEntity>, expected: &str) {
            let mut actual = String::new();
            for (index, x) in store
                .entities
                .iter()
                .map(|x| x.artifact.0.as_str())
                .enumerate()
            {
                if index > 0 {
                    actual.push(',');
                }
                actual += x;
            }

            assert_eq!(actual, expected)
        }

        

        assert_entities(store.read().unwrap().deref(), "1,2,3");

        machine.step_game(1000.0);
        assert_entities(store.read().unwrap().deref(), "1,2,3");

        sender.send(MyCommand(vec![2, 3, 4])).unwrap();
        assert_entities(store.read().unwrap().deref(), "1,2,3");

        machine.step_game(1000.0);
        assert_entities(store.read().unwrap().deref(), "2,3,4");

        sender.send(MyCommand(vec![4, 2, 5])).unwrap();

        machine.step_game(1000.0);
        assert_entities(store.read().unwrap().deref(), "2,4,5");
    }
}
