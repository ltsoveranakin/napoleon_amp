use eframe::egui::{Context, Id, Modal, SidePanel};
use eframe::{App, Frame};

use napoleon_amp_core::data::folder::content::FolderContentVariant;
use napoleon_amp_core::data::folder::{Folder, FolderImpl, GetOrLoadContent};
use napoleon_amp_core::instance::NapoleonInstance;
use std::rc::Rc;

enum CreateFolderContentDialogVariant {
    SubFolder,
    Playlist,
}

struct CreateFolderContentDialog {
    variant: CreateFolderContentDialogVariant,
    name: String,
}

enum Dialog {
    CreateFolderContent(CreateFolderContentDialog),
}

pub(super) struct NapoleonClientApp {
    core_instance: NapoleonInstance,
    current_folder: Rc<Folder>,
    dialog: Option<Dialog>,
}

impl NapoleonClientApp {
    pub(super) fn new() -> Self {
        let core_instance = NapoleonInstance::new();
        let current_folder = core_instance.get_base_folder();

        Self {
            core_instance,
            current_folder,
            dialog: None,
        }
    }
}

impl App for NapoleonClientApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        SidePanel::left("folder_list").show(ctx, |ui| {
            if let Some(dialog) = &mut self.dialog {
                let mut should_close = false;
                match dialog {
                    Dialog::CreateFolderContent(create_folder_content_dialog) => {
                        let name = &mut create_folder_content_dialog.name;
                        let modal =
                            Modal::new(Id::new("Create Content Modal")).show(ui.ctx(), |ui| {
                                ui.set_width(250.);

                                let heading = match create_folder_content_dialog.variant {
                                    CreateFolderContentDialogVariant::SubFolder => "folder",
                                    CreateFolderContentDialogVariant::Playlist => "playlist",
                                };

                                ui.heading(format!("Create {}", heading));

                                ui.label("Name: ");
                                ui.text_edit_singleline(name);

                                ui.horizontal(|ui| {
                                    if ui.button("Create").clicked() {
                                        if !name.is_empty() {
                                            self.current_folder.add_folder(name.clone());
                                        }
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
                }

                if should_close {
                    self.dialog = None;
                }
            }

            ui.horizontal(|ui| {
                if ui.button("New Folder").clicked() {
                    self.dialog = Some(Dialog::CreateFolderContent(CreateFolderContentDialog {
                        name: String::new(),
                        variant: CreateFolderContentDialogVariant::SubFolder,
                    }));
                }
                if ui.button("New PlayList").clicked() {
                    self.dialog = Some(Dialog::CreateFolderContent(CreateFolderContentDialog {
                        name: String::new(),
                        variant: CreateFolderContentDialogVariant::Playlist,
                    }));
                }
            });

            for folder_content in self.current_folder.get_or_load_content().iter() {
                match &folder_content.variant {
                    FolderContentVariant::Playlist(playlist) => {}

                    FolderContentVariant::SubFolder(folder) => {
                        if ui.button(&folder.path_named.name).clicked() {}
                    }
                }
            }
        });
    }
}
