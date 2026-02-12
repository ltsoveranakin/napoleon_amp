#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{Vec2, ViewportBuilder};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::panic;
use std::time::SystemTime;

use eframe::{egui, NativeOptions};

use napoleon_amp_client_ui::{log_dir, log_file_time_now, NapoleonClientApp};

fn main() {
    init_crash_logger();

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

fn init_crash_logger() {
    let old_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        old_hook(panic_info);

        let timestamp = SystemTime::now();

        let panicking_payload = panic_info
            .payload_as_str()
            .unwrap_or("Unknown panicking reason");

        let panic_location = panic_info
            .location()
            .map_or("Unknown panic location".to_string(), |location| {
                format!("{}:{}", location.file(), location.line())
            });

        let logging_directory = log_dir();

        if !logging_directory.try_exists().unwrap_or(false) {
            create_dir_all(logging_directory).ok();
        }

        let mut log_file = match File::create_new(log_file_time_now()) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Unable to create log file; {}", e);
                return;
            }
        };

        writeln!(log_file, "Napoleon Amp crashed :(",).ok();
        writeln!(log_file, "Crashed at: {:?}", timestamp).ok();
        writeln!(log_file, "Panic payload: {}", panicking_payload).ok();
        writeln!(log_file, "Panic location: {}", panic_location).ok();
    }));
}
