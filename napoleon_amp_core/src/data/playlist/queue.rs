use crate::data::song::Song;

#[derive(Clone, Debug)]
pub struct Queue {
    pub(super) indexes: Vec<usize>,
    index: usize,
}

impl Queue {
    pub(super) fn new(start_index: usize, songs: &[Song]) -> Self {
        let mut indexes = Vec::with_capacity(songs.len());

        for index in 0..songs.len() {
            indexes.push(index);
        }

        Self {
            indexes,
            index: start_index,
        }
    }

    pub fn current_queue(&self) -> &[usize] {
        &self.indexes[self.index..]
    }

    pub fn index(&self) -> usize {
        self.index
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
        self.index
    }

    pub(super) fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    pub(super) fn reset_queue(&mut self) {
        self.index = 0;
    }
}
