pub mod chosen_state;
pub mod found_words_state;
pub mod grid_input;
pub mod puzzle;

use std::time::Duration;
use glam::{FloatExt, Vec2};
use itertools::Itertools;
use leptos::logging::log;
use leptos::prelude::*;
use state_machine_games::leptos::pointer_input_event::Location;
use state_machine_games::leptos::{ prelude::*};
use state_machine_games::prelude::*;
use ws_core::{ArrayVec, Character, LevelTrait, Tile, Tile4x4, Ustr};

use crate::{
    chosen_state::ChosenState, found_words_state::FoundWordsState, grid_input::{GridCommand, GridInputState},
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

    let (game_state_read, game_state_write) = signal(QuizSaladGameState::new(&puzzle));   
    
    let (settings, _) = signal(());
    let (assets, _) = signal(QuizSaladAssets { puzzle });
    let (storage, _) = signal(());

    view! {
        <main>
        {game_view_component(1052.0, 1080.0, game_state_read, game_state_write,  settings, assets, storage)}
        </main>
    }
}

#[derive(Debug)]
pub struct QuizSaladGameState {
    pub current_clue: usize,    
    pub found_words: FoundWordsState,
    pub chosen_state: ChosenState,
    pub word_just_found: bool,
}

impl QuizSaladGameState {
    pub fn new(level: &impl LevelTrait<4, 16>) -> Self {
        let found_words = FoundWordsState::new_from_level(level);

        Self {
            current_clue: Default::default(),
            
            found_words,
            chosen_state: Default::default(),
            word_just_found: false,

        }
    }
}

pub struct QuizSaladAssets {
    pub puzzle: Puzzle,
}
impl GameAssets for QuizSaladAssets {}

#[derive(Debug, PartialEq)]
pub enum QuizGameEntity {

    BackgroundRect,

    ClueText {
        text: Ustr,
    },
    TileRect {
        tile: Tile4x4,
        selected: bool,
        unneeded: bool,
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
        unneeded: bool,
    },
    NextButton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QuizGameEntityKey {
    BackgroundRect,
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
            QuizGameEntity::BackgroundRect => QuizGameEntityKey::BackgroundRect,
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
    type Command = GridCommand;
    type Entity = QuizGameEntity;
    type EntityKey = QuizGameEntityKey;
    type InputState = GridInputState;

    fn get_entities(
        &self,
        _settings: &Self::Settings,
        assets: &Self::Assets,
        _storage: &Self::Storage,
    ) -> impl Iterator<Item = Self::Entity> {
        let rect = [QuizGameEntity::BackgroundRect];
        let buttons = [QuizGameEntity::NextButton].into_iter();

        let text = assets
            .puzzle
            .words
            .get(self.current_clue)
            .and_then(|x| x.clue)
            .unwrap_or_default();

        let unneeded_tiles = self.found_words.unneeded_tiles;

        let clues = [QuizGameEntity::ClueText { text }].into_iter();
        let tiles = Tile::iter_by_row()
        
        .map(move |tile|{
            let selected = self.chosen_state.solution.contains(&tile);
            QuizGameEntity::TileRect { tile, selected, unneeded: unneeded_tiles.get_bit(&tile) }
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


        let tile_texts = assets.puzzle.grid.enumerate().map(move |(tile, &character)| {
            let selected = self.chosen_state.solution.contains(&tile);
            QuizGameEntity::TileText { tile, character, selected, unneeded: unneeded_tiles.get_bit(&tile) } 
        });

        
        rect.into_iter()
        .chain(buttons)
        .chain(clues)
        .chain(tiles)
        .chain(circle)
        .chain(line)
        .chain(tile_texts)
        .chain([QuizGameEntity::NextButton])
    }


    fn apply_command(
        &mut self,
        command: Self::Command,
        _settings: &Self::Settings,
        assets: &Self::Assets,
        _storage: &Self::Storage,
    ) -> MutationResult {
        match command {
            GridCommand::SetChosen(chosen_state) => {
                self.chosen_state = chosen_state;

                match assets.puzzle.check_solution(&self.chosen_state.solution){
                    Some(word_index) => {
                        let completion_index= self.found_words.word_completions.iter().filter(|x|x.is_complete()).count();
                        match self.found_words.word_completions.get_mut(word_index){
                            Some(completion) => {
                                
                                *completion = found_words_state::Completion::Complete { index: completion_index as u8 };
                                self.word_just_found = true;
                                return MutationResult{changed: true, transition_callback_in: Some(Duration::from_millis(1))};
                            },
                            None => {},
                        }
                    },
                    None => {
                        //not a valid solution - do nothing
                    },
                }

            },
        }

        return MutationResult::CHANGED_NO_TRANSITION;
    }

    fn maybe_transition(
        &mut self,
        _settings: &Self::Settings,
        assets: &Self::Assets,
        _storage: &Self::Storage,
    ) -> MutationResult {

        if self.word_just_found{
            self.chosen_state.solution = ArrayVec::new();

            let new_unneeded_tiles = assets.puzzle.calculate_unneeded_tiles(self.found_words.unneeded_tiles, |x| self.found_words.get_completion(x).is_complete());
            self.found_words.unneeded_tiles = new_unneeded_tiles;

            if self.found_words.get_completion(self.current_clue).is_complete(){
                let new_current_clue = (0..assets.puzzle.words.len()).cycle().skip(self.current_clue).take(assets.puzzle.words.len())
                .filter(|&index| !self.found_words.get_completion(index).is_complete()).next();

                self.current_clue = new_current_clue.unwrap_or_default();
            }

            MutationResult::CHANGED_NO_TRANSITION
        }else{
            MutationResult::NO_CHANGE
        }

        
    }
}

impl LeptosGameState for QuizSaladGameState {}

const SCALE: f32 = 258.0;
const TOP_OFFSET: f32 = 10.0;
const LEFT_OFFSET: f32 = 10.0;

pub fn tile_position(tile: Tile4x4, from_centre: bool) -> Vec2 {    
    let rect_size = SCALE * 0.9;

    let mut pos = glam::Vec2 {
        x: tile.x() as f32,
        y: tile.y() as f32,
    };

    if from_centre {
        pos = pos + Vec2::splat(0.45);
    }

    let x = LEFT_OFFSET + (pos.x * SCALE) + ((SCALE - rect_size) * 0.5);
    let y = TOP_OFFSET + (pos.y * SCALE) + ((SCALE - rect_size) * 0.5);

    Vec2 { x, y }
}

pub fn location_to_tile(location: Location) -> Option<Tile4x4>{
    let x = location.x - LEFT_OFFSET;
    let y = location.y - TOP_OFFSET;

    let x_index = x / SCALE;
    let y_index = y / SCALE;

    let x2 = x_index.floor() as u8;
    let y2 = y_index.floor() as u8;

    let tile = Tile4x4::try_new(x2, y2);

    //log!("Event: {x} {y} Tile {tile:?}");

    tile
}
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum QuizSaladInputEvent{
    TileClicked(Tile4x4)
}

impl GameInputEvent<QuizSaladGameState> for QuizSaladInputEvent{
    fn handle_event(
            &self,
            _input_state: &mut <QuizSaladGameState as GameState>::InputState,
            game_state: &QuizSaladGameState,
            _settings: &<QuizSaladGameState as GameState>::Settings,
            _assets: &<QuizSaladGameState as GameState>::Assets,
            _storage: &<QuizSaladGameState as GameState>::Storage,
        ) -> Option<<QuizSaladGameState as GameState>::Command> {
        let QuizSaladInputEvent::TileClicked(clicked_tile) = *self;

        if game_state.found_words.unneeded_tiles.get_bit(&clicked_tile){
            return Some(GridCommand::SetChosen(ChosenState::default()));
        }

        let current_solution = game_state.chosen_state.current_solution();

        let Some(last_tile) = current_solution.last().copied() else{
            return Some(GridCommand::SetChosen(ChosenState{solution: ArrayVec::from_iter([clicked_tile])}));                
        };

        if clicked_tile == last_tile{
            let mut new_solution = current_solution.clone();
            new_solution.pop();
            return Some(GridCommand::SetChosen(ChosenState { solution: new_solution }));
        }

        if let Some(position) = current_solution.iter().position(|&x| x == clicked_tile) {
            let mut new_solution = current_solution.clone();
            new_solution.truncate(position + 1);
            return Some(GridCommand::SetChosen(ChosenState { solution: new_solution }));
        }

        if clicked_tile.is_adjacent_to(&last_tile){
            let mut new_solution = current_solution.clone();
            new_solution.push(clicked_tile);
            return Some(GridCommand::SetChosen(ChosenState { solution: new_solution }));
        }
        else{
            return Some(GridCommand::SetChosen(ChosenState::default()));
        }
    }
}

impl LeptosGameEntity for QuizGameEntity {
    fn render(
        &self,
        meta: &StoredEntityMeta<Self>,
        sender: CommandSender<Self::GameState>,
        start_time: f64,
        current_time: Signal<f64>,
    ) -> AnyView {
        
        const FONT_SIZE: f32 = SCALE * 0.5;
        const RECT_SIZE: f32 = SCALE * 0.9;
        const PATH_STROKE_WIDTH: f32 = SCALE * 0.5;
        const RADIUS: f32 = RECT_SIZE * 0.05;
        const FILL_COLOR : &'static str = "#97FCFF";
        const CLUE_FONT_SIZE: f32 = 40.0;    
        const FONT_FAMILY: &'static str = "Montserrat";
        const SELECTED_TEXT_COLOR: &'static str = "#FFFFFF";
        const UNSELECTED_TEXT_COLOR: &'static str = "#202251";

        match self {
            QuizGameEntity::BackgroundRect => {
                view! {
                    <rect x=0 y=0 width="100%" height = "100%" fill = "#EECCCC">
                    </rect>
                }.into_any()
            }


            QuizGameEntity::ClueText { text } => view! {
                <text x=0 y=20 font-size=CLUE_FONT_SIZE font-weight="600" font-family={FONT_FAMILY} dominant-baseline="central" style="text-align: left; text-anchor: left;">{text.to_string()}</text>
            }
            .into_any(),
            QuizGameEntity::TileRect {
                tile,                
                selected:_,
                unneeded
            } => {
                let Vec2 { x, y } = tile_position(*tile, false);              
                let sender = sender.clone();
                let tile = *tile;

                let scale = if *unneeded {0.0} else {1.0};
                let style = format!("transform: scale({scale}) ; transform-box: content-box; transform-origin: center center; transition: transform 1s;");

                view! {
                    <rect width={RECT_SIZE} height={RECT_SIZE} x={x} y={y} rx={RADIUS} ry={RADIUS} fill={FILL_COLOR} style=style
                    
                    on:click={move|_|{
                        sender.handle_game_input_event(QuizSaladInputEvent::TileClicked(tile)); 
                    }}
                    
                    
                    >  </rect>
                    
                }.into_any()
            },
            QuizGameEntity::TileText {
                tile,
                character,
                selected,
                unneeded
            } => {
                let Vec2 { x, y } = tile_position(*tile, true);              
                let color = if *selected {SELECTED_TEXT_COLOR} else {UNSELECTED_TEXT_COLOR};

                let scale = if *unneeded {0.0} else {1.0};
                let style = format!("transform: scale({scale}) ; transform-box: content-box; transform-origin: center center; transition: transform 1s;");

                view! {
                    <text x={x} y={y} style=style dominant-baseline="central" text-anchor="middle" fill={color} font-size={FONT_SIZE} font-family={FONT_FAMILY} font-weight={600} pointer-events="none">{character.as_char()} </text>
                    
                }.into_any()
            },
            QuizGameEntity::WordLineSingleCircle { tile }=>{
                let Vec2 { x: x1, y: y1 } = tile_position(*tile, true);

                let color = wordline_color(0);
                view!{
                    <circle cx={x1} cy={y1} fill={color} r={PATH_STROKE_WIDTH * 0.5} pointer-events="none"/>
                }.into_any()

            }
            QuizGameEntity::WordLineSegment { index:_, t1, t2, segment_index }=>{
                let v1 = tile_position(*t1, true);
                let v2 = tile_position(*t2, true);
                // let length = v1.distance(v2);
                let Vec2 { x: x1, y: y1 } = v1;
                let Vec2 { x: x2, y: y2 } = v2;
                let color = wordline_color(*segment_index as usize);

                let stroke_width = if meta.is_dead(){
                    0.0
                } else{
                    PATH_STROKE_WIDTH
                };
                let is_new = meta.is_new();

                let (x_from, y_from)  = if is_new{
                    (x1, y1)
                }else{
                    (x2, y2)
                };

                let x2_animated = animate_value(start_time, current_time, x_from, x2, 500.0, Lerp32);
                let y2_animated = animate_value(start_time, current_time, y_from, y2, 500.0, Lerp32);

                //todo nice line disappear
                //todo line pulsing if close to the answer


                view!{
                        <line x1={x1} y1={y1} x2={x2_animated} y2={y2_animated} visibility="visible" stroke={color} stroke-linecap="round" stroke-width={stroke_width} pointer-events="none" >
                            // {animate_x2_y2}
                            // {animate_stroke_width}
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