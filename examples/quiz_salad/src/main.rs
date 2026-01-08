pub mod animated_text;
pub mod chosen_state;
pub mod clue;
pub mod colors;
pub mod found_words_state;
pub mod grid_input;
pub mod layout;
pub mod layout_util;
pub mod lozenge_entity;
pub mod puzzle;
pub mod quiz_salad_command;
pub mod quiz_salad_game_state;
pub mod tile;
pub mod util;
pub mod word_line;
pub mod background_color;

use animated_text::*;
use clue::*;
use lozenge_entity::*;
use quiz_salad_game_state::*;
use tile::*;
use word_line::*;
use background_color::*;

use crate::colors::CLASSIC_COLOR_SCHEME;
use crate::found_words_state::Completion;
use crate::layout::*;
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

type Stores = (
    ArcRwSignal<SingleTypeEntityStore<TileEntity>>,
    ArcRwSignal<SingleTypeEntityStore<ClueEntity>>,    
    ArcRwSignal<SingleTypeEntityStore<WordLineSectionEntity>>,
    ArcRwSignal<SingleTypeEntityStore<LozengeEntity>>,
    ArcRwSignal<SingleTypeEntityStore<AnimatedTextEntity>>,
    ResourceStore<BackgroundColor>
);


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
    // Effect::new(||{
    //     let puzzle = puzzle_memo.get();
    // });
    let puzzle = puzzle_memo.get_untracked();

    let state = QuizSaladGameState::new(puzzle);

    let stores = Stores::new(&state);

    let tiles_store1 = stores.0.clone();
    let tiles_store2 = stores.0.clone();
    let clues_store = stores.1.clone();
    let word_line_store = stores.2.clone();
    let lozenge_store = stores.3.clone();
    let animated_text_store = stores.4.clone();
    let background_color = stores.5.artifact.color;

    let machine = GameMachine::new(state, stores);

    let cs1 = machine.command_sender();
    let cs2 = machine.command_sender();

    machine.run_game();

    
    let svg_style = move ||{       

        format!("max-width: 100%; max-height: 100%; user-select:none; position:fixed; margin:auto; inset: 0px; background: {}", background_color.get().to_hex())
    };

    view! {
        <div style="height: 100vh; width: 100vw; overflow: hidden;">
            <svg viewBox=format!("0 0 {GAME_WIDTH} {GAME_HEIGHT}") style={svg_style}>


            {move || SingleTypeEntityStore::render(tiles_store1.clone(),RenderTileFill, cs1.clone())} // Tile
            {move || SingleTypeEntityStore::render(clues_store.clone(),(), ())} 
            {move || SingleTypeEntityStore::render(word_line_store.clone(),(), ())}
            {move || SingleTypeEntityStore::render(tiles_store2.clone(),RenderTileText, ())} 
            {move || SingleTypeEntityStore::render(lozenge_store.clone(),(), cs2.clone())}
            {move || SingleTypeEntityStore::render(animated_text_store.clone(),(), ())} // Animated Text

            </svg>
            // <div>
            //     {move || SingleTypeEntityStore::render(stores.4.clone(), button_sender.clone())}
            // </div>
        </div>

    }
}
