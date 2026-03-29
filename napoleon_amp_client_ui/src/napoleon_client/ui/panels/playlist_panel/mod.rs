mod modals;

use crate::napoleon_client::colors::{
    Average, DEFAULT_TEXT_COLOR, SELECTED_TEXT_COLOR, SONG_PLAYING_TEXT_COLOR,
};
use crate::napoleon_client::ui::panels::get_song_data_display_str;
use crate::napoleon_client::ui::panels::playlist_panel::modals::PlaylistModals;
use crate::napoleon_client::ui::panels::queue_panel::QueuePanel;
use eframe::egui::*;
use egui_extras::{Column, TableBuilder, TableRow};
use napoleon_amp_core::content::playlist::manager::{MusicManager, SongStatus};
use napoleon_amp_core::content::playlist::{Playlist, PlaylistVariant};
use napoleon_amp_core::content::song::song_data::MAX_RATING;

use crate::napoleon_client::ui::helpers::scroll_area_styled;
use napoleon_amp_core::instance::NapoleonInstance;
use napoleon_amp_core::paths::show_file_in_explorer;
use napoleon_amp_core::read_rwlock;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

pub(crate) struct PlaylistPanel {
    pub(crate) current_playlist: Rc<Playlist>,
    playlist_modal: PlaylistModals,
    delete_original_files: bool,
    filter_search_content: String,
    pub(crate) queue_panel: QueuePanel,
}

impl PlaylistPanel {
    pub(crate) fn new(current_playlist: Rc<Playlist>) -> Self {
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
        if matches!(self.current_playlist.variant, PlaylistVariant::Normal) {
            ui.heading(
                self.current_playlist
                    .get_or_load_user_data()
                    .content_data
                    .name
                    .clone(),
            );

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
                    self.current_playlist.next_sorting_by();
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

        self.render_modal(ui);

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

    fn render_modal(&mut self, ui: &mut Ui) {
        self.playlist_modal
            .render(ui, &self.current_playlist, &mut self.delete_original_files);
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
                    let current_playing_height = ui.ctx().data(|d| {
                        d.get_temp(current_playing_id).unwrap_or(80.)
                    });

                    let height = ui.available_height() - current_playing_height;
                    height..=height
                } else {
                    0.0..=f32::INFINITY
                };

                ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
                ui.set_height_range(height_range);

                TableBuilder::new(ui).striped(true)
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .column(Column::remainder()).column(Column::remainder())
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
                                let song_data = song.get_song_data();

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
                                        RichText::new(&song_data.title).color(button_text_color);

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
                                                napoleon_instance.try_queue_song(Arc::clone(song)).expect("Checked can queue song above");
                                            }
                                        }

                                        ui.menu_button("Delete", |ui| {
                                            if ui.button("From this playlist").clicked() {
                                                song_index_to_delete = Some(song_index);
                                            }
                                        });

                                        if ui.button("Edit song data").clicked() {
                                            self.playlist_modal = PlaylistModals::EditSong {
                                                song: Arc::clone(song),
                                                editing_song_data: song_data.clone(),
                                                artist_list: current_playlist.get_artist_list(),
                                                album_list: current_playlist.get_album_list(),
                                            };
                                        }

                                        if ui.button("Copy selected").clicked() {
                                            napoleon_instance.copy_selected_songs(current_playlist);
                                        }

                                        ui.menu_button("Open song location", |ui| {
                                            if ui.button("Audio file").clicked() {
                                                show_file_in_explorer(&song.song_audio_path).expect("Error showing file in explorer")
                                            }

                                            if ui.button("Song data").clicked() {
                                                show_file_in_explorer(&song.song_data_path).expect("Error showing file in explorer")
                                            }
                                        });
                                    });
                                });

                                row.col(|ui| {
                                    ui.label(&song_data.artist.full_artist_string);
                                });

                                row.col(|ui| {
                                    ui.label(&song_data.album);
                                });

                                let song_data = Self::col_return(&mut row, |ui| {
                                    let mut update_rating = None;

                                    ui.horizontal(|ui| {
                                        ui.style_mut().spacing.item_spacing.x = 2.;

                                        let mut rating = song_data.rating as i8;

                                        for star_index in 0..MAX_RATING {
                                            let image_source = if rating > 0 {
                                                rating -= 1;
                                                include_image!(
                                                "../../../../../../assets/sprites/star_full1.png"
                                            )
                                            } else {
                                                include_image!(
                                                "../../../../../../assets/sprites/star_empty3.png"
                                            )
                                            };

                                            let star_button = ui.add(Image::new(image_source).sense(Sense::click()).max_size(Vec2::splat(10.)));

                                            if star_button.clicked() {
                                                update_rating = Some(star_index + 1);
                                            }

                                            if star_button.secondary_clicked() {
                                                update_rating = Some(0);
                                            }
                                        }
                                    });

                                    if let Some(updated_rating) = update_rating {
                                        drop(song_data);

                                        song.get_song_data_mut().rating = updated_rating;
                                        song.save_song_data();

                                        song.get_song_data()
                                    } else {
                                        song_data
                                    }
                                });

                                row.col(|ui| {
                                    ui.label(&song_data.user_tag.inner);
                                });

                                row.col(|ui| {
                                    ui.label(Self::secs_to_str(song_data.meta.as_ref().unwrap().length as u64));
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
        current_playing_id: Id,
    ) {
        let mut should_stop_music = false;

        if let Some(music_manager) = self.current_playlist.get_music_manager().deref() {
            let song_status = music_manager.get_song_status();
            let song_data = song_status.song().get_song_data();

            let height = ui
                .scope(|ui| {
                    ui.heading(get_song_data_display_str(&song_data));
                    ui.label(format!(
                        "By: {}",
                        song_data.artist.full_artist_string.replace("/", ", ")
                    ));

                    should_stop_music = self.render_currently_playing_song_controls(
                        ctx,
                        ui,
                        music_manager,
                        &song_status,
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
    ) -> bool {
        let mut volume = (self.current_playlist.get_volume() * 100.) as i32;
        let mut should_stop = false;

        ui.horizontal(|ui| {
            ui.label("Vol:");

            if ui
                .add(Slider::new(&mut volume, 0..=100).trailing_fill(true))
                .drag_stopped()
            {
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
                should_stop = true
            }

            if let Some(total_duration) = song_status.total_duration() {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
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

            if ui
                .add(
                    Slider::new(&mut progress_f32, 0f32..=total_duration.as_secs_f32())
                        .show_value(false)
                        .trailing_fill(true),
                )
                .drag_stopped()
            {
                let seek_pos = Duration::from_secs_f32(progress_f32);
                music_manager.try_seek(seek_pos).expect("Failed to seek");
            }

            ctx.request_repaint();
        }

        should_stop
    }

    fn duration_to_str(duration: Duration) -> String {
        Self::secs_to_str(duration.as_secs())
    }

    fn secs_to_str(secs: u64) -> String {
        let seconds = secs % 60;
        let minutes = secs / 60;

        format!("{}:{:02}", minutes, seconds)
    }

    fn col_return<R>(row: &mut TableRow, add_content: impl FnOnce(&mut Ui) -> R) -> R {
        let mut value = None;

        row.col(|ui| {
            value = Some(add_content(ui));
        });

        value.unwrap()
    }
}
