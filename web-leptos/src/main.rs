mod core;

use cgmath::num_traits::Float;
use leptos::attr::Width;
use leptos::prelude::*;
use leptos_use::use_element_size;
use leptos_use::use_throttle_fn;
use leptos_use::use_throttle_fn_with_arg;
use leptos_use::UseElementSizeReturn;
use log::trace;
use wasm_bindgen::convert::IntoWasmAbi;
use web_sys::PointerEvent;

use leptos::mount::mount_to_body;
// use leptos_use::docs::{demo_or_body, BooleanDisplay};
use leptos_use::use_interval;
use leptos_use::UseIntervalReturn;
use leptos_use::{use_window_size, UseWindowSizeReturn};

use leptos::html;

use shared::Event;

use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

#[allow(unused)]
use log::{debug, error, info, warn};
use web_sys::WheelEvent;

type DragStart = Option<[i32; 2]>;

#[component]
fn GameCanvas(
    //
    #[prop(into)] //
    view: Signal<shared::ViewModel>,
    set_event: WriteSignal<Event>,
) -> impl IntoView {
    let canvas_ref = NodeRef::<html::Canvas>::new();

    let UseElementSizeReturn { width, height } = use_element_size(canvas_ref);
    Effect::new(move |_| {
        let width = width.get() as f32;
        let height = height.get() as f32;
        set_event.set(Event::CameraSize([width, height]));
    });

    let (drag_start, set_drag_start) = signal(DragStart::default());
    let (drag_end, set_drag_end) = signal([0_i32, 0]);
    let (camera_old, set_camera_old) = signal([0_f32, 0.0]);
    Effect::new(move |old_v: Option<[f32; 2]>| {
        let old_v = old_v.unwrap_or_default();
        if let Some(start_pos) = drag_start.get() {
            let end_pos = drag_end.get();
            let drag = [end_pos[0] - start_pos[0], end_pos[1] - start_pos[1]];
            let drag = drag.map(|e| e as f32);
            let old_cam = camera_old.get();
            let new_pos = [old_cam[0] - drag[0], old_cam[1] - drag[1]];
            if new_pos != old_v {
                set_event.set(Event::CameraPan(new_pos));
            }
            new_pos
        } else {
            old_v
        }
    });
    let (zoom_pow, set_zoom_pow) = signal(1_f32);
    // let zoom = move || 2_f32.powf(zoom_pow.get()) / 2.0;
    // let (zoom, set_zoom) = signal(0_f32);

    let wheel_handler = use_throttle_fn_with_arg(
        move |dy: f64| {
            set_camera_old.set(view.get().camera_pan);

            set_zoom_pow.update(|old_pow| {
                let dy = dy as f32;
                let new_pow = (*old_pow + dy / 2000.0).clamp(-4.0, 4.0);
                let zoom = 2.0.powf(new_pow) / 2.0;
                // debug!("new zoom is {zoom} from pow: {new_pow}");
                set_event.set(Event::CameraZoom(zoom));

                *old_pow = new_pow
            });
        },
        20.0,
    );

    Effect::new(move |_| {
        if let Some(canvas) = canvas_ref.get() {
            let width = width.get() as f64;
            let height = height.get() as f64;
            let view = view.get();
            let cell_size = view.cell_size as f64;
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

            ctx.set_fill_style_str("red");
            ctx.begin_path();
            ctx.set_line_width(2.0);
            // let camx = camera_pos.get()[0] as f64 + width / 2.0;
            // let camy = camera_pos.get()[1] as f64 + height / 2.0;

            if cell_size > 20.0 {
                let mut x = view.modx() as f64;
                let mut y = view.mody() as f64;
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

                // if let Some(view) = view.get() {
            }
            for [x, y] in &view.cell_coords {
                let x = *x as f64;
                let y = *y as f64;
                ctx.rect(x, y, cell_size, cell_size);
            }

            // }
            ctx.fill();
            ctx.stroke();
        }
    });
    let click_handler = move |location: [i32; 2]| {
        debug!("location: {:?}", location);
        set_event.set(Event::ToggleScreenCoord(location.map(|e| e as f32)))
    };

    view! {
        <canvas id="canvas" on:pointerup=move |ev|{
            set_drag_start.set(None);
            set_camera_old.set(view.get().camera_pan);
            click_handler([ev.offset_x(), ev.offset_y()]);
        }
        on:pointerdown=move|ev|{
            set_drag_start.set(Some([ev.offset_x(), ev.offset_y()]));
        }
        on:pointermove=move |ev: PointerEvent|{
            set_drag_end.set([ev.offset_x(), ev.offset_y()]);
        }

        on:wheel=move |ev: WheelEvent| {
            wheel_handler(ev.delta_y());
        }
        node_ref=canvas_ref width=800 height=800 style="width:80vw; height: 60vh; border:2px solid #000000;">
        </canvas>
    }
}

#[component]
fn root_component() -> impl IntoView {
    let core = core::new();

    let (event, set_event) = signal(Event::Render);
    let (view, set_view) = signal(core.view());

    let (running, set_run) = signal(false);
    let millis = 15;
    let UseIntervalReturn {
        counter,
        pause,
        resume,
        ..
    } = use_interval(millis);

    let _timer = Effect::new(move || if running.get() { resume() } else { pause() });

    let _time_stepper = Effect::watch(
        move || counter.get(),
        move |_, _, _| set_event.set(Event::Step),
        false,
    );
    let _event_processor = Effect::new(move || {
        let event = event.get();
        trace!("got event: {:#?}", event);
        let effects = core.process_event(event);
        for effect in effects {
            match effect {
                shared::Effect::Alert(_) => todo!(),
                shared::Effect::FileIO(_) => todo!(),
                shared::Effect::Render(_) => set_view.set(core.view()),
            }
        }
    });

    view! { <>
    <main>
    <section class="section has-text-centered">
        <p class="title">{"Crux of Life"}</p>
        <p class="is-size-5">{"Rust Core, Leptos Shell"}</p>
        <GameCanvas view=view set_event=set_event/>
        <div class="buttons section is-centered">
            <button class="button is-primary is-warning"
                on:click=move |_| set_run.update(|state| *state = !*state)>
                    {"play/pause"}
            </button>
            <button class="button is-primary is-warning"
                on:click=move |_| set_event.update(|value| *value = Event::Step)>
                    {"Step"}
            </button>
        </div>

    </section>
    </main>
    </>
    }
}

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! { <RootComponent /> }
    });
}
