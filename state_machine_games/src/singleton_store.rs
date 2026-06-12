use crate::prelude::*;

use leptos::{
    IntoView,
    prelude::{Effect, GetUntracked, Read, RwSignal, Signal, Update, UpdateUntracked},
};

pub struct SingletonEntityStore<E: SingletonEntity> {
    pub artifact: E::Artifact,
    pub animations: AnimationList<E::Artifact>,
    pub segment: E::Segment,
}

impl<E: SingletonEntity> SingletonEntityStore<E> {
    pub fn new(segment: E::Segment) -> Self {
        let (artifact, animations) = E::init(&segment);
        Self {
            artifact,
            animations,
            segment,
        }
    }

    pub(crate) fn has_animations(&self) -> bool {
        !self.animations.is_empty()
    }

    pub(crate) fn step_animate(&mut self, delta_ms: f64) -> bool {
        self.animations.inner.retain_mut(|animation| {
            match animation.step(&mut self.artifact, delta_ms) {
                AnimateResult::Continue => true,
                AnimateResult::FinishStep => false,
            }
        });

        !self.animations.is_empty()
    }
}

pub trait SingletonEntity: Send + Sync + 'static + Sized {
    //The type of the state that this singleton can be derived from
    type Segment: PartialEq + Clone + Send + Sync + 'static + Sized;
    type Artifact: GameArtifact;

    fn init(segment: &Self::Segment) -> (Self::Artifact, AnimationList<Self::Artifact>);

    fn update(
        artifact: &mut Self::Artifact,
        current_segment: &Self::Segment,
        previous_segment: &Self::Segment,
        previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact>;

    fn render_singleton<LR: LeptosRender<Artifact = Self::Artifact>>(
        state_signal: Signal<Self::Segment>,
    ) -> impl IntoView {
        // leptos::logging::log!("Rendering singleton for {}", std::any::type_name::<Self>());
        let store1: RwSignal<SingletonEntityStore<Self>> = Self::track_singleton(state_signal);

        move || LR::render(store1.read().artifact.clone()) 
    }

    fn track_singleton(
        state_signal: Signal<Self::Segment>,
    ) -> RwSignal<SingletonEntityStore<Self>> {
        let store1: RwSignal<SingletonEntityStore<Self>> =
            RwSignal::new(SingletonEntityStore::new(state_signal.get_untracked()));

        let store = store1.clone();
        let pause_raf = leptos_use::use_raf_fn(move |a| {
            store.update_untracked(move |s| s.step_animate(a.delta));
        });

        let effect_fn = move || {
            store.maybe_update(move |store| {
                let segment = state_signal.read();
                if store.segment.eq(&segment) {
                    return false;
                }

                let prev_animations = std::mem::take(&mut store.animations);
                let new_animations = Self::update(
                    &mut store.artifact,
                    &segment,
                    &store.segment,
                    prev_animations,
                );
                store.animations = new_animations;

                store.segment = segment.clone();

                if store.has_animations() {
                    //(pause_raf.clone().resume)();
                } else {
                    //(pause.clone())()
                }

                true
            });
        };

        Effect::new(effect_fn);
        store1
    }
}
