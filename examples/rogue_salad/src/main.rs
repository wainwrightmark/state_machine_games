pub mod background_color;
pub mod colors;
pub mod layout;
pub mod layout_util;

use core::f32;

use crate::background_color::BackgroundColor;
use glam::Vec2;
use layout::{GAME_HEIGHT, GAME_WIDTH};
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
    let state = RogueSaladGameState::init();

    let background_color: ResourceStore<BackgroundColor> = InitFromGameState::init(&state);
    let background_color_value = background_color.artifact.color;
    let mut machine = GameMachine::new(state);
    let sender = machine.command_sender().clone();

    machine.add_change_watcher(background_color);
    machine.run_game();

    let node_ref = NodeRef::<Svg>::new();

    provide_context(node_ref);

    let svg_style = "max-width: 100%; max-height: 100%; user-select:none; position:fixed; margin:auto; inset: 0px;";

    let div_style = move || {
        format!(
            "height: 100vh; width: 100vw; overflow: hidden; background: {}",
            background_color_value.get().to_hex()
        )
    };

    let node_ref = NodeRef::<Svg>::new();

    view! {
        <div style=div_style>
            <svg node_ref=node_ref viewBox=format!("0 0 {GAME_WIDTH} {GAME_HEIGHT}") style={svg_style}
            // on:pointerdown=on_pointer_down
            // on:pointerup=on_pointer_up
            // on:pointermove=on_pointer_move
            >

            // <g id="tiles">
            // {move || SingleTypeEntityStore::render(tiles.clone(),RenderTileFill, ())} // Tile
            // </g>
            // <g id="clues">
            // {move || SingleTypeEntityStore::render(clues.clone(),(), ())}
            // </g>
            // <g id="word_line">
            // {move || SingleTypeEntityStore::render(word_line.clone(),(), ())}
            // </g>
            // <g id="tile_text">
            // {move || SingleTypeEntityStore::render(tiles2.clone(),RenderTileText, ())}
            // </g>
            // <g id="lozenges">
            // {move || SingleTypeEntityStore::render(lozenges.clone(),(), cs2.clone())}
            // </g>
            // <g id="animated_text">
            // {move || SingleTypeEntityStore::render(animated_text.clone(),(), ())} // Animated Text
            // </g>

            //todo render the tile text twice with a wordline luminosity mask

            // {timer_component(seconds)}

            </svg>
        </div>

    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RogueSaladGameState {
    pub global: GlobalState,
    pub dictionary: Dictionary,
    pub phase: Phase,
}

impl RogueSaladGameState {
    pub fn init() -> Self {
        RogueSaladGameState {
            global: GlobalState {},
            dictionary: Dictionary {},
            phase: Phase::ChoosePath(ChoosePathState {}),
        }
    }
}

impl GameState for RogueSaladGameState {
    type Command = RogueSaladCommand;

    fn maybe_transition(&mut self) -> MutationResult {
        MutationResult::NO_CHANGE
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RogueSaladCommand {
    ChoosePath(usize),
    ChooseBoon(usize),
}

impl GameCommand<RogueSaladGameState> for RogueSaladCommand{
    fn apply_command(&self, game_state: &mut RogueSaladGameState) -> MutationResult {
        //todo
        MutationResult::NO_CHANGE
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalState {}

#[derive(Debug, Clone, PartialEq)]
pub struct Dictionary {}

#[derive(Debug, Clone, PartialEq)]
pub enum Phase {
    ChooseBoon(ChooseBoonState),
    PlayPuzzle(PlayPuzzleState),
    ChoosePath(ChoosePathState),
    GameOver,
    Victory,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChooseBoonState {}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayPuzzleState {}

#[derive(Debug, Clone, PartialEq)]
pub struct ChoosePathState {}
