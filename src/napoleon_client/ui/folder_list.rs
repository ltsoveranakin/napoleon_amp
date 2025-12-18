use crate::napoleon_client::{CreateFolderContentDialog, CreateFolderContentDialogVariant};
use eframe::egui::{CursorIcon, Id, Modal, ScrollArea, Ui};
use napoleon_amp_core::data::folder::content::FolderContentVariant;
use napoleon_amp_core::data::folder::Folder;
use napoleon_amp_core::data::playlist::Playlist;
use napoleon_amp_core::data::NamedPathLike;
use std::rc::Rc;

enum CurrentFolder {}

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

        self.render_header_buttons(ui);

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
                                Folder::add_folder(&self.current_folder, name.clone());
                            }

                            CreateFolderContentDialogVariant::Playlist => {
                                Folder::add_playlist(&self.current_folder, name.clone());
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

    fn render_header_buttons(&mut self, ui: &mut Ui) {
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

        if let Some(parent_folder) = &self.current_folder.parent {
            if ui.button("Back").clicked() {
                let parent = parent_folder.upgrade().expect("TODO: ");
                self.current_folder = parent;
            }
        }
    }

    fn render_folder_content(&mut self, ui: &mut Ui, current_playlist: &mut Option<Rc<Playlist>>) {
        ScrollArea::vertical().show(ui, |ui| {
            let mut next_folder_folder = None;
            let mut next_playlist_content = None;

            for folder_content in Folder::get_or_load_content(&self.current_folder).iter() {
                ui.separator();

                match &folder_content.variant {
                    FolderContentVariant::Playlist(playlist) => {
                        if ui
                            .label(playlist.name())
                            .on_hover_cursor(CursorIcon::PointingHand)
                            .clicked()
                        {
                            next_playlist_content = Some(Rc::clone(playlist));
                        }
                    }

                    FolderContentVariant::SubFolder(folder) => {
                        if ui
                            .label(folder.name())
                            .on_hover_cursor(CursorIcon::PointingHand)
                            .clicked()
                        {
                            next_folder_folder = Some(Rc::clone(folder));
                        }
                    }
                }
            }

            if let Some(next_folder) = next_folder_folder {
                self.current_folder = next_folder;
            }

            if let Some(next_playlist_content) = next_playlist_content {
                *current_playlist = Some(next_playlist_content);
            }
        });
    }
}
