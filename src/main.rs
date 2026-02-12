#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{Vec2, ViewportBuilder};

use eframe::{egui, NativeOptions};

use napoleon_amp_client_ui::NapoleonClientApp;

fn main() {
    let options = NativeOptions {
        viewport: ViewportBuilder {
            inner_size: Some(Vec2::new(1280.0, 720.0)),
            title: Some("Napoleon Amp".to_string()),
            icon: Some(std::sync::Arc::new(egui::IconData {
                rgba: image::load_from_memory(include_bytes!("../assets/sprites/NapoleonIcon.png"))
                    .unwrap()
                    .to_rgba8()
                    .to_vec(),
                width: 512,
                height: 512,
                ..Default::default()
            })),
            ..Default::default()
        },
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
