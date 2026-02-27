use crate::content::song::Song;
use crate::id_generator::Id;
use crate::song_pool::SONG_POOL;
use crate::{read_rwlock, write_rwlock, ReadWrapper};
use serbytes::prelude::SerBytes;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock};

pub type SongVec = Arc<RwLock<Vec<Arc<Song>>>>;

#[derive(SerBytes, Default, Debug, Copy, Clone)]
pub enum SortByVariant {
    #[default]
    Title,
    Artist,
    Album,
}

impl Display for SortByVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display_str = match self {
            Self::Title => "Title",

            Self::Artist => "Artist",

            Self::Album => "Album",
        };

        f.write_str(display_str)
    }
}

#[derive(SerBytes, Default, Debug, Copy, Clone)]
pub struct SortBy {
    pub sort_by_variant: SortByVariant,
    pub inverted: bool,
}

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

    pub(super) fn push_songs(&mut self, song_id_list: &[Id]) {
        let songs_set = &mut self.songs_set;
        let mut songs_vec = write_rwlock(&self.songs_vec);

        songs_set.reserve(song_id_list.len());
        songs_vec.reserve_exact(song_id_list.len());

        for song_id in song_id_list {
            Self::push_song0(*song_id, songs_set, &mut songs_vec, None);
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

    pub(super) fn push_new_song(&mut self, song_id: Id, original_name: &str) {
        let songs_set = &mut self.songs_set;
        let mut songs_vec = write_rwlock(&self.songs_vec);

        Self::push_song0(song_id, songs_set, &mut songs_vec, Some(original_name));
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

    pub(super) fn sort_songs(&self, sort_by: SortBy) {
        write_rwlock(&self.songs_vec).sort_by(|a, b| {
            let a_song_data = a.get_song_data();
            let b_song_data = b.get_song_data();

            let (sort_str_a, sort_str_b) = match sort_by.sort_by_variant {
                SortByVariant::Title => (&a_song_data.title, &b_song_data.title),
                SortByVariant::Artist => (
                    &a_song_data.artist.artist_string,
                    &b_song_data.artist.artist_string,
                ),
                SortByVariant::Album => (&a_song_data.album, &b_song_data.album),
            };

            if !sort_by.inverted {
                sort_str_a.cmp(sort_str_b)
            } else {
                sort_str_b.cmp(sort_str_a)
            }
        });
    }

    fn push_song0(
        song_id: Id,
        songs_set: &mut HashSet<Arc<Song>>,
        songs_vec: &mut Vec<Arc<Song>>,
        original_name: Option<&str>,
    ) {
        if let Some(original_name) = original_name {
            SONG_POOL.register_song(song_id, original_name.to_string());
        }

        let song = SONG_POOL.get_song_by_id(song_id);

        {
            let mut song_data = song.get_song_data().clone();

            if song_data.title.is_empty() {
                song_data.title = original_name.unwrap_or("Unnamed Song").to_string();
                song.set_song_data(song_data);
            }
        }

        if !songs_set.contains(&song) {
            songs_set.insert(Arc::clone(&song));
            songs_vec.push(song);
        }
    }
}
