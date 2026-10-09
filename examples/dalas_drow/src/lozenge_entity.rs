use ws_core::prelude::{Ustr, WordTrait};

use crate::*;

define_signal_lens!(LozengeArtifactFillLens, LozengeArtifact, Srgba, fill);
define_signal_lens!(
    LozengeArtifactStrokeWidthLens,
    LozengeArtifact,
    f32,
    stroke_width
);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LozengeSelection {
    None,
    Semi,
    Full,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LozengeEntity {
    pub text: Ustr,
    pub index: usize,
    pub lozenge_count: usize,

    pub completed: bool,

    pub selection_status: LozengeSelection,
}

impl LozengeEntity {
    pub const fn position(&self) -> Vec2 {
        layout::lozenge_position(self.index, self.lozenge_count, PositionOrigin::TopLeft)
    }

    pub const fn stroke_width(&self) -> f32 {
        match self.selection_status {
            LozengeSelection::None => 0.0,
            LozengeSelection::Semi => 1.0,
            LozengeSelection::Full => 2.0,
        }
    }

    pub const fn fill(&self) -> Srgba {
        if self.completed {
            CLASSIC_COLOR_SCHEME.lozenge_completed
        } else {
            CLASSIC_COLOR_SCHEME.lozenge_normal
        }
    }
}

impl GameEntity for LozengeEntity {
    type Artifact = LozengeArtifact;
    type Key = usize;
    type Segment = DalasDrowGameState;

    fn key(&self) -> Self::Key {
        self.index
    }

    fn get_entities(segment: &Self::Segment) -> impl Iterator<Item = Self> {
        let lozenge_count = segment.puzzle.words.len();
        log!("Getting lozenge entities");
        segment
            .puzzle
            .words
            .iter()
            .enumerate()
            .map(move |(index, word)| {
                let completed = word.find_solution(segment.grid).is_some();

                let selection_status = match segment.selected_tile {
                    Some(selected_tile) => {
                        let tile_char = segment.grid[selected_tile];
                        let word_char_count =
                            word.characters.iter().filter(|x| **x == tile_char).count();
                        if word_char_count == 0 {
                            LozengeSelection::None
                        } else {
                            let grid_char_count =
                                segment.grid.iter().filter(|x| **x == tile_char).count();
                            if word_char_count == grid_char_count {
                                LozengeSelection::Full
                            } else {
                                LozengeSelection::Semi
                            }
                        }
                    }
                    None => LozengeSelection::None,
                };

                LozengeEntity {
                    text: word.text,
                    index,
                    lozenge_count,
                    selection_status,
                    completed,
                }
            })
    }

    fn should_reset_store(new_state: &Self::Segment, prev_state: &Self::Segment) -> bool {
        new_state.start_timestamp != prev_state.start_timestamp
    }
    

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        let position = self.position();

        LozengeArtifact {
            text: self.text,
            index: self.index,
            position: position,
            stroke_width: RwSignal::new(self.stroke_width()),
            fill: RwSignal::new(self.fill()),
        }
        .with_animations([])
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        former_entity_lifecycle: EntityLifecycle,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        if former_entity_lifecycle == EntityLifecycle::Reset {
            artifact.fill.set(self.fill());
            artifact.stroke_width.set(self.stroke_width());

            return AnimationList::EMPTY;
        }

        artifact.update_animations([
            animate_towards::<LozengeArtifactFillLens>(self.fill(), 1.0 / 1000.0).to_stage(),
            animate_towards::<LozengeArtifactStrokeWidthLens>(self.stroke_width(), 20.0 / 1000.0)
                .to_stage(),
        ])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LozengeArtifact {
    pub index: usize,
    pub position: Vec2,
    pub text: Ustr,
    pub fill: RwSignal<Srgba>,
    pub stroke_width: RwSignal<f32>,
}

impl GameArtifact for LozengeArtifact {}

impl LeptosRender for LozengeArtifact {
    type Artifact = Self;
    fn render(artifact: Self::Artifact) -> impl IntoView {
        let Self {
            index: _,
            position: Vec2 { x, y },
            fill,
            stroke_width,
            text,
        } = artifact;

        view! {
            <rect x={x} y={y}
            width={LOZENGE_WIDTH}
            height={LOZENGE_HEIGHT}
            stroke-width={stroke_width}
            stroke={CLASSIC_COLOR_SCHEME.lozenge_selection_stroke.to_hex()}
            fill={move || fill.get().to_hex()}
            rx={LOZENGE_RADIUS}
            ry={LOZENGE_RADIUS} >
            </rect>

            <text
            x={x + (LOZENGE_WIDTH * 0.5)}
            y={y + (LOZENGE_HEIGHT * 0.5)}
            dominant-baseline="central"
            text-anchor="middle"
            fill={colors::CLASSIC_COLOR_SCHEME.clue_text.to_hex()}
            font-size={30}
            font-family={FONT_FAMILY}
            font-weight={500}

            >
                {text.to_string()}
            </text>
        }
    }
}
