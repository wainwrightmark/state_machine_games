use crate::*;

#[derive(Debug, Clone)]
pub struct WordLineArtifact {
    pub v1: Vec2,
    pub v2: RwSignal<Vec2>,
    pub segment_index: usize,
    pub stroke_width_ratio: RwSignal<f32>,
}

//define_signal_lens!(WordLineArtifactV1Lens, WordLineArtifact, Vec2, v1);
define_signal_lens!(WordLineArtifactV2Lens, WordLineArtifact, Vec2, v2);

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
