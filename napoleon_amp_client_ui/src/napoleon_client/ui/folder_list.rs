use crate::napoleon_client::ui::helpers::scroll_area_styled;
use crate::napoleon_client::ui::playlist_panel::PlaylistPanel;
use eframe::egui::{CursorIcon, Id, Modal, Popup, Response, ScrollArea, Ui};
use napoleon_amp_core::data::folder::content::FolderContentVariant;
use napoleon_amp_core::data::folder::Folder;
use napoleon_amp_core::data::playlist::Playlist;
use napoleon_amp_core::data::NamedPathLike;
use std::rc::Rc;

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
                            Folder::add_folder(&current_folder, name.clone());
                        }

                        CreateFolderContentDialogVariant::Playlist => {
                            Folder::add_playlist(&current_folder, name.clone());
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

    pub(crate) fn render(&mut self, ui: &mut Ui, playlist_panel: &mut Option<PlaylistPanel>) {
        self.render_current_modal(ui);

        // self.render_new_content_modal(ui);
        // self.render_change_name_modal(ui);

        self.render_header_buttons(ui);

        let current_folder = Rc::clone(&self.current_folder);

        self.render_folder_content(ui, &current_folder, playlist_panel, false);
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
                self.current_modal = Some(FolderListModal::CreateFolderContent {
                    name: String::new(),
                    variant: CreateFolderContentDialogVariant::SubFolder,
                    current_folder: Rc::clone(&self.current_folder),
                });
            }

            if ui.button("New PlayList").clicked() {
                self.current_modal = Some(FolderListModal::CreateFolderContent {
                    name: String::new(),
                    variant: CreateFolderContentDialogVariant::Playlist,
                    current_folder: Rc::clone(&self.current_folder),
                });
            }
        });

        if let Some(parent_folder) = &self.current_folder.parent {
            if ui.button("Back").clicked() {
                let parent = parent_folder.upgrade().expect("TODO: ");
                self.current_folder = parent;
            }
        }
    }

    fn render_folder_content(
        &mut self,
        ui: &mut Ui,
        folder: &Rc<Folder>,
        playlist_panel: &mut Option<PlaylistPanel>,
        is_sub: bool,
    ) {
        scroll_area_styled(ui, ScrollArea::vertical(), |ui| {
            let mut next_folder_folder = None;
            let mut next_playlist = None;

            if !is_sub {
                ui.separator();

                if self.playlist_button(ui, "All Songs").clicked() {
                    next_playlist = Some(Rc::new(Playlist::all_songs()))
                }
            }

            let mut delete_index = None;

            for (current_index, folder_content) in
                Folder::get_or_load_content(folder).iter().enumerate()
            {
                ui.separator();

                match &folder_content.variant {
                    FolderContentVariant::Playlist(playlist) => {
                        let playlist_label =
                            self.playlist_button(ui, playlist.get_path_name_ref().name());

                        if playlist_label.clicked() {
                            next_playlist = Some(Rc::clone(playlist));
                        }

                        Popup::context_menu(&playlist_label).show(|ui| {
                            if ui.button("Rename Playlist").clicked() {
                                self.current_modal = Some(FolderListModal::RenamePlaylist {
                                    name: String::new(),
                                    playlist: Rc::clone(playlist),
                                });
                            }

                            #[cfg(not(target_os = "android"))]
                            if ui.button("Open File Location").clicked() {
                                if open::that_detached(
                                    playlist
                                        .get_path_name_ref()
                                        .path()
                                        .parent()
                                        .expect("File will always have parent directory"),
                                )
                                .is_err()
                                {
                                    todo!()
                                }
                            }

                            Self::popup_delete_button(ui, current_index, &mut delete_index)
                        });
                    }

                    FolderContentVariant::SubFolder(folder) => {
                        ui.horizontal(|ui| {
                            let folder_item = ui.collapsing(folder.name(), |ui| {
                                self.render_folder_content(ui, folder, playlist_panel, true);
                            });

                            if folder_item.header_response.middle_clicked() {
                                next_folder_folder = Some(Rc::clone(folder));
                            }

                            Popup::context_menu(&folder_item.header_response).show(|ui| {
                                Self::popup_delete_button(ui, current_index, &mut delete_index)
                            })
                        });
                    }
                }
            }

            if let Some(index) = delete_index {
                folder.delete_content(index);
            }

            if let Some(next_folder) = next_folder_folder {
                self.current_folder = next_folder;
            }

            if let Some(next_playlist_content) = next_playlist {
                *playlist_panel = Some(PlaylistPanel::new(next_playlist_content));
            }
        });
    }

    fn popup_delete_button(ui: &mut Ui, index: usize, delete_index: &mut Option<usize>) {
        if ui.button("Delete").clicked() {
            *delete_index = Some(index);
        }
    }

    fn playlist_button(&mut self, ui: &mut Ui, label: &str) -> Response {
        let playlist_label = ui.label(label).on_hover_cursor(CursorIcon::PointingHand);

        playlist_label
    }
}
