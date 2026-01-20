use eframe::egui::{Context, Event, Id, Modal, ScrollArea, Slider, TextWrapMode, Ui};
use std::ops::Deref;

use crate::napoleon_client::ui::helpers::scroll_area_styled;
use crate::napoleon_client::ui::queue_panel::QueuePanel;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use napoleon_amp_core::data::playlist::manager::{MusicManager, SongStatus};
use napoleon_amp_core::data::playlist::{Playlist, PlaylistVariant};
use napoleon_amp_core::data::NamedPathLike;
use napoleon_amp_core::instance::NapoleonInstance;
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
    filter_search_content: String,
    pub(crate) queue_panel: QueuePanel,
}

impl PlaylistPanel {
    pub(crate) fn new(current_playlist: Rc<Playlist>) -> Self {
        Self {
            current_playlist,
            songs_imported: None,
            delete_original_files: false,
            filter_search_content: String::new(),
            queue_panel: QueuePanel::new(),
        }
    }

    pub(crate) fn render(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        volume: &mut i32,
        napoleon_instance: &mut NapoleonInstance,
    ) {
        if matches!(self.current_playlist.variant, PlaylistVariant::PlaylistFile) {
            ui.heading(self.current_playlist.name());

            #[cfg(not(target_os = "android"))]
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

        if ui
            .text_edit_singleline(&mut self.filter_search_content)
            .changed()
        {
            let search_text = &self.filter_search_content;

            let search_query = if search_text.chars().count() > 0 {
                Some(&*self.filter_search_content)
            } else {
                None
            };

            self.current_playlist.set_search_query(search_query);
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
            ctx.input(|state| state.key_pressed(egui::Key::A) && state.modifiers.command);

        // let copy_keystroke_pressed =
        //     ctx.input(|state| state.key_pressed(egui::Key::C) && state.modifiers.command);
        //
        // let paste_keystroke_pressed =
        //     ctx.input(|state| state.key_pressed(egui::Key::V) && state.modifiers.command);

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
            // ui.set_width(250.0);

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

            // println!(failed_song_indexes.len());
            //
            // ScrollArea::vertical().show_rows(
            //     ui,
            //     20.0,
            //     failed_song_indexes.len(),
            //     |ui, row_range| {
            //         for row in row_range {
            //             let failed_song_path = &songs_imported.paths[row];
            //
            //             ui.label(failed_song_path.to_str().expect("Valid utf8 path"));
            //         }
            //     },
            // );

            scroll_area_styled(ui, ScrollArea::vertical().max_height(250.0), |ui| {
                for failed_song_index in failed_song_indexes {
                    let failed_song_path = &songs_imported.paths[*failed_song_index];
                    ui.label(failed_song_path.to_str().expect("Valid utf8 path"));
                }
            });

            // ScrollArea::vertical().show(ui, |ui| {
            //     for failed_song_index in failed_song_indexes {
            //         let failed_song_path = &songs_imported.paths[*failed_song_index];
            //         ui.label(failed_song_path.to_str().expect("Valid utf8 path"));
            //     }
            // });

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

    fn render_song_list(&self, ui: &mut Ui, current_playlist: &Playlist, volume: i32) {
        let max_height = if self.current_playlist.get_music_manager().is_some() {
            ui.available_height() - 80.
        } else {
            f32::INFINITY
        };

        let songs = &*current_playlist.get_or_load_songs();
        let selected_songs = current_playlist.get_selected_songs_variant();

        ui.scope(|ui| {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);

            TableBuilder::new(ui)
                .column(Column::remainder())
                .column(Column::remainder())
                .column(Column::remainder())
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Name");
                    });

                    header.col(|ui| {
                        ui.heading("Artist");
                    });

                    header.col(|ui| {
                        ui.heading("Album");
                    });
                })
                .body(|body| {
                    body.rows(20.0, songs.len(), |mut row| {
                        let row_index = row.index();
                        let song = &songs[row_index];

                        row.col(|ui| {
                            if ui.label(&song.get_or_load_song_data().title).clicked() {
                                current_playlist.start_play_song(row_index, volume as f32 / 100.);
                            }
                        });

                        row.col(|ui| {
                            ui.label(&song.get_or_load_song_data().artist);
                        });

                        row.col(|ui| {
                            ui.label(&song.get_or_load_song_data().album);
                        });
                    });
                });
        });

        // scroll_area_named_list(
        //     ui,
        //     ScrollArea::vertical()
        //         .max_height(max_height)
        //         .id_salt(current_playlist.name()),
        //     songs,
        //     |song_index, _| selected_songs.is_selected(song_index),
        //     |song_index| {
        //         current_playlist.select_single(song_index);
        //     },
        //     |song_index| {
        //         current_playlist.start_play_song(song_index, volume as f32 / 100.);
        //     },
        // );

        // scroll_area_styled(ui, ScrollArea::vertical(), |ui| {
        //     let songs = current_playlist.get_or_load_songs();
        //     let selected_songs = current_playlist.get_selected_songs();
        //
        //     for (song_index, song) in songs.iter().enumerate() {
        //         let current_song_selected_as_single =
        //             if let SelectedSongs::Single(index) = selected_songs {
        //                 song_index == index
        //             } else {
        //                 false
        //             };
        //
        //         if ui
        //             .selectable_label(selected_songs.is_selected(song_index), song.name())
        //             .clicked()
        //         {
        //             if current_song_selected_as_single {
        //                 current_playlist.start_play_song(song_index, volume as f32 / 100.);
        //             }
        //             self.current_playlist.select_single(song_index);
        //         }
        //
        //         if song_index != songs.len() - 1 {
        //             ui.separator();
        //         }
        //     }
        // });
    }

    fn render_currently_playing(&self, ctx: &Context, ui: &mut Ui, volume: &mut i32) {
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
        volume: &mut i32,
        music_manager: &MusicManager,
        song_status: &SongStatus,
    ) -> bool {
        let should_stop = ui
            .horizontal(|ui| {
                ui.label("Vol:");
                if ui.add(Slider::new(volume, 0..=100)).changed() {
                    music_manager.set_volume(*volume as f32 / 100.);
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
                let _ = music_manager.try_seek(seek_pos);
            }

            ctx.request_repaint();
        }

        should_stop
    }
}

fn songs_plural(count: usize) -> &'static str {
    if count == 1 { "song" } else { "songs" }
}
