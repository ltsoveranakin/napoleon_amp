mod colors;
mod texture_pool;
mod ui;

use crate::napoleon_client::texture_pool::TexturePool;
use crate::napoleon_client::ui::panels::folder_list::FolderList;
use crate::napoleon_client::ui::panels::playlist_panel::PlaylistPanel;
use crate::napoleon_client::ui::panels::top_menu_bar::TopMenuBar;
use eframe::egui::{CentralPanel, Context, MenuBar, SidePanel, TopBottomPanel};
use eframe::{App, Frame};
use napoleon_amp_core::instance::NapoleonInstance;
use std::rc::Rc;
use std::time::Duration;

pub struct NapoleonClientApp {
    napoleon_instance: NapoleonInstance,
    folder_list: FolderList,
    top_menu_bar: TopMenuBar,
    playlist_panel: Option<PlaylistPanel>,
    texture_pool: TexturePool,
}

impl NapoleonClientApp {
    pub fn new() -> Self {
        let core_instance = NapoleonInstance::new();
        let current_folder = Rc::clone(&core_instance.base_folder);

        Self {
            napoleon_instance: core_instance,
            folder_list: FolderList::new(current_folder),
            top_menu_bar: TopMenuBar::new(),
            playlist_panel: None,
            texture_pool: TexturePool::new(),
        }
    }
}

impl App for NapoleonClientApp {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                self.top_menu_bar.render(ui, &mut self.napoleon_instance);
            });
        });

        SidePanel::left("folder_list").show(ctx, |ui| {
            self.folder_list.render(
                ui,
                &mut self.playlist_panel,
                &mut self.texture_pool,
                &mut self.napoleon_instance,
            );
        });

        if let Some(ref mut playlist_panel) = self.playlist_panel {
            if let Some(ref music_manager) = *playlist_panel.current_playlist.get_music_manager() {
                SidePanel::right("queue_list").show(ctx, |ui| {
                    playlist_panel.queue_panel.render(ui, music_manager);
                });
            }

            CentralPanel::default().show(ctx, |ui| {
                playlist_panel.render(ctx, ui, &mut self.napoleon_instance);
            });
        }
    }
}

pub(super) fn duration_to_str(duration: Duration) -> String {
    secs_to_str(duration.as_secs())
}

pub(super) fn secs_to_str(secs: u64) -> String {
    let seconds = secs % 60;
    let minutes_total = secs / 60;
    let minutes = minutes_total % 60;
    let hours = minutes_total / 60;

    let hours_minutes_str = if hours != 0 {
        format!("{hours}:{:02}", minutes)
    } else {
        minutes.to_string()
    };

    format!("{hours_minutes_str}:{:02}", seconds)
}
