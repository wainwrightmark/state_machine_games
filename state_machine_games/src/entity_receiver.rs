use std::collections::HashSet;

use leptos::prelude::Notify;

use crate::prelude::*;

pub struct GeneralEntityReceiver<'s, C: GameCommand, K: GameEntityKey> {
    store: &'s mut EntityStore<C, K>,
    remaining_keys: HashSet<K>,
    changed: bool,
}

impl<'s, C: GameCommand, K: GameEntityKey> GeneralEntityReceiver<'s, C, K> {
    pub fn new(store: &'s mut EntityStore<C, K>) -> Self {
        let remaining_keys = store.entities.keys().copied().collect();
        Self {
            store,
            remaining_keys,
            changed: false,
        }
    }

    pub fn finish(&mut self) {
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

        if self.changed {
            self.store.trigger.notify();
        }
    }
}

impl<'s, C: GameCommand, K: GameEntityKey> EntityReceiver<C, K>
    for GeneralEntityReceiver<'s, C, K>
{
    fn receive<E: GameEntity<Command = C, EntityKey = K>>(
        &mut self,
        entities: impl Iterator<Item = E>,
    ) {
        for entity in entities {
            let key = entity.key();

            self.remaining_keys.remove(&key);

            match self.store.entities.entry(key) {
                std::collections::btree_map::Entry::Vacant(vacant_entry) => {
                    let se: StoredEntity<E> = StoredEntity::new(entity);
                    vacant_entry.insert(Box::new(se));
                    self.changed = true;
                }
                std::collections::btree_map::Entry::Occupied(mut occupied_entry) => {
                    let oe = occupied_entry.get_mut();

                    if let Some(new_box) = StoredEntity::update_into_box(entity, oe) {
                        *oe = new_box;
                    }
                }
            }
        }
    }
}

pub trait EntityReceiver<C: GameCommand, K: GameEntityKey> {
    fn receive<E: GameEntity<Command = C, EntityKey = K>>(
        &mut self,
        entities: impl Iterator<Item = E>,
    );
}