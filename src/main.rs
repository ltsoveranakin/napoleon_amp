#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::ViewportBuilder;
use eframe::NativeOptions;
use napoleon_amp_client_ui::NapoleonClientApp;

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

// #[cfg(target_os = "android")]
// ndk_glue::main!(main);
