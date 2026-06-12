use std::sync::mpsc::Sender;

use geometrid::{prelude::TileMap, vector::Vector};
use leptos::prelude::*;
use rand::{Rng, RngExt, SeedableRng};
use state_machine_games::prelude::*;
use strum::{EnumCount, FromRepr};

pub type Match3Tile = geometrid::tile::Tile<8, 8>;
pub type Match3Grid = geometrid::tile_map::TileMap<Option<Gem>, 8, 8, 64>;

pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    mount_to_body(|| game_component());
}

fn game_component() -> impl IntoView {
    let state = RwSignal::new(Match3Game::new_random(123));
    let sender = run_game(state);

    provide_context(sender);

    let score = ScoreTextEntity::render_singleton::<TextArtifact>(Memo::new(move |_x| state.read().score).into());
    let moves =  MovesLeftEntity::render_singleton::<TextArtifact>(Memo::new(move |_x| state.read().score).into());

    let match3_tiles = Match3TileEntity::render_entities::<TileArtifact>(state.into());

    view! {
        <svg viewBox="0 0 800.0 800.0"  style="max-width: 800px;  margin-inline: auto; ">
        {score}
        {moves}
        {match3_tiles}
        </svg>

    }
}

const FONT_SIZE: f32 = 32.0;
const SCALE: f32 = 80.0;
const TOP_OFFSET: f32 = 120.0;
const SQUARE_SIZE: f32 = SCALE * 0.9;
const RECT_OFFSET: f32 = SQUARE_SIZE * 0.5;
#[derive(Debug, Clone, PartialEq)]
pub struct Match3Game {
    pub score: u64,
    pub moves_left: u64,
    pub next_index: u32,

    pub next_tiles: [Gem; 8],

    pub grid: Match3Grid,

    pub selected_tile: Option<Match3Tile>,

    pub rng_state: TinyRng,
}

impl Match3Game {
    pub fn new_random(seed: u64) -> Self {
        let mut rng_state = TinyRng::seed_from_u64(seed);
        let mut next_index = 0;

        let arr = [None; 64];

        let next_tiles = std::array::from_fn(|_| {
            let gem = Gem {
                gem_type: GemType::new_random(&mut rng_state),
                index: next_index,
            };
            next_index += 1;
            gem
        });

        Self {
            grid: TileMap::from_inner(arr),
            selected_tile: None,
            score: 0,
            moves_left: 3,
            next_index,
            next_tiles,
            rng_state,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Gem {
    index: u32,
    gem_type: GemType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromRepr, EnumCount)]
pub enum GemType {
    Blue,
    Yellow,
    Orange,
    Pink,
    Green,
    Purple,
}

impl GemType {
    pub fn fill(self) -> &'static str {
        match self {
            GemType::Blue => "#01befe",
            GemType::Yellow => "#ffdd00",
            GemType::Orange => "#ff7d00",
            GemType::Pink => "#ff006d",
            GemType::Green => "#adff02",
            GemType::Purple => "#8f00ff",
        }
    }

    pub fn new_random(rng: &mut impl Rng) -> Self {
        let i = rng.random_range(0..Self::COUNT);
        GemType::from_repr(i).unwrap()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Match3Command {
    TileClicked(Match3Tile),
}

impl GameCommand<Match3Game> for Match3Command {
    fn apply_command(&self, game_state: &mut Match3Game) -> MutationResult {
        if game_state.grid.iter().any(|x| x.is_none()) {
            return MutationResult::NO_CHANGE;
        }

        let Match3Command::TileClicked(tile) = *self;

        //log::info!("Tile Clicked {tile}");

        match game_state.selected_tile {
            Some(former_selected_tile) => {
                if former_selected_tile == tile {
                    //Unselect the tile
                    game_state.selected_tile = None;
                } else if former_selected_tile.is_contiguous_with(&tile) {
                    //swap the tiles

                    if let Some(new_moves_left) = game_state.moves_left.checked_sub(1) {
                        game_state.grid.swap(tile, former_selected_tile);
                        game_state.moves_left = new_moves_left;
                        game_state.selected_tile = None;
                    }

                    //TODO match 3 logic
                } else {
                    game_state.selected_tile = Some(tile);
                }
            }
            None => {
                game_state.selected_tile = Some(tile);
            }
        }
        let transition_callback_in_ms = if true { Some(1000.0) } else { None };

        MutationResult {
            changed: true,
            transition_callback_in_ms,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextArtifact {
    pub x: f32,
    pub y: f32,
    pub font_size: f32,
    pub text: ArcRwSignal<String>,
}

impl GameArtifact for TextArtifact {}

impl LeptosRender for TextArtifact {
    type Artifact = Self;
    fn render(artifact: Self::Artifact) -> impl IntoView {
        view! {
            <text x=artifact.x y=artifact.y font-size=artifact.font_size style="user-select: none;">
                {artifact.text}
            </text>
        }
    }
}

#[derive(Debug, Clone)]
pub struct TileArtifact {
    pub tile: ArcRwSignal<Match3Tile>,
    pub x: ArcRwSignal<f32>,
    pub y: ArcRwSignal<f32>,
    pub fill: &'static str,
    pub selected: ArcRwSignal<bool>,
    pub scale: ArcRwSignal<f32>,
}

state_machine_games::define_signal_lens!(TileArtifactXLens, TileArtifact, f32, x);
state_machine_games::define_signal_lens!(TileArtifactYLens, TileArtifact, f32, y);
state_machine_games::define_signal_lens!(TileArtifactScaleLens, TileArtifact, f32, scale);

impl GameArtifact for TileArtifact {}

impl LeptosRender for TileArtifact {
    type Artifact = Self;
    fn render(artifact: Self::Artifact) -> impl IntoView {
        let sender = expect_context::<Sender<Box<dyn GameCommand<Match3Game>>>>();
        view! {
            <rect x={artifact.x} y={artifact.y}
            width={SQUARE_SIZE}
            height ={SQUARE_SIZE}
            rx="2%" ry="2%"
            transform-origin="center"
            fill={artifact.fill}
            stroke={move || if artifact.selected.get() { "#222222FF" } else { "#00000000" }}
            style= {move || format!("transform-box:fill-box; transform: scale({}); stroke-width: 10px;  transition: stroke 1s;", artifact.scale.get())}
            on:click=move|_|{ sender.send_command(Box::new(Match3Command::TileClicked(artifact.tile.get_untracked())));  } >
            </rect>
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScoreTextEntity {
    pub score: u64,
}

impl SingletonEntity for ScoreTextEntity {
    type Artifact = TextArtifact;
    type Segment = u64;

    fn init(segment: &Self::Segment) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        let artifact = TextArtifact {
            text: ArcRwSignal::new(format!("Score: {}", *segment)),
            font_size: FONT_SIZE,
            x: 10.0,
            y: 40.0,
        };

        artifact.with_animations([])
    }

    fn update(
        artifact: &mut Self::Artifact,
        current_segment: &Self::Segment,
        _previous_segment: &Self::Segment,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        artifact.text.set(format!("Score: {}", *current_segment));
        AnimationList::EMPTY
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MovesLeftEntity {
    pub moves: u64,
}

impl SingletonEntity for MovesLeftEntity {
    type Artifact = TextArtifact;
    type Segment = u64;

    fn init(segment: &Self::Segment) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        let artifact = TextArtifact {
            text: ArcRwSignal::new(format!("Moves Left: {}", *segment)),
            font_size: FONT_SIZE,
            x: 10.0,
            y: 100.0,
        };

        artifact.with_animations([])
    }

    fn update(
        artifact: &mut Self::Artifact,
        current_segment: &Self::Segment,
        _previous_segment: &Self::Segment,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        artifact.text.set(format!("Moves: {}", *current_segment));
        AnimationList::EMPTY
    }
}

#[derive(Debug, PartialEq)]
pub struct Match3TileEntity {
    pub index: u32,
    pub tile: Match3Tile,
    pub gem_type: GemType,
    pub selected: bool,
}

impl Match3TileEntity {
    fn get_x(col: u8) -> f32 {
        (SCALE * (col as f32 + 0.5)) - RECT_OFFSET
    }

    fn get_y(row: u8) -> f32 {
        (SCALE * (row as f32 + 0.5)) + TOP_OFFSET - RECT_OFFSET
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TileKey(u32);

impl GameEntity for Match3TileEntity {
    type Artifact = TileArtifact;
    type Key = TileKey;
    type Segment = Match3Game;

    fn get_entities(game_state: &Self::Segment) -> impl Iterator<Item = Self> {
        let selected_tile = game_state.selected_tile;
        game_state.grid.enumerate().flat_map(move |(tile, gem)| {
            if let Some(Gem {
                index,
                gem_type: tile_type,
            }) = *gem
            {
                Some(Match3TileEntity {
                    tile,
                    index: index,
                    gem_type: tile_type,
                    selected: Some(tile) == selected_tile,
                })
            } else {
                None
            }
        })
    }

    fn key(&self) -> Self::Key {
        TileKey(self.index)
    }

    fn on_death(
        &self,
        artifact: &mut Self::Artifact,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        artifact.selected.set(self.selected);
        artifact.tile.set(self.tile);

        artifact.update_animations([
            animate_towards::<TileArtifactScaleLens>(0.0, 1.0 / 1000.0).to_stage(),
            animate_towards::<TileArtifactXLens>(Self::get_x(self.tile.x()), SCALE as f64 / 1000.0)
                .to_stage(),
            animate_towards::<TileArtifactYLens>(Self::get_y(self.tile.y()), SCALE as f64 / 1000.0)
                .to_stage(),
        ])
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        let artifact = TileArtifact {
            tile: ArcRwSignal::new(self.tile),
            x: ArcRwSignal::new(Self::get_x(self.tile.x())),
            y: ArcRwSignal::new(100.0),
            fill: self.gem_type.fill(),
            selected: ArcRwSignal::new(self.selected),
            scale: ArcRwSignal::new(0.0),
        };

        let animations = [
            animate_towards::<TileArtifactScaleLens>(1.0, 1.0 / 1000.0).to_stage(),
            animate_towards::<TileArtifactYLens>(Self::get_y(self.tile.y()), SCALE as f64 / 100.0)
                .to_stage(),
        ];

        artifact.with_animations(animations)
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        _former_entity_state: EntityLifecycle,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        artifact.selected.set(self.selected);
        artifact.tile.set(self.tile);
        //change x,y,selected,scale
        vec![
            animate_towards::<TileArtifactXLens>(Self::get_x(self.tile.x()), SCALE as f64 / 1000.0)
                .to_stage(),
            animate_towards::<TileArtifactYLens>(Self::get_y(self.tile.y()), SCALE as f64 / 1000.0)
                .to_stage(),
        ]
        .into()
    }
}

impl GameState for Match3Game {
    fn maybe_transition(&mut self) -> MutationResult {
        let mut changed = false;

        let mut new_grid = self.grid.clone();

        //find all lines
        let mut any_removed = false;

        for i_is_x in [false, true] {
            for i in 0..8u8 {
                let mut previous_gem_type: Option<GemType> = None;
                let mut consecutive = 0;
                for j in 0..8u8 {
                    let tile = if i_is_x {
                        Match3Tile::try_new(i, j).unwrap()
                    } else {
                        Match3Tile::try_new(j, i).unwrap()
                    };
                    let gem_type = self.grid[tile].map(|x| x.gem_type);

                    if let Some(gem_type) = gem_type {
                        if Some(gem_type) == previous_gem_type {
                            consecutive += 1;
                            if consecutive >= 3 {
                                changed = true;
                                self.score += 1;
                                new_grid[tile] = None;
                                if consecutive == 3 {
                                    //also remove the two previous tiles
                                    any_removed = true;
                                    for sub in [2, 1] {
                                        let tile_to_remove = if i_is_x {
                                            Match3Tile::try_new(i, j - sub).unwrap()
                                        } else {
                                            Match3Tile::try_new(j - sub, i).unwrap()
                                        };
                                        new_grid[tile_to_remove] = None;
                                    }
                                }
                            }
                        } else {
                            previous_gem_type = Some(gem_type);
                            consecutive = 1;
                        }
                    } else {
                        previous_gem_type = None;
                        consecutive = 0;
                    }
                }
            }
        }
        if any_removed {
            self.moves_left += 1;
        }

        //move down all tiles with a space below them
        for tile in Match3Tile::iter_by_row().rev() {
            let gem = new_grid[tile];
            if gem.is_none() {
                if let Some(above_tile) = tile.const_add(&Vector::NORTH) {
                    //log::info!("Moving gem down to {tile}");
                    //move the tile from above down
                    new_grid.swap(tile, above_tile);
                    if new_grid[tile].is_some() {
                        changed = true;
                    }
                } else {
                    //log::info!("Creating new gem at {tile}");
                    //this tile is on the top row so create a new tile
                    let new_gem_type = GemType::new_random(&mut self.rng_state);
                    let new_index = self.next_index;
                    self.next_index += 1; //todo be careful of this
                    changed = true;
                    new_grid[tile] = Some(self.next_tiles[tile.x() as usize]);
                    self.next_tiles[tile.x() as usize] = Gem {
                        index: new_index,
                        gem_type: new_gem_type,
                    };
                }
            }
        }

        self.grid = new_grid;

        MutationResult {
            changed,
            transition_callback_in_ms: if changed { Some(1000.0) } else { None },
        }
    }
}

#[cfg(test)]
mod tests {

    // #[test]
    // pub fn test_game(){
    //     let settings = ();
    //     let assets = ();
    //     let storage = ();

    //     let mut game = Match3Game::new_random(123);

    //     let commands =
    //     //[(3,6)]

    //      [(3,6),(2,6),(2,3),(1,3),(3,4),(3,5),(5,4),(4,4),(7,7),(0,2),(1,2),(0,2),(1,2),(1,3),(2,3),(3,2),(2,2),]
    //     .map(|(x,y)|Match3Tile::try_new(x, y).unwrap())
    //     .map(|x|Match3Command::TileClicked(x));

    //     let mut entity_store: LeptosEntityStore<Match3Entity> = EntityStore::new(game.get_entities(&settings, &assets, &storage));

    //     let mut now = web_time::Instant::now();
    //     for command in commands{

    //         let _ = game.apply_command(command, &settings, &assets, &storage);
    //         now += Duration::from_hours(1);
    //         let _ = entity_store. .update(game.get_entities(&settings, &assets, &storage), now);
    //         game.fast_forward_transitions(&settings, &assets, &storage);
    //         now += Duration::from_hours(1);
    //         let _ = entity_store.update(game.get_entities(&settings, &assets, &storage), now);
    //     }
    //     now += Duration::from_hours(1);
    //     let _ = entity_store.update(game.get_entities(&settings, &assets, &storage), now);

    //     let svg_inner = entity_store.stored_entities.iter()

    //     //.filter(|x|x.signal.read_untracked().0.is_score_text())
    //     .map(|x|{
    //         let read_guard = x.signal.read_untracked();
    //         let svg = read_guard.0.svg_text(&read_guard.1);

    //         svg
    //     }) .join("\n");

    //     let svg = format!(r#"
    //     <svg viewbox="0 0 800 800" style="width: 800px; height: 800px; background:white;">
    //         {svg_inner}
    //     </svg>
    //     "#);

    //     insta::assert_snapshot!(svg);

    // }
}
