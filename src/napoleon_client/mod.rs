use eframe::egui::{Context, SidePanel};
use eframe::{App, Frame};

use napoleon_amp_core::collection::folder::{Folder, FolderContent};
use napoleon_amp_core::instance::NapoleonInstance;

pub(super) struct NapoleonClientApp {
    core_instance: NapoleonInstance,
    current_folder: Folder,
    current_folder_contents: Vec<FolderContent>,
}

impl NapoleonClientApp {
    pub(super) fn new() -> Self {
        let core_instance = NapoleonInstance::init().unwrap();
        let current_folder = core_instance.home_folder.clone();
        let current_folder_contents = current_folder.get_contents().unwrap();

        Self {
            core_instance,
            current_folder,
            current_folder_contents,
        }
    }
}

impl App for NapoleonClientApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        SidePanel::left("folder_list").show(ctx, |ui| {
            if ui.button("New Folder").clicked() {}
            if ui.button("New PlayList").clicked() {}

            for folder_content in &self.current_folder_contents {
                match folder_content {
                    FolderContent::Playlist(playlist) => {}

                    FolderContent::Folder(folder) => if ui.button(&folder.name).clicked() {},
                }
            }
        });
    }
}
