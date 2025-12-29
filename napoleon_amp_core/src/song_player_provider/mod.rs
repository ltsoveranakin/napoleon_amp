use crate::data::song::Song;
use std::sync::{Arc, RwLock};

pub mod rodio_player_provider;

pub trait SongPlayerProvider: Default {
    fn new(songs_arc: Arc<RwLock<Vec<Song>>>, start_index: usize, volume: f32) -> Self;
}
