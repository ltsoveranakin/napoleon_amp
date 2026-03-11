use crate::napoleon_client::ui::helpers::{scroll_area_iter, ScrollListDisplay};
use crate::napoleon_client::ui::panels::get_song_data_display_str;
use eframe::egui::{ScrollArea, Ui};
use napoleon_amp_core::content::playlist::manager::MusicManager;
use napoleon_amp_core::content::playlist::queue::Queue;
use napoleon_amp_core::content::playlist::Playlist;
use napoleon_amp_core::read_rwlock;

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
        ui.heading("Queued Songs:");

        let songs_vec = current_playlist.get_or_load_songs_unfiltered();
        let songs = read_rwlock(&songs_vec);
        let queue = music_manager.queue();

        let current_queue = queue.current_queue();
        let current_queue_length = Queue::queue_length(current_queue);

        scroll_area_iter(
            ui,
            ScrollArea::vertical(),
            current_queue
                .0
                .iter()
                .chain(current_queue.1)
                .chain(current_queue.2.iter().map(|song_index| &songs[*song_index])),
            current_queue_length,
            |_, song| {
                ScrollListDisplay::new(false, get_song_data_display_str(&song.get_song_data()))
            },
            |clicked_queue_index| {
                music_manager.set_queue_index(clicked_queue_index);
            },
            |_| {},
        );
    }
}
