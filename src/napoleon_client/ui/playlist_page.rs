use eframe::egui::{CursorIcon, Id, Modal, ScrollArea, Slider, TextWrapMode, Ui};
use napoleon_amp_core::data::playlist::Playlist;
use napoleon_amp_core::data::NamedPathLike;
use napoleon_amp_core::unwrap_lock;
use std::path::PathBuf;
use std::rc::Rc;

struct SongsImported {
    paths: Vec<PathBuf>,
    failed_song_indexes: Option<Vec<usize>>,
}

pub(crate) struct PlaylistPanel {
    pub(crate) current_playlist: Rc<Playlist>,
    songs_imported: Option<SongsImported>,
    delete_original_files: bool,
}

impl PlaylistPanel {
    pub(crate) fn new(current_playlist: Rc<Playlist>) -> Self {
        Self {
            current_playlist,
            songs_imported: None,
            delete_original_files: false,
        }
    }

    pub(crate) fn render(&mut self, ui: &mut Ui, volume: &mut f32) {
        ui.heading(self.current_playlist.name());
        if ui.button("Add Songs").clicked() {
            if let Some(paths) = rfd::FileDialog::new().pick_files() {
                self.songs_imported = Some(SongsImported {
                    paths,
                    failed_song_indexes: None,
                });
            }
        }

        let current_playlist_rc = Rc::clone(&self.current_playlist);

        self.render_modal(ui);

        self.render_song_list(ui, &current_playlist_rc);

        self.render_currently_playing(ui, volume);
    }

    fn render_modal(&mut self, ui: &mut Ui) {
        if self.songs_imported.is_none() {
            return;
        }

        if self
            .songs_imported
            .as_ref()
            .expect("Checked None above")
            .failed_song_indexes
            .is_some()
        {
            self.draw_modal_failed_import(ui)
        } else {
            self.draw_main_import_modal(ui);
        }
    }

    fn draw_modal_failed_import(&mut self, ui: &mut Ui) {
        let songs_imported = self
            .songs_imported
            .as_ref()
            .expect("Songs imported checked None");
        let modal = Modal::new(Id::new("Failed Import Songs Modal")).show(ui.ctx(), |ui| {
            ui.set_width(250.);

            let failed_song_indexes = songs_imported
                .failed_song_indexes
                .as_ref()
                .expect("Songs failed checked None");

            let failed_count = failed_song_indexes.len();

            ui.heading(format!(
                "Failed to import the following {} {}",
                failed_count,
                songs_plural(failed_count)
            ));

            for failed_song_index in failed_song_indexes {
                let failed_song_path = &songs_imported.paths[*failed_song_index];

                ui.label(failed_song_path.to_str().expect("Valid utf8 path"));
            }

            if ui.button("Ok").clicked() {
                true
            } else {
                false
            }
        });

        if modal.inner || modal.should_close() {
            self.songs_imported = None;
        }
    }

    fn draw_main_import_modal(&mut self, ui: &mut Ui) {
        let songs_imported = self
            .songs_imported
            .as_mut()
            .expect("Songs imported checked None");
        let modal = Modal::new(Id::new("Import Songs Modal")).show(ui.ctx(), |ui| {
            ui.set_width(250.);

            let songs_imported_paths = &songs_imported.paths;

            let count = songs_imported_paths.len();

            ui.heading(format!("Importing {} new {}", count, songs_plural(count)));

            ui.checkbox(&mut self.delete_original_files, "Delete Original Files");

            let r = ui.horizontal(|ui| {
                if ui.button("Import").clicked() {
                    return if let Err(failed_song_indexes) = self
                        .current_playlist
                        .as_ref()
                        .import_songs(songs_imported_paths, self.delete_original_files)
                    {
                        songs_imported.failed_song_indexes = Some(failed_song_indexes);

                        false
                    } else {
                        true
                    };
                }

                if ui.button("Cancel").clicked() {
                    true
                } else {
                    false
                }
            });

            return r.inner;
        });

        if modal.inner || modal.should_close() {
            self.songs_imported = None;
        }
    }

    fn render_song_list(&self, ui: &mut Ui, current_playlist: &Playlist) {
        let max_height = if self.current_playlist.current_song_status().is_some() {
            ui.available_height() - 80.
        } else {
            f32::INFINITY
        };

        ScrollArea::vertical()
            .max_height(max_height)
            .show(ui, |ui| {
                let songs = current_playlist.get_or_load_songs();

                for (song_index, song) in songs.iter().enumerate() {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);

                    if ui
                        .label(song.name())
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .clicked()
                    {
                        current_playlist.play_song(song_index);
                    }

                    if song_index != songs.len() - 1 {
                        ui.separator();
                    }
                }
            });
    }

    fn render_currently_playing(&self, ui: &mut Ui, volume: &mut f32) {
        if let Some(current_song_status) = self.current_playlist.current_song_status() {
            let song_status = unwrap_lock(&*current_song_status);

            ui.heading(song_status.song.name());

            ui.horizontal(|ui| {
                if ui.button("Prev").clicked() {
                    self.current_playlist.previous();
                }

                let toggle_playback_text = if self.current_playlist.is_playing() {
                    "Pause"
                } else {
                    "Play"
                };

                if ui.button(toggle_playback_text).clicked() {
                    self.current_playlist.toggle_playback();
                }

                if ui.button("Next").clicked() {
                    self.current_playlist.next();
                }

                if ui.button("Stop").clicked() {
                    self.current_playlist.stop();
                }

                if ui.add(Slider::new(volume, 0f32..=1f32)).changed() {
                    self.current_playlist.set_volume(*volume);
                }
            });
        }
    }
}

fn songs_plural(count: usize) -> &'static str {
    if count == 1 { "song" } else { "songs" }
}
