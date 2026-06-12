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
    type Segment = QuizSaladGameState;

    fn key(&self) -> Self::Key {
        self.word_index
    }

    fn get_entities(segment: &Self::Segment) -> impl Iterator<Item = Self> {
        if !segment.chosen_state.word_just_found {
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
        let position = tile_position(self.tile, PositionOrigin::Center); //todo offset

        let artifact = Self::Artifact {
            text: self.text,
            position: ArcRwSignal::new(position),
            scale: ArcRwSignal::new(1.0),
            color: CLASSIC_COLOR_SCHEME.animated_word,
        };

        let duration_ms = 2000.0f64;
        let target_position =
            lozenge_position(self.word_index, self.lozenge_count, PositionOrigin::Center);

        let distance = position.distance(target_position) as f64;

        log!("Position {position:?} target position {target_position} distance {distance}");

        let animations = [
            animate_towards::<AnimatedTextArtifactPositionLens>(
                target_position,
                (distance / duration_ms).abs(),
            )
            .to_stage(),
            animate_set_value::<AnimatedTextArtifactScaleLens>(0.0)
                .to_stage()
                .precede_with(animate_wait(1000.0))
                .precede_with(animate_towards::<AnimatedTextArtifactScaleLens>(
                    0.5,
                    1.0 / duration_ms,
                )),
        ];

        artifact.with_animations(animations)
    }

    fn on_update(
        &self,
        _artifact: &mut Self::Artifact,
        _former_entity_state: EntityLifecycle,
        previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        log!("Entity Updated: {}", std::any::type_name::<Self>());
        previous_animations
    }

    fn on_death(
        &self,
        _artifact: &mut Self::Artifact,
        previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        log!("Entity Died: {}", std::any::type_name::<Self>());
        previous_animations
    }
}

#[derive(Debug, Clone)]
pub struct AnimatedTextArtifact {
    pub text: Ustr,
    pub position: ArcRwSignal<Vec2>,
    pub scale: ArcRwSignal<f32>,
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

impl GameArtifact for AnimatedTextArtifact {}

impl LeptosRender for AnimatedTextArtifact {
    type Artifact = Self;
    fn render(artifact: Self::Artifact) -> impl IntoView {
        let apx = artifact.position.clone();
        let apy = artifact.position;
        view! {

            <text x={move|| apx.get().x} y={move|| apy.get().y}
                font-size={ANIMATED_WORD_FONT_SIZE}
                font-weight="600"
                font-family={FONT_FAMILY}
                dominant-baseline="central"
                fill={artifact.color.to_hex()}
                transform-origin="center"
                transform={move || format!("scale({})", artifact.scale.get()) }
                style="text-align: center; text-anchor: middle; transform-box: fill-box;">{
                    {artifact.text.to_string()}
                }</text>

        }
    }
}
