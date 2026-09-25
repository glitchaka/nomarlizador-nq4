#[path = "../icon_design.rs"]
mod design;

use eframe::egui;

pub fn window_icon() -> egui::IconData {
    const SIZE: u32 = 64;

    egui::IconData {
        rgba: design::rgba_icon(SIZE),
        width: SIZE,
        height: SIZE,
    }
}
