use crate::content::song::Song;
use crate::song_pool::SONG_POOL;
use crate::{read_rwlock, write_rwlock, ReadWrapper};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};

pub type SongVec = Arc<RwLock<Vec<Arc<Song>>>>;

/// A list of songs that may at most contain one of each song

#[derive(Debug)]
pub(super) struct SongList {
    songs_vec: SongVec,
    songs_set: HashSet<Arc<Song>>,
}

impl SongList {
    pub(super) fn new() -> Self {
        Self {
            songs_vec: Arc::new(RwLock::new(Vec::new())),
            songs_set: HashSet::new(),
        }
    }

    pub(super) fn push_songs(&mut self, song_name_list: &[String]) {
        let songs_set = &mut self.songs_set;
        let mut songs_vec = write_rwlock(&self.songs_vec);

        songs_set.reserve(song_name_list.len());
        songs_vec.reserve_exact(song_name_list.len());

        for song_name in song_name_list {
            Self::push_song0(song_name.clone(), songs_set, &mut songs_vec);
        }
    }

    pub(super) fn push_songs_arc_list(&mut self, song_name_list: &[Arc<Song>]) {
        let songs_set = &mut self.songs_set;
        let mut songs_vec = write_rwlock(&self.songs_vec);

        songs_set.reserve(song_name_list.len());
        songs_vec.reserve_exact(song_name_list.len());

        for song in song_name_list {
            if !songs_set.contains(song) {
                songs_set.insert(Arc::clone(song));
                songs_vec.push(Arc::clone(song));
            }
        }
    }

    pub(super) fn push_song(&mut self, song_name: String) {
        let songs_set = &mut self.songs_set;
        let mut songs_vec = write_rwlock(&self.songs_vec);

        Self::push_song0(song_name, songs_set, &mut songs_vec);
    }

    pub(super) fn remove_song_at(&mut self, index: usize) {
        let songs_set = &mut self.songs_set;
        let mut songs_vec = write_rwlock(&self.songs_vec);

        let song = songs_vec.remove(index);
        songs_set.remove(&song);
    }

    pub(super) fn reserve(&mut self, additional: usize) {
        self.songs_set.reserve(additional);
        write_rwlock(&self.songs_vec).reserve_exact(additional);
    }

    pub(super) fn songs(&self) -> ReadWrapper<'_, Vec<Arc<Song>>> {
        read_rwlock(&self.songs_vec)
    }

    pub(super) fn songs_arc(&self) -> SongVec {
        Arc::clone(&self.songs_vec)
    }

    fn push_song0(
        song_name: String,
        songs_set: &mut HashSet<Arc<Song>>,
        songs_vec: &mut Vec<Arc<Song>>,
    ) {
        let song = SONG_POOL.get_song_by_name(song_name);

        if !songs_set.contains(&song) {
            songs_set.insert(Arc::clone(&song));
            songs_vec.push(song);
        }
    }
}
