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

pub async fn update(core: &Core, event: Event, render: WriteSignal<ViewModel>) {
    log::debug!("event: {:?}", event);

    for effect in core.process_event(event) {
        process_effect(core, effect, render);
    }
}

pub fn process_effect(core: &Core, effect: Effect, render: WriteSignal<ViewModel>) {
    log::debug!("effect: {:?}", effect);

    match effect {
        Effect::Render(_) => {
            info!("perfomring render effect");
            render.update(|view| *view = core.view());
        }
        Effect::FileIO(_) => todo!(),
        Effect::Alert(_) => todo!(),
    };
}
