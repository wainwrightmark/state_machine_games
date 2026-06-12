use shaped_word_grid_generator::{WordTrait, grid_layout::TileShape, prelude::GridSet};

use crate::*;

#[derive(Debug, Clone)]
pub struct TileArtifact {
    pub center_position: RwSignal<Vec2>,
    pub grapheme: String,
    pub tile_fill: RwSignal<Srgba>,
    pub text_fill: RwSignal<Srgba>,
    pub scale: RwSignal<f32>,
}

impl GameArtifact for TileArtifact {}

define_signal_lens!(TileArtifactScaleLens, TileArtifact, f32, scale);
define_signal_lens!(TileArtifactTextFillLens, TileArtifact, Srgba, text_fill);
define_signal_lens!(TileArtifactTileFillLens, TileArtifact, Srgba, tile_fill);
define_signal_lens!(
    TileArtifactPositionLens,
    TileArtifact,
    Vec2,
    center_position
);

pub struct TileRender;

impl LeptosRender for TileRender {
    type Artifact = TileArtifact;

    fn render(artifact: Self::Artifact) -> impl IntoView + 'static {
        // let points = move || {
        //     let Vec2 { x, y } = artifact.center_position.get();

        // };

        let points = match LayoutType::TILE_SHAPE {
            TileShape::Square | TileShape::HexagonPointyTop => {
                shaped_word_grid_generator::svg_hexagon::get_hexagon_points_pointy_top(
                    Vec2 { x: 0.0, y: 0.0 },
                    TILE_RADIUS * TILE_PROPORTION,
                )
            }
            TileShape::HexagonFlatTop => {
                shaped_word_grid_generator::svg_hexagon::get_hexagon_points_flat_top(
                    Vec2 { x: 0.0, y: 0.0 },
                    TILE_RADIUS * TILE_PROPORTION,
                )
            }
        };

        let style = move || {
            let Vec2 { x, y } = artifact.center_position.get();
            let scale = artifact.scale.get();
            format!("transform:translate({x}px, {y}px) scale({scale});")
        };

        // let style = move || format!("transform: scale({});", artifact.scale.get());

        view! {
            <polygon
            points={points}
            fill={move || artifact.tile_fill.get().to_hex()  }


            class="game__tile"
            style=style

            />

        }
    }
}

pub struct TileTextRender;

impl LeptosRender for TileTextRender {
    type Artifact = TileArtifact;
    fn render(artifact: Self::Artifact) -> impl IntoView {
        // let x = move || artifact.center_position.get().x;
        // let y = move || artifact.center_position.get().y;

        let style = move || {
            let Vec2 { x, y } = artifact.center_position.get();
            let scale = artifact.scale.get();
            format!("transform:translate({x}px, {y}px) scale({scale});")
        };

        // let style = move || format!("transform: scale({});", artifact.scale.get());

        let class = "game__tile-text";
        let fill = move || artifact.text_fill.get().to_hex();

        let font_size = match artifact.grapheme.len() {
            0..=1 => TILE_RADIUS * 0.8,
            2 => TILE_RADIUS * 0.6,
            3 => TILE_RADIUS * 0.5,
            4 => TILE_RADIUS * 0.4,
            n => TILE_RADIUS * 1.8 / n as f32,
        };

        view! {
            <text
            // x={x} y={y}
            style=style
            class=class
            dominant-baseline="central"
            text-anchor="middle"
            fill={fill}
            font-size={font_size}
            font-family={FONT_FAMILY}
            font-weight={600}

            pointer-events="none">
                {artifact.grapheme}
            </text>

        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TileType {
    Playing(GridTile),
    Unneeded { base_tile: GridTile },
    PostGame { index: usize, word_length: usize },
    PostGameUnneeded { base_tile: GridTile },
}
impl TileType {
    pub fn position(&self, origin: PositionOrigin) -> Vec2 {
        match self {
            TileType::Playing(tile)
            | TileType::Unneeded { base_tile: tile }
            | TileType::PostGameUnneeded { base_tile: tile } => offset_tile_position(*tile, origin),

            TileType::PostGame { index, word_length } => {
                postgame_tile_position(*index, *word_length, origin)
            }
        }
    }

    pub fn scale(&self) -> f32 {
        match self {
            TileType::Playing(_) => 1.0,
            TileType::Unneeded { base_tile: _ } => 0.0,
            TileType::PostGameUnneeded { base_tile: _ } => 0.0,
            TileType::PostGame {
                index: _,
                word_length,
            } => 6.0 / (*word_length as f32),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TileEntity {
    base_tile: GridTile,
    tile_type: TileType,
    grapheme: String,
    is_first_letter: bool,
    usages: usize,
    selected: bool,
}

impl TileEntity {
    pub const fn text_fill(&self) -> Srgba {
        if self.selected {
            CLASSIC_COLOR_SCHEME.tile_letter_selected
        } else {
            CLASSIC_COLOR_SCHEME.tile_letter_unselected
        }
    }

    pub fn tile_fill(&self) -> Srgba {
        if self.is_first_letter {
            colors::CLASSIC_COLOR_SCHEME.wordline_10
        } else {
            colors::CLASSIC_COLOR_SCHEME.tile
        }
        // let lightness = (0.9 - (0.05 * self.usages as f32)).max(0.0);
        // Srgba::gray(lightness)
    }
}

impl GameEntity for TileEntity {
    type Artifact = TileArtifact;
    type Key = u8;

    type Segment = QuizSaladGameState;

    fn key(&self) -> Self::Key {
        self.base_tile.0
    }

    fn get_entities(game_state: &Self::Segment) -> impl Iterator<Item = Self> {
        let solution = game_state.chosen_state.solution.clone();

        let postgame_solution = if game_state.finish_seconds.is_some() {
            game_state
                .puzzle
                .words
                .get(game_state.current_clue)
                .and_then(|x| x.find_solution::<LayoutType>(game_state.puzzle.grid))
                .unwrap_or_default()
        } else {
            ArrayVec::new()
        };

        let first_letters: GridSet = game_state
            .puzzle
            .words
            .iter()
            .zip(game_state.found_words.word_completions.iter())
            .filter(|(_, completion)| !completion.is_complete())
            .filter_map(|(word, _)| {
                word.find_solutions_with_tiles::<LayoutType>(
                    game_state.puzzle.grid,
                    game_state.found_words.unneeded_tiles,
                )
                .next()
            })
            .filter_map(|x| x.first().cloned())
            .map(|x| x.inner_u32())
            .collect();

        let mut tile_usages = [0usize; NUM_TILES];

        for (word, _) in game_state
            .puzzle
            .words
            .iter()
            .zip(game_state.found_words.word_completions.iter())
            .filter(|(_, completion)| !completion.is_complete())
        {
            if let Some(solution) = word
                .find_solutions_with_tiles::<LayoutType>(
                    game_state.puzzle.grid,
                    game_state.found_words.unneeded_tiles,
                )
                .next()
            {
                for tile in solution {
                    tile_usages[tile.inner_usize()] += 1;
                }
            }
        }

        // let  = GridSet::from_iter(
        //     le
        // );

        //let unneeded_tiles = game_state.found_words.unneeded_tiles;
        game_state
            .puzzle
            .grid
            .enumerate()
            .map(move |(tile, character)| {
                let tile_type = if game_state.finish_seconds.is_some() {
                    if let Some(index) = postgame_solution.iter().position(|x| x.eq(&tile)) {
                        TileType::PostGame {
                            index,
                            word_length: postgame_solution.len(),
                        }
                    } else {
                        TileType::PostGameUnneeded { base_tile: tile }
                    }
                } else if game_state
                    .found_words
                    .unneeded_tiles
                    .contains_const(tile.inner_u32())
                {
                    TileType::Unneeded { base_tile: tile }
                } else {
                    TileType::Playing(tile)
                };

                let selected = solution.contains(&tile);
                Self {
                    base_tile: tile,
                    tile_type,
                    grapheme: character
                        .to_tile_string(&game_state.puzzle.special_characters)
                        .to_string(),
                    selected,
                    is_first_letter: first_letters.contains_const(tile.inner_u32()),
                    usages: tile_usages[tile.inner_usize()],
                }
            })
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        Self::Artifact {
            center_position: RwSignal::new(self.tile_type.position(PositionOrigin::Center)),
            grapheme: self.grapheme.clone(),
            scale: RwSignal::new(self.tile_type.scale()),
            text_fill: RwSignal::new(self.text_fill()),
            tile_fill: RwSignal::new(self.tile_fill()), //is_first_letter: RwSignal::new(self.is_first_letter),
        }
        .with_animations([])
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        _former_entity_state: EntityLifecycle,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        artifact.tile_fill.set(self.tile_fill()); //animate this

        artifact.update_animations([
            animate_towards::<TileArtifactScaleLens>(self.tile_type.scale(), 1.0 / 1000.0)
                .to_stage(),
            animate_towards::<TileArtifactTextFillLens>(self.text_fill(), 1.0 / 1000.0).to_stage(),
            animate_towards::<TileArtifactTileFillLens>(self.tile_fill(), 1.0 / 1000.0).to_stage(),
            animate_towards::<TileArtifactPositionLens>(
                self.tile_type.position(PositionOrigin::Center),
                1000.0 / 1000.0,
            )
            .to_stage(),
        ])
    }
}
