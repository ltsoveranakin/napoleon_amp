mod ui;

use eframe::egui::{CentralPanel, Context, SidePanel};
use eframe::{App, Frame};

use crate::napoleon_client::ui::folder_list::FolderList;
use crate::napoleon_client::ui::playlist_page::PlaylistPanel;

use napoleon_amp_core::instance::NapoleonInstance;

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
    folder_list: FolderList,
    playlist_panel: Option<PlaylistPanel>,
    volume: f32,
}

impl NapoleonClientApp {
    pub(super) fn new() -> Self {
        let core_instance = NapoleonInstance::new();
        let current_folder = core_instance.get_base_folder();

        Self {
            core_instance,
            folder_list: FolderList::new(current_folder),
            playlist_panel: None,
            volume: 1.,
        }
    }
}

impl App for NapoleonClientApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        SidePanel::left("folder_list").show(ctx, |ui| {
            self.folder_list.render(ui, &mut self.playlist_panel);
        });

        CentralPanel::default().show(ctx, |ui| {
            if let Some(playlist_panel) = &mut self.playlist_panel {
                playlist_panel.render(ctx, ui, &mut self.volume);
            }
        });
    }
}
