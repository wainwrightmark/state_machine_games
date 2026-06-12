use leptos::logging;

use crate::*;

#[derive(Debug, Clone)]
pub struct TileArtifact {
    pub tile: Tile4x4,
    pub character: RwSignal<Character>,
    pub tile_fill: RwSignal<Srgba>,
}

impl GameArtifact for TileArtifact {}

define_signal_lens!(TileArtifactFillLens, TileArtifact, Srgba, tile_fill);

pub struct TileRender;

impl LeptosRender for TileRender {
    type Artifact = TileArtifact;

    fn render(artifact: Self::Artifact) -> impl IntoView + 'static {
        let position = tile_position(artifact.tile, PositionOrigin::TopLeft);
        let x = position.x;
        let y = position.y;

        let sender = expect_context::<Sender<Box<dyn GameCommand<DalasDrowGameState> + 'static>>>();

        let on_click = move |_| {
            sender
                .send(Box::new(DalasDrowCommand::ClickTile(artifact.tile)))
                .unwrap();
        };

        let fill = move || artifact.tile_fill.get().to_hex();

        view! {
            <rect
            width={TILE_SIZE}
            height={TILE_SIZE}
            x={x}
            y={y}
            rx={TILE_RADIUS}
            ry={TILE_RADIUS}
            fill={fill}
            style="transform-box: content-box; transform-origin: center center;"
            on:click=on_click
            >  </rect>

        }
    }
}

pub struct TileTextRender;

impl LeptosRender for TileTextRender {
    type Artifact = TileArtifact;
    fn render(artifact: Self::Artifact) -> impl IntoView {
        let position = tile_position(artifact.tile, PositionOrigin::Center);
        let x = position.x;
        let y = position.y;

        let style = move || format!("transform-box: content-box; transform-origin: center;",);

        view! {
            <text x={x} y={y}
            style=style
            dominant-baseline="central"
            text-anchor="middle"
            fill={colors::CLASSIC_COLOR_SCHEME.tile_letter_unselected.to_hex()}
            font-size={TILE_LETTER_FONT_SIZE}
            font-family={FONT_FAMILY}
            font-weight={600}

            pointer-events="none">
                {move || {
                    let c = artifact.character.get().as_char();
                    logging::log!("Tile {}: {c}", artifact.tile);
                    c
                }}
            </text>

        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TileEntity {
    base_tile: Tile4x4,
    character: Character,
    selected: bool,
}

impl TileEntity {
    pub const fn tile_fill(&self) -> Srgba {
        if self.selected {
            CLASSIC_COLOR_SCHEME.wordline_0
        } else {
            CLASSIC_COLOR_SCHEME.tile
        }
    }
}

impl GameEntity for TileEntity {
    type Artifact = TileArtifact;
    type Key = u8;

    type Segment = DalasDrowGameState;

    fn key(&self) -> Self::Key {
        self.base_tile.inner()
    }

    fn get_entities(game_state: &Self::Segment) -> impl Iterator<Item = Self> {
        game_state.grid.enumerate().map(move |(tile, &character)| {
            let selected = game_state.selected_tile == Some(tile);
            Self {
                base_tile: tile,

                character,
                selected,
            }
        })
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        Self::Artifact {
            character: RwSignal::new(self.character),

            tile_fill: RwSignal::new(self.tile_fill()),
            tile: self.base_tile,
        }
        .with_animations([])
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        _former_entity_state: EntityLifecycle,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        artifact.character.maybe_update(|x| {
            if *x == self.character {
                false
            } else {
                *x = self.character;
                true
            }
        });

        artifact.update_animations([animate_towards::<TileArtifactFillLens>(
            self.tile_fill(),
            10.0 / 1000.0,
        )
        .to_stage()])
    }
}
