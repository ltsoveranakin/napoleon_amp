use crate::data::playlist::PlaybackMode;
use crate::data::song::Song;
use rand::Rng;

#[derive(Clone, Debug)]
pub struct Queue {
    pub(super) indexes: Vec<usize>,
    index: usize,
}

impl Queue {
    pub(super) fn new(mut start_index: usize, songs: &[Song], playback_mode: PlaybackMode) -> Self {
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

    pub(super) fn next(&mut self) -> usize {
        self.index += 1;
        self.index
    }

    pub(super) fn previous(&mut self) -> usize {
        self.index -= 2;
        self.index
    }

    pub(super) fn get_current(&self) -> usize {
        self.indexes[self.index]
    }

    pub(super) fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    pub(super) fn reset_queue(&mut self) {
        self.index = 0;
    }
}
