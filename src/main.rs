#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod napoleon_client;

use crate::napoleon_client::NapoleonClientApp;
use eframe::egui::ViewportBuilder;
use eframe::NativeOptions;

fn main() {
    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([1280.0, 720.0]),

        ..Default::default()
    };

    let _ = eframe::run_native(
        "Egui App",
        options,
        Box::new(|cc| Ok(Box::new(NapoleonClientApp::new()))),
    );
}
