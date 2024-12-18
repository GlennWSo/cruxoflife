mod core;

use cgmath::num_traits::Float;
use cgmath::InnerSpace;
use leptos::attr::default;
use leptos::attr::Width;
use leptos::prelude::*;
use leptos::tachys::dom::window;
use leptos_use::core::IntoElementMaybeSignal;
use leptos_use::use_element_size;
use leptos_use::use_throttle_fn;
use leptos_use::use_throttle_fn_with_arg;
use leptos_use::UseElementSizeReturn;
use log::trace;
use shared::Vec2;
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
use web_sys::TouchEvent;
use web_sys::WheelEvent;
use web_sys::Window;

type DragStart = Option<[i32; 2]>;

fn is_touch_device(window: Window) -> bool {
    let navigator = window.navigator();
    navigator.max_touch_points() > 0
}

fn avg_touch_pos(ev: &TouchEvent) -> Option<Vec2> {
    let n = ev.touches().length();
    if n == 0 {
        return None;
    }
    let touch_sum: Vec2 = (0..n)
        .filter_map(|i| ev.touches().get(i))
        .map(|t| Vec2::new(t.client_x() as f32, t.client_y() as f32))
        .sum();
    let avg = touch_sum / (n as f32);
    Some(avg)
}

fn avg_touch_spread(ev: &TouchEvent) -> Option<f32> {
    let Some(avg) = avg_touch_pos(ev) else {
        return None;
    };
    let n = ev.touches().length();
    let avg_dist_sum: f32 = (0..n)
        .filter_map(|i| ev.touches().get(i))
        .map(|t| (avg - Vec2::new(t.client_x() as f32, t.client_y() as f32)).magnitude())
        .sum();

    let avg_dist = avg_dist_sum / (n as f32);
    if avg_dist > 4.0 {
        Some(avg_dist)
    } else {
        None
    }
}

fn avg_touch_moved_pos(ev: &TouchEvent) -> Option<Vec2> {
    let total_n = ev.touches().length();
    let changed_n = ev.changed_touches().length();
    if changed_n < 1 {
        return None;
    }
    let touch_sum: Vec2 = (0..changed_n)
        .filter_map(|i| ev.touches().get(i))
        .map(|t| Vec2::new(t.client_x() as f32, t.client_y() as f32))
        .sum();
    let avg = touch_sum / (total_n as f32);
    Some(avg)
}

#[component]
fn GameCanvas(
    //
    #[prop(into)] //
    view: Signal<shared::ViewModel>,
    set_event: WriteSignal<Event>,
    is_touch: bool,
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
    let (zoom_pow, set_zoom_pow) = signal(1_f32);
    let (zoom, set_zoom) = signal(1_f32);
    Effect::new(move |old_v: Option<f32>| {
        let old_v = old_v.unwrap_or(1.0);
        let zoom = zoom.get();
        if let Some(start_pos) = drag_start.get() {
            let end_pos = drag_end.get();
            let drag = [end_pos[0] - start_pos[0], end_pos[1] - start_pos[1]];
            let drag = drag.map(|e| e as f32);
            let old_cam = camera_old.get();
            let new_pos = [old_cam[0] - drag[0], old_cam[1] - drag[1]];
            set_event.set(Event::CameraPanZoom([new_pos[0], new_pos[1], zoom]));
            zoom
        } else {
            if zoom != old_v {
                set_event.set(Event::CameraZoom(zoom));
            }
            zoom
        }
    });

    let wheel_handler = use_throttle_fn_with_arg(
        move |dy: f64| {
            set_camera_old.set(view.get().camera_pan);

            set_zoom_pow.update(|old_pow| {
                let dy = dy as f32;
                let new_pow = (*old_pow + dy / 2000.0).clamp(-4.0, 4.0);
                let zoom = 2.0.powf(new_pow) / 2.0;
                *old_pow = new_pow;
                set_zoom.set(zoom);
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

            let draw_grid = cell_size > 13.0;
            if draw_grid {
                ctx.set_line_width(2.0);
                let mut x = view.modx as f64;
                let mut y = view.mody as f64;
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
            } else {
                // ctx.set_line_width(0.0);
            }
            for [x, y] in &view.cell_coords {
                let x = *x as f64;
                let y = *y as f64;
                ctx.rect(x, y, cell_size, cell_size);
            }

            // }
            ctx.fill();
            if draw_grid {
                ctx.stroke();
            }
        }
    });
    let click_handler = move |location: [i32; 2]| {
        trace!("location: {:?}", location);
        set_event.set(Event::ToggleScreenCoord(location.map(|e| e as f32)))
    };

    let handle_pointerup = move |ev: PointerEvent| {
        if !is_touch {
            set_drag_start.set(None);
            set_camera_old.set(view.get().camera_pan);
            click_handler([ev.offset_x(), ev.offset_y()]);
            // window().alert_with_message("derp");
            // panic!();
        }
    };
    let handle_pointerdown = move |ev: PointerEvent| {
        if !is_touch {
            set_drag_start.set(Some([ev.offset_x(), ev.offset_y()]));
        }
    };
    let handle_pointermove = move |ev: PointerEvent| {
        if !is_touch {
            set_drag_end.set([ev.offset_x(), ev.offset_y()]);
        }
    };

    let handle_touchdown = move |ev: TouchEvent| {
        if let Some(avg) = avg_touch_pos(&ev) {
            debug!("t start: {avg:?}");
            set_drag_start.set(Some([avg.x.floor() as i32, avg.y.floor() as i32]));
            set_drag_end.set([avg.x.floor() as i32, avg.y.floor() as i32]);
        }
    };
    let (touch_spread, set_spread) = signal(Option::<f32>::default());

    let pinch2zoom = move |ev: &TouchEvent| -> Option<f32> {
        let new_spread = avg_touch_spread(ev);
        let Some(mut spread) = new_spread else {
            set_spread.set(None);
            return None;
        };
        let Some(old_spread) = touch_spread.get() else {
            set_spread.set(Some(spread));
            return None;
        };
        set_spread.set(Some(spread));
        set_zoom.update(|zoom| {
            *zoom = *zoom * spread / old_spread;
            spread = *zoom;
        });
        Some(spread)
    };

    let handle_touchmove = move |ev: TouchEvent| {
        if let Some(avg) = avg_touch_pos(&ev) {
            debug!("t move: {avg:?}");
            set_drag_end.set([avg.x.floor() as i32, avg.y.floor() as i32]);
            ev.prevent_default();
            if let Some(zoom) = pinch2zoom(&ev) {
                // set_event.set(Event::CameraZoom(zoom));
                set_zoom.set(zoom);
            };

            // ev.cancel_bubble();
        }
    };
    let handle_touchup = move |_ev: TouchEvent| {
        // if let Some(avg) = avg_touch_pos(&ev) {
        let start: Vec2 = drag_start.get().unwrap().map(|e| e as f32).into();
        let end: Vec2 = drag_end.get().map(|e| e as f32).into();
        debug!("drag d: {:#?}", end - start);

        set_drag_start.set(None);
        set_camera_old.set(view.get().camera_pan);
        click_handler(drag_end.get());
        // window().alert();
        // }
    };

    view! {
        <canvas  id="canvas"
            on:pointerdown=handle_pointerdown
            on:pointermove=handle_pointermove
            on:pointerup=handle_pointerup
            on:wheel=move |ev: WheelEvent| {wheel_handler(ev.delta_y());}

            on:touchstart=handle_touchdown
            on:touchend=handle_touchup
            on:touchmove=handle_touchmove

        node_ref=canvas_ref width=800 height=800 style="width:101vw; height: 100vh; border:2px solid #000000; position: absolute; top:0px; left:0px;">
        </canvas>
    }
}

#[component]
fn root_component() -> impl IntoView {
    let core = core::new();
    let touch_device = is_touch_device(window());

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

    let touch_label = if touch_device {
        Some(view! {<p class="is-size-7"> {"Touch Device detected"}</p> })
    } else {
        None
    };

    view! {
    <main >
    <section class="section has-text-centered" style="display:flex; flex-direction:column; justify-content:space-between; height:100vh">
        <GameCanvas view=view set_event=set_event is_touch=touch_device />
        <div style="margin:0 auto;">
            <h1 class="is-size-3 px-4 py-3 has-background-primary" style="position:relative; z-index:1; width:fit-content; border-radius:0.3em;">
                <p style="line-height: 100%;">{"Crux of Life"}</p>
                <p class="is-size-6">{"Rust Core, Leptos Shell"}</p>
                {touch_label}
            </h1>
        </div>
        // </header>
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
    }
}

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! { <RootComponent /> }
    });
}
