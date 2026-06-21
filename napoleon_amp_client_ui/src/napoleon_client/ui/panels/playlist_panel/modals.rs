use crate::napoleon_client::ui::helpers::{duration_input, scroll_area_styled};

use crate::napoleon_client::ui::helpers::custom_modal::custom_modal;
use crate::napoleon_client::ui::panels::CloseResult;
use eframe::egui::{Id, Modal, ScrollArea, Slider, Ui};
use egui_autocomplete::AutoCompleteTextEdit;
use napoleon_amp_core::content::playlist::PlaylistType;
use napoleon_amp_core::content::song::Song;
use napoleon_amp_core::content::song::song_data::meta::SongDataMetaV2;
use napoleon_amp_core::content::song::song_data::{SongData, SongDataStd};
use std::mem;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

pub(super) enum PlaylistModals {
    SongsImported {
        paths: Vec<PathBuf>,
        song_already_exists_indexes: Option<Vec<usize>>,
    },
    EditSong {
        song: Arc<Song>,
        editing_song_data: SongData,
        artist_list: Vec<String>,
        album_list: Vec<String>,
    },
    None,
}

impl PlaylistModals {
    pub(super) fn render(
        &mut self,
        ui: &mut Ui,
        current_playlist: &PlaylistType,
        delete_original_files: &mut bool,
    ) {
        let mut clear_modals = false;
        let mut save_song_data = false;

        match self {
            PlaylistModals::SongsImported {
                paths,
                song_already_exists_indexes,
            } => {
                clear_modals =
                    if let Some(song_already_exists_indexes) = song_already_exists_indexes {
                        Self::draw_modal_failed_import(ui, song_already_exists_indexes, paths)
                    } else {
                        Self::draw_main_import_modal(
                            ui,
                            paths,
                            song_already_exists_indexes,
                            current_playlist,
                            delete_original_files,
                        )
                    }
            }

            PlaylistModals::EditSong {
                editing_song_data,
                artist_list,
                album_list,
                ..
            } => {
                let close_result = Self::draw_edit_song_data_modal(
                    ui,
                    &mut editing_song_data.inner,
                    artist_list,
                    album_list,
                );

                clear_modals = close_result.should_close();
                save_song_data = close_result.should_save();
            }

            PlaylistModals::None => {}
        };

        if save_song_data {
            let edit_song_playlist_modal = mem::replace(self, PlaylistModals::None);

            match edit_song_playlist_modal {
                PlaylistModals::EditSong {
                    song,
                    editing_song_data,
                    ..
                } => {
                    song.set_song_data_and_save(editing_song_data);
                }

                _ => {
                    unreachable!("Only edit song will set save_song_data to true");
                }
            }
        } else if clear_modals {
            *self = PlaylistModals::None;
        }
    }

    fn draw_modal_failed_import(
        ui: &mut Ui,
        song_already_exists_indexes: &[usize],
        song_imported_paths: &[PathBuf],
    ) -> bool {
        let modal = Modal::new(Id::new("Failed Import Songs Modal")).show(ui.ctx(), |ui| {
            let failed_count = song_already_exists_indexes.len();

            ui.heading(format!(
                "The following {} {} already exist, as such the files were not overwritten, nor deleted",
                failed_count,
                Self::songs_plural(failed_count)
            ));

            scroll_area_styled(ui, ScrollArea::vertical().max_height(250.0), |ui| {
                for failed_song_index in song_already_exists_indexes {
                    let failed_song_path = &song_imported_paths[*failed_song_index];
                    ui.label(failed_song_path.to_str().expect("Valid utf8 path"));
                }
            });

            if ui.button("Ok").clicked() {
                true
            } else {
                false
            }
        });

        modal.inner || modal.should_close()
    }

    fn draw_main_import_modal(
        ui: &mut Ui,
        songs_imported_paths: &Vec<PathBuf>,
        song_already_exists_indexes_vec: &mut Option<Vec<usize>>,
        current_playlist: &PlaylistType,
        delete_original_files: &mut bool,
    ) -> bool {
        let modal = Modal::new(Id::new("Import Songs Modal")).show(ui.ctx(), |ui| {
            ui.set_width(250.);

            let count = songs_imported_paths.len();

            ui.heading(format!(
                "Importing {} new {}",
                count,
                Self::songs_plural(count)
            ));

            ui.checkbox(delete_original_files, "Delete original files");

            ui.horizontal(|ui| {
                if ui.button("Import").clicked() {
                    return if let Err(song_already_exists_indexes) =
                        current_playlist.import_songs(songs_imported_paths, *delete_original_files)
                    {
                        song_already_exists_indexes_vec.replace(song_already_exists_indexes);

                        false
                    } else {
                        true
                    };
                }

                ui.button("Cancel").clicked()
            })
            .inner
        });

        modal.inner || modal.should_close()
    }

    fn draw_edit_song_data_modal(
        ui: &mut Ui,
        editing_song_data: &mut SongDataStd,
        artist_list: &[String],
        album_list: &[String],
    ) -> CloseResult {
        custom_modal(ui, "Edit Song", |ui| {
            ui.label("Title:");
            ui.text_edit_singleline(&mut editing_song_data.title);

            ui.label("Artist:");
            ui.add(AutoCompleteTextEdit::new(
                &mut editing_song_data
                    .meta
                    .inner
                    .artist
                    .as_mut()
                    .unwrap()
                    .full_artist_string,
                artist_list,
            ));

            ui.label("Album:");
            ui.add(AutoCompleteTextEdit::new(
                editing_song_data.meta.inner.album.as_mut().unwrap(),
                album_list,
            ));

            ui.label("User Tag:");
            ui.text_edit_singleline(&mut editing_song_data.user_tag);

            ui.separator();

            ui.horizontal(|ui| {
                ui.label(format!(
                    "Times listened: {}",
                    editing_song_data.times_listened
                ));

                if ui.button("Clear").clicked() {
                    editing_song_data.times_listened = 0;
                }
            });

            ui.horizontal(|ui| {
                ui.label(format!(
                    "Times skipped: {}",
                    editing_song_data.times_skipped.inner
                ));

                if ui.button("Clear").clicked() {
                    editing_song_data.times_skipped.inner = 0;
                }
            });

            ui.separator();

            Self::time_ui(
                ui,
                "Start offset",
                &mut editing_song_data.start_offset.inner,
                Duration::ZERO,
            );

            let song_length = *editing_song_data.meta.inner.song_length.as_ref().unwrap() as u64;

            Self::time_ui(
                ui,
                "End time",
                &mut editing_song_data.end_time.inner,
                Duration::from_secs(song_length),
            );

            ui.separator();

            ui.label("Custom Volume:")
                .on_hover_text("The custom volume which will be multiplied by the playlist volume");
            ui.add(
                Slider::new(&mut editing_song_data.custom_volume.inner, 0.0..=4.0).show_value(true),
            );

            ui.separator();

            if ui.button("Clear metadata cache").clicked() {
                editing_song_data.meta.inner = SongDataMetaV2::default().into();
            }

            ui.separator();
        })
        .inner
    }

    fn time_ui(ui: &mut Ui, label: &str, duration: &mut Option<Duration>, default_value: Duration) {
        ui.horizontal(|ui| {
            let mut is_checked = duration.is_some();

            if ui.checkbox(&mut is_checked, label).changed() {
                if is_checked {
                    *duration = Some(default_value);
                } else {
                    *duration = None;
                }
            }

            if is_checked {
                duration_input(
                    ui,
                    Id::new(label),
                    duration
                        .as_mut()
                        .expect("is_checked is only true if duration is Some"),
                );
            }
        });
    }

    fn songs_plural(count: usize) -> &'static str {
        if count == 1 { "song" } else { "songs" }
    }
}
