use crate::{little_bag::LittleBag, prelude::*};
use std::{fmt::Debug, sync::Mutex};

pub struct EntityStore<T: GameEntity, S: ValueSignal<Value = T::Artifact>> {
    pub stored_entities: Vec<StoredEntity<T, S>>,

    animated_entity_indices: Mutex<Vec<usize>>,
    swap_vec1: Vec<T>,
    swap_vec2: Vec<T>,
}

pub struct StoredEntity<T: GameEntity, S: ValueSignal<Value = T::Artifact>> {
    pub entity: T,
    pub artifact_signal: S,

    pub animations: LittleBag<Box<dyn Animation<T::Artifact>>>,
    pub state: EntityState,
}

impl<T: GameEntity + Debug, S: ValueSignal<Value = T::Artifact>> Debug for StoredEntity<T, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoredEntity")
            .field("entity", &self.entity)
            //.field("animations", &self.animations.len())
            .field("state", &self.state)
            .finish()
    }
}

impl<T: GameEntity, S: ValueSignal<Value = T::Artifact>> StoredEntity<T, S> {
    pub fn new(entity: T) -> Self {
        let (artifact, animations) = entity.on_new();
        let artifact_signal = S::new(artifact);

        let animations = LittleBag::new_from_vec(animations);        

        Self {
            entity,
            artifact_signal,
            animations,
            state: EntityState::Alive,
        }
    }

    pub fn key(&self) -> T::EntityKey {
        self.entity.key()
    }

    /// Set this entity to dead.
    /// Returns whether this should be retained
    pub fn set_to_dead(&mut self) -> bool {
        if !self.state.is_dead() {
            self.state = EntityState::Dead;

            self.artifact_signal.maybe_update(|artifact| {
                let animations = self.entity.on_death(artifact);

                let r = !animations.is_empty();
                self.animations = LittleBag::new_from_vec(animations);
                r
            });
        }

        //entity should be retained if and only if it still has animations
        self.animations.len() > 0
    }

    ///
    pub fn update(&mut self, new_value: T) {
        if self.state.is_alive() && self.entity == new_value {
            return;
        }

        self.artifact_signal.maybe_update(|artifact| {
            self.animations = LittleBag::new_from_vec(new_value.on_update(artifact, self.state));
            true
        });

        self.state = EntityState::Alive;
    }
}

impl<T: GameEntity, S: ValueSignal<Value = T::Artifact>> EntityStore<T, S> {
    pub fn artifact_signals(&self) -> impl Iterator<Item = (T::EntityKey, S)> {
        self.stored_entities
            .iter()
            .map(|x| (x.key(), x.artifact_signal.clone()))
    }

    pub fn new(entities: impl Iterator<Item = T>) -> Self {
        let mut entities: Vec<StoredEntity<T, S>> = entities
            .map(|entity| {
                let stored_entity: StoredEntity<T, S> = StoredEntity::new(entity);
                stored_entity
            })
            .collect();

        entities.sort_by_key(|x| x.key());
        let animated_entity_indices = Mutex::new(Self::collect_animated_entity_keys(&entities));

        Self {
            stored_entities: entities,
            swap_vec1: vec![],
            swap_vec2: vec![],
            animated_entity_indices,
        }
    }

    fn collect_animated_entity_keys(
        entities: &Vec<StoredEntity<T, S>>,
        //animated_entities: &mut Vec<usize>,
    ) -> Vec<usize> {
        // animated_entities.clear();
        // animated_entities.extend(
        //     entities
        //         .iter()
        //         .enumerate()
        //         .filter(|(_, x)| !x.animations.is_empty())
        //         .map(|(i, _)| i),
        // );

        entities
            .iter()
            .enumerate()
            .filter(|(_, x)| !x.animations.is_empty())
            .map(|(i, _)| i)
            .collect()
    }

    /// Update the entities.
    /// Returns whether the entities map was changed (ignores whether the values inside are changed)
    pub fn update(&mut self, entities: impl Iterator<Item = T>) -> bool {
        let mut changed = false;

        self.swap_vec1.clear();
        self.swap_vec2.clear();

        self.swap_vec1.extend(entities);
        self.swap_vec1.sort_by_key(|x| x.key());

        let mut swap_iter = self.swap_vec1.drain(..);
        let mut current = swap_iter.next();

        self.stored_entities.retain_mut(|stored_entity| {
            loop {
                match current.take() {
                    Some(current_entity) => {
                        let current_key = current_entity.key();
                        match stored_entity.key().cmp(&current_key) {
                            std::cmp::Ordering::Less => {
                                //This key is less than the current key. Therefore this entity should be deleted
                                //Current Should be put back to be compared against the next thing
                                current = Some(current_entity);

                                if stored_entity.set_to_dead() {
                                    return true;
                                } else {
                                    changed = true;
                                    return false;
                                }
                            }
                            std::cmp::Ordering::Equal => {
                                //this entity matches the current entity
                                current = swap_iter.next();
                                stored_entity.update(current_entity);
                                return true;
                            }
                            std::cmp::Ordering::Greater => {
                                //This element comes after the current entity.
                                //Therefore we don't know what to do with it
                                //But we do know we need to add the current entity later

                                self.swap_vec2.push(current_entity);
                                current = swap_iter.next();
                            }
                        }
                    }
                    None => {
                        //we have run out of entities. delete this
                        if stored_entity.set_to_dead() {
                            return true;
                        } else {
                            changed = true;
                            return false;
                        }
                    }
                }
            }
        });

        let mut any_added = false;

        for new_entity in self.swap_vec2.drain(..).chain(current).chain(swap_iter) {
            any_added = true;
            self.stored_entities.push(StoredEntity::new(new_entity));
        }

        if any_added {
            changed = true;
            self.stored_entities.sort_by_key(|x| x.key());
        }

        self.animated_entity_indices =
            Mutex::new(Self::collect_animated_entity_keys(&self.stored_entities));

        changed
    }

    #[must_use]
    pub fn try_animate_step(&self, delta_ms: f64) -> Option<()> {
        let mut guard = self.animated_entity_indices.lock().ok()?;

        guard.retain(|&index| {
            if let Some(stored_entity) = self.stored_entities.get(index) {
                stored_entity.animations.retain(|animation| {
                    match stored_entity
                        .artifact_signal
                        .try_update(|artifact| animation.step(artifact, delta_ms))
                    {
                        Some(AnimateResult::Continue) => true,
                        Some(AnimateResult::DeleteAnimation) => false,
                        None => false,
                    }
                });
                !stored_entity.animations.is_empty()
            } else {
                false
            }
        });

        Some(())
    }
}
