pub mod animated_text;
pub mod background_color;
pub mod chosen_state;
pub mod clue;
pub mod colors;
pub mod database_handler;
pub mod found_words_state;
pub mod grid_input;
pub mod layout;
pub mod layout_util;
pub mod lozenge_entity;
pub mod puzzle;
pub mod qs_database_handler;
pub mod quiz_salad_command;
pub mod quiz_salad_game_state;
pub mod svg_coordinates;
pub mod tile;
pub mod util;
pub mod word_line3;

use std::sync::mpsc::Sender;

use animated_text::*;
use background_color::*;
use clue::*;
use leptos::svg::Svg;
use leptos_use::use_timestamp;
use lozenge_entity::*;
use quiz_salad_game_state::*;
use tile::*;
use web_sys::PointerEvent;
use web_sys::js_sys;
use ws_core::HasCenter;

use crate::colors::CLASSIC_COLOR_SCHEME;
use crate::found_words_state::Completion;
use crate::grid_input::GridInputCommand;
use crate::grid_input::GridInputState;
use crate::layout::*;
use crate::qs_database_handler::FoundWordsStateTracker;
use crate::qs_database_handler::QSDatabaseCommand;
use crate::quiz_salad_command::QuizSaladCommand;
use crate::word_line3::*;
use bevy_color::Srgba;
use itertools::Itertools;
use leptos::ev::MouseEvent;
#[allow(unused_imports)]
use leptos::logging::log;
use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::components::Router;
use leptos_router::components::Routes;
use leptos_router::params::Params;
use state_machine_games::define_signal_lens;
use state_machine_games::prelude::*;
use ws_core::{ArrayVec, Character, LevelTrait, Tile4x4, Ustr};

use crate::{chosen_state::ChosenState, found_words_state::FoundWordsState, puzzle::Puzzle};

pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    mount_to_body(app);
}

#[derive(Params, PartialEq)]
struct GameParams {
    encoded: Option<String>,
}
//spellchecker:disable-next-line
const DEFAULT_PUZZLE: &'static str = "LYCAWAURSGPIHOND	Super Salad 10	Song[A word that might follow 'bird' or 'love']	Capri[An Italian island]	Lycra[A synthetic fibre]	Pugwash[A cartoon captain]	Lagos[A Nigerian city]	Dinosaur[A way to describe an out of touch person] 	Posh[One of the Spice Girls]	Dingo[An Australian animal]	Cupid[A romantic messenger]	Pugh[A famous florence]";

pub fn app() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| game_component_no_path()>
                  <Route path=leptos_router::path!("/game/:encoded") view={move ||game_component_with_path()}/>
            </Routes>
        </Router>
    }
}

fn game_component_no_path() -> impl IntoView {
    let puzzle = Memo::new(move |_| Puzzle::from_tsv_line(DEFAULT_PUZZLE).unwrap());
    game_component(puzzle)
}

fn game_component_with_path() -> impl IntoView {
    let params = leptos_router::hooks::use_params::<GameParams>();

    let puzzle = Memo::new(move |_| {
        let puzzle = params
            .read()
            .as_ref()
            .ok()
            .and_then(|params| params.encoded.clone())
            .and_then(|x| Puzzle::try_from_encoded(&x));

        puzzle.unwrap_or_else(|| Puzzle::from_tsv_line(DEFAULT_PUZZLE).unwrap())
    });

    game_component(puzzle)
}

fn game_component(puzzle_memo: Memo<Puzzle>) -> impl IntoView {
    let puzzle = puzzle_memo.get_untracked();
    let now = js_sys::Date::now();

    let state = QuizSaladGameState::new(puzzle.clone(), now);
    let (db_command_sender, db_command_receiver) = async_channel::unbounded::<QSDatabaseCommand>();

    db_command_sender
        .try_send(QSDatabaseCommand::LoadLevel(puzzle.clone()))
        .unwrap();

    Effect::new({
        let db_command_sender = db_command_sender.clone();
        move || {
            let puzzle = puzzle_memo.get();
            db_command_sender
                .try_send(QSDatabaseCommand::LoadLevel(puzzle))
                .unwrap();
        }
    });

    let tiles: ArcRwSignal<SingleTypeEntityStore<TileEntity>> = InitFromGameState::init(&state);
    let clues: ArcRwSignal<SingleTypeEntityStore<ClueEntity>> = InitFromGameState::init(&state);
    let word_line: ArcRwSignal<SingleTypeEntityStore<WordLineEntity>> =
        InitFromGameState::init(&state);
    let lozenges: ArcRwSignal<SingleTypeEntityStore<LozengeEntity>> =
        InitFromGameState::init(&state);
    let animated_text: ArcRwSignal<SingleTypeEntityStore<AnimatedTextEntity>> =
        InitFromGameState::init(&state);
    let background_color: ResourceStore<BackgroundColor> = InitFromGameState::init(&state);
    let found_words_state_tracker = FoundWordsStateTracker::new(
        puzzle.clone(),
        FoundWordsState::new_from_level(&puzzle),
        db_command_sender,
    );

    let start_time_watcher = ValueWatcher::<StartTimeLens>::init(&state);
    let start_time = start_time_watcher.value_signal;

    let finish_time_watcher = ValueWatcher::<FinishTimeLens>::init(&state);
    let finish_time = finish_time_watcher.value_signal;

    let timestamp = use_timestamp();

    let seconds: Memo<u32> = Memo::new(move |_| {
        let start_time = start_time.get();
        let finish_seconds = finish_time.get();
        let timestamp: f64 = timestamp.get();

        if let Some(finish_seconds) = finish_seconds {
            return finish_seconds;
        }

        let seconds = (timestamp - start_time) / 1000.0;

        seconds.floor().max(0.0) as u32
    });

    let background_color_value = background_color.artifact.color;

    let mut machine = GameMachine::new(state);
    machine.add_change_watcher(tiles.clone());
    machine.add_change_watcher(clues.clone());
    machine.add_change_watcher(word_line.clone());
    machine.add_change_watcher(lozenges.clone());
    machine.add_change_watcher(animated_text.clone());
    machine.add_change_watcher(background_color);
    machine.add_change_watcher(found_words_state_tracker);
    machine.add_change_watcher(start_time_watcher);
    machine.add_change_watcher(finish_time_watcher);

    let cs2 = machine.command_sender();
    let cs3 = machine.command_sender();
    let cs4 = machine.command_sender();
    let cs5 = machine.command_sender();
    let cs6 = machine.command_sender();

    machine.run_game();

    let svg_style = "max-width: 100%; max-height: 100%; user-select:none; position:fixed; margin:auto; inset: 0px;";

    let div_style = move || {
        format!(
            "height: 100vh; width: 100vw; overflow: hidden; background: {}",
            background_color_value.get().to_hex()
        )
    };

    let node_ref = NodeRef::<Svg>::new();

    let on_pointer_down = move |ev: PointerEvent| {
        on_pointer_down(ev, node_ref, &cs3);
    };

    let on_pointer_up = move |ev: PointerEvent| {
        on_pointer_up(ev, node_ref, &cs4);
    };

    let on_pointer_move = move |ev: PointerEvent| {
        on_pointer_move(ev, node_ref, &cs5);
    };

    leptos::task::spawn_local(qs_database_handler::handle_db_messages(
        cs6,
        db_command_receiver,
    ));

    let tiles2 = tiles.clone();

    view! {
        <div style=div_style>
            <svg node_ref=node_ref viewBox=format!("0 0 {GAME_WIDTH} {GAME_HEIGHT}") style={svg_style}
            on:pointerdown=on_pointer_down
            on:pointerup=on_pointer_up
            on:pointermove=on_pointer_move
            >

            <g id="tiles">
            {move || SingleTypeEntityStore::render(tiles.clone(),RenderTileFill, ())} // Tile
            </g>
            <g id="clues">
            {move || SingleTypeEntityStore::render(clues.clone(),(), ())}
            </g>
            <g id="word_line">
            {move || SingleTypeEntityStore::render(word_line.clone(),(), ())}
            </g>
            <g id="tile_text">
            {move || SingleTypeEntityStore::render(tiles2.clone(),RenderTileText, ())}
            </g>
            <g id="lozenges">
            {move || SingleTypeEntityStore::render(lozenges.clone(),(), cs2.clone())}
            </g>
            <g id="animated_text">
            {move || SingleTypeEntityStore::render(animated_text.clone(),(), ())} // Animated Text
            </g>

            //todo render the tile text twice with a wordline luminosity mask

            {timer_component(seconds)}

            </svg>
        </div>

    }
}

fn timer_component(seconds: Memo<u32>) -> impl IntoView {
    let time_str: Memo<String> = Memo::new(move |_| {
        let total_seconds = seconds.get();

        let hours = total_seconds / 3600;
        let minutes = (total_seconds / 60) % 60;
        let extra_seconds = total_seconds % 60;

        if hours > 0 {
            format!("{hours:02}:{minutes:02}:{extra_seconds:02}")
        } else {
            format!("{minutes:02}:{extra_seconds:02}")
        }
    });

    view! {
        <text
        style="transform-box: content-box; transform-origin: center;"
        x="240" y="120"
        dominant-baseline="central"
        text-anchor="start"
        fill="#043E40"
        font-size="60"
        font-family="Montserrat"
        font-weight="600"
        pointer-events="none">
            {time_str}
        </text>
    }
}

fn get_tile_from_position(position: Vec2, sensitivity: f32) -> Option<Tile4x4> {
    let y = position.y - TOP_OFFSET;
    let x = position.x - LEFT_OFFSET;

    if x < 0.0 || y < 0.0 {
        return None;
    }

    const TILE_SIZE: f32 = BOARD_SIZE / 4.0;

    let x = x / TILE_SIZE;
    let y = y / TILE_SIZE;
    let x = x as u8;
    let y = y as u8;

    let tile = Tile4x4::try_new(x, y)?;

    let c = tile.get_center(TILE_SIZE);
    let distances = ((c + Vec2 {
        x: LEFT_OFFSET,
        y: TOP_OFFSET,
    } - position)
        / TILE_SIZE)
        .abs();

    // log!(
    //     "Position: {position}\n
    // Tile {tile}\n
    //  Sensitivity {sensitivity}\n
    //  Distances {distances}"
    // );

    if distances.x <= sensitivity && distances.y <= sensitivity {
        //leptos::logging::log!("GIC: {gic:?}");
        Some(tile)
    } else {
        None
    }
}

fn on_pointer_down(ev: PointerEvent, node_ref: NodeRef<Svg>, sender: &Sender<QuizSaladCommand>) {
    let Some(element) = node_ref.get() else {
        return;
    };
    let click_position = crate::svg_coordinates::get_svg_coordinates(
        ev,
        element,
        Vec2 { x: 0.0, y: 0.0 },
        Vec2 {
            x: GAME_WIDTH,
            y: GAME_HEIGHT,
        },
    );

    let tile = get_tile_from_position(click_position, 0.9);
    let command = GridInputCommand::Start(tile);

    match sender.send(QuizSaladCommand::BoardPointerEvent(command)) {
        Ok(()) => {}
        Err(_err) => {
            leptos::logging::error!("Could not send command");
        }
    }
}

fn on_pointer_up(ev: PointerEvent, node_ref: NodeRef<Svg>, sender: &Sender<QuizSaladCommand>) {
    let Some(element) = node_ref.get() else {
        return;
    };
    let click_position = crate::svg_coordinates::get_svg_coordinates(
        ev,
        element,
        Vec2 { x: 0.0, y: 0.0 },
        Vec2 {
            x: GAME_WIDTH,
            y: GAME_HEIGHT,
        },
    );

    let tile = get_tile_from_position(click_position, 0.9);
    let command = GridInputCommand::End(tile);

    match sender.send(QuizSaladCommand::BoardPointerEvent(command)) {
        Ok(()) => {}
        Err(_err) => {
            leptos::logging::error!("Could not send command");
        }
    }
}

fn on_pointer_move(ev: PointerEvent, node_ref: NodeRef<Svg>, sender: &Sender<QuizSaladCommand>) {
    if ev.pressure() == 0.0 {
        return;
    }

    let Some(element) = node_ref.get() else {
        return;
    };
    let click_position = crate::svg_coordinates::get_svg_coordinates(
        ev,
        element,
        Vec2 { x: 0.0, y: 0.0 },
        Vec2 {
            x: GAME_WIDTH,
            y: GAME_HEIGHT,
        },
    );

    let Some(tile) = get_tile_from_position(click_position, 0.3) else {
        return;
    };
    let command = GridInputCommand::Move(tile);

    match sender.send(QuizSaladCommand::BoardPointerEvent(command)) {
        Ok(()) => {}
        Err(_err) => {
            leptos::logging::error!("Could not send command");
        }
    }
}
