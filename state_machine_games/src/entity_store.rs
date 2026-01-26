use std::sync::{Arc, RwLock};

use crate::prelude::*;
use const_sized_bit_set::prelude::*;
use impl_trait_for_tuples::impl_for_tuples;

#[derive(Debug)]
pub enum StateChangeReason<GS: GameState> {
    InitialState,
    Transition,
    Command(GS::Command),
}

pub trait ChangeWatcher<GS: GameState>: Send + Sync + 'static {
    //Regather entities from the state
    //Returns `true` if at least one entity has been added or removed
    fn on_state_change(&mut self, state: &GS, reason: &StateChangeReason<GS>) -> bool;

    //Step all animations in this store
    //This should never change the structure of the store
    fn step_animations(&mut self, delta_ms: f64);
}

pub trait InitFromGameState<GS: GameState> {
    fn init(state: &GS) -> Self;
}

// #[impl_for_tuples(1, 8)]
// impl<GS: GameState> ChangeWatcher<GS> for Tuple {
//     for_tuples!( where #( Tuple: ChangeWatcher<GS> )* );

//     fn on_state_change(&mut self, state: &GS, reason: &StateChangeReason<GS>) -> bool {
//         for_tuples! (#(self.Tuple.on_state_change(state, reason))| *)
//     }

//     fn step_animations(&mut self, delta_ms: f64) {
//         for_tuples! (#(self.Tuple.step_animations(delta_ms);) *)
//     }
// }

// #[impl_for_tuples(1, 8)]
// impl<GS: GameState> InitFromGameState<GS> for Tuple {
//     fn init(state: &GS) -> Self {
//         (for_tuples! (#(Tuple::init(state) ), *))
//     }
// }

pub struct SingleTypeEntityStore<E: GameEntity> {
    pub entities: Vec<StoredEntity<E>>,
    ///indexes to entities in entities
    pub animated_entities: BitSetVec,
    pub segment: E::StateSegment,
}

impl<E: GameEntity, GS: GameState + HasSegment<E::StateSegment>> ChangeWatcher<GS>
    for SingleTypeEntityStore<E>
{
    fn on_state_change(&mut self, state: &GS, _reason: &StateChangeReason<GS>) -> bool {
        if state.segment_eq(&self.segment) {
            return false;
        }
        self.segment = state.get_segment();

        let mut receiver = GeneralEntityReceiver::new(self);
        receiver.receive();
        receiver.finish()
    }

    fn step_animations(&mut self, delta_ms: f64) {
        self.animated_entities.retain(|key| {
            if let Some(b) = self.entities.get_mut(*key as usize) {
                b.step_animate(delta_ms)
            } else {
                false
            }
        });
    }
}

impl<E: GameEntity, GS: GameState + HasSegment<E::StateSegment>> InitFromGameState<GS>
    for SingleTypeEntityStore<E>
{
    fn init(state: &GS) -> Self {
        let mut s = Self {
            entities: Vec::new(),
            animated_entities: BitSetVec::EMPTY,
            segment: state.get_segment(),
        };

        let mut receiver = GeneralEntityReceiver::new(&mut s);
        receiver.receive();
        receiver.finish();

        s
    }
}

impl<GS: GameState, T: ChangeWatcher<GS>> ChangeWatcher<GS> for std::sync::Arc<RwLock<T>> {
    fn on_state_change(&mut self, state: &GS, reason: &StateChangeReason<GS>) -> bool {
        self.write().unwrap().on_state_change(state, reason)
    }

    fn step_animations(&mut self, delta_ms: f64) {
        self.write().unwrap().step_animations(delta_ms);
    }
}

impl<GS: GameState, T: InitFromGameState<GS>> InitFromGameState<GS> for std::sync::Arc<RwLock<T>> {
    fn init(state: &GS) -> Self {
        Arc::new(RwLock::new(T::init(state)))
    }
}

#[cfg(feature = "leptos")]
pub trait LeptosRenderable<Argument: Clone + Send + Sync + 'static, Command: Send + 'static>:
    Sized
{
    fn render(
        signal: leptos::prelude::ArcRwSignal<Self>,
        argument: Argument,
        sender: impl CommandSender<Command>,
    ) -> impl leptos::IntoView;
}

#[cfg(feature = "leptos")]
impl<
    Argument: Clone + Send + Sync + 'static,
    A: LeptosGameArtifact<Argument>,
    E: GameEntity<Artifact = A>,
> LeptosRenderable<Argument, A::Command> for SingleTypeEntityStore<E>
{
    fn render(
        signal: leptos::prelude::ArcRwSignal<Self>,
        argument: Argument,
        sender: impl CommandSender<A::Command>,
    ) -> impl leptos::IntoView {
        leptos::control_flow::For(leptos::prelude::ForProps {
            each: {
                let signal = signal.clone();
                move || {
                    let read_guard = leptos::prelude::Read::read(&signal);
                    read_guard
                        .entities
                        .iter()
                        .map(|x| x.entity.key())
                        .collect::<Vec<_>>()
                }
            },
            key: |k| k.clone(),
            children: move |k| {
                let read_guard: leptos::prelude::guards::ReadGuard<_, _> =
                    leptos::prelude::Read::read(&signal);

                //log!("Rendering child");
                let entities = &read_guard.entities;

                match entities.binary_search_by_key(&k, |x| x.entity.key()) {
                    Ok(index) => match entities.get(index) {
                        Some(stored_entity) => Some(
                            stored_entity
                                .artifact
                                .clone()
                                .render(argument.clone(), sender.clone()),
                        ),
                        None => None,
                    },
                    Err(_) => None,
                }
            },
        })
    }
}

pub struct StoredEntity<E: GameEntity> {
    pub entity: E,
    pub artifact: E::Artifact,
    pub animations: AnimationList<E::Artifact>,
    pub state: EntityState,
}

impl<E: GameEntity> StoredEntity<E> {
    pub fn new(entity: E) -> Self {
        let (artifact, animations) = entity.on_new();

        Self {
            entity,
            artifact,
            animations,
            state: EntityState::Alive,
        }
    }

    pub(crate) fn update(new_entity: E, prev: &mut StoredEntity<E>) {
        if prev.state == EntityState::Alive && prev.entity == new_entity {
            //do nothing
        } else {
            let previous_animations = std::mem::take(&mut prev.animations);
            prev.animations =
                new_entity.on_update(&mut prev.artifact, prev.state, previous_animations);
            prev.entity = new_entity;
            prev.state = EntityState::Alive;
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

    pub(crate) fn kill(&mut self) -> bool {
        match self.state {
            EntityState::Alive => {
                let previous_animations = std::mem::take(&mut self.animations);
                self.animations = self
                    .entity
                    .on_death(&mut self.artifact, previous_animations);
                self.state = EntityState::Dead;
            }
            EntityState::Dead => {}
        }

        self.animations.is_empty()
    }
}
