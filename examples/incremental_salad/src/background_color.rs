use bevy_color::Srgba;
use leptos::prelude::RwSignal;
use state_machine_games::{
    define_signal_lens,
    prelude::{GameArtifact, animate_towards},
};

use crate::{colors::CLASSIC_COLOR_SCHEME, *};

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

impl HasSegment<BackgroundTargetColor> for RogueSaladGameState {
    fn get_segment(&self) -> BackgroundTargetColor {
        BackgroundTargetColor {
            color: match &self.phase {
                Phase::ChooseBoon(..) => CLASSIC_COLOR_SCHEME.background_complete,
                Phase::PlayPuzzle(..) => CLASSIC_COLOR_SCHEME.background_incomplete,
                Phase::ChoosePath(..) => CLASSIC_COLOR_SCHEME.background_complete,
                Phase::GameOver => CLASSIC_COLOR_SCHEME.background_complete,
                Phase::Victory => CLASSIC_COLOR_SCHEME.background_complete,
            },
        }
    }

    fn segment_eq(&self, s: &BackgroundTargetColor) -> bool {
        <Self as HasSegment<BackgroundTargetColor>>::get_segment(self).eq(s)
    }
}

impl ResourceValue for BackgroundColor {
    type Segment = BackgroundTargetColor;

    type GS = RogueSaladGameState;

    fn update_value(
        segment: &Self::Segment,
        artifact: &mut Self,
        animations: &mut AnimationList<Self>,
    ) -> bool {
        animations.clear();
        animations
            .push(animate_towards::<BackgroundColorLens>(segment.color, 1.0 / 1000.0).to_stage());

        true
    }
}
