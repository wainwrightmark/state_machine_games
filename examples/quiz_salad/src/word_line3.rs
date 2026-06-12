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

impl LeptosRender for WordLineArtifact {
    type Artifact = Self;
    fn render(artifact: Self::Artifact) -> impl IntoView {
        let opacity = artifact.opacity;

        let segments = leptos::control_flow::For(leptos::prelude::ForProps {
            each: { move || make_line_segments(artifact.solution.get()) },
            key: |segment| segment.clone(),
            children: move |segment| {
                let color = crate::colors::CLASSIC_COLOR_SCHEME
                    .wordline_color(segment.index)
                    .to_hex();

                let Vec2 { x: x1, y: y1 } = tile_position(segment.from, PositionOrigin::Center);
                let Vec2 { x: x2, y: y2 } = tile_position(segment.to, PositionOrigin::Center);

                let this_segment_length = move || {
                    let total_segment_length = artifact.total_segments_length.get();
                    let this_index = segment.index as f32;
                    (total_segment_length - this_index - 1.0).clamp(0.0, 1.0)
                };

                let x2 = move || x1.lerp(x2, this_segment_length());

                let y2 = move || y1.lerp(y2, this_segment_length());
                let stroke_width = move || {
                    let ratio = artifact.stroke_width_ratio.get();
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

        // view!{
        //     <text>
        //         {move||{
        //             format!("{} {} {} {}", self.opacity.get(), self.solution.get().iter().map(|x|x.to_string()).join(", "), self.stroke_width_ratio.get(), self.total_segments_length.get())
        //         }}
        //     </text>
        // }
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
    pub close_to_solution: bool,
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

impl From<&QuizSaladGameState> for WordLineStateSegment {
    fn from(value: &QuizSaladGameState) -> Self {
        Self {
            chosen: value.chosen_state.clone(),
            is_close_to_word: value.is_close_to_solution(),
        }
    }
}

// impl HasSegment<WordLineStateSegment> for QuizSaladGameState {
//     fn get_segment(&self) -> WordLineStateSegment {
//         WordLineStateSegment {
//             chosen: self.chosen_state.clone(),
//             is_close_to_word: self.is_close_to_solution(),
//         }
//     }

//     fn segment_eq(&self, s: &WordLineStateSegment) -> bool {
//         self.chosen_state == s.chosen && s.is_close_to_word == self.is_close_to_solution()
//     }
// }

impl SingletonEntity for WordLineEntity {
    type Segment = WordLineStateSegment;
    type Artifact = WordLineArtifact;

    fn init(segment: &Self::Segment) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        let artifact = WordLineArtifact {
            stroke_width_ratio: RwSignal::new(1.0),
            total_segments_length: RwSignal::new(0.5),
            opacity: RwSignal::new(1.0),
            solution: RwSignal::new(segment.chosen.solution.clone()),
        };
        artifact.with_animations([animate_spring::<WordLineArtifactSegmentsLens>(
            segment.chosen.solution.len() as f32,
            1.0 / 1000.0,
            1.0,
        )
        .to_stage()])
    }

    fn update(
        artifact: &mut Self::Artifact,
        current_segment: &Self::Segment,
        previous_segment: &Self::Segment,
        previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        if current_segment.chosen.solution.is_empty() {
            if current_segment.chosen.word_just_found {
                return artifact.update_animations([
                    animate_towards::<WordLineArtifactOpacityLens>(0.0, 1.0 / 1000.0).to_stage(),
                ]);
            } else {
                return artifact.update_animations([animate_towards::<WordLineArtifactWidthLens>(
                    0.0,
                    1.0 / 1000.0,
                )
                .to_stage()]);
            }
        }

        artifact.solution.maybe_update(|a_s| {
            if !should_line_retract(&current_segment.chosen.solution, a_s) {
                *a_s = current_segment.chosen.solution.clone();
                true
            } else {
                false
            }
        });

        let line_width = if current_segment.is_close_to_word {
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
                current_segment.chosen.solution.len() as f32,
                2.0 / 1000.0,
                1.0,
            )
            .to_stage(),
            line_width,
            animate_towards::<WordLineArtifactOpacityLens>(1.0, 1.0 / 1000.0).to_stage(),
        ]);

        list
    }
}
