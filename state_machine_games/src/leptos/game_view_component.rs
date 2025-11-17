use std::{ops::Deref, time::Duration};

use leptos::prelude::*;

use crate::{
    entity_store::EntityStore,
    leptos::{
        command_sender::CommandSender,
        game_entity::{LeptosGameEntity, LeptosGameState}, leptos_entity_store::LeptosEntityStore, 
    },
};

pub fn game_view_component<T: LeptosGameState>(
    viewbox_width: f32,
    viewbox_height: f32,
    signal: ReadSignal<T>,
    write_signal: WriteSignal<T>,
    settings: ReadSignal<T::Settings>,
    assets: ReadSignal<T::Assets>,
    storage: ReadSignal<T::Storage>,
) -> impl IntoView {
    let action_sender: CommandSender<T> = CommandSender::new(
        write_signal,
        settings,
        assets,
        storage,
        Some(Duration::from_secs(0)),
    );

    let entity_store: Memo<LeptosEntityStore<T::Entity>> =
        Memo::new_owning(move |prev_map: Option<LeptosEntityStore<T::Entity>>| {
            let settings = settings.read();
            let assets = assets.read();
            let storage = storage.read();

            let state = signal.read();

            let entities_iter =
                state.get_entities(settings.deref(), assets.deref(), storage.deref());

            match prev_map {
                Some(mut entity_store) => {
                    let now = web_time::Instant::now();
                    let changed = entity_store.update(entities_iter, now);
                    //log::info!("Entity Store {}. {} entities", if changed {"changed"} else {"unchanged"}, entity_store.entities.len());
                    return (entity_store, changed);
                }
                None => {
                    let store = EntityStore::new(entities_iter);

                    return (store, true);
                }
            }
        });

    view! {
        <svg viewBox=format!("0 0 {viewbox_width} {viewbox_height}")  style="width: 800px; ">
            <For
            each = move || entity_store.read().entities.clone()
            key = |stored_entity_signal| stored_entity_signal.key
            children = move |stored_entity_signal|{
                let action_sender = action_sender.clone();


                view!{
                    {move || {
                        let r =stored_entity_signal.signal.read();
                        r.0.render(&r.1, action_sender.clone())
                    }}
                }
            }

             />

        </svg>
    }
}
