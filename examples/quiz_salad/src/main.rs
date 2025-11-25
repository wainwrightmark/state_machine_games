pub mod chosen_state;
pub mod found_words_state;
pub mod grid_input;
pub mod puzzle;

use std::sync::mpsc;

use itertools::Itertools;
use leptos::prelude::*;
use state_machine_games::prelude::*;
use state_machine_games::{define_signal_lens, prelude::*};
use ws_core::{ArrayVec, Character, LevelTrait, Tile, Tile4x4, Ustr};

use crate::{chosen_state::ChosenState, found_words_state::FoundWordsState, puzzle::Puzzle};

type Stores = (
    ArcRwSignal<SingleTypeEntityStore<ClueEntity>>,
    ArcRwSignal<SingleTypeEntityStore<TileRectEntity>>,
    ArcRwSignal<SingleTypeEntityStore<WordLineSectionEntity>>,
    ArcRwSignal<SingleTypeEntityStore<TileTextEntity>>,
    ArcRwSignal<SingleTypeEntityStore<NextButtonArtifact>>,
);

pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    mount_to_body(app);
}

pub fn app() -> impl IntoView {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    mount_to_body(|| game_component());
}

fn game_component() -> impl IntoView {
    let puzzle = Puzzle::from_tsv_line(
        "LYCAWAURSGPIHOND	Super Salad 10	Song[A word that might follow 'bird' or 'love']	Capri[An Italian island]	Lycra[A synthetic fibre]	Pugwash[A cartoon captain]	Lagos[A Nigerian city]	Dinosaur[A way to describe an out of touch person] 	Posh[One of the Spice Girls]	Dingo[An Australian animal]	Cupid[A romantic messenger]	Pugh[A famous florence]",
    )
    .unwrap();

    let state = QuizSaladGameState::new(puzzle);

    let (click_sender, click_receiver) = mpsc::channel::<QuizSaladCommand>();
    let (button_sender, button_receiver) = mpsc::channel::<NextButtonArtifact>();
    let stores = Stores::default();
    let machine = GameMachine::new(state, stores.clone(), (click_receiver, button_receiver));

    machine.run_game();

    view! {
        <svg viewBox="0 0 1052.0 1080.0"  style="max-width: 800px;  margin-inline: auto; ">
        {move || SingleTypeEntityStore::render(stores.0.clone(), ())}
        {move || SingleTypeEntityStore::render(stores.1.clone(), click_sender.clone())}
        {move || SingleTypeEntityStore::render(stores.2.clone(), ())}
        {move || SingleTypeEntityStore::render(stores.3.clone(), ())}

        </svg>
        <div>
            {move || SingleTypeEntityStore::render(stores.4.clone(), button_sender.clone())}
        </div>

    }
}

#[derive(Debug)]
pub struct QuizSaladGameState {
    pub puzzle: Puzzle,
    pub current_clue: usize,
    pub found_words: FoundWordsState,
    pub chosen_state: ChosenState,
    pub word_just_found: bool,
}

impl QuizSaladGameState {
    pub fn new(puzzle: Puzzle) -> Self {
        let found_words = FoundWordsState::new_from_level(&puzzle);

        Self {
            current_clue: Default::default(),
            puzzle,
            found_words,
            chosen_state: Default::default(),
            word_just_found: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WordLineArtifact {
    pub v1: Vec2,
    pub v2: RwSignal<Vec2>,
    pub segment_index: usize,
    pub stroke_width_ratio: RwSignal<f32>,
}

//define_signal_lens!(WordLineArtifactV1Lens, WordLineArtifact, Vec2, v1);
define_signal_lens!(WordLineArtifactV2Lens, WordLineArtifact, Vec2, v2);
// define_signal_lens!(
//     WordLineArtifactSegmentIndexLens,
//     WordLineArtifact,
//     usize,
//     segment_index
// );
define_signal_lens!(
    WordLineArtifactStrokeWidthRatioLens,
    WordLineArtifact,
    f32,
    stroke_width_ratio
);

impl GameArtifact for WordLineArtifact {
    type Command = ();

    fn render(self, _sender: impl CommandSender<Self::Command>) -> impl IntoView {
        // let v1 = tile_position(*t1, true);
        // let v2 = tile_position(*t2, true);
        // // let length = v1.distance(v2);
        // let Vec2 { x: x1, y: y1 } = v1;
        // let Vec2 { x: x2, y: y2 } = v2;
        // let color = wordline_color(*segment_index as usize);

        // let stroke_width = if meta.is_dead() {
        //     0.0
        // } else {
        //     PATH_STROKE_WIDTH
        // };
        // let is_new = meta.is_new();

        // let (x_from, y_from) = if is_new { (x1, y1) } else { (x2, y2) };

        // let x2_animated = animate_value(start_time, current_time, x_from, x2, 500.0, Lerp32);
        // let y2_animated = animate_value(start_time, current_time, y_from, y2, 500.0, Lerp32);

        //todo nice line disappear
        //todo line pulsing if close to the answer

        view! {
            <line
             x1= self.v1.x
             y1= self.v1.y
             x2={move || self.v2.get().x}
             y2={move || self.v2.get().y}
             visibility="visible"
             stroke={ wordline_color(self.segment_index as usize)}
             stroke-linecap="round"
             stroke-width={move ||{self.stroke_width_ratio.get() * PATH_STROKE_WIDTH}}
             pointer-events="none" >
                // {animate_x2_y2}
                // {animate_stroke_width}
            </line>
        }

        // QuizGameEntity::WordLineSingleCircle { tile } => {
        //         let Vec2 { x: x1, y: y1 } = tile_position(*tile, true);

        //         let color = wordline_color(0);
        //         view!{
        //             <circle cx={x1} cy={y1} fill={color} r={PATH_STROKE_WIDTH * 0.5} pointer-events="none"/>
        //         }.into_any()
        //     }
        //     QuizGameEntity::WordLineSegment {
        //         index: _,
        //         t1,
        //         t2,
        //         segment_index,
        //     } => {

        //     }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WordLineSectionEntity {
    Circle {
        tile: Tile4x4,
    },
    LineSegment {
        index: u8,
        segment_index: u8,
        t1: Tile4x4,
        t2: Tile4x4,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WordLineSectionKey {
    index: u8,
    tile1: Tile4x4,
}

impl GameEntityKey for WordLineSectionKey {}

impl GameEntityKey for WordLineSectionEntity {}

impl GameEntity for WordLineSectionEntity {
    type Artifact = WordLineArtifact;
    type Key = WordLineSectionKey;
    type GameState = QuizSaladGameState;

    fn key(&self) -> Self::Key {
        match self {
            WordLineSectionEntity::Circle { tile } => WordLineSectionKey {
                index: 0,
                tile1: *tile,
            },
            WordLineSectionEntity::LineSegment { index, t1, .. } => WordLineSectionKey {
                index: *index,
                tile1: *t1,
            },
        }
    }

    fn get_entities(game_state: &Self::GameState) -> impl Iterator<Item = Self> {
        let circle = if game_state.chosen_state.solution.len() == 1 {
            Some(WordLineSectionEntity::Circle {
                tile: game_state.chosen_state.solution[0],
            })
        } else {
            None
        };

        let mut segment_index1: u8 = 0;
        let mut last_relative_point = None;

        let line = game_state
            .chosen_state
            .solution
            .iter()
            .tuple_windows()
            .enumerate()
            .map(move |(index, (t1, t2))| {
                let relative_point = Some(tile_position(*t1, true) - tile_position(*t2, true));
                let segment_index = segment_index1;
                if Some(relative_point) != last_relative_point {
                    last_relative_point = Some(relative_point);
                    segment_index1 = segment_index1.wrapping_add(1);
                }

                WordLineSectionEntity::LineSegment {
                    index: index as u8,
                    segment_index,
                    t1: *t1,
                    t2: *t2,
                }
            });

        return circle.into_iter().chain(line);
    }

    fn on_death(&self, artifact: &mut Self::Artifact) -> AnimationList<Self::Artifact> {
        vec![animate_towards::<WordLineArtifactStrokeWidthRatioLens>(
            0.0,
            1.0 / 1000.0,
        )]
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        let v1: Vec2; //= tile_position(self.v1, true);
        let v2: Vec2; //= tile_position(self.v2, true);
        let si: u8;
        let initial_width_ratio: f32;

        match self {
            WordLineSectionEntity::Circle { tile } => {
                si = 0;
                v1 = tile_position(*tile, true);
                v2 = tile_position(*tile, true);
                initial_width_ratio = 0.0;
            }
            WordLineSectionEntity::LineSegment {
                segment_index,
                t1,
                t2,
                ..
            } => {
                si = *segment_index;
                v1 = tile_position(*t1, true);
                v2 = tile_position(*t2, true);
                initial_width_ratio = 1.0;
            }
        }

        let artifact = Self::Artifact {
            v1,
            v2: RwSignal::new(v1), //yes - start with v1
            segment_index: si as usize,
            stroke_width_ratio: RwSignal::new(initial_width_ratio),
        };

        let animations = vec![
            animate_towards::<WordLineArtifactV2Lens>(v2, 1.0),
            animate_towards::<WordLineArtifactStrokeWidthRatioLens>(1.0, 1.0 / 250.0),
        ];
        (artifact, animations)
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        former_entity_state: EntityState,
    ) -> AnimationList<Self::Artifact> {
        let v1: Vec2; //= tile_position(self.v1, true);
        let v2: Vec2; //= tile_position(self.v2, true);
        let si: u8;
        let initial_width_ratio: f32;

        match self {
            WordLineSectionEntity::Circle { tile } => {
                si = 0;
                v1 = tile_position(*tile, true);
                v2 = tile_position(*tile, true);
                initial_width_ratio = 0.0;
            }
            WordLineSectionEntity::LineSegment {
                segment_index,
                t1,
                t2,
                ..
            } => {
                si = *segment_index;
                v1 = tile_position(*t1, true);
                v2 = tile_position(*t2, true);
                initial_width_ratio = 1.0;
            }
        }

        let animations = vec![
            animate_towards::<WordLineArtifactV2Lens>(v2, 1.0),
            animate_towards::<WordLineArtifactStrokeWidthRatioLens>(1.0, 1.0 / 250.0),
        ];

        animations
    }
}

impl GameState for QuizSaladGameState {
    fn maybe_transition(&mut self) -> MutationResult {
        if self.word_just_found {
            self.chosen_state.solution = ArrayVec::new();

            let new_unneeded_tiles = self
                .puzzle
                .calculate_unneeded_tiles(self.found_words.unneeded_tiles, |x| {
                    self.found_words.get_completion(x).is_complete()
                });
            self.found_words.unneeded_tiles = new_unneeded_tiles;

            if self
                .found_words
                .get_completion(self.current_clue)
                .is_complete()
            {
                let new_current_clue = (0..self.puzzle.words.len())
                    .cycle()
                    .skip(self.current_clue)
                    .take(self.puzzle.words.len())
                    .filter(|&index| !self.found_words.get_completion(index).is_complete())
                    .next();

                self.current_clue = new_current_clue.unwrap_or_default();
            }

            MutationResult::CHANGED_NO_TRANSITION
        } else {
            MutationResult::NO_CHANGE
        }
    }
}

const SCALE: f32 = 258.0;
const TOP_OFFSET: f32 = 10.0;
const LEFT_OFFSET: f32 = 10.0;

pub fn tile_position(tile: Tile4x4, from_centre: bool) -> Vec2 {
    let rect_size = SCALE * 0.9;

    let mut pos = Vec2 {
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

// pub fn location_to_tile(location: Location) -> Option<Tile4x4>{
//     let x = location.x - LEFT_OFFSET;
//     let y = location.y - TOP_OFFSET;

//     let x_index = x / SCALE;
//     let y_index = y / SCALE;

//     let x2 = x_index.floor() as u8;
//     let y2 = y_index.floor() as u8;

//     let tile = Tile4x4::try_new(x2, y2);

//     //log!("Event: {x} {y} Tile {tile:?}");

//     tile
// }
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum QuizSaladCommand {
    TileClicked(Tile4x4),
}

impl AnyGameCommand for QuizSaladCommand {}

impl GameCommand<QuizSaladGameState> for QuizSaladCommand {
    fn apply_command(&self, game_state: &mut QuizSaladGameState) -> MutationResult {
        let QuizSaladCommand::TileClicked(clicked_tile) = *self;
        if game_state.found_words.unneeded_tiles.get_bit(&clicked_tile) {
            game_state.chosen_state = ChosenState::default();
            return MutationResult::CHANGED_NO_TRANSITION;
        }

        let current_solution = game_state.chosen_state.current_solution();

        let Some(last_tile) = current_solution.last().copied() else {
            game_state.chosen_state = ChosenState {
                solution: ArrayVec::from_iter([clicked_tile]),
            };
            return MutationResult::CHANGED_NO_TRANSITION;
        };

        if clicked_tile == last_tile {
            let mut new_solution = current_solution.clone();
            new_solution.pop();
            game_state.chosen_state = ChosenState {
                solution: new_solution,
            };
            return MutationResult::CHANGED_NO_TRANSITION;
        }

        if let Some(position) = current_solution.iter().position(|&x| x == clicked_tile) {
            let mut new_solution = current_solution.clone();
            new_solution.truncate(position + 1);
            game_state.chosen_state = ChosenState {
                solution: new_solution,
            };

            return MutationResult::CHANGED_NO_TRANSITION;
        }

        if clicked_tile.is_adjacent_to(&last_tile) {
            let mut new_solution = current_solution.clone();
            new_solution.push(clicked_tile);
            game_state.chosen_state = ChosenState {
                solution: new_solution,
            };
            if let Some(solution_index) = game_state.puzzle.check_solution(&game_state.chosen_state.solution){
                if !game_state.found_words.get_completion(solution_index).is_complete(){
                    game_state.word_just_found = true;
                    return MutationResult::changed_with_transition(500.0);
                }
            }

            return MutationResult::CHANGED_NO_TRANSITION;
            
        } else {
            game_state.chosen_state = ChosenState::default();

            return MutationResult::CHANGED_NO_TRANSITION;
        }
    }
}

const FONT_SIZE: f32 = SCALE * 0.5;
const RECT_SIZE: f32 = SCALE * 0.9;
const PATH_STROKE_WIDTH: f32 = SCALE * 0.5;
const RADIUS: f32 = RECT_SIZE * 0.05;
const FILL_COLOR: &'static str = "#97FCFF";
const CLUE_FONT_SIZE: f32 = 40.0;
const FONT_FAMILY: &'static str = "Montserrat";
const SELECTED_TEXT_COLOR: &'static str = "#FFFFFF";
const UNSELECTED_TEXT_COLOR: &'static str = "#202251";

#[derive(Debug, Clone)]
pub struct ClueArtifact {
    pub text: RwSignal<Ustr>,
}

impl GameArtifact for ClueArtifact {
    type Command = ();

    fn render(self, _sender: impl CommandSender<Self::Command>) -> impl IntoView {
        view! {
            <text x=0 y=20
            font-size=CLUE_FONT_SIZE
            font-weight="600"
            font-family={FONT_FAMILY}
            dominant-baseline="central"
            style="text-align: left; text-anchor: left;">{
                move ||self.text.get().to_string()
            }</text>
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ClueEntity {
    pub text: Ustr,
}

impl GameEntity for ClueEntity {
    type Artifact = ClueArtifact;
    type Key = ();
    type GameState = QuizSaladGameState;

    fn key(&self) -> Self::Key {
        ()
    }

    fn get_entities(game_state: &Self::GameState) -> impl Iterator<Item = Self> {
        let text = game_state
            .puzzle
            .words
            .get(game_state.current_clue)
            .and_then(|x| x.clue)
            .unwrap_or_default();

        [Self { text }].into_iter()
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        (
            Self::Artifact {
                text: RwSignal::new(self.text),
            },
            vec![],
        )
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        former_entity_state: EntityState,
    ) -> AnimationList<Self::Artifact> {
        artifact.text.set(self.text);
        vec![]
    }
}

#[derive(Debug, Clone)]
pub struct TileRectArtifact {
    pub tile: Tile4x4,
    pub scale: RwSignal<f32>,
}

define_signal_lens!(TileRectArtifactScaleLens, TileRectArtifact, f32, scale);
define_signal_lens!(TileTextArtifactScaleLens, TileTextArtifact, f32, scale);

impl GameArtifact for TileRectArtifact {
    type Command = QuizSaladCommand;

    fn render(self, sender: impl CommandSender<Self::Command>) -> impl IntoView {
        let Vec2 { x, y } = tile_position(self.tile, false);

        let tile = self.tile;

        //let style = format!("transform: scale({scale}) ; transform-box: content-box; transform-origin: center center; transition: transform 1s;");

        view! {
            <rect width={move || RECT_SIZE * self.scale.get()} height={move|| RECT_SIZE * self.scale.get()} x={x} y={y} rx={RADIUS} ry={RADIUS} fill={FILL_COLOR}

            on:click={move|_|{
                sender.send_command(QuizSaladCommand::TileClicked(tile));
            }}
            >  </rect>

        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TileRectEntity {
    tile: Tile4x4,
    unneeded: bool,
}

impl GameEntity for TileRectEntity {
    type Artifact = TileRectArtifact;
    type Key = u8;
    type GameState = QuizSaladGameState;

    fn key(&self) -> Self::Key {
        self.tile.inner()
    }

    fn get_entities(game_state: &Self::GameState) -> impl Iterator<Item = Self> {
        let unneeded_tiles = game_state.found_words.unneeded_tiles;
        Tile::iter_by_row().map(move |tile| TileRectEntity {
            tile,
            unneeded: unneeded_tiles.get_bit(&tile),
        })
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        (
            TileRectArtifact {
                tile: self.tile,
                scale: RwSignal::new(if self.unneeded { 0.0 } else { 1.0 }),
            },
            vec![],
        )
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        former_entity_state: EntityState,
    ) -> AnimationList<Self::Artifact> {
        vec![animate_towards::<TileRectArtifactScaleLens>(
            if self.unneeded { 0.0 } else { 1.0 },
            1.0 / 1000.0,
        )]
    }
}

#[derive(Debug, Clone)]
pub struct TileTextArtifact {
    pub tile: Tile4x4,
    pub character: Character,
    pub selected: RwSignal<bool>,
    pub scale: RwSignal<f32>,
}

impl GameArtifact for TileTextArtifact {
    type Command = ();

    fn render(self, sender: impl CommandSender<Self::Command>) -> impl IntoView {
        let Vec2 { x, y } = tile_position(self.tile, true);
        let color = move || {
            if self.selected.get() {
                SELECTED_TEXT_COLOR
            } else {
                UNSELECTED_TEXT_COLOR
            }
        };

        let style = move || {
            format!(
                "transform: scale({}) ; transform-box: content-box; transform-origin: center center; transition: transform 1s;",
                self.scale.get()
            )
        };

        view! {
            <text x={x} y={y} style=style dominant-baseline="central" text-anchor="middle" fill={color} font-size={FONT_SIZE} font-family={FONT_FAMILY} font-weight={600} pointer-events="none">{self.character.as_char()} </text>

        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TileTextEntity {
    tile: Tile4x4,
    character: Character,
    selected: bool,
    unneeded: bool,
}

impl GameEntity for TileTextEntity {
    type Artifact = TileTextArtifact;
    type Key = u8;

    type GameState = QuizSaladGameState;

    fn key(&self) -> Self::Key {
        self.tile.inner()
    }

    fn get_entities(game_state: &Self::GameState) -> impl Iterator<Item = Self> {
        let solution = game_state.chosen_state.solution.clone();
        let unneeded_tiles = game_state.found_words.unneeded_tiles;
        game_state
            .puzzle
            .grid
            .enumerate()
            .map(move |(tile, &character)| {
                let selected = solution.contains(&tile);
                Self {
                    tile,
                    character,
                    selected,
                    unneeded: unneeded_tiles.get_bit(&tile),
                }
            })
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        (
            Self::Artifact {
                tile: self.tile,
                character: self.character,
                scale: RwSignal::new(if self.unneeded { 0.0 } else { 1.0 }),
                selected: RwSignal::new(self.selected),
            },
            vec![],
        )
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        former_entity_state: EntityState,
    ) -> AnimationList<Self::Artifact> {
        artifact.selected.set(self.selected);
        vec![animate_towards::<TileTextArtifactScaleLens>(
            if self.unneeded { 0.0 } else { 1.0 },
            1.0 / 1000.0,
        )]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NextButtonArtifact;

impl AnyGameCommand for NextButtonArtifact {}

impl GameCommand<QuizSaladGameState> for NextButtonArtifact {
    fn apply_command(&self, game_state: &mut QuizSaladGameState) -> MutationResult {
        let num_clues = game_state.puzzle.words.len();
        game_state.current_clue = (0..num_clues)
            .cycle()
            .skip(game_state.current_clue + 1)
            .take(num_clues)
            .filter(|index| game_state.found_words.get_completion(*index).is_complete())
            .next()
            .unwrap_or_default();

        MutationResult::CHANGED_NO_TRANSITION
    }
}

impl GameArtifact for NextButtonArtifact {
    type Command = NextButtonArtifact;

    fn render(self, sender: impl CommandSender<Self::Command>) -> impl IntoView {
        view! {
            <button on:click=move|_|sender.send_command(NextButtonArtifact)>
                "Next"
            </button>
        }
    }
}
impl GameEntity for NextButtonArtifact {
    type Artifact = Self;
    type Key = ();
    type GameState = QuizSaladGameState;

    fn key(&self) -> Self::Key {
        ()
    }

    fn get_entities(game_state: &Self::GameState) -> impl Iterator<Item = Self> {
        [Self].into_iter()
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        (Self, vec![])
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        former_entity_state: EntityState,
    ) -> AnimationList<Self::Artifact> {
        vec![]
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
