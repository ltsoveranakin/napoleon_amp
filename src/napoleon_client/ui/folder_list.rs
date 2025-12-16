use crate::napoleon_client::{CreateFolderContentDialog, CreateFolderContentDialogVariant};
use eframe::egui::{Id, Modal, Ui};
use napoleon_amp_core::data::folder::content::FolderContentVariant;
use napoleon_amp_core::data::folder::{Folder, FolderImpl, GetOrLoadContent};
use napoleon_amp_core::data::playlist::Playlist;
use napoleon_amp_core::data::NamedPathLike;
use std::rc::Rc;

pub(crate) struct FolderList {
    pub(crate) current_folder: Rc<Folder>,
    dialog: Option<CreateFolderContentDialog>,
}

impl FolderList {
    pub(crate) fn new(current_folder: Rc<Folder>) -> Self {
        Self {
            current_folder,
            dialog: None,
        }
    }

    pub(crate) fn draw(&mut self, ui: &mut Ui, current_playlist: &mut Option<Rc<Playlist>>) {
        self.render_modal(ui);

        self.render_new_buttons(ui);

        self.render_folder_content(ui, current_playlist);
    }

    fn render_modal(&mut self, ui: &mut Ui) {
        let mut should_close = false;
        if let Some(dialog) = &mut self.dialog {
            let name = &mut dialog.name;
            let create_folder_variant = &dialog.variant;

            let modal = Modal::new(Id::new("Create Content Modal")).show(ui.ctx(), |ui| {
                ui.set_width(250.);

                let heading = match create_folder_variant {
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

                        match create_folder_variant {
                            CreateFolderContentDialogVariant::SubFolder => {
                                self.current_folder.add_folder(name.clone());
                            }

                            CreateFolderContentDialogVariant::Playlist => {
                                self.current_folder.add_playlist(name.clone())
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
        }

        if should_close {
            self.dialog = None;
        }
    }

    fn render_new_buttons(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("New Folder").clicked() {
                self.dialog = Some(CreateFolderContentDialog {
                    name: String::new(),
                    variant: CreateFolderContentDialogVariant::SubFolder,
                });
            }
            if ui.button("New PlayList").clicked() {
                self.dialog = Some(CreateFolderContentDialog {
                    name: String::new(),
                    variant: CreateFolderContentDialogVariant::Playlist,
                });
            }
        });
    }

    fn render_folder_content(&mut self, ui: &mut Ui, current_playlist: &mut Option<Rc<Playlist>>) {
        let mut next_folder_folder_content = None;
        let mut next_playlist_content = None;

        for folder_content in self.current_folder.get_or_load_content().iter() {
            match &folder_content.variant {
                FolderContentVariant::Playlist(playlist) => {
                    if ui.button(playlist.name()).clicked() {
                        next_playlist_content = Some(Rc::clone(playlist));
                    }
                }

                FolderContentVariant::SubFolder(folder) => {
                    if ui.button(folder.name()).clicked() {
                        next_folder_folder_content = Some(Rc::clone(folder));
                    }
                }
            }
        }

        if let Some(next_folder_content) = next_folder_folder_content {
            self.current_folder = next_folder_content;
        }

        if let Some(next_playlist_content) = next_playlist_content {
            *current_playlist = Some(next_playlist_content);
        }
    }
}
