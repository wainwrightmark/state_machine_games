pub mod chosen_state;
pub mod found_words_state;
pub mod grid_input;
pub mod puzzle;

use std::time::Duration;

use glam::Vec2;
use itertools::Itertools;
use leptos::prelude::*;

use state_machine_games::leptos::prelude::*;
use state_machine_games::prelude::*;

use ws_core::{Character, GridTile, LevelTrait, Solution4x4, Tile, Tile4x4, Ustr};

use crate::{
    chosen_state::ChosenState, found_words_state::FoundWordsState, grid_input::GridInputState,
    puzzle::Puzzle,
};

pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    mount_to_body(app);
}

pub fn app() -> impl IntoView {
    let puzzle = Puzzle::from_tsv_line(
        "LYCAWAURSGPIHOND	Super Salad 10	Song[A word that might follow 'bird' or 'love']	Capri[An Italian island]	Lycra[A synthetic fibre]	Pugwash[A cartoon captain]	Lagos[A Nigerian city]	Dinosaur[A way to describe an out of touch person] 	Posh[One of the Spice Girls]	Dingo[An Australian animal]	Cupid[A romantic messenger]	Pugh[A famous florence]",
    )
    .unwrap();

    let (state_signal, write_signal) = signal(QuizSaladGameState::new(&puzzle));
    let (settings, _) = signal(());
    let (assets, _) = signal(QuizSaladAssets { puzzle });
    let (storage, _) = signal(());

    view! {
        <main>
        {game_view_component(1920.0, 1080.0, state_signal, write_signal, settings, assets, storage)}
        </main>
    }
}

#[derive(Debug)]
pub struct QuizSaladGameState {
    pub current_clue: usize,
    pub grid_input_state: GridInputState,
    pub found_words: FoundWordsState,
    pub chosen_state: ChosenState,
}

impl QuizSaladGameState {
    pub fn new(level: &impl LevelTrait<4, 16>) -> Self {
        let found_words = FoundWordsState::new_from_level(level);

        Self {
            current_clue: Default::default(),
            grid_input_state: Default::default(),
            found_words,
            chosen_state: Default::default(),
        }
    }
}

pub struct QuizSaladAssets {
    pub puzzle: Puzzle,
}
impl GameAssets for QuizSaladAssets {}

#[derive(Debug, Clone)]
pub enum QuizSaladCommand {
    InputStart(Tile4x4),
    InputMove(Tile4x4),
    InputEnd(Tile4x4),
    InputStartNoLocation,
    InputEndNoLocation,
}

impl GameCommand for QuizSaladCommand {}

#[derive(Debug, PartialEq)]
pub enum QuizGameEntity {
    ClueText {
        text: Ustr,
    },
    TileRect {
        tile: Tile4x4,
        selected: bool,
    },
    WordLineSingleCircle{
        tile: Tile4x4,
    },

    WordLineSegment {
        index: u8,
        segment_index: u8,
        t1: Tile4x4,
        t2: Tile4x4,
    },
    TileText {
        tile: Tile4x4,
        character: Character,
        selected: bool,
    },
    NextButton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QuizGameEntityKey {
    ClueText,
    TileRect(Tile4x4),
    WordLineSingleCircle(Tile4x4),
    WordLineSegment(u8),
    TileText(Tile4x4),    
    NextButton,
}

impl GameEntity for QuizGameEntity {
    type GameState = QuizSaladGameState;

    fn key(&self) -> <<Self as GameEntity>::GameState as GameState>::EntityKey {
        match self {
            QuizGameEntity::ClueText { .. } => QuizGameEntityKey::ClueText,
            QuizGameEntity::TileRect { tile, .. } => QuizGameEntityKey::TileRect(*tile),
            QuizGameEntity::WordLineSingleCircle { tile } => QuizGameEntityKey::WordLineSingleCircle(*tile),
            QuizGameEntity::WordLineSegment { index, .. } => QuizGameEntityKey::WordLineSegment(*index),
            QuizGameEntity::TileText { tile, .. } => QuizGameEntityKey::TileText(*tile),
            QuizGameEntity::NextButton => QuizGameEntityKey::NextButton,
        }
    }

    fn death_duration(&self) -> Option<Duration> {
        None
    }
}

impl GameEntityKey for QuizGameEntityKey {}

impl GameState for QuizSaladGameState {
    type Settings = ();
    type Assets = QuizSaladAssets;
    type Storage = ();
    type Command = QuizSaladCommand;
    type Entity = QuizGameEntity;
    type EntityKey = QuizGameEntityKey;

    fn get_entities(
        &self,
        _settings: &Self::Settings,
        assets: &Self::Assets,
        _storage: &Self::Storage,
    ) -> impl Iterator<Item = Self::Entity> {
        let buttons = [QuizGameEntity::NextButton].into_iter();

        let text = assets
            .puzzle
            .words
            .get(self.current_clue)
            .and_then(|x| x.clue)
            .unwrap_or_default();

        let clues = [QuizGameEntity::ClueText { text }].into_iter();
        let tiles = Tile::iter_by_row().map(|tile|{
            let selected = self.chosen_state.solution.contains(&tile);
            QuizGameEntity::TileRect { tile, selected }
        });
                
        let circle = if self.chosen_state.solution.len() == 1{
            Some(
                QuizGameEntity::WordLineSingleCircle { tile: self.chosen_state.solution[0] }
            )
        }else {
            None
        };

        let mut segment_index1:u8 = 0;
        let mut last_relative_point = None;

        let line = self.chosen_state.solution.iter().tuple_windows().enumerate() .map(move |(index, (t1,t2))|{

            let relative_point = Some(tile_position(*t1, true) - tile_position(*t2, true));
            let segment_index = segment_index1;
            if Some(relative_point) != last_relative_point {
                last_relative_point = Some(relative_point);
                segment_index1 = segment_index1.wrapping_add(1);
            }            
            
            QuizGameEntity::WordLineSegment { index: index as u8, segment_index, t1: *t1, t2: *t2 }
        });


        let tile_texts = assets.puzzle.grid.enumerate().map(|(tile, &character)| {
            let selected = self.chosen_state.solution.contains(&tile);
            QuizGameEntity::TileText { tile, character, selected } 
        });

        buttons.chain(clues).chain(tiles).chain(circle).chain(line).chain(tile_texts).chain([QuizGameEntity::NextButton])
    }

    fn apply_command(
        &mut self,
        command: Self::Command,
        _settings: &Self::Settings,
        assets: &Self::Assets,
        _storage: &Self::Storage,
    ) -> MutationResult {
        match command {
            QuizSaladCommand::InputStart(tile) => {
                self.grid_input_state.handle_input_start(
                    &mut self.chosen_state,
                    tile,
                    &assets.puzzle.grid,
                    &self.found_words,
                );
            }
            QuizSaladCommand::InputMove(tile) => {
                self.grid_input_state.handle_input_move(
                    &mut self.chosen_state,
                    tile,
                    &assets.puzzle.grid,
                    &self.found_words,
                );
            }
            QuizSaladCommand::InputEnd(tile) => {
                self.grid_input_state
                    .handle_input_end(&mut self.chosen_state, tile);
            }
            QuizSaladCommand::InputStartNoLocation => {
                self.grid_input_state.handle_input_start_no_location();
            }
            QuizSaladCommand::InputEndNoLocation => {
                self.grid_input_state.handle_input_end_no_location(
                    &mut self.chosen_state,
                    self.found_words.is_level_complete(),
                );
            }
        }

        return MutationResult::CHANGED_NO_TRANSITION;
    }

    fn maybe_transition(
        &mut self,
        _settings: &Self::Settings,
        _assets: &Self::Assets,
        _storage: &Self::Storage,
    ) -> MutationResult {
        MutationResult::NO_CHANGE
    }
}

impl LeptosGameState for QuizSaladGameState {}

pub fn tile_position(tile: Tile<4, 4>, from_centre: bool) -> Vec2 {
    let scale = 258.0;
    //let font_size = scale * 0.5;
    let rect_size = scale * 0.9;

    let mut pos = glam::Vec2 {
        x: tile.x() as f32,
        y: tile.y() as f32,
    };

    if from_centre {
        pos = pos + Vec2::splat(0.45);
    }

    let x = 10.0 + (pos.x * scale) + ((scale - rect_size) * 0.5);
    let y = 10.0 + (pos.y * scale) + ((scale - rect_size) * 0.5);

    Vec2 { x, y }
}

impl LeptosGameEntity for QuizGameEntity {
    fn render(
        &self,
        meta: &StoredEntityMeta<Self>,
        sender: CommandSender<Self::GameState>,
    ) -> AnyView {
        const SCALE: f32 = 258.0;
        const FONT_SIZE: f32 = SCALE * 0.5;
        const RECT_SIZE: f32 = SCALE * 0.9;
        const PATH_STROKE_WIDTH: f32 = SCALE * 0.5;
        const RADIUS: f32 = RECT_SIZE * 0.05;
        const FILL_COLOR : &'static str = "#97FCFF";
        const CLUE_FONT_SIZE: f32 = 40.0;
        



        const FONT_FAMILY: &'static str = "Montserrat";
        // const FONT_COLOR: &'static str =  "#000000";
        // const BOX_FONT_COLOR: &'static str =  "#000000";
        // const BOX_FONT_COLOR: &'static str =  "#000000";

        // const UNSELECTED_FILL_COLOR: &'static str = "#111111";
        // const SELECTED_FILL_COLOR: &'static str = "#333333";

        const SELECTED_TEXT_COLOR: &'static str = "#FFFFFF";
        const UNSELECTED_TEXT_COLOR: &'static str = "#202251";
        
        

        //let width = ((SIDE_LENGTH as f32) + 1.0) * RECT_SIZE;
        //let height = RECT_SIZE * ((GRID_TOP_OFFSET + GRID_WORDS_GAP) + (SIDE_LENGTH as f32));

        match self {
            QuizGameEntity::ClueText { text } => view! {
                <text x=0 y=20 font-size=CLUE_FONT_SIZE font-weight="600" font-family={FONT_FAMILY} dominant-baseline="central" style="text-align: left; text-anchor: left;">{text.to_string()}</text>
            }
            .into_any(),
            QuizGameEntity::TileRect {
                tile,                
                selected:_,
            } => {
                let Vec2 { x, y } = tile_position(*tile, false);              
                let sender = sender.clone();
                let tile = *tile;

                view! {
                    <rect width={RECT_SIZE} height={RECT_SIZE} x={x} y={y} rx={RADIUS} ry={RADIUS} fill={FILL_COLOR} on:click={move|_|{
                        sender.send_command(QuizSaladCommand::InputStart(tile));
                    }}>  </rect>
                    
                }.into_any()
            },
            QuizGameEntity::TileText {
                tile,
                character,
                selected,
            } => {
                let Vec2 { x, y } = tile_position(*tile, true);              
                let color = if *selected {SELECTED_TEXT_COLOR} else {UNSELECTED_TEXT_COLOR};               

                view! {
                    <text x={x} y={y} dominant-baseline="central" text-anchor="middle" fill={color} font-size={FONT_SIZE} font-family={FONT_FAMILY} font-weight={600} pointer-events="none">{character.as_char()} </text>
                    
                }.into_any()
            },
            QuizGameEntity::WordLineSingleCircle { tile }=>{
                let Vec2 { x: x1, y: y1 } = tile_position(*tile, true);

                let color = wordline_color(0);
                view!{
                    <circle cx={x1} cy={y1} fill={color} r=25 pointer-events="none"/>
                }.into_any()

            }
            QuizGameEntity::WordLineSegment { index:_, t1, t2, segment_index }=>{
                let v1 = tile_position(*t1, true);
                let v2 = tile_position(*t2, true);
                //let length = v1.distance(v2);
                let Vec2 { x: x1, y: y1 } = v1;
                let Vec2 { x: x2, y: y2 } = v2;

                let color = wordline_color(*segment_index as usize);

                view!{
                    <line x1={x1} y1={y1} x2={x2} y2={y2} visibility="visible" stroke={color} stroke-linecap="round" stroke-width={PATH_STROKE_WIDTH} pointer-events="none">            
                    </line>
                }.into_any()

            }
            QuizGameEntity::NextButton => {
                view! {
                    <rect />

                }.into_any()
            },
        }
    }
}


fn wordline_color(index: usize) -> &'static str {
    match index % 12 {
        0 => "#006AFF",
        1 => "#0015FF",
        2 => "#4000FF",
        3 => "#9500FF",
        4 => "#EA00FF",
        5 => "#FF00BF",
        6 => "#FF006A",
        7 => "#FF0015",
        8 => "#FF4000",
        9 => "#FF9D00",
        10 => "#E8AE00",
        11 | _ => "#C9B900",
    }
}