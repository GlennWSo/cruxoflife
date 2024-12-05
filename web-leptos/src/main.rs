mod core;

use leptos::prelude::*;
use std::f64::consts::PI;
use std::ops::Deref;

use leptos::{attr::Height, mount::mount_to_body};
use leptos_use::{use_window_size, UseWindowSizeReturn};

use leptos::html;

use log::{debug, info};
use shared::Event;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_leptos::{draw_canvas, DrawScope};
use web_sys::CanvasRenderingContext2d;

#[component]
fn root_component() -> impl IntoView {
    let core = core::new();

    // create event signals
    let (view, render) = create_signal(core.view());
    let (event, set_event) = create_signal(Event::Step);
    create_effect(move |_| {
        core::update(&core, event.get(), render);
    });

    // draw the canvas
    let canvas_ref = create_node_ref::<html::Canvas>();
    let UseWindowSizeReturn { width, height } = use_window_size();
    create_effect(move |_| {
        if let Some(canvas) = canvas_ref.get() {
            // let width = canvas.offset_width() as u32;
            // let height = canvas.offset_height() as u32;

            let width = width.get();
            let height = height.get();
            // let h = height as f64;
            let cell_size = 50.0;
            let zoom = 1.0;

            let ncol = (width / cell_size) as u32;

            debug!("{:#?}", ncol);
            canvas.set_width(width as u32);
            canvas.set_height(height as u32);

            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();

            ctx.set_fill_style(&JsValue::from_str("red"));
            ctx.begin_path();
            ctx.set_line_width(2.0);

            let mut x = 0.0;
            for _ in 0..ncol {
                x += cell_size;
                ctx.move_to(x, 0.0);
                ctx.line_to(x, height);
            }

            ctx.stroke();
            // ctx.fill();
        }
    });

    let view = view! { <>
    <main>
    <section class="section has-text-centered">
        <p class="title">{"Crux Counter Example"}</p>
        <p class="is-size-5">{"Rust Core, Rust Shell (Leptos)"}</p>
        <p class="is-size-5">{move || view.get().to_string()}</p>
        <p class="is-size-5">{width}" "{height}</p>
        <div class="container">
            <canvas id="canvas" node_ref=canvas_ref width=800 height=800 style="width:80vw; height: 80vh; border:2px solid #000000;">
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
