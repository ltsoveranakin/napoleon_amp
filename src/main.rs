mod napoleon_client;

use crate::napoleon_client::NapoleonClientApp;
use eframe::egui::ViewportBuilder;
use eframe::NativeOptions;

fn main() {
    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Egui App",
        options,
        Box::new(|cc| Ok(Box::new(NapoleonClientApp::new()))),
    );
}
