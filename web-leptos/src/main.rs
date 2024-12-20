mod core;

use std::fmt::Debug;
use std::fmt::Display;
use std::time::Duration;

use cgmath::num_traits::Float;
use cgmath::InnerSpace;
use js_sys::ArrayBuffer;
use js_sys::DataView;
use leptos::attr::default;
use leptos::attr::Width;
use leptos::prelude::document;
use leptos::prelude::*;
use leptos::tachys::dom::window;
use leptos::task::spawn_local;
use leptos_use::core::IntoElementMaybeSignal;
use leptos_use::storage::use_local_storage;
use leptos_use::storage::use_local_storage_with_options;
use leptos_use::storage::UseStorageOptions;
use leptos_use::use_element_size;
use leptos_use::use_throttle_fn;
use leptos_use::use_throttle_fn_with_arg;
use leptos_use::use_timeout_fn;
use leptos_use::use_window;
use leptos_use::UseElementSizeReturn;
use leptos_use::UseTimeoutFnReturn;
use log::trace;
use shared::ExportOperation;
use shared::Vec2;
use wasm_bindgen::convert::IntoWasmAbi;
use web_sys::Blob;
use web_sys::MouseEvent;
use web_sys::Url;
use web_sys::{File, PointerEvent};

use codee::string::{FromToStringCodec, JsonSerdeCodec};

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

type DragStart = Option<Vec2>;

const LOREM_IPSUM: &'static str = r#"Lorem Ipsum is simply dummy text of the printing
 and typesetting industry. Lorem Ipsum has been the industry's standard dummy text
ever since the 1500s, when an unknown printer took a galley of type and scrambled it to
make a type specimen book. It has survived not only five centuries, but also the leap
into electronic typesetting, remaining essentially unchanged. It was popularised in
the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more
recently with desktop publishing software like Aldus PageMaker including versions of
Lorem Ipsum."#;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
enum NoticeKind {
    #[default]
    Hidden,
    Success,
    Error,
}

impl Display for NoticeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            NoticeKind::Hidden => "is-hidden",
            NoticeKind::Success => "is-success",
            NoticeKind::Error => "is-danger",
        };
        write!(f, "{txt}")
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
struct Notice {
    msg: String,
    kind: NoticeKind,
}

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

fn alert_msg(msg: &str) {
    let doc = document();
    // doc.window
    // let navi = use_window().navigator();
    let window = window();
    if let Err(err) = window.alert_with_message(msg) {
        error!("alert failed with: {err:#?}");
    };
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

    let (drag_start, set_drag_start) = signal(false);
    // let drag_start = use_throttle_fn( 10)
    let (drag_end, set_drag_end) = signal([0_f32, 0.0]);

    // let drag_vector = move || {
    //     let Some(start) = drag_start.get() else {
    //         return None;
    //     };
    //     let start: Vec2 = start.into();
    //     let end: Vec2 = drag_end.get().into();
    //     let vector = end - start;
    //     Some(vector)
    // };

    // let drag_dist = move || {
    //     let Some(start) = drag_start.get() else {
    //         return None;
    //     };
    //     let start = Vec2::new(start[0] as f32, start[1] as f32);
    //     let end = drag_end.get();
    //     let end = Vec2::new(end[0] as f32, end[1] as f32);
    //     Some((end - start).magnitude())
    // };
    let (zoom_pow, set_zoom_pow) = signal(1_f32);
    let (zoom, set_zoom) = signal(1_f32);
    // set_event.set(Event::CameraPanZoom([camera_pan[0], camera_pan[1], 1.0]));
    let _send_camera_events = Effect::new(move |_old_v: Option<f32>| {
        // let old_v = old_v.unwrap_or(1.0);
        let zoom = zoom.get();
        if drag_start.get() {
            // let old_cam = view.get().camera_pan;
            // let new_pos = [old_cam[0] - drag.x, old_cam[1] - drag.y];
            let drag = drag_end.get();

            let cam_update = [drag[0], drag[1], zoom];
            // info!("cam update {cam_update:#?}");
            set_event.set(Event::ChangePanZoom(cam_update));
            zoom
        } else {
            if zoom != 1.0 {
                set_event.set(Event::ChangeZoom(zoom));
            }
            zoom
        }
    });

    let wheel_handler = use_throttle_fn_with_arg(
        move |dy: f64| {
            let dy = dy as f32;
            let zoom = 2.0.powf(1.0 + dy / 600.0) / 2.0;
            set_zoom.set(zoom);
        },
        20.0,
    );

    let _draw = Effect::new(move |_| {
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

    let (short_press, set_short_press) = signal(false);

    let click_handler = move |location: [f32; 2]| {
        info!("location: {:?}", location);
        if short_press.get() {
            set_event.set(Event::ToggleScreenCoord(location))
        }
    };

    let handle_pointerup = move |ev: PointerEvent| {
        if !is_touch {
            click_handler([ev.offset_x() as f32, ev.offset_y() as f32]);
            set_drag_start.set(false);
        }
    };
    let handle_pointerdown = move |ev: PointerEvent| {
        if !is_touch {
            set_short_press.set(true);
            set_timeout(
                move || set_short_press.set(false),
                Duration::from_millis(300),
            );
            let pos = [ev.offset_x() as f32, ev.offset_y() as f32];
            set_drag_end.set(pos);
            set_event.set(Event::AnchorDrag(pos));
            set_drag_start.set(true);
        }
    };
    let handle_pointermove = move |ev: PointerEvent| {
        if !is_touch {
            set_zoom.set(1.0);
            set_drag_end.set([ev.offset_x() as f32, ev.offset_y() as f32]);
        }
    };

    let handle_touchdown = move |ev: TouchEvent| {
        set_short_press.set(true);
        set_timeout(
            move || set_short_press.set(false),
            Duration::from_millis(300),
        );
        if let Some(avg) = avg_touch_pos(&ev) {
            let pos = [avg.x, avg.y];
            set_event.set(Event::AnchorDrag(pos));
            set_drag_start.set(true);
            set_drag_end.set(pos);
        }
    };
    let (touch_spread, set_spread) = signal(Option::<f32>::default());

    let pinch2zoom = move |ev: &TouchEvent| -> f32 {
        let new_spread = avg_touch_spread(ev);
        let Some(spread) = new_spread else {
            set_spread.set(None);
            return 1.0;
        };
        let Some(old_spread) = touch_spread.get() else {
            set_spread.set(Some(spread));
            return 1.0;
        };
        set_spread.set(Some(spread));
        let zoom_change = spread / old_spread;
        zoom_change
    };

    let handle_touchmove = move |ev: TouchEvent| {
        if let Some(avg) = avg_touch_pos(&ev) {
            ev.prevent_default();
            set_drag_end.set([avg.x, avg.y]);
            set_zoom.set(pinch2zoom(&ev));
        }
    };
    let handle_touchup = move |_ev: TouchEvent| {
        click_handler(drag_end.get());
        set_drag_start.set(false);
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
    let view = Memo::new(move |_| view.get());

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

    let export_node = NodeRef::<html::A>::new();
    let (show_menu, set_show_menu) = signal(false);
    let (notice, set_notice) = signal(<Notice>::default());

    let close_notice = move || set_notice.update(|n| n.kind = NoticeKind::Hidden);

    let UseTimeoutFnReturn {
        start: start_notice_timer,
        ..
    } = use_timeout_fn(move |()| close_notice(), 5000.0);

    let _event_processor = Effect::new(move || {
        let event = event.get();
        trace!("got event: {:#?}", event);
        let effects = core.process_event(event);
        for effect in effects {
            match effect {
                shared::Effect::Alert(_) => todo!(),
                shared::Effect::FileIO(req) => {
                    let op: ExportOperation = req.operation;
                    match op {
                        ExportOperation::Copy(data) => {
                            let clipboard = web_sys::window().unwrap().navigator().clipboard();

                            if clipboard.is_undefined() {
                                let kind = NoticeKind::Error;
                                set_show_menu.set(false);
                                set_notice.set(Notice {
                                    msg: "Access to clipboard was denied".to_string(),
                                    kind,
                                });
                                start_notice_timer(());
                                return;
                            };
                            if let Ok(txt) = std::str::from_utf8(&data) {
                                let _promise = clipboard.write_text(txt);
                                let msg = format!(
                                    "Copied: {} ...",
                                    txt.chars().take(30).collect::<String>()
                                );
                                let kind = NoticeKind::Success;
                                set_show_menu.set(false);
                                set_notice.set(Notice { msg, kind });
                                start_notice_timer(());
                            } else {
                                let msg = format!("failed to get world data as txt");
                                let kind = NoticeKind::Success;
                                set_show_menu.set(false);
                                set_notice.set(Notice { msg, kind });
                                log::error!("failed to parsing world data");
                            }
                        }
                        ExportOperation::Save(data) => {
                            let link = export_node
                                .get()
                                .expect("The Anchor must exist to preform the file save");
                            let blob = gloo_file::Blob::new(data.as_slice());
                            let url = gloo_file::ObjectUrl::from(blob);
                            link.set_attribute("href", &url).unwrap();
                            link.set_attribute("download", "exported_life.json")
                                .unwrap();
                            // link.query_selector
                            let click_event: web_sys::Event =
                                MouseEvent::new("click").unwrap().into();
                            link.dispatch_event(&click_event).unwrap();
                        }
                    };
                }
                shared::Effect::Render(_) => set_view.set(core.view()),
            }
        }
    });

    let touch_label = if touch_device {
        Some(view! {<p class="is-size-7"> {"Touch Device detected"}</p> })
    } else {
        None
    };
    // let (show_info, set_show_info) = signal(true);
    // let info_active = move || set_show_info.get().then_some("is-active");
    let (show_info, set_show_info, _) = use_local_storage_with_options::<bool, JsonSerdeCodec>(
        "show_info",
        UseStorageOptions::default().initial_value(true),
    );

    let background = r#"
    Crux of life is an implementation of John Conway's "Game of Life"
    John (December 26, 1937 – April 11, 2020) was a mathematician and professor who hates his creation because
    the game of life overshadowed his research.

    The game of life is an abstract simulation of life. So what is life about?
    That's what you make it into, the interesting thing is that even though the simulation has extremely simple rules, the future is often unpredictable.

    Here is an interview with the mathematician https://www.youtube.com/watch?v=E8kUJL04ELA&list=PLt5AfwLFPxWIL8XA1npoNAHseS-j1y-7V&index=1
    "#;
    let background_info = view! {
        <section class="is-size-5 p-4 has-background-dark has-text-light" style="border-radius:0.3em;">
            <h3 class="is-size-3">"What is this"</h3>
            <p>r#"
            Crux of life is an implementation of John Conway's "Game of Life"
            John (December 26, 1937 – April 11, 2020) was a mathematician and professor who hates his creation because
            the game of life overshadowed his research.
            "#
            </p>
            <a class="" style=":hover{color:white;}"
                href=r#"https://www.youtube.com/watch?v=E8kUJL04ELA&list=PLt5AfwLFPxWIL8XA1npoNAHseS-j1y-7V&index=1"#
                target="_blank"
            >
                Here is an interview with the mathematician
            </a>
            <h3 class="mt-3 is-size-3">"The Game of Life"</h3>
            <p class="is-size-5 ">r#"
            The game of life is an abstract simulation of life. So what is life about?
            It is what up to you, this is sandbox game. Edit the cells and run the simulations as you like.

            "#
            </p>
            <ul class="section" style="list-style:inherit;">
                <b>The rules of the simulation</b>
                <li>"Cells that have 2 or 3 neighbors survive"</li>
                <li>"Empty regions spawn new cells if they have 3 neighbors"</li>
            </ul>
            <p>
            While the rules are simple, how a given pattern will evolve is very hard to predict.
            Now close this window draw some patterns and experiment!
            </p>
            <div class="mt-4 container is-justify-content-center is-flex">
            <button class="button is-primary"
            on:click=move |_ev| set_show_info.set(false)>
                "Let's play"
            </button>
            </div>

        </section>
    };

    let info_modal = view! {
            <div class="modal" class:is-active=show_info on:keydown=move |ev|{
                info!("keydown");
            }>
              <div class="modal-background"></div>
              <div class="modal-content">
                <h2 class="is-size-3 mb-4 px-4 py-3 has-background-primary has-text-centered" style="position:relative; border-radius:0.3em;">
                    <p style="line-height: 100%;">{"crux of life"}</p>
                    <p class="is-size-6">{"rust core, leptos shell"}</p>
                    {touch_label}
                </h2>
                {background_info}

              </div>
              <button class="modal-close is-large" aria-label="close"
                on:click=move |ev| set_show_info.set(false)
              />
              <div>
                  links
              </div>
            </div>

    };

    let input_element: NodeRef<html::Input> = NodeRef::new();

    let load_world_from_js_file = Closure::new(move |js: JsValue| {
        if let Ok(buff) = js.dyn_into::<ArrayBuffer>() {
            let byte_length = buff.byte_length() as usize;
            let data = DataView::new(&buff, 0, byte_length);
            let data: Vec<_> = (0..byte_length).map(|i| data.get_uint8(i)).collect();
            #[cfg(debug_assertions)]
            {
                let txt = String::from_utf8(data.clone());
                info!("info {txt:#?}");
            }
            set_event.set(Event::LoadWorld(data));
        }
    });

    let menu = view! {<>
        <div class="buttons m-4"  style="position:absolute; z-index:3;" >
            <img alt="info" width="64px" src="/assets/menu-icon.svg" hidden=move||{show_menu.get()}
                style="border:none; background:none; position:relative; z-index:1;"
                on:click=move |_| set_show_menu.set(true) />
        </div>

        <aside class="menu m-4 p-4 has-background-primary" class:is-hidden=move||{!show_menu.get()}
            style="position:absolute; z-index:3; border-radius:0.6em;">
          <p class="menu-label">Genereal</p>
          <ul class="menu-list">
            <li><a>
            <label for="importworld">Import World </label>
            <input node_ref=input_element class="input is-hidden" id="importworld" type="file"
                on:change=move |ev|{
                    set_show_menu.set(false);
                    let files = ev.target()
                        .expect("event target should exist")
                        .unchecked_ref::<web_sys::HtmlInputElement>()
                        .files()
                        .expect("target element must be input of type 'file'");

                    if let Some(file) = files.get(0) {
                        info!("file uploaded: {file:?}");
                        let buff = file.array_buffer();
                        let _promise = buff.then(&load_world_from_js_file);
                    } else{
                        log::error!("Expected a file in the input element");
                    }

                }
                />
            </a>

            </li>
            <li on:click=move |_|{
                set_event.set(Event::SaveWorld);
            }>
                <a>Export World</a>
            </li>

            <li on:click=move |_|{
                set_event.set(Event::CopyWorld);
            }>
                <a>Copy World to clipboard</a>
            </li>
            <li on:click=move |_|{
                set_show_info.set(true);
                set_show_menu.set(false);
            }><a>About</a></li>
          </ul>
        </aside>
        </>
    };

    let notice_class = move || {
        let kind = notice.get().kind.to_string();
        format!("notification {kind}")
    };

    view! { <main>
    <div class="p-4" style="position: absolute; width:100%; z-index=10;">
        <a class="is-hidden" node_ref=export_node>Export link</a>
        {info_modal}
        <div class=notice_class style="position:relative; z-index:10; display:flex; justify-content:center;">
            <button class="delete"
                on:click=move |_| close_notice()
            />
            {move || notice.get().msg}
        </div>
        {menu}
    </div>
    <section class="section pt-5 has-text-centered" style="display:flex; flex-direction:column; justify-content:space-between; height:100vh"
        on:click=move |ev| set_show_menu.set(false)>
    <GameCanvas view=view set_event=set_event is_touch=touch_device />
    <div/> // spacer
        <div class="buttons is-centered mb-5 is-flex" style="position:absolute; bottom: 5dvh; justify-content: center; width:100%;">
            <button class="button is-success" class:is-danger=running
            on:click=move |_| set_run.update(|state| *state = !*state)>
                {move || if running.get() {"Stop"} else {"Run"}}
            </button>
            <button class="button is-warning"
                on:click=move |_| {
                    set_run.update(|state| *state = false);
                    set_event.update(|value| *value = Event::Step);
                }>
                "Step"
            </button>
            <p>""</p>
        </div>

    </section>
    </main>}
}

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! { <RootComponent /> }
    });
}
