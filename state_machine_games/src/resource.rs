use crate::prelude::{
    AnimateResult, AnimationList, ChangeWatcher, GameArtifact, GameState, HasSegment,
};

pub trait ResourceValue: GameArtifact + Default + Send + Sync + 'static {
    type Segment: Send + Sync + 'static;
    type GS: GameState + HasSegment<Self::Segment>;

    ///Returns whether anything was changed
    fn update_value(segment: &Self::Segment, artifact: &mut Self, animations: &mut AnimationList<Self>)-> bool;
}


pub struct ResourceStore<R: ResourceValue> {
    pub artifact: R,
    animations: AnimationList<R>,
    segment: R::Segment,
}

impl<R: ResourceValue> ChangeWatcher<R::GS> for ResourceStore<R> {
    fn on_state_change(
        &mut self,
        state: &R::GS,
        _reason: &crate::prelude::StateChangeReason<R::GS>,
    ) -> bool {
        if state.segment_eq(&self.segment){return false;};
        self.segment = state.get_segment();

        R::update_value(&self.segment, &mut self.artifact, &mut self.animations)

    }

    fn step_animations(&mut self, delta_ms: f64) {
        self.animations.inner.retain_mut(|animation| {
            match animation.step(&mut self.artifact, delta_ms) {
                AnimateResult::Continue => true,
                AnimateResult::FinishStep => false,
            }
        });
    }

    fn new(state: &R::GS) -> Self {
        let segment = state.get_segment();
        let mut artifact = R::default();
        let mut animations = AnimationList::default();

        R::update_value(&segment,&mut artifact,&mut animations);
        
        Self {
            artifact,
            animations,
            segment,
        }
    }
}
