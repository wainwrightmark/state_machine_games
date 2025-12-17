use crate::prelude::*;
use impl_trait_for_tuples::impl_for_tuples;
use std::collections::BTreeMap;

pub trait EntityStoreCombination<GS: GameState>: Send + Sync + 'static {
    //Regather entities from the state
    //Returns `true` if at least one entity has been added or removed
    fn gather_entities(&mut self, state: &GS) -> bool;

    //Step all animations in this store
    //This should never change the structure of the store
    fn step_animations(&mut self, delta_ms: f64);
}

#[impl_for_tuples(1, 8)]
impl<GS: GameState> EntityStoreCombination<GS> for Tuple {
    for_tuples!( where #( Tuple: EntityStoreCombination<GS> )* );

    fn gather_entities(&mut self, state: &GS) -> bool {
        for_tuples! (#(self.Tuple.gather_entities(state))| *)
    }

    fn step_animations(&mut self, delta_ms: f64) {
        for_tuples! (#(self.Tuple.step_animations(delta_ms);) *)
    }
}

pub struct SingleTypeEntityStore<E: GameEntity> {
    pub entities: BTreeMap<E::Key, StoredEntity<E>>, //todo just use a vec here
    pub animated_entities: Vec<E::Key>,              //todo use indices rather than keys
}

impl<E: GameEntity> Default for SingleTypeEntityStore<E> {
    fn default() -> Self {
        Self { entities: Default::default(), animated_entities: Default::default() }
    }
}

impl<E: GameEntity> EntityStoreCombination<E::GameState> for SingleTypeEntityStore<E> {
    fn gather_entities(&mut self, state: &E::GameState) -> bool {
        let mut receiver = GeneralEntityReceiver::new(self);
        receiver.receive(E::get_entities(state));        
        receiver.finish()
    }

    fn step_animations(&mut self, delta_ms: f64) {
        self.animated_entities.retain(|key| {
            if let Some(b) = self.entities.get_mut(key) {
                b.step_animate(delta_ms)
            } else {
                false
            }
        });
    }
}

#[cfg(feature = "leptos")]
impl<A: LeptosGameArtifact, E: GameEntity<Artifact = A>> SingleTypeEntityStore<E> {
    
    pub fn render(
        signal: leptos::prelude::ArcRwSignal<Self>,
        sender: impl CommandSender<<E::Artifact as GameArtifact>::Command>,
    ) -> impl leptos::IntoView {
        leptos::control_flow::For(leptos::prelude::ForProps {
            each: {
                let signal = signal.clone();
                move || {
                    let read_guard = leptos::prelude::Read::read(&signal);
                    //read_guard.trigger.track();
                    read_guard.entities.keys().copied().collect::<Vec<_>>()
                }
            },
            key: |&k| k,
            children: move |k| {
                let read_guard: leptos::prelude::guards::ReadGuard<_, _> = leptos::prelude::Read::read(&signal);

                let entity = read_guard.entities.get(&k).unwrap();
                //log!("Rendering child");

                entity.artifact.clone().render(sender.clone())
            },
        })
    }
}

impl<E: GameEntity> SingleTypeEntityStore<E> {
    pub fn new() -> Self {
        Self {
            entities: BTreeMap::new(),
            animated_entities: vec![],
        }
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
            prev.animations = new_entity.on_update(&mut prev.artifact, prev.state);
            prev.entity = new_entity;
            prev.state = EntityState::Alive;
        }
    }

    pub(crate) fn has_animations(&self) -> bool {
        !self.animations.is_empty()
    }

    pub(crate) fn step_animate(&mut self, delta_ms: f64) -> bool {
        self.animations.retain_mut(|animation| {
            match animation.step(&mut self.artifact, delta_ms) {
                AnimateResult::Continue => true,
                AnimateResult::DeleteAnimation => false,
            }
        });

        !self.animations.is_empty()
    }

    pub(crate) fn kill(&mut self) -> bool {
        match self.state {
            EntityState::Alive => {
                self.animations = self.entity.on_death(&mut self.artifact);
                self.state = EntityState::Dead;
            }
            EntityState::Dead => {}
        }

        self.animations.is_empty()
    }
}
