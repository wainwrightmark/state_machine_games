use std::{ops::Deref, time::Duration};

use leptos::prelude::*;
use leptos_use::use_timestamp;

use crate::{
    entity_store::EntityStore,
    leptos::{
        command_sender::CommandSender,
        game_entity::{LeptosGameEntity, LeptosGameState},
        leptos_entity_store::LeptosEntityStore,
    },
};

pub fn game_view_component<T: LeptosGameState>(
    viewbox_width: f32,
    viewbox_height: f32,
    game_state_read: ReadSignal<T>,
    game_state_write: WriteSignal<T>,
    settings: ReadSignal<T::Settings>,
    assets: ReadSignal<T::Assets>,
    storage: ReadSignal<T::Storage>,
) -> impl IntoView {
    let command_sender: CommandSender<T> = CommandSender::new(
        game_state_read,
        game_state_write,
        settings,
        assets,
        storage,
        Some(Duration::from_secs(0)),
    );

    let timestamp = use_timestamp();

    //use_raf_fn(|x|x.)

    let entity_store: Memo<LeptosEntityStore<T::Entity>> =
        Memo::new_owning(move |prev_map: Option<LeptosEntityStore<T::Entity>>| {
            let settings = settings.read();
            let assets = assets.read();
            let storage = storage.read();

            let state = game_state_read.read();

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

    // let node_ref = NodeRef::<Svg>::new();

    // let cs1 = command_sender.clone();
    // let cs2 = command_sender.clone();
    // let cs3 = command_sender.clone();
    // let cs4 = command_sender.clone();

    view! {
        <svg viewBox=format!("0 0 {viewbox_width} {viewbox_height}")  style="max-width: 800px;  margin-inline: auto; "
        // node_ref=node_ref
        // on:pointerdown = { move|event: PointerEvent|{ cs1.handle_event(PointerEventType::Start, event, node_ref);}}
        // on:pointerup = { move|event: PointerEvent|{ cs2.handle_event(PointerEventType::End, event, node_ref);}}
        // on:pointermove = { move|event: PointerEvent|{ cs3.handle_event(PointerEventType::Move, event, node_ref);}}
        // on:pointercancel = { move|event: PointerEvent|{ cs4.handle_event(PointerEventType::Cancel, event, node_ref);}}

        >
            <For
            each = move || entity_store.read().entities.clone()
            key = |stored_entity_signal| stored_entity_signal.key
            children = move |stored_entity_signal|{
                let action_sender = command_sender.clone();
                let now = timestamp.get_untracked();


                view!{
                    {move || {
                        let r =stored_entity_signal.signal.read();
                        r.0.render(&r.1, action_sender.clone(), now, timestamp)
                    }}
                }
            }

             />

        </svg>
    }
}
