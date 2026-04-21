use eframe::egui::{Id, Modal, Ui};
use napoleon_amp_core::content::folder::Folder;
use napoleon_amp_core::content::playlist::dynamic_playlist_data::DynamicPlaylistDataStd;
use napoleon_amp_core::content::playlist::filter::{
    ComparisonMethod, FilterRule, FilterRules, ValuesType,
};
use napoleon_amp_core::content::playlist::{Playlist, PlaylistType};
use std::ops::Deref;
use std::rc::Rc;

pub(super) type EditPlaylistType =
    PlaylistType<Rc<PlaylistType>, (DynamicPlaylistDataStd, Rc<PlaylistType>)>;

pub(super) enum CreatePlaylistVariant {
    Standard,
    Dynamic,
}

pub(super) enum CreateFolderContentDialogVariant {
    SubFolder,
    Playlist(CreatePlaylistVariant),
}

pub(super) enum FolderListModals {
    CreateFolderContent {
        variant: CreateFolderContentDialogVariant,
        name: String,
        current_folder: Rc<Folder>,
    },
    EditPlaylist {
        name: String,
        edit_playlist_type: EditPlaylistType,
    },
    None,
}

impl FolderListModals {
    pub(super) fn create_folder(current_folder: Rc<Folder>) -> Self {
        Self::create(CreateFolderContentDialogVariant::SubFolder, current_folder)
    }

    pub(super) fn create_playlist(
        current_folder: Rc<Folder>,
        create_playlist_variant: CreatePlaylistVariant,
    ) -> Self {
        Self::create(
            CreateFolderContentDialogVariant::Playlist(create_playlist_variant),
            current_folder,
        )
    }

    fn create(variant: CreateFolderContentDialogVariant, current_folder: Rc<Folder>) -> Self {
        Self::CreateFolderContent {
            variant,
            name: String::new(),
            current_folder,
        }
    }

    pub(super) fn render(&mut self, ui: &mut Ui) {
        let should_clear_modal = match self {
            Self::CreateFolderContent {
                variant,
                name,
                current_folder,
            } => Self::render_create_folder_content(ui, variant, name, current_folder),
            Self::EditPlaylist {
                name,
                edit_playlist_type: playlist,
            } => Self::render_edit_playlist(ui, name, playlist),
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
                CreateFolderContentDialogVariant::Playlist(playlist_variant) => {
                    match playlist_variant {
                        CreatePlaylistVariant::Standard => "standard",
                        CreatePlaylistVariant::Dynamic => "dynamic",
                    }
                }
            };

            ui.heading(format!("Create {} playlist", heading));

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

                        CreateFolderContentDialogVariant::Playlist(playlist_variant) => {
                            match playlist_variant {
                                CreatePlaylistVariant::Standard => {
                                    Folder::create_standard_playlist(&current_folder, name.clone())
                                        .expect("Error creating standard playlist");
                                }

                                CreatePlaylistVariant::Dynamic => {
                                    Folder::create_dynamic_playlist(&current_folder, name.clone())
                                        .expect("Error creating dynamic playlist");
                                }
                            }
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

    fn render_edit_playlist(
        ui: &mut Ui,
        name: &mut String,
        edit_playlist: &mut EditPlaylistType,
    ) -> bool {
        let mut should_close = false;
        let mut rename = false;

        let modal = Modal::new(Id::new("Create Content Modal")).show(ui.ctx(), |ui| {
            ui.set_width(250.);

            ui.heading("Edit Playlist");

            ui.label("Name: ");
            ui.text_edit_singleline(name);

            if let PlaylistType::Dynamic((dyn_user_data, dyn_playlist)) = edit_playlist {
                ui.label("wip (all playlist only)");

                for filter in &mut dyn_user_data.rules.filters {
                    let (value_type, cmp_method) = filter.get_mut_values_pair();

                    let mut string_val = match value_type {
                        ValuesType::Str(s) => s.to_string(),
                        ValuesType::U8(int) => int.to_string(),
                    };

                    ui.horizontal(|ui| {
                        ui.label(cmp_method.to_string());
                        ui.text_edit_singleline(&mut string_val);
                    });
                }

                ui.menu_button("Add filter", |ui| {
                    if ui.button("Title").clicked() {
                        dyn_user_data
                            .rules
                            .filters
                            .push(FilterRules::Title(FilterRule::new(
                                "<Track Title>".to_string(),
                                ComparisonMethod::Contains,
                            )))
                    }
                });
            }

            ui.horizontal(|ui| {
                if ui.button("Ok").clicked() {
                    if name.is_empty() {
                        return;
                    }

                    rename = true;

                    should_close = true;
                }

                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

        if rename {
            let playlist: &dyn Playlist = match edit_playlist {
                EditPlaylistType::Standard(playlist) => (**playlist).deref(),
                EditPlaylistType::Dynamic((_, playlist)) => (**playlist).deref(),
                EditPlaylistType::AllSongs(playlist) => playlist,
            };

            playlist.rename(name.clone()).expect("Editing playlist");
        }

        if modal.should_close() {
            should_close = true;
        }

        should_close
    }
}
