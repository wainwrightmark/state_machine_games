use crate::*;
use ws_core::Solution4x4;

fn make_line_segments(solution: Solution4x4) -> ArrayVec<LineSegment, 15> {
    if let Ok(&s) = solution.iter().exactly_one() {
        return ArrayVec::from_iter([LineSegment {
            from: s,
            to: s,
            index: 0,
        }]);
    }

    return ArrayVec::from_iter(
        solution
            .into_iter()
            .tuple_windows()
            .enumerate()
            .map(|(index, (from, to))| LineSegment { from, to, index }),
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineSegment {
    pub from: Tile4x4,
    pub to: Tile4x4,
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WordLineArtifact {
    pub stroke_width_ratio: RwSignal<f32>,
    pub total_segments_length: RwSignal<f32>,
    pub opacity: RwSignal<f32>,
    /// The full solution, including additional tiles beyond the end
    pub solution: RwSignal<Solution4x4>,
}

impl LeptosGameArtifact for WordLineArtifact {
    type Command = ();

    fn render(
        self,
        _argument: (),
        _sender: impl state_machine_games::prelude::CommandSender<Self::Command>,
    ) -> impl IntoView {
        let opacity = self.opacity;

        let segments = leptos::control_flow::For(leptos::prelude::ForProps {
            each: { move || make_line_segments(self.solution.get()) },
            key: |segment| segment.clone(),
            children: move |segment| {
                let color = crate::colors::CLASSIC_COLOR_SCHEME
                    .wordline_color(segment.index)
                    .to_hex();

                let Vec2 { x: x1, y: y1 } = tile_position(segment.from, PositionOrigin::Center);
                let Vec2 { x: x2, y: y2 } = tile_position(segment.to, PositionOrigin::Center);

                let this_segment_length = move || {
                    let total_segment_length = self.total_segments_length.get();
                    let this_index = segment.index as f32;
                    (total_segment_length - this_index - 1.0).clamp(0.0, 1.0)
                };

                let x2 = move || x1.lerp(x2, this_segment_length());

                let y2 = move || y1.lerp(y2, this_segment_length());
                let stroke_width = move || {
                    let ratio = self.stroke_width_ratio.get();
                    // if segment.index > 0 && this_segment_length() <= 0.0 {
                    //     0.0
                    // } else {
                    //     (ratio) * PATH_STROKE_WIDTH
                    // }
                    (ratio) * PATH_STROKE_WIDTH
                };

                let segment_opacity = move || {
                    if segment.index == 0 {
                        1.0
                    } else {
                        (this_segment_length() * 2.0).clamp(0.0, 1.0)
                    }
                };

                view! {
                    <line
                                x1=x1
                                y1=y1
                                x2=x2
                                y2=y2
                                opacity=segment_opacity
                                visibility="visible"
                                stroke={color}
                                stroke-linecap="round"
                                stroke-width=stroke_width
                                pointer-events="none" >
                                </line>
                }
            },
        });

        view! {
            <g id="word_line" opacity={opacity}>
                {segments}
            </g>
        }
    }
}

impl GameArtifact for WordLineArtifact {}

define_signal_lens!(
    WordLineArtifactWidthLens,
    WordLineArtifact,
    f32,
    stroke_width_ratio
);
define_signal_lens!(
    WordLineArtifactSegmentsLens,
    WordLineArtifact,
    f32,
    total_segments_length
);
define_signal_lens!(WordLineArtifactOpacityLens, WordLineArtifact, f32, opacity);

#[derive(Debug, PartialEq)]
pub struct WordLineEntity {
    /// The current solution
    pub solution: Solution4x4,
    pub word_jut_found: bool,
}

fn should_line_retract(new: &Solution4x4, old: &Solution4x4) -> bool {
    if new.len() == 1 && old.len() > 2 {
        return false;
    }

    new.len() <= old.len() && new.iter().zip(old.iter()).all(|(a, b)| a == b)
}
#[derive(Debug, Clone, PartialEq)]
pub struct WordLineStateSegment {
    pub chosen: ChosenState,
    pub is_close_to_word: bool,
}
impl HasSegment<WordLineStateSegment> for QuizSaladGameState {
    fn get_segment(&self) -> WordLineStateSegment {
        WordLineStateSegment {
            chosen: self.chosen_state.clone(),
            is_close_to_word: self.is_close_to_solution(),
        }
    }

    fn segment_eq(&self, s: &WordLineStateSegment) -> bool {
        self.chosen_state == s.chosen && s.is_close_to_word == self.is_close_to_solution()
    }
}

impl GameEntity for WordLineEntity {
    type Artifact = WordLineArtifact;
    type Key = Tile4x4;
    type StateSegment = WordLineStateSegment;

    fn key(&self) -> Self::Key {
        self.solution
            .iter()
            .copied()
            .next()
            .unwrap_or_else(|| Tile4x4::NORTH_WEST)
    }

    fn get_entities(segment: &Self::StateSegment) -> impl Iterator<Item = Self> {
        let wle = WordLineEntity {
            solution: segment.chosen.solution.clone(),
            word_jut_found: segment.is_close_to_word,
        };
        [wle].into_iter().filter(|x| !x.solution.is_empty())
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        let artifact = WordLineArtifact {
            stroke_width_ratio: RwSignal::new(1.0),
            total_segments_length: RwSignal::new(0.5),
            opacity: RwSignal::new(1.0),
            solution: RwSignal::new(self.solution.clone()),
        };
        artifact.with_animations([animate_spring::<WordLineArtifactSegmentsLens>(
            self.solution.len() as f32,
            1.0 / 1000.0,
            1.0,
        )
        .to_stage()])
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        former_entity_state: EntityState,
        previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        artifact.solution.maybe_update(|a_s| {
            if !should_line_retract(&self.solution, a_s) {
                *a_s = self.solution.clone();
                true
            } else {
                false
            }
        });

        let line_width = if self.word_jut_found {
            let mut animation = animate_towards::<WordLineArtifactWidthLens>(1.0, 0.2 / 1000.0)
                .to_stage()
                .precede_with(animate_towards::<WordLineArtifactWidthLens>(
                    0.9,
                    0.2 / 1000.0,
                ));
            animation.loop_forever();
            animation
        } else {
            animate_towards::<WordLineArtifactWidthLens>(1.0, 1.0 / 1000.0).to_stage()
        };

        let list = artifact.update_animations([
            animate_spring::<WordLineArtifactSegmentsLens>(
                self.solution.len() as f32,
                2.0 / 1000.0,
                1.0,
            )
            .to_stage(),
            line_width,
            animate_towards::<WordLineArtifactOpacityLens>(1.0, 1.0 / 1000.0).to_stage(),
        ]);

        list
    }

    fn on_death(
        &self,
        artifact: &mut Self::Artifact,
        previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        if self.word_jut_found {
            artifact.update_animations([animate_towards::<WordLineArtifactOpacityLens>(
                0.0,
                1.0 / 1000.0,
            )
            .to_stage()])
        } else {
            artifact.update_animations([animate_towards::<WordLineArtifactWidthLens>(
                0.0,
                1.0 / 1000.0,
            )
            .to_stage()])
        }
    }
}
