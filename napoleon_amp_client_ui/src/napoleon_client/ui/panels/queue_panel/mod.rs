use crate::napoleon_client::ui::helpers::{scroll_area_iter, ScrollListDisplay};
use eframe::egui::{ScrollArea, Ui};
use napoleon_amp_core::content::playlist::manager::MusicManager;
use napoleon_amp_core::content::playlist::queue::Queue;
use napoleon_amp_core::content::playlist::StaticPlaylist;
use napoleon_amp_core::read_rwlock;

pub struct QueuePanel;

impl QueuePanel {
    pub(super) fn new() -> Self {
        Self
    }

    pub(crate) fn render(
        &mut self,
        ui: &mut Ui,
        current_playlist: &StaticPlaylist,
        music_manager: &MusicManager,
    ) {
        ui.heading("Queued Songs:");

        let songs_vec = current_playlist.get_or_load_songs_unfiltered();
        let songs = read_rwlock(&songs_vec);
        let queue = music_manager.queue();

        let current_queue = queue.current_queue();
        let current_queue_length = Queue::queue_length(current_queue);

        scroll_area_iter(
            ui,
            ScrollArea::vertical(),
            current_queue.iter().flat_map(|inner| inner.iter()),
            current_queue_length,
            |_, song_index| {
                ScrollListDisplay::new(false, songs[*song_index].get_song_data().title.clone())
            },
            |clicked_queue_index| {
                music_manager.set_queue_index(clicked_queue_index);
            },
            |_| {},
        );
    }
}
