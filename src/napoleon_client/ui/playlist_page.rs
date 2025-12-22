use eframe::egui::{Context, CursorIcon, Id, Modal, ScrollArea, Slider, TextWrapMode, Ui};
use std::ops::Deref;

use napoleon_amp_core::data::playlist::manager::{MusicManager, SongStatus};
use napoleon_amp_core::data::playlist::{Playlist, PlaylistVariant};
use napoleon_amp_core::data::NamedPathLike;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

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

    pub(crate) fn render(&mut self, ctx: &Context, ui: &mut Ui, volume: &mut f32) {
        if matches!(self.current_playlist.variant, PlaylistVariant::PlaylistFile) {
            ui.heading(self.current_playlist.name());

            if ui.button("Add Songs").clicked() {
                if let Some(paths) = rfd::FileDialog::new().pick_files() {
                    self.songs_imported = Some(SongsImported {
                        paths,
                        failed_song_indexes: None,
                    });
                }
            }
        } else {
            ui.heading("All Songs");
        }

        let current_playlist_rc = Rc::clone(&self.current_playlist);

        self.render_modal(ui);

        self.render_song_list(ui, &current_playlist_rc, *volume);

        self.render_currently_playing(ctx, ui, volume);
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

    fn render_song_list(&self, ui: &mut Ui, current_playlist: &Playlist, volume: f32) {
        let max_height = if self.current_playlist.get_music_manager().is_some() {
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
                        current_playlist.play_song(song_index, volume);
                    }

                    if song_index != songs.len() - 1 {
                        ui.separator();
                    }
                }
            });
    }

    fn render_currently_playing(&self, ctx: &Context, ui: &mut Ui, volume: &mut f32) {
        let mut should_stop_music = false;

        if let Some(music_manager) = self.current_playlist.get_music_manager().deref() {
            let song_status = music_manager.get_song_status_ref();

            ui.heading(song_status.song().name());

            should_stop_music = self.render_currently_playing_song_controls(
                ctx,
                ui,
                volume,
                music_manager,
                &song_status,
            );
        }

        if should_stop_music {
            self.current_playlist.stop_music();
        }
    }

    fn render_currently_playing_song_controls(
        &self,
        ctx: &Context,
        ui: &mut Ui,
        volume: &mut f32,
        music_manager: &MusicManager,
        song_status: &SongStatus,
    ) -> bool {
        let should_stop = ui
            .horizontal(|ui| {
                ui.label("Vol:");
                if ui.add(Slider::new(volume, 0f32..=1f32)).changed() {
                    music_manager.set_volume(*volume);
                }

                if ui.button("Prev").clicked() {
                    music_manager.previous();
                }

                let toggle_playback_text = if music_manager.is_playing() {
                    "Pause"
                } else {
                    "Play"
                };

                if ui.button(toggle_playback_text).clicked() {
                    music_manager.toggle_playback();
                }

                if ui.button("Next").clicked() {
                    music_manager.next();
                }

                if ui.button("Stop").clicked() {
                    true
                } else {
                    false
                }
            })
            .inner;

        if let Some(total_duration) = song_status.total_duration() {
            let pos = music_manager.get_song_pos();
            let mut progress = pos.as_secs_f32();

            if ui
                .add_sized(
                    ui.available_size(),
                    Slider::new(&mut progress, 0f32..=total_duration.as_secs_f32())
                        .show_value(false),
                )
                .drag_stopped()
            {
                let seek_pos = Duration::from_secs_f32(progress);
                music_manager.try_seek(seek_pos).ok();
            }

            ctx.request_repaint();
        }

        should_stop
    }
}

fn songs_plural(count: usize) -> &'static str {
    if count == 1 { "song" } else { "songs" }
}
