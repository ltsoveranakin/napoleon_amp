use eframe::egui::{Id, Modal, TextWrapMode, Ui};
use napoleon_amp_core::data::playlist::Playlist;
use napoleon_amp_core::data::NamedPathLike;
use std::path::PathBuf;
use std::rc::Rc;

pub(crate) struct PlaylistPage {
    pub(crate) current_playlist: Option<Rc<Playlist>>,
    songs_added: Option<Vec<PathBuf>>,
    delete_original_files: bool,
}

impl PlaylistPage {
    pub(crate) fn new() -> Self {
        Self {
            current_playlist: None,
            songs_added: None,
            delete_original_files: false,
        }
    }

    pub(crate) fn draw(&mut self, ui: &mut Ui) {
        if let Some(current_playlist) = &self.current_playlist {
            ui.heading(current_playlist.name());
            if ui.button("Add Songs").clicked() {
                if let Some(paths) = rfd::FileDialog::new().pick_files() {
                    self.songs_added = Some(paths);
                }
            }

            let current_playlist_rc = Rc::clone(current_playlist);

            self.draw_modal(ui, &current_playlist_rc);

            self.draw_songs(ui, &current_playlist_rc);
        }
    }

    fn draw_modal(&mut self, ui: &mut Ui, current_playlist: &Playlist) {
        if let Some(songs_added) = &self.songs_added {
            let modal = Modal::new(Id::new("Import Songs Modal")).show(ui.ctx(), |ui| {
                ui.set_width(250.);

                let songs_text = if songs_added.len() == 1 {
                    "song"
                } else {
                    "songs"
                };

                ui.heading(format!(
                    "Importing {} new {}",
                    songs_added.len(),
                    songs_text
                ));

                ui.checkbox(&mut self.delete_original_files, "Delete Original Files");

                let r = ui.horizontal(|ui| {
                    if ui.button("Import").clicked() {
                        current_playlist.import_songs(songs_added, self.delete_original_files);
                        return true;
                    }

                    if ui.button("Cancel").clicked() {
                        return true;
                    }

                    return false;
                });

                return r.inner;
            });

            if modal.inner || modal.should_close() {
                self.songs_added = None;
            }
        }
    }

    fn draw_songs(&self, ui: &mut Ui, current_playlist: &Playlist) {
        for song in current_playlist.get_or_load_songs().iter() {
            // ui.horizontal(|ui| {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
            if ui.label(song.name()).clicked() {
                current_playlist.set_playing_song(song);
            }
            // });
        }
    }
}
