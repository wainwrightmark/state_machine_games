use std::sync::mpsc;

use leptos::prelude::*;
use state_machine_games::prelude::*;
type Stores = (
    ArcRwSignal<SingleTypeEntityStore<SquareButton>>,
    ArcRwSignal<SingleTypeEntityStore<Circle>>,
);

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    mount_to_body(|| game_component());
}

fn game_component() -> impl IntoView {
    let state = CounterGameState { n: 2 };

    let (sender, receiver) = mpsc::channel::<CounterCommand>();
    let stores = Stores::default();
    let machine = GameMachine::new(state, stores.clone(), receiver);

    machine.run_game();

    view! {
        <svg viewBox="0 0 800.0 800.0"  style="max-width: 800px;  margin-inline: auto; ">
        {move || SingleTypeEntityStore::render(stores.0.clone(), sender.clone())}
        {move || SingleTypeEntityStore::render(stores.1.clone(), ())}
        </svg>
    }
}

pub struct CounterGameState {
    pub n: usize,
}

impl GameState for CounterGameState {
    fn maybe_transition(&mut self) -> MutationResult {
        MutationResult::NO_CHANGE
    }
}

#[derive(Debug, Clone)]
pub enum CounterCommand {
    IncrementCount(usize),
}
impl AnyGameCommand for CounterCommand {}

impl GameCommand<CounterGameState> for CounterCommand {
    fn apply_command(&self, games_state: &mut CounterGameState) -> MutationResult {
        match self {
            CounterCommand::IncrementCount(n) => {
                games_state.n += n;
                MutationResult::CHANGED_NO_TRANSITION
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key {
    SquareButton,
    Circle(u8),
}

impl GameEntityKey for Key {}

#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    pub inner: u8,
}

impl GameEntity for Circle {
    type GameState = CounterGameState;
    type Artifact = CircleArtifact;
    type Key = Key;

    fn get_entities(game_state: &Self::GameState, receiver: &mut impl EntityReceiver<Self>) {
        receiver.receive((0..game_state.n).map(|n| Circle { inner: n as u8 }));
    }

    fn key(&self) -> Self::Key {
        Key::Circle(self.inner)
    }

    fn on_death(&self, _artifact: &mut Self::Artifact) -> AnimationList<Self::Artifact> {
        vec![]
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        (
            CircleArtifact {
                size: 10.0,
                x: 20.0 + (30.0 * self.inner as f32),
                y: 10.0,
            },
            vec![],
        )
    }

    fn on_update(
        &self,
        _artifact: &mut Self::Artifact,
        _former_entity_state: EntityState,
    ) -> AnimationList<Self::Artifact> {
        vec![]
    }
}

#[derive(Debug, Clone)]
pub struct CircleArtifact {
    pub x: f32,
    pub y: f32,
    pub size: f32,
}

impl GameArtifact for CircleArtifact {
    type Command = ();

    fn render(self, _sender: impl CommandSender<Self::Command>) -> impl IntoView {
        view! {<circle cx=self.x cy=self.y r=self.size fill="#1111EE" />}
    }
}

#[derive(Debug, PartialEq)]
pub struct SquareButton {
    pub n: usize,
}

impl GameEntity for SquareButton {
    type GameState = CounterGameState;
    type Artifact = SquareButtonArtifact;
    type Key = Key;

    fn key(&self) -> Self::Key {
        Key::SquareButton
    }

    fn get_entities(game_state: &Self::GameState, receiver: &mut impl EntityReceiver<Self>) {
        receiver.receive([SquareButton { n: game_state.n }].into_iter());
    }

    fn on_death(&self, _artifact: &mut Self::Artifact) -> AnimationList<Self::Artifact> {
        vec![]
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        (
            SquareButtonArtifact {
                text: self.n.to_string(),
                x: 400.0,
                y: 400.0,
                size: self.n as f32 * 10.0,
            },
            vec![],
        )
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        _former_entity_state: EntityState,
    ) -> AnimationList<Self::Artifact> {
        artifact.text = self.n.to_string();

        let size_animation = animate_towards::<TextBoxArtifactSizeLens>(self.n as f32 * 10.0, 0.01);

        vec![size_animation]
    }
}

state_machine_games::define_lens!(TextBoxArtifactSizeLens, SquareButtonArtifact, f32, size);

//state_machine_games::define_signal_lens!(TextBoxArtifactSizeLens, SquareButtonArtifact, f32, size);

#[derive(Debug, Clone)]
pub struct SquareButtonArtifact {
    pub text: String,
    pub x: f32,
    pub y: f32,

    pub size: f32,
}

impl GameArtifact for SquareButtonArtifact {
    type Command = CounterCommand;

    fn render(self, sender: impl CommandSender<Self::Command>) -> impl leptos::IntoView {
        view! {
            <rect x=self.x rx="5%" ry="5%" y=self.y width=self.size height=self.size fill="#EE1111" on:click=move |_| sender.send_command(CounterCommand::IncrementCount(1)) />
            <text font-size={move || format!("{}px", self.size * 0.5)} x={move||{self.x + (self.size * 0.5)} } y={move ||{self.y + (self.size *0.5)}} style="pointer-events: none;user-select: none;font-family: monospace;dominant-baseline: central;text-anchor: middle;">
                {self.text}
            </text>
        }
    }
}
