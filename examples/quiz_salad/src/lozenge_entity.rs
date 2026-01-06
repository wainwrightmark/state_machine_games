use crate::{quiz_salad_command::{LozengeClickedCommand, QuizSaladCommand}, *};

define_signal_lens!(LozengeArtifactFillLens, LozengeArtifact, Srgba, fill);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LozengeSelection {
    None,
    Selected,
    Finished,
}

impl LozengeSelection {
    pub const fn color(&self) -> Srgba {
        match self {
            LozengeSelection::None => CLASSIC_COLOR_SCHEME.lozenge_normal,
            LozengeSelection::Selected => CLASSIC_COLOR_SCHEME.lozenge_selected,
            LozengeSelection::Finished => CLASSIC_COLOR_SCHEME.lozenge_completed,
        }
    }

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
    pub selected: LozengeSelection,
}

impl LozengeEntity {
    pub const fn position(&self) -> Vec2 {
        layout::lozenge_position(self.index, self.lozenge_count, PositionOrigin::TopLeft)
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
                selected: LozengeSelection::new(completion, index == segment.current_clue),
            })
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        let position = self.position();

        (
            LozengeArtifact {
                index: self.index,
                x: position.x,
                y: position.y,
                fill: RwSignal::new(self.selected.color()),
            },
            AnimationList::new(),
        )
    }

    fn on_update(
        &self,
        _artifact: &mut Self::Artifact,
        _former_entity_state: EntityState,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        let animations = vec![animate_towards::<LozengeArtifactFillLens>(
            self.selected.color(),
            1.0 / 1000.0,
        )];

        animations
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LozengeArtifact {
    pub index: usize,
    pub x: f32,
    pub y: f32,
    pub fill: RwSignal<bevy_color::Srgba>,
}

impl GameArtifact for LozengeArtifact {
    type Command = QuizSaladCommand;
}

impl LeptosGameArtifact for LozengeArtifact {
    fn render(
        self,
        sender: impl state_machine_games::prelude::CommandSender<Self::Command>,
    ) -> impl IntoView {
        let Self { index, x, y, fill } = self;
        let on_click = move |_: MouseEvent| {
            sender.send_command(QuizSaladCommand::LozengeClicked(LozengeClickedCommand(index)));
        };
        view! {
            <rect x={x} y={y} width={LOZENGE_WIDTH} height={LOZENGE_HEIGHT} fill={move || fill.get().to_hex()} rx={LOZENGE_RADIUS} ry={LOZENGE_RADIUS} on:click=on_click>
            </rect>
        }
    }
}