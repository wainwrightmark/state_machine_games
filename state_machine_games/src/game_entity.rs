use const_sized_bit_set::prelude::BitSet;
use leptos::prelude::*;

use crate::prelude::*;

pub trait GameEntity: PartialEq + Send + Sync + 'static + Sized {
    type Artifact: GameArtifact;
    type Key: GameEntityKey;
    //The type of the state that these entities can be gotten from
    type Segment: PartialEq + Clone + Send + Sync + 'static + Sized;

    fn key(&self) -> Self::Key;

    //todo give the game state the opportunity to promise that the entities are sorted
    fn get_entities(segment: &Self::Segment) -> impl Iterator<Item = Self>;

    //todo reuse the same vec for all the animations

    #[allow(unused)]
    /// animations to run when this entity dies
    /// the entity will not be deleted until all animations have finished
    fn on_death(
        &self,
        artifact: &mut Self::Artifact,
        previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        AnimationList::EMPTY
    }

    /// what to do when this entity is new
    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>);

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        former_entity_lifecycle: EntityLifecycle,
        previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact>; //todo try mutating the animations

    fn track_entities(state_signal: Signal<Self::Segment>) -> RwSignal<EntityStore<Self>> {
        let store1: RwSignal<EntityStore<Self>> =
            RwSignal::new(EntityStore::new(state_signal.get_untracked()));

        let store = store1.clone();
        let pause_raf = leptos_use::use_raf_fn(move |a| {
            store.update_untracked(move |s| {
                // if s.animated_entities.count() > 0 {
                //     log!("Animating {} Entities of type {}", s.animated_entities.count(), std::any::type_name::<Self>());
                // }

                s.animated_entities.retain(|key| {
                    if let Some(b) = s.entities.get_mut(*key as usize) {
                        let retain = b.step_animate(a.delta);
                        if !retain {
                            //log!("Finished animating Entity of type {}",  std::any::type_name::<Self>());
                        }
                        retain
                    } else {
                        false
                    }
                });
            });
        });
        let store = store1.clone();
        //todo actually pause
        let _pause = pause_raf.pause.clone();
        let effect_fn = move || {
            store.maybe_update(move |store| {
                let segment = state_signal.read();
                if store.segment.eq(&segment) {
                    return false;
                }

                let store_reset = Self::should_reset_store(&segment, &store.segment);

                store.segment = segment.clone();

                if store_reset {
                    for entity in store.entities.iter_mut(){
                        entity.lifecycle = EntityLifecycle::Reset;
                    }
                }

                let mut receiver = GeneralEntityReceiver::new(store);
                receiver.receive();
                let changed = receiver.finish() || store_reset;
                if changed {
                    if store.animated_entities.is_empty() {

                        //(pause.clone())()
                        //(Clone::clone(&pause_raf.pause))();
                    } else {
                        //(pause_raf.clone().resume)();
                    }
                }

                changed
            });
        };

        Effect::new(effect_fn);
        store1
    }

    fn should_reset_store(_new_state: &Self::Segment, _prev_state: &Self::Segment) -> bool {
        false
    }

    fn render_entities<LR: LeptosRender<Artifact = Self::Artifact>>(
        state_signal: Signal<Self::Segment>,
    ) -> impl IntoView {
        let store1 = Self::track_entities(state_signal);

        let store = store1.clone();
        let for_props = leptos::prelude::ForProps {
            each: move || {
                let artifacts: Vec<Self::Key> = store
                    .read()
                    .entities
                    .iter()
                    .map(|x| x.entity.key())
                    .collect();

                // leptos::logging::log!(
                //     "{} artifacts of {}",
                //     artifacts.len(),
                //     std::any::type_name::<Self>()
                // );

                artifacts
            },
            key: |key| *key,
            children: move |key| {
                let read_guard = &store.read_untracked();

                // leptos::logging::log!(
                //     "Rendering {} Artifact with key {key:?}",
                //     std::any::type_name::<Self>()
                // );
                let entities = &read_guard.entities;

                match entities.binary_search_by_key(&key, |x| x.entity.key()) {
                    Ok(index) => match entities.get(index) {
                        Some(stored_entity) => Some(LR::render(stored_entity.artifact.clone())),
                        None => None,
                    },
                    Err(_) => None,
                }
            },
        };

        leptos::control_flow::For(for_props)
    }
}
