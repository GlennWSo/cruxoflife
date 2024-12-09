use std::sync::Arc;

// use leptos::{spawn_local, SignalUpdate, WriteSignal};
use leptos::prelude::*;
use log::info;
use shared::{App, Effect, Event, ViewModel};

pub type Core = Arc<shared::Core<Effect, App>>;
// pub type Core = shared::Core<Effect, App>;

pub fn new() -> Core {
    Arc::new(shared::Core::new())
}
