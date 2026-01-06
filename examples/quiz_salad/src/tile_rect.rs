use crate::{quiz_salad_command::{QuizSaladCommand, TileClickedCommand}, *};

#[derive(Debug, Clone)]
pub struct TileRectArtifact {
    pub tile: Tile4x4,
    pub scale: RwSignal<f32>,
}

define_signal_lens!(TileRectArtifactScaleLens, TileRectArtifact, f32, scale);


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
                sender.send_command(QuizSaladCommand::TileClicked(TileClickedCommand(tile)));
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