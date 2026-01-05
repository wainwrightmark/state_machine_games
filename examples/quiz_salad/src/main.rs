pub mod chosen_state;
pub mod colors;
pub mod found_words_state;
pub mod grid_input;
pub mod layout;
pub mod layout_util;
pub mod puzzle;
pub mod util;

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
use std::sync::mpsc;
use ws_core::{ArrayVec, Character, LevelTrait, Tile, Tile4x4, Ustr};

use crate::{chosen_state::ChosenState, found_words_state::FoundWordsState, puzzle::Puzzle};

type Stores = (
    ArcRwSignal<SingleTypeEntityStore<ClueEntity>>,
    ArcRwSignal<SingleTypeEntityStore<TileRectEntity>>,
    ArcRwSignal<SingleTypeEntityStore<WordLineSectionEntity>>,
    ArcRwSignal<SingleTypeEntityStore<TileTextEntity>>,
    ArcRwSignal<SingleTypeEntityStore<LozengeEntity>>,
    ArcRwSignal<SingleTypeEntityStore<AnimatedTextEntity>>,
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
    let machine = GameMachine::new(state, stores.clone());

    let cs1 = machine.command_sender();
    let cs2 = machine.command_sender();

    machine.run_game();

    view! {
        <div style="height: 100vh; width: 100vw; overflow: hidden;">
            <svg viewBox=format!("0 0 {GAME_WIDTH} {GAME_HEIGHT}") style="max-width: 100%; max-height: 100%; user-select:none; position:fixed; margin:auto; inset: 0px;">

            {move || SingleTypeEntityStore::render(stores.1.clone(), cs1.clone())}
            {move || SingleTypeEntityStore::render(stores.2.clone(), ())}
            {move || SingleTypeEntityStore::render(stores.3.clone(), ())}
            {move || SingleTypeEntityStore::render(stores.0.clone(), ())}
            {move || SingleTypeEntityStore::render(stores.4.clone(), cs2.clone())}
            {move || SingleTypeEntityStore::render(stores.5.clone(), ())}

            </svg>
            // <div>
            //     {move || SingleTypeEntityStore::render(stores.4.clone(), button_sender.clone())}
            // </div>
        </div>

    }
}

#[derive(Debug, Clone, PartialEq)]
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

define_signal_lens!(LozengeArtifactFillLens, LozengeArtifact, Srgba, fill);
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
}

impl LeptosGameArtifact for WordLineArtifact {
    fn render(self, _sender: impl CommandSender<Self::Command>) -> impl IntoView {
        //todo nice line disappear - dependent on reason for disappear
        //todo line pulsing if close to the answer

        view! {
            <line
             x1= self.v1.x
             y1= self.v1.y
             x2={move || self.v2.get().x}
             y2={move || self.v2.get().y}
             visibility="visible"
             stroke={colors::CLASSIC_COLOR_SCHEME.wordline_color(self.segment_index as usize).to_hex()}
             stroke-linecap="round"
             stroke-width={move ||{self.stroke_width_ratio.get() * PATH_STROKE_WIDTH}}
             pointer-events="none" >
            </line>
        }
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
    type StateSegment = QuizSaladGameState;

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

    fn get_entities(game_state: &Self::StateSegment) -> impl Iterator<Item = Self> {
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
                let relative_point = Some(
                    tile_position(*t1, PositionOrigin::Center)
                        - tile_position(*t2, PositionOrigin::Center),
                );
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

    fn on_death(
        &self,
        artifact: &mut Self::Artifact,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
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
                v1 = tile_position(*tile, PositionOrigin::Center);
                v2 = tile_position(*tile, PositionOrigin::Center);
                initial_width_ratio = 0.0;
            }
            WordLineSectionEntity::LineSegment {
                segment_index,
                t1,
                t2,
                ..
            } => {
                si = *segment_index;
                v1 = tile_position(*t1, PositionOrigin::Center);
                v2 = tile_position(*t2, PositionOrigin::Center);
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
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        let v1: Vec2;
        let v2: Vec2;
        let si: u8;
        let initial_width_ratio: f32;

        match self {
            WordLineSectionEntity::Circle { tile } => {
                si = 0;
                v1 = tile_position(*tile, PositionOrigin::Center);
                v2 = tile_position(*tile, PositionOrigin::Center);
                initial_width_ratio = 0.0;
            }
            WordLineSectionEntity::LineSegment {
                segment_index,
                t1,
                t2,
                ..
            } => {
                si = *segment_index;
                v1 = tile_position(*t1, PositionOrigin::Center);
                v2 = tile_position(*t2, PositionOrigin::Center);
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
    type Command = QuizSaladCommand;

    fn apply_command(&mut self, command: &Self::Command) -> MutationResult {
        match command {
            QuizSaladCommand::TileClicked(clicked_tile) => {
                let clicked_tile = *clicked_tile;
                if self.found_words.unneeded_tiles.get_bit(&clicked_tile) {
                    self.chosen_state = ChosenState::default();
                    return MutationResult::CHANGED_NO_TRANSITION;
                }

                let current_solution = self.chosen_state.current_solution();

                let Some(last_tile) = current_solution.last().copied() else {
                    self.chosen_state = ChosenState {
                        solution: ArrayVec::from_iter([clicked_tile]),
                    };
                    return MutationResult::CHANGED_NO_TRANSITION;
                };

                if clicked_tile == last_tile {
                    let mut new_solution = current_solution.clone();
                    new_solution.pop();
                    self.chosen_state = ChosenState {
                        solution: new_solution,
                    };
                    return MutationResult::CHANGED_NO_TRANSITION;
                }

                if let Some(position) = current_solution.iter().position(|&x| x == clicked_tile) {
                    let mut new_solution = current_solution.clone();
                    new_solution.truncate(position + 1);
                    self.chosen_state = ChosenState {
                        solution: new_solution,
                    };

                    return MutationResult::CHANGED_NO_TRANSITION;
                }

                if clicked_tile.is_adjacent_to(&last_tile) {
                    let mut new_solution = current_solution.clone();
                    new_solution.push(clicked_tile);
                    self.chosen_state = ChosenState {
                        solution: new_solution,
                    };
                    if let Some(solution_index) =
                        self.puzzle.check_solution(&self.chosen_state.solution)
                    {
                        let c_index = self
                            .found_words
                            .word_completions
                            .iter()
                            .filter(|x| x.is_complete())
                            .count() as u8;

                        if let Some(completion) =
                            self.found_words.word_completions.get_mut(solution_index)
                        {
                            if !completion.is_complete() {
                                *completion =
                                    found_words_state::Completion::Complete { index: c_index };
                                self.word_just_found = true;
                                return MutationResult::changed_with_transition(500.0);
                            }
                        }
                    }

                    return MutationResult::CHANGED_NO_TRANSITION;
                } else {
                    self.chosen_state = ChosenState::default();

                    return MutationResult::CHANGED_NO_TRANSITION;
                }
            }
            QuizSaladCommand::LozengeClicked(index) => {
                self.current_clue = *index;
                return MutationResult::CHANGED_NO_TRANSITION;
            }
        }
    }

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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum QuizSaladCommand {
    TileClicked(Tile4x4),
    LozengeClicked(usize),
}

impl GameCommand for QuizSaladCommand {}

//const FILL_COLOR: &'static str = "#97FCFF";

//const SELECTED_TEXT_COLOR: &'static str = "#FFFFFF";
//const UNSELECTED_TEXT_COLOR: &'static str = "#202251";

#[derive(Debug, Clone)]
pub struct ClueArtifact {
    pub text: RwSignal<Ustr>,
}

impl GameArtifact for ClueArtifact {
    type Command = ();
}
impl LeptosGameArtifact for ClueArtifact {
    fn render(self, _sender: impl CommandSender<Self::Command>) -> impl IntoView {
        move || match util::split_two_line_ustr(self.text.get(), 30) {
            itertools::Either::Left(a) => leptos::either::Either::Left(view! {
                <text x=320 y=950
                font-size={CLUE_FONT_SIZE}
                font-weight="600"
                font-family={FONT_FAMILY}
                dominant-baseline="central"
                fill={CLASSIC_COLOR_SCHEME.clue_text.to_hex()}
                style="text-align: center; text-anchor: middle;">{
                    a.to_string()
                }</text>
            }),
            itertools::Either::Right((a, b)) => leptos::either::Either::Right(view! {
                <text x=320 y=930
                font-size={CLUE_FONT_SIZE}
                font-weight="600"
                font-family={FONT_FAMILY}
                dominant-baseline="central"
                fill={CLASSIC_COLOR_SCHEME.clue_text.to_hex()}
                style="text-align: center; text-anchor: middle;">{
                    a.to_string()
                }</text>
                <text x=320 y=970
                font-size={CLUE_FONT_SIZE}
                font-weight="600"
                font-family={FONT_FAMILY}
                dominant-baseline="central"
                fill={CLASSIC_COLOR_SCHEME.clue_text.to_hex()}
                style="text-align: center; text-anchor: middle;">{
                    b.to_string()
                }</text>
            }),
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
    type StateSegment = QuizSaladGameState;

    fn key(&self) -> Self::Key {
        ()
    }

    fn get_entities(game_state: &Self::StateSegment) -> impl Iterator<Item = Self> {
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
        _previous_animations: AnimationList<Self::Artifact>,
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
define_signal_lens!(TileTextArtifactFillLens, TileTextArtifact, Srgba, fill);

impl GameArtifact for TileRectArtifact {
    type Command = QuizSaladCommand;
}

impl LeptosGameArtifact for TileRectArtifact {
    fn render(self, sender: impl CommandSender<Self::Command>) -> impl IntoView {
        let Vec2 { x, y } = tile_position(self.tile, PositionOrigin::TopLeft);

        let tile = self.tile;

        // let style = move || {
        //     format!(
        //         "transform-box: content-box; transform-origin: center center;",
        //         self.scale.get()
        //     )
        // };

        view! {
            <rect
            width={TILE_SIZE}
            height={TILE_SIZE}
            x={x}
            y={y}
            rx={TILE_RADIUS}
            ry={TILE_RADIUS}
            fill={colors::CLASSIC_COLOR_SCHEME.tile.to_hex()}
            transform={move || format!("scale({})", self.scale.get())}
            style="transform-box: content-box; transform-origin: center center;"
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
    type StateSegment = FoundWordsState;

    fn key(&self) -> Self::Key {
        self.tile.inner()
    }

    fn get_entities(found_words: &Self::StateSegment) -> impl Iterator<Item = Self> {
        let unneeded_tiles = found_words.unneeded_tiles;
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
        _artifact: &mut Self::Artifact,
        _former_entity_state: EntityState,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        vec![animate_towards::<TileRectArtifactScaleLens>(
            if self.unneeded { 0.0 } else { 1.0 },
            1.0 / 1000.0,
        )]
    }
}

impl HasSegment<FoundWordsState> for QuizSaladGameState {
    fn get_segment(&self) -> FoundWordsState {
        self.found_words.clone()
    }

    fn segment_eq(&self, s: &FoundWordsState) -> bool {
        self.found_words.eq(s)
    }
}

#[derive(Debug, Clone)]
pub struct TileTextArtifact {
    pub tile: Tile4x4,
    pub character: Character,
    pub fill: RwSignal<Srgba>,
    //pub selected: RwSignal<bool>,
    pub scale: RwSignal<f32>,
}

impl GameArtifact for TileTextArtifact {
    type Command = ();
}

impl LeptosGameArtifact for TileTextArtifact {
    fn render(self, sender: impl CommandSender<Self::Command>) -> impl IntoView {
        let Vec2 { x, y } = tile_position(self.tile, PositionOrigin::Center);
        // let color = move || {
        //     if self.selected.get() {
        //         SELECTED_TEXT_COLOR
        //     } else {
        //         UNSELECTED_TEXT_COLOR
        //     }
        // };

        let style = move || format!("transform-box: content-box; transform-origin: center;",);
        let fill = move || self.fill.get().to_hex();

        view! {
            <text x={x} y={y}
            style=style
            dominant-baseline="central"
            text-anchor="middle"
            fill={fill}
            font-size={TILE_LETTER_FONT_SIZE}
            font-family={FONT_FAMILY}
            font-weight={600}
            transform={move ||format!("scale({})", self.scale.get())}
            pointer-events="none">
                {self.character.as_char()}
            </text>

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

impl TileTextEntity {
    pub const fn fill(&self) -> Srgba {
        if self.selected {
            CLASSIC_COLOR_SCHEME.tile_letter_selected
        } else {
            CLASSIC_COLOR_SCHEME.tile_letter_unselected
        }
    }
}

impl GameEntity for TileTextEntity {
    type Artifact = TileTextArtifact;
    type Key = u8;

    type StateSegment = QuizSaladGameState;

    fn key(&self) -> Self::Key {
        self.tile.inner()
    }

    fn get_entities(game_state: &Self::StateSegment) -> impl Iterator<Item = Self> {
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
                fill: RwSignal::new(self.fill()),
            },
            vec![],
        )
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        former_entity_state: EntityState,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        vec![
            animate_towards::<TileTextArtifactScaleLens>(
                if self.unneeded { 0.0 } else { 1.0 },
                1.0 / 1000.0,
            ),
            animate_towards::<TileTextArtifactFillLens>(self.fill(), 1.0 / 1000.0),
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LozengeSelection {
    None,
    Selected,
    Finished,
}

impl LozengeSelection {
    pub const fn color(&self) -> Srgba {
        match self {
            LozengeSelection::None => CLASSIC_COLOR_SCHEME.lozenge_normal,
            LozengeSelection::Selected => CLASSIC_COLOR_SCHEME.lozenge_selected,
            LozengeSelection::Finished => CLASSIC_COLOR_SCHEME.lozenge_completed,
        }
    }

    pub const fn new(completion: &Completion, selected: bool) -> Self {
        if selected {
            return Self::Selected;
        } else if completion.is_complete() {
            return Self::Finished;
        } else {
            return Self::None;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LozengeEntity {
    pub index: usize,
    pub lozenge_count: usize,
    pub selected: LozengeSelection,
}

impl LozengeEntity {
    pub const fn position(&self) -> Vec2 {
        layout::lozenge_position(self.index, self.lozenge_count, PositionOrigin::TopLeft)
    }
}

impl GameEntity for LozengeEntity {
    type Artifact = LozengeArtifact;
    type Key = usize;
    type StateSegment = QuizSaladGameState;

    fn key(&self) -> Self::Key {
        self.index
    }

    fn get_entities(segment: &Self::StateSegment) -> impl Iterator<Item = Self> {
        let lozenge_count = segment.found_words.word_completions.len();
        segment
            .found_words
            .word_completions
            .iter()
            .enumerate()
            .map(move |(index, completion)| LozengeEntity {
                index,
                lozenge_count,
                selected: LozengeSelection::new(completion, index == segment.current_clue),
            })
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        let position = self.position();

        (
            LozengeArtifact {
                index: self.index,
                x: position.x,
                y: position.y,
                fill: RwSignal::new(self.selected.color()),
            },
            AnimationList::new(),
        )
    }

    fn on_update(
        &self,
        _artifact: &mut Self::Artifact,
        _former_entity_state: EntityState,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        let animations = vec![animate_towards::<LozengeArtifactFillLens>(
            self.selected.color(),
            1.0 / 1000.0,
        )];

        animations
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LozengeArtifact {
    pub index: usize,
    pub x: f32,
    pub y: f32,
    pub fill: RwSignal<bevy_color::Srgba>,
}

impl GameArtifact for LozengeArtifact {
    type Command = QuizSaladCommand;
}

impl LeptosGameArtifact for LozengeArtifact {
    fn render(
        self,
        sender: impl state_machine_games::prelude::CommandSender<Self::Command>,
    ) -> impl IntoView {
        let Self { index, x, y, fill } = self;
        let on_click = move |_: MouseEvent| {
            sender.send_command(QuizSaladCommand::LozengeClicked(index));
        };
        view! {
            <rect x={x} y={y} width={LOZENGE_WIDTH} height={LOZENGE_HEIGHT} fill={move || fill.get().to_hex()} rx={LOZENGE_RADIUS} ry={LOZENGE_RADIUS} on:click=on_click>
            </rect>
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct AnimatedTextEntity {
    pub word_index: usize,
    pub lozenge_count: usize,
    pub tile: Tile4x4,
    pub text: Ustr,
}

impl GameEntity for AnimatedTextEntity {
    type Artifact = AnimatedTextArtifact;
    type Key = usize;
    type StateSegment = QuizSaladGameState;

    fn key(&self) -> Self::Key {
        self.word_index
    }

    fn get_entities(segment: &Self::StateSegment) -> impl Iterator<Item = Self> {
        if !segment.word_just_found {
            return None.into_iter();
        }

        let tile = segment
            .chosen_state
            .solution
            .last()
            .copied()
            .unwrap_or_default();

        let Some((word_index, word)) = segment
            .found_words
            .most_recently_completed_word(&segment.puzzle)
        else {
            return None.into_iter();
        };
        let lozenge_count = segment.found_words.word_completions.len();
        let entity = AnimatedTextEntity {
            word_index,
            lozenge_count,
            tile,
            text: word.text,
        };
        Some(entity).into_iter()
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        let position = tile_position(self.tile, PositionOrigin::Center);

        let artifact = Self::Artifact {
            text: self.text,
            position: RwSignal::new(position),
            scale: RwSignal::new(1.0),
            color: CLASSIC_COLOR_SCHEME.animated_word,
        };

        let duration_ms = 2000.0f64;
        let target_position =
            lozenge_position(self.word_index, self.lozenge_count, PositionOrigin::Center);

        let distance = position.distance(target_position) as f64;

        //log!("Position {position:?} target position {target_position} distance {distance}");

        let animations = vec![
            animate_towards::<AnimatedTextArtifactPositionLens>(
                target_position,
                (distance / duration_ms).abs(),
            ),
            animate_towards::<AnimatedTextArtifactScaleLens>(0.5, 1.0 / duration_ms),
        ];

        (artifact, animations)
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        former_entity_state: EntityState,
        previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        previous_animations
    }

    fn on_death(
        &self,
        _artifact: &mut Self::Artifact,
        previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        previous_animations
    }
}

#[derive(Debug, Clone)]
pub struct AnimatedTextArtifact {
    pub text: Ustr,
    pub position: RwSignal<Vec2>,
    pub scale: RwSignal<f32>,
    pub color: Srgba,
}

define_signal_lens!(
    AnimatedTextArtifactPositionLens,
    AnimatedTextArtifact,
    Vec2,
    position
);
define_signal_lens!(
    AnimatedTextArtifactScaleLens,
    AnimatedTextArtifact,
    f32,
    scale
);

impl GameArtifact for AnimatedTextArtifact {
    type Command = ();
}

impl LeptosGameArtifact for AnimatedTextArtifact {
    fn render(
        self,
        sender: impl state_machine_games::prelude::CommandSender<Self::Command>,
    ) -> impl IntoView {
        view! {

            <text x={move|| self.position.get().x} y={move|| self.position.get().y}
                font-size={ANIMATED_WORD_FONT_SIZE}
                font-weight="600"
                font-family={FONT_FAMILY}
                dominant-baseline="central"
                fill={self.color.to_hex()}
                transform-origin="center"
                transform={move || format!("scale({})", self.scale.get()) }
                style="text-align: center; text-anchor: middle; transform-box: fill-box;">{
                    {self.text.to_string()}
                }</text>

        }
    }
}
