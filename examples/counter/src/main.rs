use core::f32;
use std::sync::mpsc::Sender;

use glam::Vec2;
use leptos::prelude::{codee::string::JsonSerdeCodec, *};
use leptos_use::storage::{UseStorageOptions, use_local_storage_with_options};
use rand::{Rng, seq::SliceRandom};
use rand_core::SeedableRng;
use serde::{Deserialize, Serialize};
use state_machine_games::{define_signal_lens, delayed_effect::MutationResult, prelude::*};

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    mount_to_body(|| game_component());
}

fn game_component() -> impl IntoView {
    let (state_signal, state_signal_write, _) =
        use_local_storage_with_options::<CounterGameState, JsonSerdeCodec>(
            "smg2_counter_state",
            UseStorageOptions::default().initial_value(CounterGameState {
                n: 2,
                rng: TinyRng::seed_from_u64(123),
            }),
        );

    let sender = run_game(state_signal_write, std::iter::empty());

    let square = SquareButton::render_singleton::<SquareButtonArtifact>(state_signal);
    let circles = Circle::render_entities::<CircleArtifact>(state_signal);

    provide_context(sender);

    let reset_action = move |_me: _| {
        state_signal_write.update(|x| {
            CounterCommand::Reset.apply_command(x);
        });
    };

    view! {

        <svg viewBox="0 0 800.0 800.0"  style="max-width: 800px;  margin-inline: auto; ">
        {square}
        {circles}
        </svg>

        <button on:click=reset_action>Reset</button>
    }
}

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct CounterGameState {
    pub n: usize,
    pub rng: TinyRng,
}

impl GameState for CounterGameState {}

#[derive(Debug, Clone)]
pub enum CounterCommand {
    IncrementCount(usize),
    Reset,
}

impl GameCommand<CounterGameState> for CounterCommand {
    fn apply_command(&self, game_state: &mut CounterGameState) -> MutationResult<CounterGameState> {
        match self {
            CounterCommand::IncrementCount(n) => {
                game_state.n += n;
                let _ = game_state.rng.next_u64();

                MutationResult::Changed(None)
                
            }
            CounterCommand::Reset => {
                game_state.n = 0;
                MutationResult::Changed(None)
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
    type Segment = CounterGameState;

    fn get_entities(game_state: &Self::Segment) -> impl Iterator<Item = Self> {
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
            size: ArcRwSignal::new(0.0),
            position: ArcRwSignal::new(position),
        };

        let size = animate_towards::<CircleSizeLens>(10.0, 0.05);

        artifact.with_animations([size.to_stage()])
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        _former_entity_state: EntityLifecycle,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        let position = animate_towards::<CirclePositionLens>(self.position(), 0.1).to_stage();
        let size = animate_towards::<CircleSizeLens>(10.0, 0.05).to_stage();
        artifact.update_animations([position, size])
    }
}

#[derive(Debug, Clone)]
pub struct CircleArtifact {
    pub position: ArcRwSignal<Vec2>,
    pub size: ArcRwSignal<f32>,
}

define_signal_lens!(CirclePositionLens, CircleArtifact, Vec2, position);
define_signal_lens!(CircleSizeLens, CircleArtifact, f32, size);

impl GameArtifact for CircleArtifact {}

impl LeptosRender for CircleArtifact {
    type Artifact = Self;

    fn render(artifact: Self::Artifact) -> impl IntoView {
        let px = artifact.position.clone();
        let py = artifact.position.clone();
        view! {<circle cx={move ||px.get().x} cy={move ||py.get().y} r=artifact.size fill="#1111EE" style="pointer-events:none;" />}
    }
}

pub const SQUARE_POSITION: Vec2 = Vec2 { x: 400.0, y: 400.0 };

#[derive(Debug, PartialEq)]
pub struct SquareButton {
    pub n: usize,
}

impl SingletonEntity for SquareButton {
    type Segment = CounterGameState;
    type Artifact = SquareButtonArtifact;

    fn init(segment: &Self::Segment) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        (
            SquareButtonArtifact {
                text: RwSignal::new(segment.n.to_string()),
                x: SQUARE_POSITION.x,
                y: SQUARE_POSITION.y,
                size: RwSignal::new(get_size(segment.n)),
            },
            AnimationList::EMPTY,
        )
    }

    fn update(
        artifact: &mut Self::Artifact,
        current_entity_state: &Self::Segment,
        _former_entity_state: &Self::Segment,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        artifact.text.set(current_entity_state.n.to_string());

        let size_animation =
            animate_towards::<TextBoxArtifactSizeLens>(get_size(current_entity_state.n), 0.01)
                .to_stage();
        artifact.update_animations([size_animation])
    }
}

fn get_size(count: usize) -> f32 {
    (count as f32 + 100.0).log10() * 50.0
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

impl LeptosRender for SquareButtonArtifact {
    type Artifact = Self;

    fn render(artifact: Self::Artifact) -> impl IntoView + 'static {
        let sender = expect_context::<Sender<Box<dyn GameCommand<CounterGameState>>>>();

        let on_click = move |_| {
            sender
                .send(Box::new(CounterCommand::IncrementCount(1)))
                .unwrap();
        };

        view! {
            <rect x={move||{artifact.x - (artifact.size.get() * 0.5)} } y={move ||{artifact.y - (artifact.size.get() *0.5)}} rx={move || artifact.size.get() * 0.2} ry={move|| artifact.size.get() * 0.2}  width=artifact.size height=artifact.size fill="#EE1111" on:click=on_click  />
            <text font-size={move || format!("{}px", artifact.size.get() * 0.5)}  x=artifact.x y=artifact.y  style="pointer-events: none;user-select: none;font-family: monospace;dominant-baseline: central;text-anchor: middle;">
                {artifact.text}
            </text>
        }
    }
}
