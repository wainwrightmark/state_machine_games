pub mod animated_text;
pub mod background_color;
pub mod chosen_state;
pub mod clue;
pub mod colors;
pub mod database_handler;
pub mod found_answer;
pub mod found_words_state;
pub mod grid_input;
pub mod layout;
pub mod layout_util;
pub mod lozenge_entity;
pub mod menu;
pub mod puzzle;
pub mod qs_database_handler;
pub mod quiz_salad_command;
pub mod quiz_salad_game_state;
pub mod tile;
pub mod util;
pub mod word_line3;

use std::sync::mpsc::Sender;

use animated_text::*;
use background_color::*;
use clue::*;
use leptos::reactive::spawn_local;
use leptos::svg::Svg;
use leptos_use::UseEventListenerOptions;
use leptos_use::use_timestamp;
use lozenge_entity::*;
use quiz_salad_game_state::*;
use shaped_word_grid_generator::grid_layout::GridLayout;
use shaped_word_grid_generator::grid_layout::Hexagon19RotatedLayout;
use shaped_word_grid_generator::prelude::DesignedLevel;
use shaped_word_grid_generator::prelude::GridTile;
use shaped_word_grid_generator::prelude::Solution;
use state_machine_games::value_watcher::watch_value;
use tile::*;
use web_sys::PointerEvent;
use web_sys::js_sys;

use crate::colors::CLASSIC_COLOR_SCHEME;
use crate::found_answer::FoundAnswerArtifact;
use crate::found_answer::FoundAnswerEntity;
use crate::found_words_state::Completion;
use crate::grid_input::GridInputCommand;
use crate::grid_input::GridInputState;
use crate::layout::*;
use crate::qs_database_handler::QSDatabaseCommand;
use crate::qs_database_handler::SavedLevelState;
use crate::qs_database_handler::handle_db_messages;
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
use shaped_word_grid_generator::{ArrayVec, LevelTrait, Ustr};
use state_machine_games::define_signal_lens;
use state_machine_games::prelude::*;

use crate::{chosen_state::ChosenState, found_words_state::FoundWordsState, puzzle::Puzzle};

pub const NUM_TILES: usize = 19;
pub type LayoutType = Hexagon19RotatedLayout;

pub type LevelType = DesignedLevel<19, LayoutType>;
pub type SolutionType = Solution<19>;
pub type DesignedLevelType = DesignedLevel<19, LayoutType>;

//pub type DisplayWordType = Display

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
const DEFAULT_PUZZLE: &'static str = r#"CREHAUSORLADIOMESTR	Test Puzzle[by mark]	aroma[You might pick it up at a coffee shop]	choir[Ones who agree with you metaphorically]	Christmas[A famous father]	Carol[Number by a door]	crusade[Campaign religiously]	Treasure[Something found at "X"]	measure[Piano Bar]	medal[Come third or better]	salome[Dancer Of The Seven Veils]	tremor[It's a fault's fault]"#;

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

    let state = QuizSaladGameState::new(puzzle.clone(), now);

    let state_signal = RwSignal::new(state);
    let (db_command_sender, db_command_receiver) = async_channel::unbounded::<QSDatabaseCommand>();

    let sender: Sender<Box<dyn GameCommand<QuizSaladGameState> + 'static>> = run_game(state_signal);

    //todo track found words

    Effect::new({
        let db_command_sender = db_command_sender.clone();
        move || {
            let puzzle = puzzle_memo.get();
            db_command_sender
                .try_send(QSDatabaseCommand::LoadLevel(puzzle))
                .unwrap();
        }
    });

    {
        let sender = sender.clone();
        spawn_local(async move {
            handle_db_messages(sender, db_command_receiver).await;
        });
    }

    let current_puzzle_and_found_words = RwSignal::new((
        puzzle_memo.get_untracked(),
        state_signal.get_untracked().found_words,
    ));

    {
        let db_command_sender = db_command_sender.clone();
        Effect::new(move || {
            let state = state_signal.read();
            let (current_puzzle, current_found_words) =
                current_puzzle_and_found_words.get_untracked();
            if state.puzzle != current_puzzle {
                current_puzzle_and_found_words
                    .set((state.puzzle.clone(), state.found_words.clone()));
            }
            if state.found_words != current_found_words {
                current_puzzle_and_found_words
                    .set((state.puzzle.clone(), state.found_words.clone()));

                let elapsed = js_sys::Date::now() - state.start_timestamp;
                let state = SavedLevelState::new(&state.puzzle, &state.found_words, elapsed);
                let db_command_sender = db_command_sender.clone();

                spawn_local(async move {
                    db_command_sender
                        .send(QSDatabaseCommand::SaveLevel(state))
                        .await
                        .expect("Could not send save level command");
                });
            }
        });
    }

    let word_line_segment: Memo<WordLineStateSegment> =
        Memo::new(move |_| WordLineStateSegment::from(&*state_signal.read()));

    let tiles = TileEntity::render_entities::<TileRender>(state_signal.into());
    let tile_texts = TileEntity::render_entities::<TileTextRender>(state_signal.into());
    let clues = ClueEntity::render_entities::<ClueArtifact>(state_signal.into());
    let found_answers =
        FoundAnswerEntity::render_entities::<FoundAnswerArtifact>(state_signal.into());
    let word_line = WordLineEntity::render_singleton::<WordLineArtifact>(word_line_segment.into());

    let lozenges = LozengeEntity::render_entities::<LozengeArtifact>(state_signal.into());
    let animated_text =
        AnimatedTextEntity::render_entities::<AnimatedTextArtifact>(state_signal.into());

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

    let on_pointer_down = {
        let sender = sender.clone();
        move |ev: PointerEvent| {
            on_pointer_down(ev, node_ref, &sender);
        }
    };

    let on_pointer_up = {
        let sender = sender.clone();
        move |ev: PointerEvent| {
            on_pointer_up(ev, node_ref, &sender);
        }
    };

    let on_pointer_move = {
        let sender = sender.clone();
        move |ev: PointerEvent| {
            on_pointer_move(ev, node_ref, &sender);
        }
    };

    let _ = leptos_use::use_event_listener_with_options(
        node_ref,
        leptos::ev::pointerdown,
        on_pointer_down,
        UseEventListenerOptions::default(), //.passive(true),
    );
    let _ = leptos_use::use_event_listener_with_options(
        node_ref,
        leptos::ev::pointerup,
        on_pointer_up,
        UseEventListenerOptions::default(), //.passive(true),
    );
    let _ = leptos_use::use_event_listener_with_options(
        node_ref,
        leptos::ev::pointermove,
        on_pointer_move,
        UseEventListenerOptions::default(), //.passive(true),
    );

    // leptos::task::spawn_local(qs_database_handler::handle_db_messages(
    //     sender.clone(),
    //     db_command_receiver,
    // ));

    let sender1 = sender.clone();
    provide_context(sender);

    view! {

        <div style=div_style>
            <svg node_ref=node_ref viewBox=format!("0 0 {GAME_WIDTH} {GAME_HEIGHT}") style={svg_style}>

            {timer_component(seconds)}

            {title_component(puzzle_memo)}

            <g id="tiles">
            {tiles}
            </g>
            <g id="clues">
            {clues}
            </g>
            <g id="found_answer">
            {found_answers}
            </g>
            <g id="word_line">
                {word_line}
            </g>
            <g id="tile_text">
                {tile_texts}
            </g>
            <g id="lozenges">
            {lozenges}
            </g>
            <g id="animated_text">
            {animated_text}
            </g>

            //todo render the tile text twice with a wordline luminosity mask



            </svg>
        </div>

         <button style="position:fixed;  top: 10px; left: 10px;" on:click=move|_|{
            match sender1.send(Box::new(QuizSaladCommand::ResetLevel)) {
                Ok(()) => {}
                Err(_err) => {
                    leptos::logging::error!("Could not send command");
                }
            }
        }>
            Reset
        </button>

    }
}

fn title_component(puzzle_memo: Memo<Puzzle>) -> impl IntoView {
    // let title_and_attribution_text = move || {
    //     let puzzle = puzzle_memo.get();
    //     let title = puzzle.title.trim();
    //     let (t, a) = if title.is_empty() {
    //         (("Quiz Salad #123"), ("by anonymous"))
    //     } else {
    //         (puzzle.title.as_str(), puzzle.category.as_str())
    //     };

    //     (t.to_string(), a.to_string())
    // };

    view! {
        <text
        style="transform-box: content-box; transform-origin: center;"
        x={LEFT_OFFSET} y="120"
        dominant-baseline="central"
        text-anchor="start"
        fill="#043E40"
        font-size="24"
        font-family="Montserrat"
        font-weight="500"
        pointer-events="none">
            {move ||puzzle_memo.get().name.to_string()}
        </text>


        <text
        style="transform-box: content-box; transform-origin: center;"
        x={LEFT_OFFSET} y="140"
        dominant-baseline="central"
        text-anchor="start"
        fill="#043E40"
        font-size="18"
        font-family="Montserrat"
        font-weight="500"
        pointer-events="none">
            {move ||puzzle_memo.get().extra_info.unwrap_or_default().to_string()}
        </text>
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
        x="500" y="120"
        dominant-baseline="central"
        text-anchor="start"
        fill="#043E40"
        font-size="42"
        font-family="Montserrat"
        font-weight="600"
        pointer-events="none">
            {time_str}
        </text>
    }
}

fn get_tile_from_position(position: Vec2, sensitivity: f32) -> Option<GridTile> {
    let position_adjusted = position
        - Vec2 {
            x: LEFT_OFFSET,
            y: TOP_OFFSET,
        };

    LayoutType::get_tile_from_position(position_adjusted, TILE_RADIUS * 2.0, sensitivity)
}

fn on_pointer_down(
    ev: PointerEvent,
    node_ref: NodeRef<Svg>,
    sender: &Sender<Box<dyn GameCommand<QuizSaladGameState> + 'static>>,
) {
    let Some(element) = node_ref.get() else {
        return;
    };
    let click_position =
        state_machine_games::svg_coordinates::get_svg_coordinates_from_pointer_event(
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

    match sender.send(Box::new(QuizSaladCommand::BoardPointerEvent(command))) {
        Ok(()) => {}
        Err(_err) => {
            leptos::logging::error!("Could not send command");
        }
    }
}

fn on_pointer_up(
    ev: PointerEvent,
    node_ref: NodeRef<Svg>,
    sender: &Sender<Box<dyn GameCommand<QuizSaladGameState> + 'static>>,
) {
    let Some(element) = node_ref.get() else {
        return;
    };
    let click_position =
        state_machine_games::svg_coordinates::get_svg_coordinates_from_pointer_event(
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

    match sender.send(Box::new(QuizSaladCommand::BoardPointerEvent(command))) {
        Ok(()) => {}
        Err(_err) => {
            leptos::logging::error!("Could not send command");
        }
    }
}

fn on_pointer_move(
    ev: PointerEvent,
    node_ref: NodeRef<Svg>,
    sender: &Sender<Box<dyn GameCommand<QuizSaladGameState> + 'static>>,
) {
    if ev.pressure() == 0.0 && ev.pointer_type() != "touch" {
        return;
    }

    let Some(element) = node_ref.get() else {
        return;
    };
    let click_position =
        state_machine_games::svg_coordinates::get_svg_coordinates_from_pointer_event(
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

    match sender.send(Box::new(QuizSaladCommand::BoardPointerEvent(command))) {
        Ok(()) => {}
        Err(_err) => {
            leptos::logging::error!("Could not send command");
        }
    }
}
