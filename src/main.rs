mod napoleon_client;

use eframe::egui::ViewportBuilder;
use eframe::NativeOptions;
use crate::napoleon_client::NapoleonClientApp;

fn main() {
    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native("Egui App", options, Box::new(|cc| {
        Ok(Box::new(NapoleonClientApp::default()))
    }));
}

