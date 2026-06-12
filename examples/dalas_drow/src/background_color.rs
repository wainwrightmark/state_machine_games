use bevy_color::Srgba;
use leptos::prelude::RwSignal;
use state_machine_games::{
    define_signal_lens,
    prelude::{GameArtifact, animate_towards},
};

use crate::*;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct BackgroundColor {
    pub color: RwSignal<Srgba>,
}

define_signal_lens!(BackgroundColorLens, BackgroundColor, Srgba, color);

impl GameArtifact for BackgroundColor {}

#[derive(Debug, PartialEq)]
pub struct BackgroundTargetColor {
    pub color: Srgba,
}



// impl HasSegment<BackgroundTargetColor> for QuizSaladGameState {
//     fn get_segment(&self) -> BackgroundTargetColor {
//         if self.found_words.is_level_complete() {
//             BackgroundTargetColor {
//                 color: CLASSIC_COLOR_SCHEME.background_complete,
//             }
//         } else {
//             BackgroundTargetColor {
//                 color: CLASSIC_COLOR_SCHEME.background_incomplete,
//             }
//         }
//     }

//     fn segment_eq(&self, s: &BackgroundTargetColor) -> bool {
//         <Self as HasSegment<BackgroundTargetColor>>::get_segment(self).eq(s)
//     }
// }

impl SingletonEntity for BackgroundColor {
    type Segment = Srgba;
    type Artifact = Self;

    fn init(segment: &Self::Segment) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        (
            Self {
                color: RwSignal::new(*segment),
            },
            AnimationList::EMPTY,
        )
    }

    fn update(
        _artifact: &mut Self::Artifact,
        current_segment: &Self::Segment,
        _previous_segment: &Self::Segment,
        mut previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        previous_animations.clear();
        previous_animations.push(
            animate_towards::<BackgroundColorLens>(current_segment.clone(), 1.0 / 1000.0)
                .to_stage(),
        );

        previous_animations
    }
    // type Segment = BackgroundTargetColor;

    // type GS = QuizSaladGameState;

    // fn update_value(
    //     segment: &Self::Segment,
    //     artifact: &mut Self,
    //     animations: &mut AnimationList<Self>,
    // ) -> bool {
    //     animations.clear();
    //     animations
    //         .push(animate_towards::<BackgroundColorLens>(segment.color, 1.0 / 1000.0).to_stage());

    //     true
    // }
}

// #[derive(Debug, PartialEq)]
// pub struct BackgroundColorEntity {
//     pub color: Srgba,
// }

// #[derive(Debug, Clone)]
// pub struct BackgroundColorArtifact {
//     pub color: RwSignal<Srgba>,
// }

// impl LeptosGameArtifact for BackgroundColor {
//     fn render(self) -> impl IntoView {
//         let style = move || {
//             format!(
//                 "height: 100vh; width: 100vw; overflow: hidden; background: {}",
//                 self.color.get().to_hex()
//             )
//         };
//         view! {
//             <div  style=style/>
//         }
//     }
// }

// define_signal_lens!(BackgroundColorArtifactColorLens, BackgroundColorArtifact, Srgba, color);

// impl GameArtifact for BackgroundColorArtifact {}

// impl GameEntity for BackgroundColorEntity {
//     type Artifact = BackgroundColorArtifact;
//     type Key = ();
//     type StateSegment = QuizSaladGameState;

//     fn key(&self) -> Self::Key {
//         ()
//     }

//     fn get_entities(segment: &Self::StateSegment) -> impl Iterator<Item = Self> {
//         let color = if segment.found_words.is_level_complete() {
//             CLASSIC_COLOR_SCHEME.background_incomplete
//         } else {
//             CLASSIC_COLOR_SCHEME.background_complete
//         };

//         [Self { color }].into_iter()
//     }

//     fn on_new(
//         &self,
//     ) -> (
//         Self::Artifact,
//         state_machine_games::prelude::AnimationList<Self::Artifact>,
//     ) {
//         BackgroundColorArtifact{
//             color: RwSignal::new(self.color)
//         }.with_animations([])
//     }

//     fn on_update(
//         &self,
//         artifact: &mut Self::Artifact,
//         former_entity_state: state_machine_games::prelude::EntityState,
//         previous_animations: state_machine_games::prelude::AnimationList<Self::Artifact>,
//     ) -> state_machine_games::prelude::AnimationList<Self::Artifact> {
//         artifact.update_animations([
//             animate_towards::<BackgroundColorArtifactColorLens>(self.color, 1.0/1000.0).to_stage()
//         ])
//     }
// }
