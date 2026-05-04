use eframe::egui::{Id, Modal, Ui};
use napoleon_amp_core::content::folder::Folder;
use napoleon_amp_core::content::playlist::dynamic_playlist_data::DynamicPlaylistDataStd;
use napoleon_amp_core::content::playlist::filter::{
    ComparisonMethod, FilterRule, FilterRules, ValuesType,
};
use napoleon_amp_core::content::playlist::{ClearSongsCache, Playlist, PlaylistType};
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
        let mut edit = false;

        let modal = Modal::new(Id::new("Create Content Modal")).show(ui.ctx(), |ui| {
            ui.set_width(250.);

            ui.heading("Edit Playlist");

            ui.label("Name: ");
            ui.text_edit_singleline(name);

            if let PlaylistType::Dynamic((dyn_user_data, _)) = edit_playlist {
                // ui.label("wip (all playlist only)");

                let len = dyn_user_data.rules.filters.len();

                for (i, filter) in dyn_user_data.rules.filters.iter_mut().enumerate() {
                    let filter_of_str = filter.get_display_str();

                    let (mut string_val, cmp_method_copy) = {
                        let (value_type, cmp_method_ref) = filter.get_mut_values_pair();

                        let string_val = match value_type {
                            ValuesType::Str(s) => s.to_string(),
                            ValuesType::U8(int) => int.to_string(),
                        };

                        (string_val, *cmp_method_ref)
                    };

                    let edited = ui
                        .horizontal(|ui| {
                            ui.menu_button(filter_of_str, |ui| {
                                for filter_rules_ty in FilterRules::values() {
                                    if ui.button(filter_rules_ty.get_display_str()).clicked() {
                                        *filter = FilterRules::from_variant(
                                            filter_rules_ty,
                                            &string_val,
                                            cmp_method_copy,
                                        );
                                    }
                                }
                            });

                            ui.menu_button(cmp_method_copy.get_display_str().to_string(), |ui| {
                                for cmp_method_item in ComparisonMethod::all_values() {
                                    if ui.button(cmp_method_item.get_display_str()).clicked() {
                                        // TODO: delete this function all together, clean it up
                                        *filter.get_mut_values_pair().1 = *cmp_method_item;
                                    }
                                }
                            });

                            if ui.text_edit_singleline(&mut string_val).changed() {
                                true
                            } else {
                                false
                            }
                        })
                        .inner;

                    if i < len.saturating_sub(1) {
                        ui.label("And");
                    }

                    if edited {
                        //TODO: handle this err
                        if filter.try_assign_from_str(&string_val).is_ok() {}
                    }
                }

                ui.menu_button("Add filter", |ui| {
                    if ui.button("Title").clicked() {
                        dyn_user_data
                            .rules
                            .filters
                            .push(FilterRules::Title(FilterRule::new(
                                "<Title>".to_string(),
                                ComparisonMethod::Contains,
                            )))
                    }

                    if ui.button("Artist").clicked() {
                        dyn_user_data
                            .rules
                            .filters
                            .push(FilterRules::Artist(FilterRule::new(
                                "<Artist>".to_string(),
                                ComparisonMethod::Contains,
                            )))
                    }

                    if ui.button("Album").clicked() {
                        dyn_user_data
                            .rules
                            .filters
                            .push(FilterRules::Album(FilterRule::new(
                                "<Album>".to_string(),
                                ComparisonMethod::Contains,
                            )))
                    }

                    if ui.button("Rating").clicked() {
                        dyn_user_data
                            .rules
                            .filters
                            .push(FilterRules::Rating(FilterRule::new(
                                0,
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

                    edit = true;

                    should_close = true;
                }

                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

        if edit {
            let playlist: &dyn Playlist = match edit_playlist {
                EditPlaylistType::Standard(playlist) => (**playlist).deref(),
                EditPlaylistType::Dynamic((dyn_playlist_data_std, playlist)) => {
                    match &**playlist {
                        PlaylistType::Dynamic(dynamic_playlist) => {
                            dynamic_playlist.get_dyn_user_data_mut().inner =
                                dyn_playlist_data_std.clone();
                        }

                        _ => {
                            unreachable!()
                        }
                    }

                    playlist
                        .save_user_data()
                        .expect("Failed to save dynamic playlist user data");
                    playlist.get_inner().clear_songs_cache();
                    (**playlist).deref()
                }
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
