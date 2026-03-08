use crate::napoleon_client::ui::helpers::scroll_area_styled;

use crate::napoleon_client::ui::panels::playlist_panel::PlaylistPanel;
use eframe::egui::{CursorIcon, Id, Modal, Popup, Response, ScrollArea, Ui};

use napoleon_amp_core::content::folder::content::FolderContentVariant;
use napoleon_amp_core::content::folder::Folder;
use napoleon_amp_core::content::playlist::Playlist;
use napoleon_amp_core::discord_rpc::set_rpc_playlist;
use napoleon_amp_core::instance::NapoleonInstance;
use std::ffi::OsStr;
use std::rc::{Rc, Weak};

enum CreateFolderContentDialogVariant {
    SubFolder,
    Playlist,
}

enum FolderListModal {
    CreateFolderContent {
        variant: CreateFolderContentDialogVariant,
        name: String,
        current_folder: Rc<Folder>,
    },
    RenamePlaylist {
        name: String,
        playlist: Rc<Playlist>,
    },
}

impl FolderListModal {
    fn create_folder(current_folder: Rc<Folder>) -> Self {
        Self::create(CreateFolderContentDialogVariant::SubFolder, current_folder)
    }

    fn create_playlist(current_folder: Rc<Folder>) -> Self {
        Self::create(CreateFolderContentDialogVariant::Playlist, current_folder)
    }

    fn create(variant: CreateFolderContentDialogVariant, current_folder: Rc<Folder>) -> Self {
        Self::CreateFolderContent {
            variant,
            name: String::new(),
            current_folder,
        }
    }

    /// Returns true if modal should be closed
    fn render(&mut self, ui: &mut Ui) -> bool {
        match self {
            Self::CreateFolderContent {
                variant,
                name,
                current_folder,
            } => Self::render_create_folder_content(ui, variant, name, current_folder),
            Self::RenamePlaylist { name, playlist } => Self::render_change_name(ui, name, playlist),
        }
    }

    fn render_create_folder_content(
        ui: &mut Ui,
        variant: &CreateFolderContentDialogVariant,
        name: &mut String,
        current_folder: &Rc<Folder>,
    ) -> bool {
        let mut should_close = false;

        let modal = Modal::new(Id::new("Create Content Modal")).show(ui.ctx(), |ui| {
            ui.set_width(250.);

            let heading = match variant {
                CreateFolderContentDialogVariant::SubFolder => "folder",
                CreateFolderContentDialogVariant::Playlist => "playlist",
            };

            ui.heading(format!("Create {}", heading));

            ui.label("Name: ");
            ui.text_edit_singleline(name);

            ui.horizontal(|ui| {
                if ui.button("Create").clicked() {
                    if name.is_empty() {
                        return;
                    }

                    match variant {
                        CreateFolderContentDialogVariant::SubFolder => {
                            Folder::create_folder(&current_folder, name.clone())
                                .expect("Err create folder");
                        }

                        CreateFolderContentDialogVariant::Playlist => {
                            Folder::create_playlist(&current_folder, name.clone())
                                .expect("Err create playlist");
                        }
                    }

                    should_close = true;
                }

                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

        if modal.should_close() {
            should_close = true;
        }
        should_close
    }

    fn render_change_name(ui: &mut Ui, name: &mut String, playlist: &Playlist) -> bool {
        let mut should_close = false;

        let modal = Modal::new(Id::new("Create Content Modal")).show(ui.ctx(), |ui| {
            ui.set_width(250.);

            ui.heading("Rename Playlist");

            ui.label("Name: ");
            ui.text_edit_singleline(name);

            ui.horizontal(|ui| {
                if ui.button("Rename").clicked() {
                    if name.is_empty() {
                        return;
                    }

                    playlist.rename(name.clone()).expect("Rename playlist");

                    should_close = true;
                }

                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

        if modal.should_close() {
            should_close = true;
        }

        should_close
    }
}

pub(crate) struct FolderList {
    pub(crate) current_folder: Rc<Folder>,
    current_modal: Option<FolderListModal>,
}

impl FolderList {
    pub(crate) fn new(current_folder: Rc<Folder>) -> Self {
        Self {
            current_folder,
            current_modal: None,
        }
    }

    pub(crate) fn render(
        &mut self,
        ui: &mut Ui,
        playlist_panel: &mut Option<PlaylistPanel>,
        napoleon_instance: &mut NapoleonInstance,
    ) {
        self.render_current_modal(ui);

        self.render_header_buttons(ui);

        let current_folder = Rc::clone(&self.current_folder);

        self.render_folder_content(ui, &current_folder, playlist_panel, napoleon_instance);
    }

    fn render_current_modal(&mut self, ui: &mut Ui) {
        let mut should_close = false;
        if let Some(current_modal) = &mut self.current_modal {
            should_close = current_modal.render(ui);
        }

        if should_close {
            self.current_modal = None;
        }
    }

    fn render_header_buttons(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("New Folder").clicked() {
                self.current_modal = Some(FolderListModal::create_folder(Rc::clone(
                    &self.current_folder,
                )))
            }

            if ui.button("New PlayList").clicked() {
                self.current_modal = Some(FolderListModal::create_playlist(Rc::clone(
                    &self.current_folder,
                )))
            }
        });

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
        folder: &Rc<Folder>,
        playlist_panel: &mut Option<PlaylistPanel>,
        napoleon_instance: &mut NapoleonInstance,
    ) {
        scroll_area_styled(ui, ScrollArea::vertical(), |ui| {
            let mut next_folder = None;
            let mut next_playlist = None;

            if folder.parent.is_none() {
                if self.playlist_button(ui, "All Songs").clicked() {
                    next_playlist = Some(Rc::new(Playlist::all_songs()))
                }
            }

            if let Some((folder, index)) = self.render_sub_folder_content(
                ui,
                folder,
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
        folder: &Rc<Folder>,
        playlist_panel: &mut Option<PlaylistPanel>,
        next_playlist: &mut Option<Rc<Playlist>>,
        next_folder: &mut Option<Rc<Folder>>,
        napoleon_instance: &mut NapoleonInstance,
    ) -> Option<(Rc<Folder>, usize)> {
        let mut delete_index = None;

        for (current_index, folder_content_variant) in
            Folder::get_contents(folder).iter().enumerate()
        {
            ui.separator();

            match folder_content_variant {
                FolderContentVariant::Playlist(playlist) => {
                    let playlist_name = playlist.get_name();

                    let playlist_button = self.playlist_button(ui, &playlist_name);

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
                            self.current_modal = Some(FolderListModal::RenamePlaylist {
                                name: String::new(),
                                playlist: Rc::clone(playlist),
                            });
                        }

                        if Self::shared_popup_ui(
                            ui,
                            "playlist",
                            playlist
                                .get_or_load_playlist_data()
                                .get_data_path()
                                .parent()
                                .expect("File will always have parent directory"),
                        ) {
                            delete_index = Some((Rc::clone(folder), current_index));
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
                            ui.menu_button("New", |ui| {
                                if ui.button("Playlist").clicked() {
                                    self.current_modal =
                                        Some(FolderListModal::create_playlist(Rc::clone(folder)))
                                }

                                if ui.button("Folder").clicked() {
                                    self.current_modal =
                                        Some(FolderListModal::create_folder(Rc::clone(folder)))
                                }
                            });

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

    /// Shared popup UI between folders and playlists
    ///
    /// Returns true if the content should be deleted
    fn shared_popup_ui(ui: &mut Ui, variant_text: &str, path: impl AsRef<OsStr>) -> bool {
        #[cfg(not(target_os = "android"))]
        if ui
            .button(format!("Open {} location", variant_text))
            .clicked()
        {
            if open::that_detached(path).is_err() {
                todo!()
            }
        }

        ui.button(format!("Delete {}", variant_text)).clicked()
    }

    fn playlist_button(&mut self, ui: &mut Ui, label: &str) -> Response {
        let playlist_label = ui.label(label).on_hover_cursor(CursorIcon::PointingHand);

        playlist_label
    }
}
