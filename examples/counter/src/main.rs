use core::f32;

use glam::Vec2;
use leptos::prelude::*;
use rand::{RngCore, seq::SliceRandom};
use rand_core::SeedableRng;
use state_machine_games::{define_signal_lens, prelude::*};

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

    let square_button: ArcRwSignal<SingleTypeEntityStore<SquareButton>> = InitFromGameState::init(&state);
    let circles: ArcRwSignal<SingleTypeEntityStore<Circle>> = InitFromGameState::init(&state);

    
    let mut machine = GameMachine::new(state);
    machine.add_change_watcher(square_button.clone());
    machine.add_change_watcher(circles.clone());
    let sender = machine.command_sender().clone();

    machine.run_game();

    view! {
        <svg viewBox="0 0 800.0 800.0"  style="max-width: 800px;  margin-inline: auto; ">
        {move || SingleTypeEntityStore::render(square_button.clone(), (), sender.clone())}
        {move || SingleTypeEntityStore::render(circles.clone(), (), ())}
        </svg>
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct CounterGameState {
    pub n: usize,
    pub rng: TinyRng,
}

impl GameState for CounterGameState {
    type Command = CounterCommand;
    fn maybe_transition(&mut self) -> MutationResult {
        MutationResult::NO_CHANGE
    }
}

#[derive(Debug, Clone)]
pub enum CounterCommand {
    IncrementCount(usize),
}

impl GameCommand<CounterGameState> for CounterCommand {
    fn apply_command(&self, game_state: &mut CounterGameState) -> MutationResult {
        match self {
            CounterCommand::IncrementCount(n) => {
                game_state.n += n;
                let _ = game_state.rng.next_u64();
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

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        let position = self.position();
        let artifact = CircleArtifact {
            size: RwSignal::new(0.0),
            position: RwSignal::new(position),
        };

        let size = animate_towards::<CircleSizeLens>(10.0, 0.05);

        artifact.with_animations([size.to_stage()])
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        _former_entity_state: EntityState,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        let position = animate_towards::<CirclePositionLens>(self.position(), 0.1).to_stage();
        let size = animate_towards::<CircleSizeLens>(10.0, 0.05).to_stage();
        artifact.update_animations([position, size])
    }
}

#[derive(Debug, Clone)]
pub struct CircleArtifact {
    pub position: RwSignal<Vec2>,
    pub size: RwSignal<f32>,
}

define_signal_lens!(CirclePositionLens, CircleArtifact, Vec2, position);
define_signal_lens!(CircleSizeLens, CircleArtifact, f32, size);

impl GameArtifact for CircleArtifact {}

impl LeptosGameArtifact for CircleArtifact {
    type Command = ();
    fn render(self, _: (), _sender: impl CommandSender<Self::Command>) -> impl IntoView {
        view! {<circle cx={move ||self.position.get().x} cy={move ||self.position.get().y} r=self.size fill="#1111EE" style="pointer-events:none;" />}
    }
}

pub const SQUARE_POSITION: Vec2 = Vec2 { x: 400.0, y: 400.0 };

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

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        (
            SquareButtonArtifact {
                text: RwSignal::new(self.n.to_string()),
                x: SQUARE_POSITION.x,
                y: SQUARE_POSITION.y,
                size: RwSignal::new(self.n as f32 * 10.0),
            },
            AnimationList::EMPTY,
        )
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        _former_entity_state: EntityState,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        artifact.text.set(self.n.to_string());

        let size_animation =
            animate_towards::<TextBoxArtifactSizeLens>(self.n as f32 * 10.0, 0.01).to_stage();
        artifact.update_animations([size_animation])
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

impl GameArtifact for SquareButtonArtifact {}

impl LeptosGameArtifact for SquareButtonArtifact {
    type Command = CounterCommand;
    fn render(self, _: (), sender: impl CommandSender<Self::Command>) -> impl leptos::IntoView {
        view! {
            <rect x={move||{self.x - (self.size.get() * 0.5)} } y={move ||{self.y - (self.size.get() *0.5)}} rx={move || self.size.get() * 0.2} ry={move|| self.size.get() * 0.2}  width=self.size height=self.size fill="#EE1111" on:click=move |_| sender.send_command(CounterCommand::IncrementCount(1)) />
            <text font-size={move || format!("{}px", self.size.get() * 0.5)}  x=self.x y=self.y  style="pointer-events: none;user-select: none;font-family: monospace;dominant-baseline: central;text-anchor: middle;">
                {self.text}
            </text>
        }
    }
}
