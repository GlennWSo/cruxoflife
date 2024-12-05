mod core;

use leptos::mount::mount_to_body;
use leptos::prelude::*;
use leptos_use::{use_window_size, UseWindowSizeReturn};

use log::info;
use shared::Event;

use web_leptos::draw_canvas;

#[component]
fn root_component() -> impl IntoView {
    let core = core::new();
    // create signals
    let (view, render) = create_signal(core.view());
    let (event, set_event) = create_signal(Event::Step);
    let UseWindowSizeReturn { width, height } = use_window_size();

    /// react to signals
    create_effect(move |_| {
        core::update(&core, event.get(), render);
        info!("w:{}, h:{}", width.get(), height.get());
        draw_canvas();
    });

    let view = view! { <>
    <main>
    <section class="section has-text-centered">
        <p class="title">{"Crux Counter Example"}</p>
        <p class="is-size-5">{"Rust Core, Rust Shell (Leptos)"}</p>
        <p class="is-size-5">{move || view.get().to_string()}</p>
        <p class="is-size-5">{width}" "{height}</p>
        <div class="container">
            <canvas id="canvas" width={width} height={height} style="width:80vw; height: 80vh; border:1px solid #000000;">
            </canvas>
            <div class="buttons section is-centered">
                <button class="button is-primary is-warning"
                    on:click=move |_| set_event.update(|value| *value = Event::Step)>
                        {"Step"}
                </button>
            </div>
        </div>

    </section>
    </main>
    </>
    };
    view
}

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! { <RootComponent /> }
    });
}
