use crate::{
    quiz_salad_command::{QuizSaladCommand, TileClickedCommand},
    *,
};

#[derive(Debug, Clone)]
pub struct TileArtifact {
    pub tile: Tile4x4,
    pub character: Character,
    pub text_fill: RwSignal<Srgba>,
    //pub selected: RwSignal<bool>,
    pub scale: RwSignal<f32>,
}

impl GameArtifact for TileArtifact {}

define_signal_lens!(TileTextArtifactScaleLens, TileArtifact, f32, scale);
define_signal_lens!(TileTextArtifactFillLens, TileArtifact, Srgba, text_fill);

#[derive(Clone)]
pub struct RenderTileFill;

impl LeptosGameArtifact<RenderTileFill> for TileArtifact {
    type Command = QuizSaladCommand;
    fn render(self, _: RenderTileFill, sender: impl CommandSender<Self::Command>) -> impl IntoView {
        let Vec2 { x, y } = tile_position(self.tile, PositionOrigin::TopLeft);

        let tile = self.tile;

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
                sender.send_command(QuizSaladCommand::TileClicked(TileClickedCommand(tile)));
            }}
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
        let Vec2 { x, y } = tile_position(self.tile, PositionOrigin::Center);

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
pub struct TileEntity {
    tile: Tile4x4,
    character: Character,
    selected: bool,
    unneeded: bool,
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
        Self::Artifact {
            tile: self.tile,
            character: self.character,
            scale: RwSignal::new(if self.unneeded { 0.0 } else { 1.0 }),
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
            animate_towards::<TileTextArtifactScaleLens>(
                if self.unneeded { 0.0 } else { 1.0 },
                1.0 / 1000.0,
            )
            .to_stage(),
            animate_towards::<TileTextArtifactFillLens>(self.fill(), 1.0 / 1000.0).to_stage(),
        ])
    }
}
