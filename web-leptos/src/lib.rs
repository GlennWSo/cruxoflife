use core::f64;
use log::info;
use wasm_bindgen::prelude::*;

pub struct DrawScope {
    pub width: f32,
    pub height: f32,
    pub cell_size: f32,
    pub camera_pos: [f32; 2],
}

pub fn draw_canvas(_scope: DrawScope) {
    let document = web_sys::window().unwrap().document().unwrap();
    let Some(canvas) = document.get_element_by_id("canvas") else {
        info!("id canvas not found");
        return;
    };
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    info!("{:#?}", canvas);
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
    info!("{:#?}", context);

    context.begin_path();

    // Draw the outer circle.
    context
        .arc(75.0, 75.0, 50.0, 0.0, f64::consts::PI * 2.0)
        .unwrap();

    // Draw the mouth.
    context.move_to(110.0, 75.0);
    context.arc(75.0, 75.0, 35.0, 0.0, f64::consts::PI).unwrap();

    // Draw the left eye.
    context.move_to(65.0, 65.0);
    context
        .arc(60.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)
        .unwrap();

    // Draw the right eye.
    context.move_to(95.0, 65.0);
    context
        .arc(90.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)
        .unwrap();
    context.set_line_width(5.0);

    context.stroke();
}
