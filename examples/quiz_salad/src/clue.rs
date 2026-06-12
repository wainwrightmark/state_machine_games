use crate::*;

#[derive(Debug, Clone)]
pub struct ClueArtifact {
    pub text: RwSignal<Ustr>,
}

impl GameArtifact for ClueArtifact {}
impl LeptosRender for ClueArtifact {
    type Artifact = Self;
    fn render(artifact: Self::Artifact) -> impl IntoView {
        move || match util::split_two_line_ustr(artifact.text.get(), 30) {
            itertools::Either::Left(a) => leptos::either::Either::Left(view! {
                <text x=320 y=950
                font-size={CLUE_FONT_SIZE}
                font-weight="600"
                font-family={FONT_FAMILY}
                dominant-baseline="central"
                fill={CLASSIC_COLOR_SCHEME.clue_text.to_hex()}
                style="text-align: center; text-anchor: middle;">{
                    a.to_string()
                }</text>
            }),
            itertools::Either::Right((a, b)) => leptos::either::Either::Right(view! {
                <text x=320 y=930
                font-size={CLUE_FONT_SIZE}
                font-weight="600"
                font-family={FONT_FAMILY}
                dominant-baseline="central"
                fill={CLASSIC_COLOR_SCHEME.clue_text.to_hex()}
                style="text-align: center; text-anchor: middle;">{
                    a.to_string()
                }</text>
                <text x=320 y=970
                font-size={CLUE_FONT_SIZE}
                font-weight="600"
                font-family={FONT_FAMILY}
                dominant-baseline="central"
                fill={CLASSIC_COLOR_SCHEME.clue_text.to_hex()}
                style="text-align: center; text-anchor: middle;">{
                    b.to_string()
                }</text>
            }),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ClueEntity {
    pub text: Ustr,
}

impl GameEntity for ClueEntity {
    type Artifact = ClueArtifact;
    type Key = ();
    type Segment = QuizSaladGameState;

    fn key(&self) -> Self::Key {
        ()
    }

    fn get_entities(game_state: &Self::Segment) -> impl Iterator<Item = Self> {
        let text = game_state
            .puzzle
            .words
            .get(game_state.current_clue)
            .and_then(|x| x.clue)
            .unwrap_or_default();

        [Self { text }].into_iter()
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        Self::Artifact {
            text: RwSignal::new(self.text),
        }
        .with_animations([])
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        former_entity_state: EntityLifecycle,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        artifact.text.set(self.text);
        AnimationList::EMPTY
    }
}
