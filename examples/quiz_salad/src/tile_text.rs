use crate::*;

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

define_signal_lens!(TileTextArtifactScaleLens, TileTextArtifact, f32, scale);
define_signal_lens!(TileTextArtifactFillLens, TileTextArtifact, Srgba, fill);

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
