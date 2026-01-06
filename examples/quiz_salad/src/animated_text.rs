use crate::*;

#[derive(Debug, PartialEq, Clone)]
pub struct AnimatedTextEntity {
    pub word_index: usize,
    pub lozenge_count: usize,
    pub tile: Tile4x4,
    pub text: Ustr,
}

impl GameEntity for AnimatedTextEntity {
    type Artifact = AnimatedTextArtifact;
    type Key = usize;
    type StateSegment = QuizSaladGameState;

    fn key(&self) -> Self::Key {
        self.word_index
    }

    fn get_entities(segment: &Self::StateSegment) -> impl Iterator<Item = Self> {
        if !segment.word_just_found {
            return None.into_iter();
        }

        let tile = segment
            .chosen_state
            .solution
            .last()
            .copied()
            .unwrap_or_default();

        let Some((word_index, word)) = segment
            .found_words
            .most_recently_completed_word(&segment.puzzle)
        else {
            return None.into_iter();
        };
        let lozenge_count = segment.found_words.word_completions.len();
        let entity = AnimatedTextEntity {
            word_index,
            lozenge_count,
            tile,
            text: word.text,
        };
        Some(entity).into_iter()
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        let position = tile_position(self.tile, PositionOrigin::Center);

        let artifact = Self::Artifact {
            text: self.text,
            position: RwSignal::new(position),
            scale: RwSignal::new(1.0),
            color: CLASSIC_COLOR_SCHEME.animated_word,
        };

        let duration_ms = 2000.0f64;
        let target_position =
            lozenge_position(self.word_index, self.lozenge_count, PositionOrigin::Center);

        let distance = position.distance(target_position) as f64;

        //log!("Position {position:?} target position {target_position} distance {distance}");

        let animations = vec![
            animate_towards::<AnimatedTextArtifactPositionLens>(
                target_position,
                (distance / duration_ms).abs(),
            ),
            animate_towards::<AnimatedTextArtifactScaleLens>(0.5, 1.0 / duration_ms),
        ];

        (artifact, animations)
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        former_entity_state: EntityState,
        previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        previous_animations
    }

    fn on_death(
        &self,
        _artifact: &mut Self::Artifact,
        previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        previous_animations
    }
}

#[derive(Debug, Clone)]
pub struct AnimatedTextArtifact {
    pub text: Ustr,
    pub position: RwSignal<Vec2>,
    pub scale: RwSignal<f32>,
    pub color: Srgba,
}

define_signal_lens!(
    AnimatedTextArtifactPositionLens,
    AnimatedTextArtifact,
    Vec2,
    position
);
define_signal_lens!(
    AnimatedTextArtifactScaleLens,
    AnimatedTextArtifact,
    f32,
    scale
);

impl GameArtifact for AnimatedTextArtifact {
    
}

impl LeptosGameArtifact for AnimatedTextArtifact {
    type Command = ();
    fn render(
        self,
        _: (),
        sender: impl state_machine_games::prelude::CommandSender<Self::Command>,
    ) -> impl IntoView {
        view! {

            <text x={move|| self.position.get().x} y={move|| self.position.get().y}
                font-size={ANIMATED_WORD_FONT_SIZE}
                font-weight="600"
                font-family={FONT_FAMILY}
                dominant-baseline="central"
                fill={self.color.to_hex()}
                transform-origin="center"
                transform={move || format!("scale({})", self.scale.get()) }
                style="text-align: center; text-anchor: middle; transform-box: fill-box;">{
                    {self.text.to_string()}
                }</text>

        }
    }
}
