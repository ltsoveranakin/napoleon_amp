use crate::napoleon_client::ui::helpers::{ScrollListDisplay, scroll_area_iter};
use crate::napoleon_client::ui::panels::get_song_data_display_str;
use eframe::egui::{ScrollArea, Ui};
use napoleon_amp_core::content::playlist::manager::MusicManager;
use napoleon_amp_core::content::playlist::queue::Queue;

pub struct QueuePanel;

impl QueuePanel {
    pub(super) fn new() -> Self {
        Self
    }

    pub(crate) fn render(
        &mut self,
        ui: &mut Ui,
        // current_playlist: &PlaylistType,
        music_manager: &MusicManager,
    ) {
        ui.heading("Queued Songs:");

        // let songs_vec = current_playlist.get_song_vec_unfiltered();
        // let songs = read_rwlock(&songs_vec);
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
                .chain(current_queue.2.iter().map(|song| song)),
            current_queue_length,
            |_, song| {
                ScrollListDisplay::new(
                    false,
                    get_song_data_display_str(&song.get_song_data().inner),
                )
            },
            |clicked_queue_index| {
                music_manager.set_queue_index(clicked_queue_index);
            },
            |_| {},
        );
    }
}
