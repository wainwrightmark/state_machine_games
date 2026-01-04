use crate::prelude::*;

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

#[cfg(feature = "leptos")]
impl<GS: GameState, S: EntityStoreCombination<GS>> EntityStoreCombination<GS> for leptos::prelude::ArcRwSignal<S> {
    fn gather_entities(&mut self, state: &GS) -> bool {
        use leptos::prelude::Update;

        self.try_maybe_update(|x| {
            let changed = x.gather_entities(state);
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

    fn new(state: &GS) -> Self {
        use leptos::prelude::ArcRwSignal;

        ArcRwSignal::new(S::new(state))
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

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use crate::prelude::*;

    #[derive(Debug, PartialEq, Clone)]
    struct MyGameState(Vec<u32>);

    impl GameState for MyGameState {
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
        fn apply_command(&self, game_state: &mut MyGameState) -> MutationResult {
            game_state.0 = self.0.clone();
            MutationResult::CHANGED_NO_TRANSITION
        }
    }

    impl GameArtifact for MyArtifact {
        type Command = MyCommand;
    }

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
            (MyArtifact(self.1.clone()), AnimationList::new())
        }

        fn on_update(
            &self,
            artifact: &mut Self::Artifact,
            _former_entity_state: crate::prelude::EntityState,
            _previous_animations: AnimationList<Self::Artifact>
        ) -> crate::prelude::AnimationList<Self::Artifact> {
            artifact.0 = self.1.clone();
            AnimationList::new()
        }
    }

    #[test]
    pub fn test_game_machine() {
        let state = MyGameState(vec![1, 2, 3]);
        let store: SingleTypeEntityStore<MyEntity> = SingleTypeEntityStore::new(&state);
        let (sender, receiver) = mpsc::channel::<MyCommand>();

        let mut machine = GameMachine::new(state, store, receiver);

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

        assert_entities(&machine.stores, "1,2,3");

        machine.step_game(1000.0);
        assert_entities(&machine.stores, "1,2,3");

        sender.send(MyCommand(vec![2, 3, 4])).unwrap();
        assert_entities(&machine.stores, "1,2,3");

        machine.step_game(1000.0);
        assert_entities(&machine.stores, "2,3,4");

        sender.send(MyCommand(vec![4, 2, 5])).unwrap();

        machine.step_game(1000.0);
        assert_entities(&machine.stores, "2,4,5");
    }
}
