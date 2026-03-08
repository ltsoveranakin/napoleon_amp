use crate::content::playlist::PlaybackMode;
use crate::content::song::Song;
use rand::RngExt;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Clone, Debug)]
enum IndexMode {
    UseQueue,
    Index(usize),
}

#[derive(Clone, Debug)]
pub struct Queue {
    pub(super) indexes: Vec<usize>,

    index: usize,
    temporary_queue: VecDeque<usize>,
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
            temporary_queue: VecDeque::new(),
        }
    }

    pub fn current_queue(&self) -> [&[usize]; 3] {
        let temporary_queue_slices = self.temporary_queue.as_slices();

        [
            temporary_queue_slices.0,
            temporary_queue_slices.1,
            &self.indexes[self.index..],
        ]
    }

    pub fn queue_length(queue_array: [&[usize]; 3]) -> usize {
        let mut length = 0;

        for arr in queue_array {
            length += arr.len();
        }

        length
    }

    pub fn push_temporary_queue(&mut self, index: usize) {
        self.temporary_queue.push_back(index);
    }

    pub(super) fn get_next_song_index(&mut self) -> usize {
        self.temporary_queue
            .pop_front()
            .unwrap_or(self.indexes[self.index])
    }

    // pub(super) fn get_current_song_index(&self) -> usize {
    //     self.indexes[self.get_wrapped_index(self.index as i32 - 1)]
    // }

    pub(super) fn next(&mut self) {
        if self.temporary_queue.is_empty() {
            self.index = self.get_wrapped_index(self.index as i32 + 1);
        }
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
