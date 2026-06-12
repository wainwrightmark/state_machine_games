pub mod background_color;

pub mod colors;

pub mod dalas_drow_command;
pub mod dalas_drow_game_state;
pub mod layout;
pub mod layout_util;
pub mod lozenge_entity;
pub mod menu;
pub mod puzzle;
pub mod tile;
pub mod util;

use std::sync::mpsc::Sender;

use background_color::*;

use dalas_drow_game_state::*;
use leptos::svg::Svg;
use leptos_use::use_timestamp;
use lozenge_entity::*;
use state_machine_games::value_watcher::watch_value;
use tile::*;
use web_sys::js_sys;
use ws_core::HasCenter;

use crate::colors::CLASSIC_COLOR_SCHEME;
use crate::dalas_drow_command::DalasDrowCommand;

use crate::layout::*;
use bevy_color::Srgba;
#[allow(unused_imports)]
use leptos::logging::log;
use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::components::Router;
use leptos_router::components::Routes;
use leptos_router::params::Params;
use state_machine_games::define_signal_lens;
use state_machine_games::prelude::*;
use ws_core::{Character, Tile4x4};

use crate::puzzle::Puzzle;

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
const DEFAULT_PUZZLE: &'static str = "ELTEKHRVISOUNBCD	5	cover	obscure	shelter	shroud	skin";

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

    //todo load found words from db

    let state = DalasDrowGameState::new(puzzle.clone(), now);

    let state_signal = RwSignal::new(state);

    let sender: Sender<Box<dyn GameCommand<DalasDrowGameState> + 'static>> = run_game(state_signal);

    let tiles = TileEntity::render_entities::<TileRender>(state_signal.into());
    let tile_texts = TileEntity::render_entities::<TileTextRender>(state_signal.into());

    let lozenges = LozengeEntity::render_entities::<LozengeArtifact>(state_signal.into());

    let background_color = BackgroundColor::track_singleton(
        Memo::new(move |_| (&*state_signal.read()).background_target_color()).into(),
    );
    let background_color = background_color.read_untracked().artifact.color;

    let start_time = watch_value::<StartTimeLens>(state_signal);
    let finish_time = watch_value::<FinishTimeLens>(state_signal);

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

    let svg_style = "max-width: 100%; max-height: 100%; user-select:none; position:fixed; margin:auto; inset: 0px;";

    let div_style = move || {
        format!(
            "height: 100vh; width: 100vw; overflow: hidden; background: {}",
            background_color.get().to_hex()
        )
    };

    let node_ref = NodeRef::<Svg>::new();

    // leptos::task::spawn_local(qs_database_handler::handle_db_messages(
    //     sender.clone(),
    //     db_command_receiver,
    // ));
    let sender1: Sender<Box<dyn GameCommand<DalasDrowGameState> + 'static>> = sender.clone();
    provide_context(sender);

    view! {
        <button style="position:fixed;" on:click=move|_|{
            match sender1.send(Box::new(DalasDrowCommand::ResetLevel)) {
                Ok(()) => {}
                Err(_err) => {
                    leptos::logging::error!("Could not send command");
                }
            }
        }>
            Reset
        </button>
        <div style=div_style>
            <svg node_ref=node_ref viewBox=format!("0 0 {GAME_WIDTH} {GAME_HEIGHT}") style={svg_style}

            >

            <g id="tiles">
            {tiles}
            </g>
            <g id="tile_text">
                {tile_texts}
            </g>
            <g id="lozenges">
            {lozenges}
            </g>

            // //todo render the tile text twice with a wordline luminosity mask

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

    let p = glam::f32::Vec2 {
        x: LEFT_OFFSET - position.x,
        y: TOP_OFFSET - position.y,
    };

    let c = tile.get_center(TILE_SIZE);
    let distances = ((c + p) / TILE_SIZE).abs();

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
