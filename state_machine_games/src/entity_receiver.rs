use crate::prelude::*;
use std::collections::HashSet;

pub trait EntityReceiver<E: GameEntity> {
    fn receive(&mut self, entities: impl Iterator<Item = E>);
}

pub struct GeneralEntityReceiver<'s, E: GameEntity> {
    store: &'s mut SingleTypeEntityStore<E>,
    remaining_keys: HashSet<E::Key>,
    changed: bool,
}

impl<'s, E: GameEntity> GeneralEntityReceiver<'s, E> {
    pub fn new(store: &'s mut SingleTypeEntityStore<E>) -> Self {
        let remaining_keys = store.entities.keys().copied().collect();
        Self {
            store,
            remaining_keys,
            changed: false,
        }
    }

    pub fn finish(&mut self)-> bool {
        for k in self.remaining_keys.drain() {
            match self.store.entities.entry(k) {
                std::collections::btree_map::Entry::Vacant(_) => {
                    //should not happen
                }
                std::collections::btree_map::Entry::Occupied(mut occupied_entry) => {
                    let oe = occupied_entry.get_mut();
                    if oe.kill() {
                        occupied_entry.remove();
                        self.changed = true
                    }
                }
            }
        }

        self.store.animated_entities.clear();
        let animated_entity_keys = self
            .store
            .entities
            .iter()
            .filter(|(_, v)| v.has_animations())
            .map(|x| x.0)
            .copied();

        self.store.animated_entities.extend(animated_entity_keys);

        self.changed
    }
}

impl<'s, E: GameEntity> EntityReceiver<E> for GeneralEntityReceiver<'s, E> {
    fn receive(&mut self, entities: impl Iterator<Item = E>) {
        for entity in entities {
            let key = entity.key();

            self.remaining_keys.remove(&key);

            match self.store.entities.entry(key) {
                std::collections::btree_map::Entry::Vacant(vacant_entry) => {
                    let se: StoredEntity<E> = StoredEntity::new(entity);
                    vacant_entry.insert(se);
                    self.changed = true;
                }
                std::collections::btree_map::Entry::Occupied(mut occupied_entry) => {
                    let oe = occupied_entry.get_mut();

                    StoredEntity::update(entity, oe)
                }
            }
        }
    }
}
