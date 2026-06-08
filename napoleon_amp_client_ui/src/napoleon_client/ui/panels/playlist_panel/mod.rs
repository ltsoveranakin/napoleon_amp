mod modals;
mod rating;

use crate::napoleon_client::colors::text_color;
use crate::napoleon_client::ui::helpers::scroll_area_styled;
use crate::napoleon_client::ui::helpers::select_button::{select_button, select_button_mut};
use crate::napoleon_client::ui::panels::get_song_data_display_str;
use crate::napoleon_client::ui::panels::playlist_panel::modals::PlaylistModals;
use crate::napoleon_client::ui::panels::playlist_panel::rating::render_rating;
use crate::napoleon_client::ui::panels::queue_panel::QueuePanel;
use derive_enum_all_values::AllValues;
use eframe::egui::*;
use egui_extras::{Column, TableBuilder};
use napoleon_amp_core::content::SaveData;
use napoleon_amp_core::content::playlist::PlaylistType;
use napoleon_amp_core::content::playlist::manager::{MusicManager, SongStatus};
use napoleon_amp_core::content::playlist::song_list::SortByVariant;
use napoleon_amp_core::instance::NapoleonInstance;
use napoleon_amp_core::paths::show_file_in_explorer;
use napoleon_amp_core::read_rwlock;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

pub(crate) struct PlaylistPanel {
    pub(crate) current_playlist: Rc<PlaylistType>,
    playlist_modal: PlaylistModals,
    delete_original_files: bool,
    filter_search_content: String,
    pub(crate) queue_panel: QueuePanel,
}

impl PlaylistPanel {
    pub(crate) fn new(current_playlist: Rc<PlaylistType>) -> Self {
        Self {
            current_playlist,
            playlist_modal: PlaylistModals::None,
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
        self.keystrokes_pressed(napoleon_instance, ctx);

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                let mut user_data_v = self.current_playlist.get_user_data_mut();
                let mut should_save_song_data = false;

                let user_data = &mut user_data_v.inner;

                ui.heading(&user_data.content_data.name);

                ui.horizontal(|ui| {
                    #[cfg(not(target_os = "android"))]
                    if ui.button("Add Songs").clicked() {
                        if let Some(paths) = rfd::FileDialog::new().pick_files() {
                            self.playlist_modal = PlaylistModals::SongsImported {
                                paths,
                                song_already_exists_indexes: None,
                            };
                        }
                    }

                    select_button_mut(ui, "Playback Mode", &mut user_data.playback_mode);

                    let sort_by = user_data.sort_by;

                    ui.menu_button(format!("Sort: {}", sort_by.sort_by_variant), |ui| {
                        for sort_by_variant in SortByVariant::all_values() {
                            if ui.button(sort_by_variant.to_string()).clicked() {
                                should_save_song_data = true;
                                user_data.sort_by.sort_by_variant = *sort_by_variant;

                                self.current_playlist.sort_songs(user_data.sort_by);
                            }
                        }
                    });

                    if ui
                        .checkbox(&mut user_data.sort_by.inverted, "Descending")
                        .changed()
                    {
                        should_save_song_data = true;
                        self.current_playlist.sort_songs(user_data.sort_by);
                    }
                });

                if should_save_song_data {
                    user_data_v
                        .save_data(self.current_playlist.id())
                        .expect("Save playlist user data");
                }
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(format!(
                    "{} songs, {}",
                    read_rwlock(&self.current_playlist.get_song_vec_unfiltered()).len(),
                    Self::secs_to_str(self.current_playlist.get_total_song_duration() as u64)
                ))
            });
        });

        if ui
            .text_edit_singleline(&mut self.filter_search_content)
            .changed()
        {
            let search_text = &self.filter_search_content;

            self.current_playlist.set_search_query_filter(search_text);
        }

        self.playlist_modal
            .render(ui, &self.current_playlist, &mut self.delete_original_files);

        let current_playing_id = ui.make_persistent_id("currently_playing_display");

        self.render_song_list(ui, napoleon_instance, current_playing_id);

        self.render_currently_playing(ctx, ui, napoleon_instance, current_playing_id);
    }

    fn keystrokes_pressed(&self, napoleon_instance: &mut NapoleonInstance, ctx: &Context) {
        if !matches!(self.playlist_modal, PlaylistModals::None) {
            return;
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
    }

    fn render_song_list(
        &mut self,
        ui: &mut Ui,
        napoleon_instance: &mut NapoleonInstance,
        current_playing_id: Id,
    ) {
        let current_playlist = &self.current_playlist;

        scroll_area_styled(ui, ScrollArea::vertical(), |ui| {
            ui.scope(|ui| {
                let height_range = if self.current_playlist.get_music_manager().is_some() {
                    let current_playing_height = ui
                        .ctx()
                        .data(|d| d.get_temp(current_playing_id).unwrap_or(80.));

                    let height = ui.available_height() - current_playing_height;
                    height..=height
                } else {
                    0.0..=f32::INFINITY
                };

                ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
                ui.set_height_range(height_range);

                TableBuilder::new(ui)
                    .striped(true)
                    // Title
                    .column(Column::remainder())
                    // Artist
                    .column(Column::remainder())
                    // Album
                    .column(Column::remainder())
                    // Rating
                    .column(Column::remainder())
                    // User Tag
                    .column(Column::remainder())
                    // Length
                    .column(Column::remainder())
                    // Times Listened
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

                        header.col(|ui| {
                            ui.heading("Rating");
                        });

                        header.col(|ui| {
                            ui.heading("User Tag");
                        });

                        header.col(|ui| {
                            ui.heading("Length");
                        });

                        header.col(|ui| {
                            ui.heading("Times Listened");
                        });
                    })
                    .body(|body| {
                        let mut song_index_to_delete = None;

                        {
                            let song_vec = &*current_playlist.get_song_vec();
                            let songs = read_rwlock(&song_vec);
                            let selected_songs = current_playlist.get_selected_songs();
                            let current_playing_song_opt =
                                current_playlist.get_current_song_playing();

                            body.rows(20.0, songs.len(), |mut row| {
                                let mut updated_rating_opt = None;
                                let song_index = row.index();
                                let song = &songs[song_index];
                                let is_selected = selected_songs.is_selected(song_index);
                                let song_data_vers = song.get_song_data();

                                row.col(|ui| {
                                    let button_text_color = text_color(
                                        is_selected,
                                        current_playing_song_opt.as_ref().is_some_and(
                                            |current_playing_song| current_playing_song == song,
                                        ),
                                    );

                                    let song_button_text =
                                        RichText::new(&song_data_vers.inner.title)
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
                                        if napoleon_instance.can_queue_song() {
                                            if ui.button("Queue Next").clicked() {
                                                napoleon_instance
                                                    .try_queue_song(Arc::clone(song))
                                                    .expect("Checked can queue song above");
                                            }
                                        }

                                        ui.menu_button("Delete", |ui| {
                                            if ui.button("From this playlist").clicked() {
                                                song_index_to_delete = Some(song_index);
                                            }
                                        });

                                        if ui.button("Edit song data").clicked() {
                                            let editing_song_data = song_data_vers.clone();

                                            self.playlist_modal = PlaylistModals::EditSong {
                                                song: Arc::clone(song),
                                                editing_song_data,
                                                artist_list: current_playlist.get_artist_list(),
                                                album_list: current_playlist.get_album_list(),
                                            };
                                        }

                                        if ui.button("Copy selected").clicked() {
                                            napoleon_instance.copy_selected_songs(current_playlist);
                                        }

                                        ui.menu_button("Open song location", |ui| {
                                            if ui.button("Audio file").clicked() {
                                                show_file_in_explorer(&song.song_audio_path)
                                                    .expect("Error showing file in explorer")
                                            }

                                            if ui.button("Song data").clicked() {
                                                show_file_in_explorer(&song.song_data_path)
                                                    .expect("Error showing file in explorer")
                                            }
                                        });
                                    });
                                });

                                row.col(|ui| {
                                    ui.label(
                                        &song_data_vers.inner.meta().artist.full_artist_string,
                                    );
                                });

                                row.col(|ui| {
                                    ui.label(&song_data_vers.inner.meta().album);
                                });

                                row.col(|ui| {
                                    updated_rating_opt =
                                        render_rating(ui, song_data_vers.inner.rating).inner;

                                    if let Some(updated_rating) = updated_rating_opt {
                                        let mut song_data_cloned = song_data_vers.clone();
                                        song_data_cloned.inner.rating = updated_rating;

                                        song.save_song_data_already_borrowed(&song_data_cloned);
                                    }
                                });

                                row.col(|ui| {
                                    ui.label(&song_data_vers.inner.user_tag);
                                });

                                row.col(|ui| {
                                    ui.label(Self::secs_to_str(
                                        song_data_vers.inner.meta().song_length as u64,
                                    ));
                                });

                                row.col(|ui| {
                                    ui.label(song_data_vers.inner.times_listened.to_string());
                                });

                                if let Some(updated_rating) = updated_rating_opt {
                                    drop(song_data_vers);
                                    song.get_song_data_mut().inner.rating = updated_rating;
                                }
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
        current_playing_id: Id,
    ) {
        let mut should_stop_music = false;

        if let Some(music_manager) = self.current_playlist.get_music_manager().deref() {
            let song_status = music_manager.get_song_status();
            let song_data_vers = song_status.song().get_song_data_mut();

            let height = ui
                .scope(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(get_song_data_display_str(&song_data_vers.inner));

                        // let rating_id = Id::new("Taskbar-Rating");
                        // let size = ui.ctx().data(|d| d.get_temp(rating_id)).unwrap_or_default();
                        //
                        //
                        // ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        //     ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
                        //         let rating_resp = render_rating(ui, song_data_vers.inner.rating);
                        //         let rating_width = rating_resp.response.rect.width();
                        //         //
                        //         ui.ctx().data_mut(|d| d.insert_temp(rating_id, rating_width));
                        //
                        //         if let Some(updated_rating) = rating_resp.inner {
                        //             song_data_vers.inner.rating = updated_rating;
                        //             song_status.song().save_song_data_already_borrowed(&song_data_vers);
                        //         }
                        //     });
                        // });
                    });

                    ui.label(format!(
                        "By: {}",
                        song_data_vers
                            .inner
                            .meta()
                            .artist
                            .full_artist_string
                            .replace("/", ", ")
                    ));

                    should_stop_music = self.render_currently_playing_song_controls(
                        ctx,
                        ui,
                        music_manager,
                        &song_status,
                        napoleon_instance,
                    );
                })
                .response
                .rect
                .height();

            ctx.data_mut(|d| d.insert_temp(current_playing_id, height));
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
        napoleon_instance: &mut NapoleonInstance,
    ) -> bool {
        let mut volume = (self.current_playlist.get_volume() * 100.) as i32;
        let mut should_stop = false;

        ui.horizontal(|ui| {
            ui.label("Vol:");

            if ui
                .add(Slider::new(&mut volume, 0..=100).trailing_fill(true))
                .drag_stopped()
            {
                self.current_playlist
                    .set_volume(volume as f32 / 100.)
                    .expect("Unable to set volume");
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
                should_stop = true
            }

            if let Some(total_duration) = song_status.total_duration() {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    select_button(ui, "Loop", &music_manager.loop_mode(), |new_loop_mode| {
                        music_manager.set_loop_mode(*new_loop_mode);
                    });

                    ui.label(format!(
                        "{}/{}",
                        Self::duration_to_str(music_manager.get_song_pos()),
                        Self::duration_to_str(total_duration)
                    ));
                });
            }
        });

        if let Some(total_duration) = song_status.total_duration() {
            let pos = music_manager.get_song_pos();

            let mut progress_f32 = pos.as_secs_f32();

            ui.spacing_mut().slider_width = ui.available_width();

            let slider_response = ui.add(
                Slider::new(&mut progress_f32, 0f32..=total_duration.as_secs_f32())
                    .show_value(false)
                    .trailing_fill(true),
            );

            if slider_response.drag_stopped() {
                let seek_pos = Duration::from_secs_f32(progress_f32);
                music_manager.try_seek(seek_pos).expect("Failed to seek");
            }

            if slider_response.hovered() {
                ctx.request_repaint();
            }
        }

        ctx.request_repaint_after(Duration::from_millis(
            napoleon_instance
                .get_client_settings()
                .inner
                .inactive_render_timeout_ms as u64,
        ));

        should_stop
    }

    fn duration_to_str(duration: Duration) -> String {
        Self::secs_to_str(duration.as_secs())
    }

    fn secs_to_str(secs: u64) -> String {
        let seconds = secs % 60;
        let minutes_total = secs / 60;
        let minutes = minutes_total % 60;
        let hours = minutes_total / 60;

        let hours_minutes_str = if hours != 0 {
            format!("{hours}:{:02}", minutes)
        } else {
            minutes.to_string()
        };

        format!("{hours_minutes_str}:{:02}", seconds)
    }
}
