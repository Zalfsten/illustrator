//! Minimal egui front-end for the constraint runtime.
//!
//! The only platform-specific code lives here: one `main` per host (native
//! window via `run_native`, browser canvas via `WebRunner`), and the choice
//! of host module. `App` and `Viewport` are shared and free of `cfg`.

mod app;
mod export;
#[cfg(test)]
mod fake_files;
#[cfg(test)]
mod fake_typesetter;
mod files;
#[cfg(not(target_arch = "wasm32"))]
mod host_native;
#[cfg(target_arch = "wasm32")]
mod host_web;
mod labels;
mod platform;
mod style_ui;
mod viewport;

use app::App;
use platform::Host;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // The App sets the drawing's title once it starts.
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
        Box::new(|cc| {
            let typesetter = label_layout::Immediate::new(label_typst::TypstTypesetter::new());
            let files = host_native::NativeFiles::new(cc.egui_ctx.clone());
            let host = Host::new(Some(Box::new(typesetter)), Box::new(files));
            Ok(Box::new(App::new(Box::new(host))))
        }),
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
                Box::new(|cc| {
                    // Typst runs in a Web Worker, fetched on the first label line.
                    let ctx = cc.egui_ctx.clone();
                    let typesetter = label_web::WorkerTypesetter::new(
                        "label_worker_boot.js",
                        move || ctx.request_repaint(),
                    );
                    let files = host_web::WebFiles::new();
                    let host = Host::new(Some(Box::new(typesetter)), Box::new(files));
                    Ok(Box::new(App::new(Box::new(host))))
                }),
            )
            .await
            .expect("failed to start eframe in the browser");
    });
}
