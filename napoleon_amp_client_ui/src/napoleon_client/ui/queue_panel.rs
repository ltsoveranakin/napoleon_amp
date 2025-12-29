use crate::napoleon_client::ui::helpers::{scroll_area_list, ScrollListDisplay};
use eframe::egui::{ScrollArea, Ui};
use napoleon_amp_core::data::playlist::manager::MusicManager;
use napoleon_amp_core::data::playlist::Playlist;
use napoleon_amp_core::data::NamedPathLike;

pub struct QueuePanel;

impl QueuePanel {
    pub(super) fn new() -> Self {
        Self
    }

    pub(crate) fn render(
        &mut self,
        ui: &mut Ui,
        current_playlist: &Playlist,
        music_manager: &MusicManager,
    ) {
        ui.heading("Queued Next:");

        let songs = &*current_playlist.get_or_load_songs();
        let queue = music_manager.queue();

        scroll_area_list(
            ui,
            ScrollArea::vertical(),
            queue.current_queue(),
            |_, song_index| ScrollListDisplay::new(false, songs[*song_index].name()),
            |clicked_queue_index| {
                music_manager.set_queue_index(clicked_queue_index);
            },
            |_| {},
        );
    }
}
