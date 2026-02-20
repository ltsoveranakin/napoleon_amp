mod colors;
mod ui;

use crate::napoleon_client::ui::panels::folder_list::FolderList;
use crate::napoleon_client::ui::panels::playlist_panel::PlaylistPanel;
use crate::napoleon_client::ui::panels::top_menu_bar::TopMenuBar;
use eframe::egui::{CentralPanel, Context, MenuBar, SidePanel, TopBottomPanel};
use eframe::{App, Frame};
use napoleon_amp_core::instance::NapoleonInstance;

pub struct NapoleonClientApp {
    core_instance: NapoleonInstance,
    folder_list: FolderList,
    top_menu_bar: TopMenuBar,
    playlist_panel: Option<PlaylistPanel>,
}

impl NapoleonClientApp {
    pub fn new() -> Self {
        let core_instance = NapoleonInstance::new();
        let current_folder = core_instance.get_base_folder();

        Self {
            core_instance,
            folder_list: FolderList::new(current_folder),
            top_menu_bar: TopMenuBar::new(),
            playlist_panel: None,
        }
    }
}

impl App for NapoleonClientApp {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                self.top_menu_bar.render(ui);
            });
        });

        SidePanel::left("folder_list").show(ctx, |ui| {
            self.folder_list.render(ui, &mut self.playlist_panel);
        });

        if let Some(ref mut playlist_panel) = self.playlist_panel {
            if let Some(ref music_manager) = *playlist_panel.current_playlist.get_music_manager() {
                SidePanel::right("queue_list").show(ctx, |ui| {
                    playlist_panel.queue_panel.render(
                        ui,
                        &playlist_panel.current_playlist,
                        music_manager,
                    );
                });
            }

            CentralPanel::default().show(ctx, |ui| {
                playlist_panel.render(ctx, ui, &mut self.core_instance);
            });
        }
    }
}
