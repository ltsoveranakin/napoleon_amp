use crate::data::song::Song;
use crate::song_player_provider::SongPlayerProvider;
use rodio::OutputStream;
use std::sync::{Arc, RwLock};

pub struct RodioPlayerProvider {
    _output_stream: OutputStream,
}

impl SongPlayerProvider for RodioPlayerProvider {
    fn new(songs_arc: Arc<RwLock<Vec<Song>>>, start_index: usize, volume: f32) -> Self {
        todo!()
    }
}

impl Default for RodioPlayerProvider {
    fn default() -> Self {
        todo!()
    }
}
