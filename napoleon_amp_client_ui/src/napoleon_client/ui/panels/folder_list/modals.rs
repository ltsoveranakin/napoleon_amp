use eframe::egui::{Id, Modal, Ui};
use napoleon_amp_core::content::folder::Folder;
use napoleon_amp_core::content::playlist::StandardPlaylist;
use std::rc::Rc;

pub(super) enum CreateFolderContentDialogVariant {
    SubFolder,
    Playlist,
}

pub(super) enum FolderListModals {
    CreateFolderContent {
        variant: CreateFolderContentDialogVariant,
        name: String,
        current_folder: Rc<Folder>,
    },
    RenamePlaylist {
        name: String,
        playlist: Rc<StandardPlaylist>,
    },
    None,
}

impl FolderListModals {
    pub(super) fn create_folder(current_folder: Rc<Folder>) -> Self {
        Self::create(CreateFolderContentDialogVariant::SubFolder, current_folder)
    }

    pub(super) fn create_playlist(current_folder: Rc<Folder>) -> Self {
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
    pub(super) fn render(&mut self, ui: &mut Ui) {
        let should_clear_modal = match self {
            Self::CreateFolderContent {
                variant,
                name,
                current_folder,
            } => Self::render_create_folder_content(ui, variant, name, current_folder),
            Self::RenamePlaylist { name, playlist } => Self::render_change_name(ui, name, playlist),
            Self::None => false,
        };

        if should_clear_modal {
            *self = Self::None;
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
                                .expect("Error creating folder");
                        }

                        CreateFolderContentDialogVariant::Playlist => {
                            Folder::create_playlist(&current_folder, name.clone())
                                .expect("Error creating playlist");
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

    fn render_change_name(ui: &mut Ui, name: &mut String, playlist: &StandardPlaylist) -> bool {
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
