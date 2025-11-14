use crate::{game_entity::GameEntity, game_state::GameState};

// use itertools::Itertools;
// use leptos::prelude::{ArcRwSignal, Update};
use strum::EnumIs;

pub struct EntityStore<T: GameEntity, S: ValueSignal<Value = (T, StoredEntityMeta<T>)>> {
    pub entities: Vec<StoredEntitySignal<T, S>>,
    swap_vec1: Vec<T>,
    swap_vec2: Vec<T>,
}

#[derive(Debug, EnumIs)]
pub enum StoredEntityMeta<T> {
    New,
    Unchanged,
    Changed { previous: T },
    Dead,
}

pub trait ValueSignal: Clone {
    type Value: Sized + 'static;

    fn new(value: Self::Value) -> Self;

    /// Updates the value of the signal, but only notifies subscribers if the function
    /// returns `true`.
    #[track_caller]
    fn maybe_update(&self, fun: impl FnOnce(&mut Self::Value) -> bool) {
        self.try_maybe_update(|val| {
            let did_update = fun(val);
            (did_update, ())
        });
    }

    /// Updates the value of the signal and notifies subscribers, returning the value that is
    /// returned by the update function, or `None` if the signal has already been disposed.
    #[track_caller]
    fn try_update<U>(&self, fun: impl FnOnce(&mut Self::Value) -> U) -> Option<U> {
        self.try_maybe_update(|val| (true, fun(val)))
    }

    /// Updates the value of the signal, notifying subscribers if the update function returns
    /// `(true, _)`, and returns the value returned by the update function,
    /// or `None` if the signal has already been disposed.
    fn try_maybe_update<U>(&self, fun: impl FnOnce(&mut Self::Value) -> (bool, U)) -> Option<U>;
}

#[derive(Debug)]
pub struct StoredEntitySignal<T: GameEntity, S: ValueSignal<Value = (T, StoredEntityMeta<T>)>> {
    pub key: <<T as GameEntity>::GameState as GameState>::EntityKey,
    pub signal: S,
    pub remove_at: Option<web_time::Instant>,
}

impl<T: GameEntity, S: ValueSignal<Value = (T, StoredEntityMeta<T>)>> Clone
    for StoredEntitySignal<T, S>
{
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            signal: self.signal.clone(),
            remove_at: self.remove_at.clone(),
        }
    }
}

impl<T: GameEntity, S: ValueSignal<Value = (T, StoredEntityMeta<T>)>> StoredEntitySignal<T, S> {
    pub fn new(t: T) -> Self {
        return Self {
            key: t.key(),
            signal: S::new((t, StoredEntityMeta::New)),
            remove_at: None,
        };
    }

    /// Set this entity to dead.
    /// Returns whether this should be retained
    pub fn set_to_dead(&mut self, now: web_time::Instant) -> bool {
        match self.remove_at {
            Some(remove_at) => {
                return remove_at < now;
            }
            None => {
                match self.signal.try_update(|x| {
                    x.1 = StoredEntityMeta::Dead;
                    x.0.death_duration()
                }) {
                    Some(Some(duration)) => {
                        self.remove_at = Some(now + duration);
                        return true;
                    }
                    Some(None) => {
                        return false;
                    }
                    None => {
                        return false;
                    }
                }
            }
        }
    }

    pub fn update(&mut self, new_value: T) {
        self.remove_at = None;

        self.signal.maybe_update(|(entity, meta)| {
            if *entity == new_value {
                if meta.is_unchanged() {
                    return false;
                } else {
                    *meta = StoredEntityMeta::Unchanged;
                    return true;
                }
            } else {
                let previous = std::mem::replace(entity, new_value);
                *meta = StoredEntityMeta::Changed { previous };
                return true;
            }
        });
    }
}

impl<T: GameEntity, S: ValueSignal<Value = (T, StoredEntityMeta<T>)>> EntityStore<T, S> {
    pub fn new(entities: impl Iterator<Item = T>) -> Self {
        let mut entities: Vec<StoredEntitySignal<T, S>> = entities
            .map(|entity| {
                //let key = entity.key();
                let ses = StoredEntitySignal::new(entity);

                ses
            })
            .collect();

        entities.sort_by_key(|x| x.key);

        Self {
            entities,
            swap_vec1: vec![],
            swap_vec2: vec![],
        }
    }

    /// Update the entities.
    /// Returns whether the entities map was changed (ignores whether the values inside are changed)
    pub fn update(&mut self, entities: impl Iterator<Item = T>, now: web_time::Instant) -> bool {
        let mut changed = false;

        self.swap_vec1.clear();
        self.swap_vec2.clear();

        self.swap_vec1.extend(entities);
        self.swap_vec1.sort_by_key(|x| x.key());

        let mut swap_iter = self.swap_vec1.drain(..);
        let mut current = swap_iter.next();

        self.entities.retain_mut(|element| {
            loop {
                match current.take() {
                    Some(current_entity) => {
                        let current_key = current_entity.key();
                        match element.key.cmp(&current_key) {
                            std::cmp::Ordering::Less => {
                                //This key is less than the current key. Therefore this entity should be deleted
                                //Current Should be put back to be compared against the next thing
                                current = Some(current_entity);

                                if element.set_to_dead(now) {
                                    return true;
                                } else {
                                    changed = true;
                                    return false;
                                }
                            }
                            std::cmp::Ordering::Equal => {
                                //this entity matches the current entity
                                current = swap_iter.next();
                                element.update(current_entity);
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
                        if element.set_to_dead(now) {
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
            self.entities.push(StoredEntitySignal::new(new_entity));
        }

        if any_added {
            changed = true;
            self.entities.sort_by_key(|x| x.key);
        }

        changed
    }
}
