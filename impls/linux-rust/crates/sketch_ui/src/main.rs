//! Minimal egui front-end for the constraint runtime.

mod app;
mod viewport;

use app::App;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([960.0, 640.0])
            .with_title("Constraint Sketch (v1)"),
        ..Default::default()
    };
    eframe::run_native(
        "Constraint Sketch (v1)",
        options,
        Box::new(|_cc| Ok(Box::new(App::with_sample()))),
    )
}
