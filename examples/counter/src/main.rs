use core::f32;
use std::sync::mpsc;

use glam::Vec2;
use leptos::prelude::*;
use rand::{RngCore, seq::SliceRandom};
use rand_core::SeedableRng;
use state_machine_games::{define_signal_lens, prelude::*};
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
    let state = CounterGameState {
        n: 2,
        rng: TinyRng::seed_from_u64(123),
    };

    let (sender, receiver) = mpsc::channel::<CounterCommand>();
    let stores: Stores = Stores::new(&state);
    let machine = GameMachine::new(state, stores.clone(), receiver);

    machine.run_game();

    view! {
        <svg viewBox="0 0 800.0 800.0"  style="max-width: 800px;  margin-inline: auto; ">
        {move || SingleTypeEntityStore::render(stores.0.clone(), sender.clone())}
        {move || SingleTypeEntityStore::render(stores.1.clone(), ())}
        </svg>
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct CounterGameState {
    pub n: usize,
    pub rng: TinyRng,
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

impl GameCommand<CounterGameState> for CounterCommand {
    fn apply_command(&self, games_state: &mut CounterGameState) -> MutationResult {
        match self {
            CounterCommand::IncrementCount(n) => {
                games_state.n += n;
                let _ = games_state.rng.next_u64();
                MutationResult::CHANGED_NO_TRANSITION
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key {
    SquareButton,
    Circle(u32),
}

impl GameEntityKey for Key {}

#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    pub k: u32,
    pub position_k: u32,
    pub n: u32,
}

impl Circle {
    pub fn position(&self) -> Vec2 {
        const CENTER: Vec2 = Vec2::splat(400.0);
        const RADIUS: f32 = 100.0;
        let theta = f32::consts::TAU * self.position_k as f32 / self.n as f32;

        let x = CENTER.x + RADIUS * f32::cos(theta);
        let y = CENTER.y + RADIUS * f32::sin(theta);

        //log!("k {} n {} Theta: {theta} x {x} y {y}", self.position_k, self.n);

        Vec2 { x, y }
    }
}

impl GameEntity for Circle {    
    type Artifact = CircleArtifact;
    type Key = Key;
    type StateSegment = CounterGameState;

    fn get_entities(game_state: &Self::StateSegment) -> impl Iterator<Item = Self> {
        let mut arr: Vec<_> = (0u32..game_state.n as u32).collect();

        let mut rng = game_state.rng.clone();
        arr.shuffle(&mut rng);

        let n = game_state.n as u32;

        (0..n).map(move |k| Circle {
            k,
            position_k: arr[k as usize],
            n,
        })
    }

    fn key(&self) -> Self::Key {
        Key::Circle(self.k)
    }

    fn on_death(&self, _artifact: &mut Self::Artifact) -> AnimationList<Self::Artifact> {
        vec![]
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        let Vec2 { x, y } = self.position();
        (
            CircleArtifact {
                size: 10.0,
                x: RwSignal::new(x),
                y: RwSignal::new(y),
            },
            vec![],
        )
    }

    fn on_update(
        &self,
        _artifact: &mut Self::Artifact,
        _former_entity_state: EntityState,
    ) -> AnimationList<Self::Artifact> {
        let Vec2 { x, y } = self.position();

        let x = animate_towards::<CircleXLens>(x, 0.1);
        let y = animate_towards::<CircleYLens>(y, 0.1);

        vec![x, y]
    }
}

#[derive(Debug, Clone)]
pub struct CircleArtifact {
    pub x: RwSignal<f32>,
    pub y: RwSignal<f32>,
    pub size: f32,
}

define_signal_lens!(CircleXLens, CircleArtifact, f32, x);
define_signal_lens!(CircleYLens, CircleArtifact, f32, y);

impl GameArtifact for CircleArtifact {
    type Command = ();
}

impl LeptosGameArtifact for CircleArtifact {
    fn render(self, _sender: impl CommandSender<Self::Command>) -> impl IntoView {
        view! {<circle cx=self.x cy=self.y r=self.size fill="#1111EE" />}
    }
}

#[derive(Debug, PartialEq)]
pub struct SquareButton {
    pub n: usize,
}

impl GameEntity for SquareButton {
    type StateSegment = CounterGameState;
    type Artifact = SquareButtonArtifact;
    type Key = Key;

    fn key(&self) -> Self::Key {
        Key::SquareButton
    }

    fn get_entities(game_state: &Self::StateSegment) -> impl Iterator<Item = Self> {
        [SquareButton { n: game_state.n }].into_iter()
    }

    fn on_death(&self, _artifact: &mut Self::Artifact) -> AnimationList<Self::Artifact> {
        vec![]
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        (
            SquareButtonArtifact {
                text: RwSignal::new(self.n.to_string()),
                x: 400.0,
                y: 400.0,
                size: RwSignal::new(self.n as f32 * 10.0),
            },
            vec![],
        )
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        _former_entity_state: EntityState,
    ) -> AnimationList<Self::Artifact> {
        artifact.text.set(self.n.to_string());

        let size_animation = animate_towards::<TextBoxArtifactSizeLens>(self.n as f32 * 10.0, 0.01);

        vec![size_animation]
    }
}

state_machine_games::define_signal_lens!(TextBoxArtifactSizeLens, SquareButtonArtifact, f32, size);

#[derive(Debug, Clone)]
pub struct SquareButtonArtifact {
    pub text: RwSignal<String>,
    pub x: f32,
    pub y: f32,

    pub size: RwSignal<f32>,
}

impl GameArtifact for SquareButtonArtifact {
    type Command = CounterCommand;
}

impl LeptosGameArtifact for SquareButtonArtifact {
    fn render(self, sender: impl CommandSender<Self::Command>) -> impl leptos::IntoView {
        view! {
            <rect x={move||{self.x - (self.size.get() * 0.5)} } y={move ||{self.y - (self.size.get() *0.5)}} rx={move || self.size.get() * 0.2} ry={move|| self.size.get() * 0.2}  width=self.size height=self.size fill="#EE1111" on:click=move |_| sender.send_command(CounterCommand::IncrementCount(1)) />
            <text font-size={move || format!("{}px", self.size.get() * 0.5)}  x=self.x y=self.y  style="pointer-events: none;user-select: none;font-family: monospace;dominant-baseline: central;text-anchor: middle;">
                {self.text}
            </text>
        }
    }
}
