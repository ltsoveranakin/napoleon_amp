mod modals;

use crate::napoleon_client::ui::helpers::scroll_area_styled;

use crate::napoleon_client::ui::panels::playlist_panel::PlaylistPanel;
use eframe::egui::{Button, IntoAtoms, Popup, Response, RichText, ScrollArea, Sense, Ui, UiBuilder};

use crate::napoleon_client::colors::SONG_PLAYING_TEXT_COLOR;
use crate::napoleon_client::ui::panels::folder_list::modals::FolderListModals;
use crate::napoleon_client::ui::panels::open_location_button;
use napoleon_amp_core::content::folder::content::FolderContentVariant;
use napoleon_amp_core::content::folder::Folder;
use napoleon_amp_core::content::playlist::Playlist;
use napoleon_amp_core::discord_rpc::set_rpc_playlist;
use napoleon_amp_core::instance::NapoleonInstance;
use std::path::Path;
use std::rc::{Rc, Weak};

pub(crate) struct FolderList {
    pub(crate) current_folder: Rc<Folder>,
    current_modal: FolderListModals,
}

impl FolderList {
    pub(crate) fn new(current_folder: Rc<Folder>) -> Self {
        Self {
            current_folder,
            current_modal: FolderListModals::None,
        }
    }

    pub(crate) fn render(
        &mut self,
        ui: &mut Ui,
        playlist_panel: &mut Option<PlaylistPanel>,
        napoleon_instance: &mut NapoleonInstance,
    ) {
        let bg = ui.scope_builder(UiBuilder::new().sense(Sense::click()), |ui| {
            self.current_modal.render(ui);

            self.render_header_buttons(ui);

            self.render_folder_content(ui, playlist_panel, napoleon_instance);
        });

        let (_, extra_space) = ui.allocate_at_least(ui.available_size(), Sense::click());

        self.new_content_only_menu(&bg.response);
        self.new_content_only_menu(&extra_space);
    }

    fn new_content_only_menu(&mut self, response: &Response) {
        Popup::context_menu(&response).show(|ui| {
            let current_folder = Rc::clone(&self.current_folder);

            self.new_content_button(ui, &current_folder);
        });
    }

    fn render_header_buttons(&mut self, ui: &mut Ui) {
        if let Some(parent_folder) = &self.current_folder.parent {
            if ui.button("Back").clicked() {
                let parent = Weak::upgrade(parent_folder).expect("TODO: ");
                self.current_folder = parent;
            }
        }
    }

    fn render_folder_content(
        &mut self,
        ui: &mut Ui,
        playlist_panel: &mut Option<PlaylistPanel>,
        napoleon_instance: &mut NapoleonInstance,
    ) {
        scroll_area_styled(ui, ScrollArea::vertical(), |ui| {
            let mut next_folder = None;
            let mut next_playlist = None;

            if self.current_folder.parent.is_none() {
                if self.playlist_button(ui, "All Songs").clicked() {
                    next_playlist = Some(Rc::new(Playlist::all_songs()))
                }
            }

            let current_folder = Rc::clone(&self.current_folder);

            if let Some((folder, index)) = self.render_sub_folder_content(
                ui,
                &current_folder,
                playlist_panel,
                &mut next_playlist,
                &mut next_folder,
                napoleon_instance,
            ) {
                Folder::delete_content(&folder, index).expect("Failed delete folder content");
            }

            if let Some(next_folder) = next_folder {
                self.current_folder = next_folder;
            }

            if let Some(next_playlist_content) = next_playlist {
                *playlist_panel = Some(PlaylistPanel::new(next_playlist_content));
            }
        });
    }

    fn render_sub_folder_content(
        &mut self,
        ui: &mut Ui,
        current_folder: &Rc<Folder>,
        playlist_panel: &mut Option<PlaylistPanel>,
        next_playlist: &mut Option<Rc<Playlist>>,
        next_folder: &mut Option<Rc<Folder>>,
        napoleon_instance: &mut NapoleonInstance,
    ) -> Option<(Rc<Folder>, usize)> {
        let mut delete_index = None;

        // let current_folder = Rc::clone(&self.current_folder);

        for (current_index, folder_content_variant) in
            Folder::get_contents(&current_folder).iter().enumerate()
        {
            ui.separator();

            match folder_content_variant {
                FolderContentVariant::Playlist(playlist) => {
                    let playlist_name = playlist.get_name();

                    let playlist_button = ui.scope(|ui| {
                        let mut rt = RichText::new(&*playlist_name);

                        if playlist.get_music_manager().is_some() {
                            rt = rt.color(SONG_PLAYING_TEXT_COLOR);
                        }

                        let playlist_button = self.playlist_button(ui, rt);

                        playlist_button
                    }).inner;


                    if playlist_button.clicked() {
                        *next_playlist = Some(Rc::clone(playlist));
                        set_rpc_playlist(playlist_name.to_string());
                    }

                    if playlist_button.double_clicked() {
                        if let Some(playlist) = next_playlist {
                            napoleon_instance.start_play_playlist(Rc::clone(playlist));
                        }
                    }

                    Popup::context_menu(&playlist_button).show(|ui| {
                        if ui.button("Rename Playlist").clicked() {
                            self.current_modal = FolderListModals::RenamePlaylist {
                                name: String::new(),
                                playlist: Rc::clone(playlist),
                            };
                        }

                        if Self::shared_popup_ui(
                            ui,
                            "playlist",
                            playlist.get_or_load_playlist_data().get_data_path(),
                        ) {
                            delete_index = Some((Rc::clone(&self.current_folder), current_index));
                        }
                    });
                }

                FolderContentVariant::Folder(folder) => {
                    ui.horizontal(|ui| {
                        let folder_item =
                            ui.collapsing(&folder.get_folder_data().content_data.name, |ui| {
                                let delete_index_sub = self.render_sub_folder_content(
                                    ui,
                                    folder,
                                    playlist_panel,
                                    next_playlist,
                                    next_folder,
                                    napoleon_instance,
                                );

                                if delete_index_sub.is_some() {
                                    delete_index = delete_index_sub;
                                }
                            });

                        if folder_item.header_response.middle_clicked() {
                            *next_folder = Some(Rc::clone(folder));
                        }

                        Popup::context_menu(&folder_item.header_response).show(|ui| {
                            self.new_content_button(ui, folder);

                            if Self::shared_popup_ui(
                                ui,
                                "folder",
                                folder.get_folder_data().get_folder_data_path(),
                            ) {
                                delete_index = Some((Rc::clone(folder), current_index));
                            }
                        })
                    });
                }
            }
        }

        delete_index
    }

    fn new_content_button(&mut self, ui: &mut Ui, parent_folder: &Rc<Folder>) {
        ui.menu_button("New", |ui| {
            if ui.button("Playlist").clicked() {
                self.current_modal = FolderListModals::create_playlist(Rc::clone(parent_folder))
            }

            if ui.button("Folder").clicked() {
                self.current_modal = FolderListModals::create_folder(Rc::clone(parent_folder))
            }
        });
    }

    /// Shared popup UI between folders and playlists
    ///
    /// Returns true if the content should be deleted
    fn shared_popup_ui(ui: &mut Ui, variant_text: &str, path: impl AsRef<Path>) -> bool {
        open_location_button(ui, variant_text, path);

        ui.button(format!("Delete {}", variant_text)).clicked()
    }

    fn playlist_button<'a>(&mut self, ui: &mut Ui, label: impl IntoAtoms<'a>) -> Response {
        let playlist_button = Button::new(label).frame(true)
            .frame_when_inactive(false);

        ui.add(playlist_button)
    }
}
