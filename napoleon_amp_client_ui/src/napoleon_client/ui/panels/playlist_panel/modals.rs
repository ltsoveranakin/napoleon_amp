use crate::napoleon_client::ui::helpers::scroll_area_styled;
use crate::napoleon_client::ui::panels::playlist_panel::songs_plural;
use eframe::egui::{Id, Modal, ScrollArea, Ui};
use egui_autocomplete::AutoCompleteTextEdit;
use napoleon_amp_core::content::playlist::Playlist;
use napoleon_amp_core::content::song::song_data::SongData;
use napoleon_amp_core::content::song::Song;
use std::mem;
use std::path::PathBuf;
use std::sync::Arc;

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
        current_playlist: &Playlist,
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
                let (should_close_modal, should_save_song_data) =
                    Self::draw_edit_song_modal(ui, editing_song_data, artist_list, album_list);

                clear_modals = should_close_modal;
                save_song_data = should_save_song_data;
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
                    song.set_song_data(editing_song_data);
                }

                _ => {
                    unreachable!("Only edit song will set save_song_data to true");
                }
            }
        } else if clear_modals {
            *self = PlaylistModals::None;
        } else {
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
                songs_plural(failed_count)
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
        current_playlist: &Playlist,
        delete_original_files: &mut bool,
    ) -> bool {
        let modal = Modal::new(Id::new("Import Songs Modal")).show(ui.ctx(), |ui| {
            ui.set_width(250.);

            let count = songs_imported_paths.len();

            ui.heading(format!("Importing {} new {}", count, songs_plural(count)));

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

    fn draw_edit_song_modal(
        ui: &mut Ui,
        editing_song_data: &mut SongData,
        artist_list: &[String],
        album_list: &[String],
    ) -> (bool, bool) {
        let modal = Modal::new(Id::new("Edit Song")).show(ui.ctx(), |ui| {
            ui.set_width(250.);

            ui.heading("Edit song properties");

            ui.label("Title:");
            ui.text_edit_singleline(&mut editing_song_data.title);

            ui.label("Artist:");
            ui.add(AutoCompleteTextEdit::new(
                &mut editing_song_data.artist.artist_string,
                artist_list,
            ));

            ui.label("Album:");
            ui.add(AutoCompleteTextEdit::new(
                &mut editing_song_data.album,
                album_list,
            ));

            let action = ui
                .horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        return (true, true);
                    }

                    if ui.button("Cancel").clicked() {
                        return (true, false);
                    }

                    (false, false)
                })
                .inner;

            if action.0 {
                return action;
            }

            (false, false)
        });

        (modal.inner.0 || modal.should_close(), modal.inner.1)
    }
}
