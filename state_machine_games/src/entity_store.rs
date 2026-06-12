use crate::prelude::*;
use const_sized_bit_set::prelude::*;
// use leptos::logging::log;

pub struct EntityStore<E: GameEntity> {
    pub entities: Vec<StoredEntity<E>>,
    ///indexes to entities in entities
    pub animated_entities: BitSetVec,
    pub segment: E::Segment,
}

impl<E: GameEntity> EntityStore<E> {
    pub fn new(state: E::Segment) -> Self {
        let mut s = Self {
            entities: Vec::new(),
            animated_entities: BitSetVec::EMPTY,
            segment: state,
        };

        let mut receiver = GeneralEntityReceiver::new(&mut s);
        receiver.receive();
        receiver.finish();

        s
    }
}

pub struct StoredEntity<E: GameEntity> {
    pub entity: E,
    pub artifact: E::Artifact,
    pub animations: AnimationList<E::Artifact>,
    pub lifecycle: EntityLifecycle,
}

impl<E: GameEntity> StoredEntity<E> {
    pub fn new(entity: E) -> Self {
        let (artifact, animations) = entity.on_new();

        Self {
            entity,
            artifact,
            animations,
            lifecycle: EntityLifecycle::Alive,
        }
    }

    pub(crate) fn update(new_entity: E, prev: &mut StoredEntity<E>) {
        if prev.lifecycle == EntityLifecycle::Alive && prev.entity == new_entity {
            //do nothing
        } else {
            let previous_animations = std::mem::take(&mut prev.animations);
            prev.animations =
                new_entity.on_update(&mut prev.artifact, prev.lifecycle, previous_animations);
            prev.entity = new_entity;
            prev.lifecycle = EntityLifecycle::Alive;
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
        match self.lifecycle {
            EntityLifecycle::Alive => {
                let previous_animations = std::mem::take(&mut self.animations);
                self.animations = self
                    .entity
                    .on_death(&mut self.artifact, previous_animations);
                self.lifecycle = EntityLifecycle::Dead;
            }
            EntityLifecycle::Dead => {}
        }

        let should_remove =  self.animations.is_empty();
        
        should_remove
    }
}
