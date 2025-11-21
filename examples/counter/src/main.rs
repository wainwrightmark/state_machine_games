use state_machine_games::prelude::*;
use leptos::prelude::*;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        leptos_game_component(200.0, 200.0, CounterGameState { n: 2 })
    });
}

pub struct CounterGameState {
    pub n: usize,
}

impl GameState for CounterGameState {
    type Command = ();
    type Key = Key;

    fn entities(&self, receiver: &mut impl EntityReceiver<Self::Command, Self::Key>) {
        receiver.receive([SquareButton { n: self.n }].into_iter());

        if self.n > 5 {
            receiver.receive(
                (0..self.n)
                    .filter(|x| x % 2 == 0)
                    .map(|n| Circle { inner: n as u8 }),
            );
        } else {
            receiver.receive((0..self.n).map(|n| Circle { inner: n as u8 }));
        }
    }

    fn apply_command(&mut self, _command: Self::Command) -> MutationResult {
        self.n += 1;
        MutationResult::CHANGED_NO_TRANSITION
    }

    fn maybe_transition(&mut self) -> MutationResult {
        MutationResult::NO_CHANGE
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
    type Artifact = CircleArtifact;
    type EntityKey = Key;
    type Command = ();

    fn key(&self) -> Self::EntityKey {
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
    type Artifact = TextBoxArtifact;
    type EntityKey = Key;

    type Command = ();

    fn key(&self) -> Self::EntityKey {
        Key::SquareButton
    }

    fn on_death(&self, _artifact: &mut Self::Artifact) -> AnimationList<Self::Artifact> {
        vec![]
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        (
            TextBoxArtifact {
                text: RwSignal::new(self.n.to_string()),
                x: 50.0,
                y: 50.0,
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

state_machine_games::define_signal_lens!(TextBoxArtifactSizeLens, TextBoxArtifact, f32, size);

#[derive(Debug, Clone)]
pub struct TextBoxArtifact {
    pub text: RwSignal<String>,
    pub x: f32,
    pub y: f32,

    pub size: RwSignal<f32>,
}

impl GameArtifact for TextBoxArtifact {
    type Command = ();

    fn render(self, mut sender: impl CommandSender<Self::Command>) -> impl leptos::IntoView {
        view! {
            <rect x=self.x rx="5%" ry="5%" y=self.y width=self.size height=self.size fill="#EE1111" on:click=move |_| sender.send_command(()) />
            <text font-size={move || format!("{}px", self.size.get() * 0.5)} x={move||{self.x + (self.size.get() * 0.5)} } y={move ||{self.y + (self.size.get() *0.5)}} style="pointer-events: none;user-select: none;font-family: monospace;dominant-baseline: central;text-anchor: middle;">
                {self.text}
            </text>
        }
    }
}
