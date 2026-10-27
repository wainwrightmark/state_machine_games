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

    pub(crate) fn refresh_animation_list(&mut self) {
        self.animated_entities.clear();
        let animated_entity_keys = self
            .entities
            .iter()
            .enumerate()
            .filter(|(_index, e)| e.has_animations())
            .map(|x| x.0);

        self.animated_entities.extend(animated_entity_keys);
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

            let prev_entity = std::mem::replace(&mut prev.entity, new_entity);
            prev.animations = prev.entity.on_update(
                &mut prev.artifact,
                prev.lifecycle,
                prev_entity,
                previous_animations,
            );

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

    pub(crate) fn kill(&mut self, segment: &E::Segment) -> bool {
        match self.lifecycle {
            EntityLifecycle::Alive => {
                let previous_animations = std::mem::take(&mut self.animations);
                self.animations = self
                    .entity
                    .on_death(segment, &mut self.artifact, previous_animations);
                self.lifecycle = EntityLifecycle::Dead;
            }
            EntityLifecycle::Dead => {}
            EntityLifecycle::Reset => {
                self.animations.clear();
                self.lifecycle = EntityLifecycle::Dead;
            }
        }

        let should_remove = self.animations.is_empty();

        should_remove
    }
}
