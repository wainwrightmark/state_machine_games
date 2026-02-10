pub mod card;
pub mod game_state;
pub mod entities;
pub mod layout;

use core::f32;

use glam::Vec2;
use leptos::prelude::*;
use rand::{RngCore, seq::SliceRandom};
use rand_core::SeedableRng;
use state_machine_games::{define_signal_lens, prelude::*};
use strum::FromRepr;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    mount_to_body(|| game_component());
}

fn game_component() -> impl IntoView {
    view! {}
}
