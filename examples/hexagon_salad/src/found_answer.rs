use crate::*;

#[derive(Debug, Clone)]
pub struct FoundAnswerArtifact {
    pub text: RwSignal<Ustr>,
}

impl GameArtifact for FoundAnswerArtifact {}
impl LeptosRender for FoundAnswerArtifact {
    type Artifact = Self;
    fn render(artifact: Self::Artifact) -> impl IntoView {
        move || match util::split_two_line_ustr(artifact.text.get(), 30) {
            itertools::Either::Left(a) => leptos::either::Either::Left(view! {
                <text x=320 y=900
                font-size={CLUE_FONT_SIZE}
                font-weight="700"
                font-family={FONT_FAMILY}
                dominant-baseline="central"
                fill={CLASSIC_COLOR_SCHEME.clue_text.to_hex()}
                style="text-align: center; text-anchor: middle;">{
                    a.to_string()
                }</text>
            }),
            itertools::Either::Right((a, b)) => leptos::either::Either::Right(view! {
                <text x=320 y=880
                font-size={CLUE_FONT_SIZE}
                font-weight="700"
                font-family={FONT_FAMILY}
                dominant-baseline="central"
                fill={CLASSIC_COLOR_SCHEME.clue_text.to_hex()}
                style="text-align: center; text-anchor: middle;">{
                    a.to_string()
                }</text>
                <text x=320 y=920
                font-size={CLUE_FONT_SIZE}
                font-weight="700"
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
pub struct FoundAnswerEntity {
    pub text: Ustr,
}

impl GameEntity for FoundAnswerEntity {
    type Artifact = FoundAnswerArtifact;
    type Key = Ustr;
    type Segment = QuizSaladGameState;

    fn key(&self) -> Self::Key {
        self.text
    }

    fn get_entities(game_state: &Self::Segment) -> impl Iterator<Item = Self> {

        if game_state.found_words.word_completions[game_state.current_clue].is_complete(){
            let text = game_state
            .puzzle
            .words
            .get(game_state.current_clue)
            .map(|x| Ustr::from(&x.text) )
            .unwrap_or_default();

        Some(Self { text }).into_iter()
        }   else{
            None.into_iter()
        }

        
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
        _former_entity_state: EntityLifecycle,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        artifact.text.set(self.text);
        AnimationList::EMPTY
    }
}
