use std::rc::Rc;

use futures_util::TryStreamExt;
use leptos::{spawn_local, SignalUpdate, WriteSignal};
use shared::{App, Effect, Event, ViewModel};

pub type Core = Rc<shared::Core<Effect, App>>;

pub fn new() -> Core {
    Rc::new(shared::Core::new())
}

pub fn update(core: &Core, event: Event, render: WriteSignal<ViewModel>) {
    log::debug!("event: {:?}", event);

    for effect in core.process_event(event) {
        process_effect(core, effect, render);
    }
}

pub fn process_effect(core: &Core, effect: Effect, render: WriteSignal<ViewModel>) {
    log::debug!("effect: {:?}", effect);

    match effect {
        Effect::Render(_) => {
            render.update(|view| *view = core.view());
        }
        Effect::FileIO(_) => todo!(),
        Effect::Alert(_) => todo!(),
    };
}
