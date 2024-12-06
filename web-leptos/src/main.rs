mod core;

use leptos::attr::height;
use leptos::prelude::*;
use std::f64::consts::PI;
use std::ops::Deref;
use web_sys::PointerEvent;

use leptos::{attr::Height, mount::mount_to_body};
use leptos_use::{use_window_size, UseWindowSizeReturn};

use leptos::html;

use log::{debug, info};
use shared::Event;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_leptos::{draw_canvas, DrawScope};
use web_sys::CanvasRenderingContext2d;

type DragStart = Option<[i32; 2]>;

#[component]
fn root_component() -> impl IntoView {
    let core = core::new();

    // create event signals
    let (view, render) = create_signal(core.view());
    let (event, set_event) = create_signal(Event::Step);
    create_effect(move |_| {
        core::update(&core, event.get(), render);
    });

    let (drag_start, set_drag_start) = signal(DragStart::default());
    let (drag_end, set_drag_end) = signal([0_i32, 0]);
    let (camera_pos, set_camera_pos) = signal([0_i32, 0]);
    let (camera_old, set_camera_old) = signal([0_i32, 0]);
    create_effect(move |_| {
        if let Some(start_pos) = drag_start.get() {
            let end_pos = drag_end.get();
            let drag = [end_pos[0] - start_pos[0], end_pos[1] - start_pos[1]];
            let old_pos = camera_old.get();
            let new_pos = [drag[0] + old_pos[0], drag[1] + old_pos[1]];
            set_camera_pos.update(|pos| *pos = new_pos);
            info!("draged:  to {:?}", new_pos);
        }
    });

    // draw the canvas
    let canvas_ref = create_node_ref::<html::Canvas>();
    let UseWindowSizeReturn { width, height } = use_window_size();
    create_effect(move |_| {
        if let Some(canvas) = canvas_ref.get() {
            let width = width.get();
            let height = height.get();
            let cell_size = 50.0;
            let zoom = 1.0;
            let ncol = (width / cell_size) as u32 + 2;
            let nrow = (height / cell_size) as u32 + 2;

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

            // draw grid

            let camx = camera_pos.get()[0] as f64;
            let camy = camera_pos.get()[1] as f64;

            let mut x = camx % cell_size - cell_size;
            let mut y = camy % cell_size - cell_size;
            for _ in 0..ncol {
                x += cell_size;
                ctx.move_to(x, 0.0);
                ctx.line_to(x, height);
            }
            for _ in 0..nrow {
                y += cell_size;
                ctx.move_to(0.0, y);
                ctx.line_to(width, y);
            }

            let view = view.get();
            for [row, col] in view.life {
                let x = camx + row as f64 * cell_size;
                let y = camy + col as f64 * cell_size;
                ctx.rect(x, y, cell_size, cell_size);
            }
            ctx.fill();
            ctx.stroke();
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
            <canvas id="canvas" on:pointerup=move |ev|{
                set_drag_start.set(None);
                set_camera_old.set(camera_pos.get());
            }
            on:pointerdown=move|ev|{
                let camera_pos = camera_pos.get();
                set_drag_start.set(Some([ev.offset_x(), ev.offset_y()]));
            }
            on:pointermove=move |ev: PointerEvent|{
                set_drag_end.set([ev.offset_x(), ev.offset_y()]);
            }

            node_ref=canvas_ref width=800 height=800 style="width:80vw; height: 80vh; border:2px solid #000000;">
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
