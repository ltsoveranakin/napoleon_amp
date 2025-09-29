use eframe::egui::{CentralPanel, Context};
use eframe::{App, Frame};
use napoleon_amp_core::song::add::add_song_from_path;
use std::path::Path;

pub(super) struct NapoleonClientApp {}

impl Default for NapoleonClientApp {
    fn default() -> Self {
        Self {}
    }
}

impl App for NapoleonClientApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            if ui.button("Load Song").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    let path = path.display().to_string();
                    let r = add_song_from_path(Path::new(&path).to_path_buf());
                    println!("{:?}", r);
                }
            }
        });
    }
}
