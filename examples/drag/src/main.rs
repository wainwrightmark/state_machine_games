use core::f32;

use glam::Vec2;
use leptos::{
    ev::{DragEvent, PointerEvent},
    prelude::*,
    svg::Svg,
};
use rand::{RngCore, seq::SliceRandom};
use rand_core::SeedableRng;
use state_machine_games::{define_signal_lens, prelude::*};

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    mount_to_body(|| game_component());
}

fn game_component() -> impl IntoView {
    let state = DragGameState { current_slot: 1 };

    let draggables: ArcRwSignal<SingleTypeEntityStore<DraggableEntity>> =
        InitFromGameState::init(&state);

    let mut machine = GameMachine::new(state);
    machine.add_change_watcher(draggables.clone());
    let sender = machine.command_sender().clone();

    machine.run_game();

    let node_ref = NodeRef::<Svg>::new();

    provide_context(node_ref);

    view! {
        <svg node_ref=node_ref viewBox="0 0 800.0 800.0"   style="max-width: 800px;  margin-inline: auto; ">
        {move || SingleTypeEntityStore::render(draggables.clone(), (), sender.clone())}
        </svg>
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DragGameState {
    pub current_slot: usize,
}

impl GameState for DragGameState {
    type Command = DragCommand;

    fn maybe_transition(&mut self) -> MutationResult {
        MutationResult::NO_CHANGE
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DragCommand {
    pub new_slot: usize,
}

impl GameCommand<DragGameState> for DragCommand {
    fn apply_command(&self, game_state: &mut DragGameState) -> MutationResult {
        game_state.current_slot = self.new_slot;
        MutationResult::CHANGED_NO_TRANSITION
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DraggableShapeArtifact {
    pub position: RwSignal<Vec2>,
}

impl GameArtifact for DraggableShapeArtifact {}

impl LeptosGameArtifact for DraggableShapeArtifact {
    type Command = DragCommand;

    fn render(
        self,
        argument: (),
        sender: impl state_machine_games::prelude::CommandSender<Self::Command>,
    ) -> impl IntoView {
        //let position = move || position(self.current_slot.get());

        let node_ref =
            use_context::<NodeRef<Svg>>().expect("Should be able to get svg node ref context");

        let offset = signal(None);

        let on_pointer_down = move |ev: PointerEvent| {
            leptos::logging::log!("drag start");

            let Some(element) = node_ref.get() else {
                return;
            };

            ev.prevent_default();
            let c = state_machine_games::svg_coordinates::get_svg_coordinates(
                ev,
                element,
                Vec2::ZERO,
                Vec2::new(800.0, 800.0),
            );
            offset.1.set(Some(c - self.position.get_untracked()));
        };

        let on_pointer_up = move |ev: PointerEvent| {
            leptos::logging::log!("drag end");
            offset.1.set(None);
        };

        let on_pointer_move = move |ev: PointerEvent| {
            if ev.pressure() == 0.0 {
                return;
            }

            let Some(element) = node_ref.get() else {
                return;
            };
            let Some(offset) = offset.0.get() else {
                return;
            };
            ev.prevent_default();
            let c = state_machine_games::svg_coordinates::get_svg_coordinates(
                ev,
                element,
                Vec2::ZERO,
                Vec2::new(800.0, 800.0),
            );

            self.position.set(c - offset);

            leptos::logging::log!("drag: {c}");
        };

        view! {
            <circle
                cx=move||self.position.get().x
                cy=move||self.position.get().y
                style="pointer-events: visiblePainted; cursor: move;"

                on:pointerdown=on_pointer_down
                on:pointerup=on_pointer_up
                on:pointermove=on_pointer_move

                draggable="true"
                r=40.0
                fill="#EE1122"
            />

        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DraggableEntity {
    pub current_slot: usize,
}

impl GameEntity for DraggableEntity {
    type Artifact = DraggableShapeArtifact;

    type Key = ();

    type StateSegment = DragGameState;

    fn key(&self) -> Self::Key {
        ()
    }

    fn get_entities(segment: &Self::StateSegment) -> impl Iterator<Item = Self> {
        [Self {
            current_slot: segment.current_slot,
        }]
        .into_iter()
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        (
            DraggableShapeArtifact {
                position: RwSignal::new(position(self.current_slot)),
            },
            Default::default(),
        )
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        former_entity_state: EntityState,
        previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        //artifact.current_slot.set(self.current_slot);
        AnimationList::EMPTY
    }
}

pub fn position(index: usize) -> Vec2 {
    let y = 100.0;
    let x = 200.0 + (50.0 * index as f32);

    Vec2 { x, y }
}
