//! Minimal egui front-end for the constraint runtime.
//!
//! The only platform-specific code lives here: one `main` per host (native
//! window via `run_native`, browser canvas via `WebRunner`). `App` and
//! `Viewport` are shared and free of `cfg`.

mod app;
mod viewport;

use app::App;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // Browser tab title comes from index.html.
    const TITLE: &str = "Constraint Sketch (v1)";
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([960.0, 640.0])
            .with_title(TITLE),
        ..Default::default()
    };
    eframe::run_native(
        TITLE,
        options,
        Box::new(|_cc| Ok(Box::new(App::with_sample()))),
    )
}

/// Browser entry: runs the same `App` on `<canvas id="the_canvas_id">`.
#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;

    let canvas = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("the_canvas_id"))
        .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .expect("index.html must contain <canvas id=\"the_canvas_id\">");

    wasm_bindgen_futures::spawn_local(async move {
        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|_cc| Ok(Box::new(App::with_sample()))),
            )
            .await
            .expect("failed to start eframe in the browser");
    });
}
