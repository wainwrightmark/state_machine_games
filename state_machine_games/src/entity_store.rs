use std::{any::Any, collections::BTreeMap};
use leptos::prelude::*;
use crate::prelude::*;

pub struct EntityStore<C: GameCommand, K: GameEntityKey> {
    pub entities: BTreeMap<K, Box<dyn DynamicStoredEntity<C>>>,
    pub animated_entities: Vec<K>,
    pub trigger: ArcTrigger,
}

impl<C: GameCommand, K: GameEntityKey> EntityStore<C, K> {
    pub fn render(signal: ArcRwSignal<Self>, sender: BasicCommandSender<C>) -> impl IntoView {
        leptos::control_flow::For(ForProps {
            each: {
                let signal = signal.clone();
                move || {
                    let read_guard = signal.read_untracked();
                    read_guard.trigger.track();
                    read_guard.entities.keys().copied().collect::<Vec<_>>()
                }
            },
            key: |&k| k,
            children: move |k| {
                let read_guard: guards::ReadGuard<
                    EntityStore<C, K>,
                    guards::Plain<EntityStore<C, K>>,
                > = signal.read();

                let entity = read_guard.entities.get(&k).unwrap();
                //log!("Rendering child");

                entity.render(sender.clone())
            },
        })
    }
}

impl<C: GameCommand, K: GameEntityKey> EntityStore<C, K> {
    pub fn new() -> Self {
        Self {
            entities: BTreeMap::new(),
            animated_entities: vec![],
            trigger: ArcTrigger::new(),
        }
    }

    pub fn animate_step(&mut self, delta_ms: f64) {
        self.animated_entities.retain(|key| {
            if let Some(b) = self.entities.get_mut(key) {
                b.step_animate(delta_ms)
            } else {
                false
            }
        });
    }
}

pub trait DynamicStoredEntity<C: GameCommand>: Send + Sync + 'static + Any {
    fn render(&self, sender: BasicCommandSender<C>) -> AnyView;

    ///Step all animations
    /// Returns true if animations remain
    fn step_animate(&mut self, delta_ms: f64) -> bool;

    fn has_animations(&self) -> bool;

    ///Returns true if the entity should be removed
    fn kill(&mut self) -> bool;
}

static_assertions::assert_obj_safe!(DynamicStoredEntity<()>);

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

    ///Attempts to update the entity inside the box
    /// If the entities are different types, return a box with the updated entity
    pub fn update_into_box(
        new_entity: E,
        previous_entity: &mut Box<dyn DynamicStoredEntity<E::Command>>,
    ) -> Option<Box<Self>> {
        match <dyn std::any::Any>::downcast_mut::<StoredEntity<E>>(previous_entity.as_mut()) {
            Some(prev) => {
                if prev.state == EntityState::Alive && prev.entity == new_entity {
                    //do nothing
                } else {
                    prev.animations = new_entity.on_update(&mut prev.artifact, prev.state);
                    prev.entity = new_entity;
                    prev.state = EntityState::Alive;
                }
                return None;
            }
            None => {
                panic!("Entity has changed type");
            }
        }
    }
}

impl<E: GameEntity> DynamicStoredEntity<E::Command> for StoredEntity<E> {
    fn render(&self, sender: BasicCommandSender<E::Command>) -> AnyView {
        self.artifact.clone().render(sender).into_any()
    }

    fn has_animations(&self) -> bool {
        !self.animations.is_empty()
    }

    fn step_animate(&mut self, delta_ms: f64) -> bool {
        self.animations.retain_mut(|animation| {
            match animation.step(&mut self.artifact, delta_ms) {
                AnimateResult::Continue => true,
                AnimateResult::DeleteAnimation => false,
            }
        });

        !self.animations.is_empty()
    }

    fn kill(&mut self) -> bool {
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
