mod core;

use leptos::{component, create_effect, create_signal, view, IntoView, SignalGet, SignalUpdate};
use shared::Event;

#[component]
fn root_component() -> impl IntoView {
    let core = core::new();
    let (view, render) = create_signal(core.view());
    let (event, set_event) = create_signal(Event::Step);

    create_effect(move |_| {
        core::update(&core, event.get(), render);
    });

    view! {
        <>
            <script type="module" src="hello.js"></script>
            <section class="section has-text-centered">
                <p class="title">{"Crux Counter Example"}</p>
                <p class="is-size-5">{"Rust Core, Rust Shell (Leptos)"}</p>
                <p class="is-size-5">{move || view.get().to_string()}</p>
            </section>
            <section class="section has-text-centered">
                <canvas id="myCanvas" width="200" class="section has-text-centered" height="100" style="border:1px solid #000000;">
                </canvas>
                <script>
                </script>
            </section>

            <section class="container has-text-centered">
                <div class="buttons section is-centered">
                    <button class="button is-primary is-warning"
                        on:click=move |_| set_event.update(|value| *value = Event::Step)
                    >
                        {"Step"}
                    </button>
                </div>
            </section>
        </>
    }
}

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    leptos::mount_to_body(|| {
        view! { <RootComponent /> }
    });
}
