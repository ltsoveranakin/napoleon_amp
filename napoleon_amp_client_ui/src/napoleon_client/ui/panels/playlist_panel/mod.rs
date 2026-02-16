use eframe::egui::*;
use std::ops::Deref;

use crate::napoleon_client::ui::helpers::scroll_area_styled;

use crate::napoleon_client::colors::{
    Average, DEFAULT_TEXT_COLOR, SELECTED_TEXT_COLOR, SONG_PLAYING_TEXT_COLOR,
};
use crate::napoleon_client::ui::panels::queue_panel::QueuePanel;
use egui_extras::{Column, TableBuilder};
use napoleon_amp_core::content::playlist::manager::{MusicManager, SongStatus};
use napoleon_amp_core::content::playlist::{Playlist, PlaylistVariant};
use napoleon_amp_core::content::song::song_data::SongData;
use napoleon_amp_core::content::song::Song;
use napoleon_amp_core::content::NamedPathLike;
use napoleon_amp_core::instance::NapoleonInstance;
use napoleon_amp_core::read_rwlock;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

enum PlaylistModals {
    SongsImported {
        paths: Vec<PathBuf>,
        song_already_exists_indexes: Option<Vec<usize>>,
    },
    EditSong {
        song: Arc<Song>,
        editing_song_data: SongData,
    },
}

pub(crate) struct PlaylistPanel {
    pub(crate) current_playlist: Rc<Playlist>,
    playlist_modal: Option<PlaylistModals>,
    delete_original_files: bool,
    filter_search_content: String,
    pub(crate) queue_panel: QueuePanel,
}

impl PlaylistPanel {
    pub(crate) fn new(current_playlist: Rc<Playlist>) -> Self {
        Self {
            current_playlist,
            playlist_modal: None,
            delete_original_files: false,
            filter_search_content: String::new(),
            queue_panel: QueuePanel::new(),
        }
    }

    pub(crate) fn render(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        napoleon_instance: &mut NapoleonInstance,
    ) {
        if matches!(self.current_playlist.variant, PlaylistVariant::PlaylistFile) {
            ui.heading(self.current_playlist.get_path_named_ref().name());

            ui.horizontal(|ui| {
                #[cfg(not(target_os = "android"))]
                if ui.button("Add Songs").clicked() {
                    if let Some(paths) = rfd::FileDialog::new().pick_files() {
                        self.playlist_modal = Some(PlaylistModals::SongsImported {
                            paths,
                            song_already_exists_indexes: None,
                        });
                    }
                }

                if ui
                    .button(format!(
                        "Playback Mode: {}",
                        self.current_playlist.playback_mode()
                    ))
                    .clicked()
                {
                    self.current_playlist.next_playback_mode();
                }

                let sort_by = self.current_playlist.get_sorting_by();

                if ui
                    .button(format!("Sort: {}", sort_by.sort_by_variant))
                    .clicked()
                {
                    self.current_playlist.next_sorting_by_and_save();
                }
            });
        } else {
            ui.heading("All Songs");
        }

        if ui
            .text_edit_singleline(&mut self.filter_search_content)
            .changed()
        {
            let search_text = &self.filter_search_content;

            self.current_playlist.set_search_query_filter(search_text);
        }

        let mut copy_keystroke_pressed = false;
        let mut paste_keystroke_pressed = false;

        ctx.input(|i| {
            for ev in &i.events {
                match ev {
                    Event::Copy => {
                        copy_keystroke_pressed = true;
                    }

                    Event::Paste(_) => {
                        paste_keystroke_pressed = true;
                    }

                    _ => {}
                }
            }
        });

        let select_all_keystroke_pressed =
            ctx.input(|state| state.key_pressed(Key::A) && state.modifiers.command);

        if select_all_keystroke_pressed {
            self.current_playlist.select_all();
        }

        if copy_keystroke_pressed {
            napoleon_instance.copy_selected_songs(&*self.current_playlist);
        }

        if paste_keystroke_pressed {
            napoleon_instance.paste_copied_songs(&*self.current_playlist);
        }

        let current_playlist_rc = Rc::clone(&self.current_playlist);

        self.render_modal(ui);

        self.render_song_list(ui, &current_playlist_rc, napoleon_instance);

        self.render_currently_playing(ctx, ui, napoleon_instance);
    }

    fn render_modal(&mut self, ui: &mut Ui) {
        let playlist_modals = if let Some(playlist_modals) = &mut self.playlist_modal {
            playlist_modals
        } else {
            return;
        };

        let clear_modals;
        let mut save_song_data = false;

        match playlist_modals {
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
                            &self.current_playlist,
                            &mut self.delete_original_files,
                        )
                    }
            }

            PlaylistModals::EditSong {
                editing_song_data, ..
            } => {
                let (should_close_modal, should_save_song_data) =
                    Self::draw_edit_song_modal(ui, editing_song_data);

                clear_modals = should_close_modal;
                save_song_data = should_save_song_data;
            }
        };

        if save_song_data {
            match self
                .playlist_modal
                .take()
                .expect("Playlist modal to be some, returns earlier if none")
            {
                PlaylistModals::EditSong {
                    song,
                    editing_song_data,
                } => {
                    song.set_song_data(editing_song_data);
                }

                _ => {
                    unreachable!("Only edit song will set save_song_data to true");
                }
            }
        } else if clear_modals {
            self.playlist_modal = None;
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

    fn draw_edit_song_modal(ui: &mut Ui, editing_song_data: &mut SongData) -> (bool, bool) {
        let modal = Modal::new(Id::new("Edit Song")).show(ui.ctx(), |ui| {
            ui.set_width(250.);

            ui.heading("Edit song properties");

            ui.label("Title:");
            ui.text_edit_singleline(&mut editing_song_data.title);

            ui.label("Artist:");
            ui.text_edit_singleline(&mut editing_song_data.artist);

            ui.label("Album:");
            ui.text_edit_singleline(&mut editing_song_data.album);

            if ui.button("Save").clicked() {
                return (true, true);
            }

            if ui.button("Cancel").clicked() {
                return (true, false);
            }

            (false, false)
        });

        (modal.inner.0 || modal.should_close(), modal.inner.1)
    }

    fn render_song_list(
        &mut self,
        ui: &mut Ui,
        current_playlist: &Rc<Playlist>,
        napoleon_instance: &mut NapoleonInstance,
    ) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.scope(|ui| {
                let height_range = if self.current_playlist.get_music_manager().is_some() {
                    let height = ui.available_height() - 80.;
                    height..=height
                } else {
                    0.0..=f32::INFINITY
                };

                ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
                ui.set_height_range(height_range);

                TableBuilder::new(ui)
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("Title");
                        });

                        header.col(|ui| {
                            ui.heading("Artist");
                        });

                        header.col(|ui| {
                            ui.heading("Album");
                        });
                    })
                    .body(|body| {
                        let mut song_index_to_delete = None;

                        {
                            let song_vec = &*current_playlist.get_or_load_songs();
                            let songs = read_rwlock(&song_vec);
                            let selected_songs = current_playlist.get_selected_songs_variant();
                            let current_playing_song = current_playlist.get_current_song_playing();

                            body.rows(20.0, songs.len(), |mut row| {
                                let song_index = row.index();
                                let song = &songs[song_index];
                                let is_selected = selected_songs.is_selected(song_index);

                                row.col(|ui| {
                                    let mut button_text_color = DEFAULT_TEXT_COLOR;

                                    if is_selected {
                                        button_text_color = SELECTED_TEXT_COLOR;
                                    }

                                    if let Some(current_playing_song) = &current_playing_song
                                        && current_playing_song == song
                                    {
                                        if is_selected {
                                            button_text_color
                                                .average_assign(SONG_PLAYING_TEXT_COLOR);
                                        } else {
                                            button_text_color = SONG_PLAYING_TEXT_COLOR;
                                        }
                                    }

                                    let song_button_text =
                                        RichText::new(&song.get_song_data().title)
                                            .color(button_text_color);

                                    let button = Button::new(song_button_text)
                                        .selected(selected_songs.is_selected(song_index))
                                        .frame(true)
                                        .frame_when_inactive(false);

                                    let button_response = ui.add(button);

                                    if button_response.clicked() {
                                        current_playlist.select_single(song_index);
                                    }

                                    if button_response.double_clicked() {
                                        napoleon_instance.start_play_song(
                                            Rc::clone(current_playlist),
                                            song_index,
                                        );
                                    }

                                    Popup::context_menu(&button_response).show(|ui| {
                                        ui.menu_button("Delete", |ui| {
                                            if ui.button("From this playlist").clicked() {
                                                song_index_to_delete = Some(song_index);
                                            }
                                        });

                                        if ui.button("Edit song data").clicked() {
                                            self.playlist_modal = Some(PlaylistModals::EditSong {
                                                song: Arc::clone(song),
                                                editing_song_data: song.get_song_data().clone(),
                                            })
                                        }
                                    });
                                });

                                row.col(|ui| {
                                    ui.label(&song.get_song_data().artist);
                                });

                                row.col(|ui| {
                                    ui.label(&song.get_song_data().album);
                                });
                            });
                        }

                        if let Some(song_index) = song_index_to_delete {
                            current_playlist.delete_song(song_index);
                        }
                    });
            });
        });
    }

    fn render_currently_playing(
        &self,
        ctx: &Context,
        ui: &mut Ui,
        napoleon_instance: &mut NapoleonInstance,
    ) {
        let mut should_stop_music = false;

        if let Some(music_manager) = self.current_playlist.get_music_manager().deref() {
            let song_status = music_manager.get_song_status();
            let song_data = song_status.song().get_song_data();

            ui.heading(&song_data.title);

            should_stop_music =
                self.render_currently_playing_song_controls(ctx, ui, music_manager, &song_status);
        }

        if should_stop_music {
            napoleon_instance.stop_music();
        }
    }

    fn render_currently_playing_song_controls(
        &self,
        ctx: &Context,
        ui: &mut Ui,
        music_manager: &MusicManager,
        song_status: &SongStatus,
    ) -> bool {
        let mut volume = (self.current_playlist.get_volume() * 100.) as i32;
        let should_stop = ui
            .horizontal(|ui| {
                ui.label("Vol:");

                if ui.add(Slider::new(&mut volume, 0..=100)).drag_stopped() {
                    self.current_playlist.set_volume(volume as f32 / 100.);
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

            ui.spacing_mut().slider_width = ui.available_width();

            if ui
                .add(
                    Slider::new(&mut progress, 0f32..=total_duration.as_secs_f32())
                        .show_value(false),
                )
                .drag_stopped()
            {
                let seek_pos = Duration::from_secs_f32(progress);
                music_manager.try_seek(seek_pos).ok();
            }

            if music_manager.is_playing() {
                ctx.request_repaint();
            }
        }

        should_stop
    }
}

fn songs_plural(count: usize) -> &'static str {
    if count == 1 { "song" } else { "songs" }
}
