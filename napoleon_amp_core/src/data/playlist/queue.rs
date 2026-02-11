use crate::data::playlist::PlaybackMode;
use crate::data::song::Song;
use rand::RngExt;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Queue {
    pub(super) indexes: Vec<usize>,
    index: usize,
}

impl Queue {
    pub(super) fn new(
        mut start_index: usize,
        songs: &[Arc<Song>],
        playback_mode: PlaybackMode,
    ) -> Self {
        let mut indexes = Vec::with_capacity(songs.len());

        for index in 0..songs.len() {
            indexes.push(index);
        }

        match playback_mode {
            PlaybackMode::Sequential => {
                // no-op
            }

            PlaybackMode::Shuffle => {
                let mut rng = rand::rng();
                indexes.swap(start_index, 0);

                for i in 1..indexes.len() {
                    let swap_to = rng.random_range(1..indexes.len());
                    indexes.swap(i, swap_to);
                }

                start_index = 0;
            }
        }

        Self {
            indexes,
            index: start_index,
        }
    }

    pub fn current_queue(&self) -> &[usize] {
        &self.indexes[self.index..]
    }

    pub(super) fn get_next_song_index(&self) -> usize {
        self.indexes[self.index]
    }

    pub(super) fn get_current_song_index(&self) -> usize {
        self.indexes[self.get_wrapped_index(self.index as i32 - 1)]
    }

    pub(super) fn next(&mut self) {
        self.index = self.get_wrapped_index(self.index as i32 + 1);
    }

    pub(super) fn previous(&mut self) {
        self.sub_index(2);
    }

    pub(super) fn restart_song(&mut self) {
        self.sub_index(1);
    }

    pub(super) fn set_index_from_queue(&mut self, index: usize) {
        self.index += index;
    }

    pub(super) fn reset_queue(&mut self) {
        self.index = 0;
    }

    fn sub_index(&mut self, amt: i32) {
        let new_index = self.index as i32 - amt;

        self.index = self.get_wrapped_index(new_index);
    }

    fn get_wrapped_index(&self, index: i32) -> usize {
        index.rem_euclid(self.indexes.len() as i32) as usize
    }
}
