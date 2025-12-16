mod ui;

use eframe::egui::{CentralPanel, Context, SidePanel};
use eframe::{App, Frame};

use crate::napoleon_client::ui::folder_list::FolderList;
use crate::napoleon_client::ui::playlist_page::PlaylistPage;

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
    playlist_page: PlaylistPage,
}

impl NapoleonClientApp {
    pub(super) fn new() -> Self {
        let core_instance = NapoleonInstance::new();
        let current_folder = core_instance.get_base_folder();

        Self {
            core_instance,
            folder_list: FolderList::new(current_folder),
            playlist_page: PlaylistPage::new(),
        }
    }
}

impl App for NapoleonClientApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        SidePanel::left("folder_list").show(ctx, |ui| {
            self.folder_list
                .draw(ui, &mut self.playlist_page.current_playlist);
        });

        CentralPanel::default().show(ctx, |ui| {
            self.playlist_page.draw(ui);
        });
    }
}
