mod app;
mod controller;
mod model;
mod service;

use app::Nq4App;
use controller::MainController;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Normalizador NQ4",
        options,
        Box::new(|_cc| Ok(Box::new(Nq4App::new(MainController::default())))),
    )
}
