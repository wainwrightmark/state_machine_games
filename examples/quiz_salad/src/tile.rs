use ws_core::WordTrait;

use crate::*;

#[derive(Debug, Clone)]
pub struct TileArtifact {
    pub position: RwSignal<Vec2>,
    pub character: Character,
    pub text_fill: RwSignal<Srgba>,
    pub scale: RwSignal<f32>,
}

impl GameArtifact for TileArtifact {}

define_signal_lens!(TileArtifactScaleLens, TileArtifact, f32, scale);
define_signal_lens!(TileArtifactFillLens, TileArtifact, Srgba, text_fill);
define_signal_lens!(TileArtifactPositionLens, TileArtifact, Vec2, position);

#[derive(Clone)]
pub struct RenderTileFill;

impl LeptosGameArtifact<RenderTileFill> for TileArtifact {
    type Command = ();
    fn render(
        self,
        _: RenderTileFill,
        _sender: impl CommandSender<Self::Command>,
    ) -> impl IntoView {
        let x = move || self.position.get().x;
        let y = move || self.position.get().y;

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

            >  </rect>

        }
    }
}

#[derive(Clone)]
pub struct RenderTileText;

impl LeptosGameArtifact<RenderTileText> for TileArtifact {
    type Command = ();
    fn render(
        self,
        _: RenderTileText,
        _sender: impl CommandSender<Self::Command>,
    ) -> impl IntoView {
        let x = move || self.position.get().x + (TILE_SIZE * 0.5);
        let y = move || self.position.get().y + (TILE_SIZE * 0.5);

        let style = move || format!("transform-box: content-box; transform-origin: center;",);
        let fill = move || self.text_fill.get().to_hex();

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
pub enum TileType {
    Playing(Tile4x4),
    Unneeded { base_tile: Tile4x4 },
    PostGame { index: usize, word_length: usize },
    PostGameUnneeded { base_tile: Tile4x4 },
}
impl TileType {
    pub fn position(&self, origin: PositionOrigin) -> Vec2 {
        match self {
            TileType::Playing(tile) | TileType::Unneeded { base_tile: tile } | TileType::PostGameUnneeded { base_tile: tile } => {
                tile_position(*tile, origin)
            }

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
            } => 4.0 / ((*word_length).max(4usize) as f32) ,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TileEntity {
    base_tile: Tile4x4,
    tile_type: TileType,
    character: Character,
    selected: bool,
}

impl TileEntity {
    pub const fn fill(&self) -> Srgba {
        if self.selected {
            CLASSIC_COLOR_SCHEME.tile_letter_selected
        } else {
            CLASSIC_COLOR_SCHEME.tile_letter_unselected
        }
    }
}

impl GameEntity for TileEntity {
    type Artifact = TileArtifact;
    type Key = u8;

    type StateSegment = QuizSaladGameState;

    fn key(&self) -> Self::Key {
        self.base_tile.inner()
    }

    fn get_entities(game_state: &Self::StateSegment) -> impl Iterator<Item = Self> {
        let solution = game_state.chosen_state.solution.clone();

        let postgame_solution = if game_state.finish_seconds.is_some() {
            game_state
                .puzzle
                .words
                .get(game_state.current_clue)
                .and_then(|x| x.find_solution(game_state.puzzle.grid))
                .unwrap_or_default()
        } else {
            ArrayVec::new()
        };

        //game_state.chosen_state.solution.clone();

        //let unneeded_tiles = game_state.found_words.unneeded_tiles;
        game_state
            .puzzle
            .grid
            .enumerate()
            .map(move |(tile, &character)| {
                let tile_type = if game_state.finish_seconds.is_some() {
                    if let Some(index) = postgame_solution.iter().position(|x| x.eq(&tile)) {
                        TileType::PostGame {
                            index,
                            word_length: postgame_solution.len(),
                        }
                    } else {
                        TileType::PostGameUnneeded  { base_tile: tile }
                    }
                } else if game_state.found_words.unneeded_tiles.get_bit(&tile) {
                    TileType::Unneeded { base_tile: tile }
                } else {
                    TileType::Playing(tile)
                };

                let selected = solution.contains(&tile);
                Self {
                    base_tile: tile,
                    tile_type,
                    character,
                    selected,
                }
            })
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        Self::Artifact {
            position: RwSignal::new(self.tile_type.position(PositionOrigin::TopLeft)),
            character: self.character,
            scale: RwSignal::new(self.tile_type.scale()),
            text_fill: RwSignal::new(self.fill()),
        }
        .with_animations([])
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        _former_entity_state: EntityState,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {

        

        artifact.update_animations([
            animate_towards::<TileArtifactScaleLens>(self.tile_type.scale(), 1.0 / 1000.0)
                .to_stage(),
            animate_towards::<TileArtifactFillLens>(self.fill(), 1.0 / 1000.0).to_stage(),


            animate_towards::<TileArtifactPositionLens>(self.tile_type.position(PositionOrigin::TopLeft), 1000.0 / 1000.0).to_stage(),
        ])
    }
}
