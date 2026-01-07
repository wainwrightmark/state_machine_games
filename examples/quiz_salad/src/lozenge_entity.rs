use crate::{
    quiz_salad_command::{LozengeClickedCommand, QuizSaladCommand},
    *,
};

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
    Selected,
    Finished,
}

impl LozengeSelection {
    pub const fn new(completion: &Completion, selected: bool) -> Self {
        if selected {
            return Self::Selected;
        } else if completion.is_complete() {
            return Self::Finished;
        } else {
            return Self::None;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LozengeEntity {
    pub index: usize,
    pub lozenge_count: usize,
    pub selected: bool,
    pub completed: bool,
}

impl LozengeEntity {
    pub const fn position(&self) -> Vec2 {
        layout::lozenge_position(self.index, self.lozenge_count, PositionOrigin::TopLeft)
    }

    pub const fn stroke_width(&self) -> f32 {
        if self.selected { 10.0 } else { 0.0 }
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
    type StateSegment = QuizSaladGameState;

    fn key(&self) -> Self::Key {
        self.index
    }

    fn get_entities(segment: &Self::StateSegment) -> impl Iterator<Item = Self> {
        let lozenge_count = segment.found_words.word_completions.len();
        segment
            .found_words
            .word_completions
            .iter()
            .enumerate()
            .map(move |(index, completion)| LozengeEntity {
                index,
                lozenge_count,
                selected: index == segment.current_clue,
                completed: completion.is_complete(),
            })
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        let position = self.position();

        LozengeArtifact {
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
        _former_entity_state: EntityState,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        artifact.update_animations([
            animate_towards::<LozengeArtifactFillLens>(self.fill(), 1.0 / 1000.0).to_stage(),
            animate_towards::<LozengeArtifactStrokeWidthLens>(self.stroke_width(), 20.0 / 1000.0).to_stage(),
        ])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LozengeArtifact {
    pub index: usize,
    pub position: Vec2,
    pub fill: RwSignal<bevy_color::Srgba>,
    pub stroke_width: RwSignal<f32>,
}

impl GameArtifact for LozengeArtifact {}

impl LeptosGameArtifact for LozengeArtifact {
    type Command = QuizSaladCommand;
    fn render(
        self,
        _: (),
        sender: impl state_machine_games::prelude::CommandSender<Self::Command>,
    ) -> impl IntoView {
        let Self {
            index,
            position: Vec2 { x, y },
            fill,
            stroke_width,
        } = self;
        let on_click = move |_: MouseEvent| {
            sender.send_command(QuizSaladCommand::LozengeClicked(LozengeClickedCommand(
                index,
            )));
        };
        view! {
            <rect x={x} y={y}
            width={LOZENGE_WIDTH}
            height={LOZENGE_HEIGHT}
            stroke-width={stroke_width}
            stroke={CLASSIC_COLOR_SCHEME.lozenge_selection_stroke.to_hex()}
            fill={move || fill.get().to_hex()}
            rx={LOZENGE_RADIUS}
            ry={LOZENGE_RADIUS} on:click=on_click>
            </rect>
        }
    }
}
