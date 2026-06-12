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
}
