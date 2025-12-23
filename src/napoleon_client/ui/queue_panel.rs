use eframe::egui::{Button, ScrollArea, Ui};

use napoleon_amp_core::data::playlist::manager::MusicManager;
use napoleon_amp_core::data::playlist::Playlist;
use napoleon_amp_core::data::NamedPathLike;

pub struct QueuePanel {}

impl QueuePanel {
    pub(super) fn new() -> Self {
        Self {}
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
        let queue_indexes = queue.indexes();

        ScrollArea::vertical().show(ui, |ui| {
            for queue_index in queue.index()..queue_indexes.len() {
                let song_index = queue_indexes[queue_index];
                let song = &songs[song_index];

                let button = Button::new(song.name()).frame(true);

                if ui.add(button).clicked() {
                    music_manager.set_queue_index(queue_index);
                }
            }
        });
    }
}
