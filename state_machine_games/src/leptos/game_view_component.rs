use std::{ops::Deref, time::Duration};
use leptos::prelude::*;
use leptos_use::{UseRafFnCallbackArgs, UseRafFnOptions, use_raf_fn, use_raf_fn_with_options};

use crate::{
    leptos::{
        command_sender::CommandSender, game_entity::LeptosGameState,
        leptos_entity_store::LeptosEntityStore, prelude::LeptosArtifact,
    },
    prelude::EntityStore,
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
                    let changed = entity_store.update(entities_iter);
                    //log::info!("Entity Store {}. {} entities", if changed {"changed"} else {"unchanged"}, entity_store.entities.len());
                    return (entity_store, changed);
                }
                None => {
                    let store = EntityStore::new(entities_iter);

                    return (store, true);
                }
            }
        });

    let entities = leptos::control_flow::For(ForProps {
        each: move || entity_store.read().artifact_signals().collect::<Vec<_>>(),
        key: |(key, _)| *key,
        children: move |(_key, signal)| {
            let command_sender = command_sender.clone();

            view! {
                {move || {
                    let r =signal.read();
                    r.render(command_sender.clone())
                }}
            }
        },
    });

    use_raf_fn(move |UseRafFnCallbackArgs{delta, timestamp: _}|{
        //entity_store.
        let entity_store =entity_store.with_untracked(|es|{
            es.animate_step(delta);
        });
        
    });

    view! {
        <svg viewBox=format!("0 0 {viewbox_width} {viewbox_height}")  style="max-width: 800px;  margin-inline: auto; "
        // node_ref=node_ref
        // on:pointerdown = { move|event: PointerEvent|{ cs1.handle_event(PointerEventType::Start, event, node_ref);}}
        // on:pointerup = { move|event: PointerEvent|{ cs2.handle_event(PointerEventType::End, event, node_ref);}}
        // on:pointermove = { move|event: PointerEvent|{ cs3.handle_event(PointerEventType::Move, event, node_ref);}}
        // on:pointercancel = { move|event: PointerEvent|{ cs4.handle_event(PointerEventType::Cancel, event, node_ref);}}

        >
        {entities}
        </svg>
    }
}
