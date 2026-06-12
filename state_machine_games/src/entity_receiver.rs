use crate::prelude::*;
use const_sized_bit_set::prelude::*;
use std::collections::HashSet;

pub trait EntityReceiver<E: GameEntity> {
    fn receive(&mut self);
}

pub struct GeneralEntityReceiver<'s, E: GameEntity> {
    store: &'s mut EntityStore<E>,
    remaining_keys: HashSet<E::Key>, //todo use a vec
    changed: bool,
}

impl<'s, E: GameEntity> GeneralEntityReceiver<'s, E> {
    pub fn new(store: &'s mut EntityStore<E>) -> Self {
        let remaining_keys = store.entities.iter().map(|x| x.entity.key()).collect();
        Self {
            store,
            remaining_keys,
            changed: false,
        }
    }

    pub fn finish(&mut self) -> bool {
        for k in self.remaining_keys.drain() {
            match self
                .store
                .entities
                .binary_search_by_key(&k, |x| x.entity.key())
            {
                Ok(index) => match self.store.entities.get_mut(index) {
                    Some(ab) => {
                        if ab.kill() {
                            self.store.entities.remove(index);
                            self.changed = true
                        }
                    }
                    None => {}
                },
                Err(_) => {
                    //should not happen
                }
            }
        }

        self.store.animated_entities.clear();
        let animated_entity_keys = self
            .store
            .entities
            .iter()
            .enumerate()
            .filter(|(_index, e)| e.has_animations())
            .map(|x| x.0);

        self.store.animated_entities.extend(animated_entity_keys);

        self.changed
    }
}

impl<'s, E: GameEntity> EntityReceiver<E> for GeneralEntityReceiver<'s, E> {
    fn receive(&mut self) {
        let entities= E::get_entities(&self.store.segment);
        for entity in entities {
            let key = entity.key();

            self.remaining_keys.remove(&key);

            match self
                .store
                .entities
                .binary_search_by_key(&key, |x| x.entity.key())
            {
                Ok(get_index) => {
                    let oe = self.store.entities.get_mut(get_index).unwrap();
                    StoredEntity::update(entity, oe)
                }
                Err(insert_index) => {
                    let se: StoredEntity<E> = StoredEntity::new(entity);
                    self.store.entities.insert(insert_index, se);
                    self.changed = true;
                }
            }
        }
    }
}
