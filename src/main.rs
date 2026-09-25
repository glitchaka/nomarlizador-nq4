#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod controller;
mod icon;
mod model;
mod service;

use app::Nq4App;
use controller::MainController;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1320.0, 860.0])
            .with_min_inner_size([1100.0, 700.0])
            .with_resizable(true)
            .with_decorations(false)
            .with_icon(std::sync::Arc::new(icon::window_icon())),
        ..Default::default()
    };

    eframe::run_native(
        "Normalizador NQ4",
        options,
        Box::new(|_cc| Ok(Box::new(Nq4App::new(MainController::default())))),
    )
}
