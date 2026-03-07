use crate::content::song::song_data::{SongData, MAX_RATING};
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
    Rating,
}

impl Display for SortByVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display_str = match self {
            Self::Title => "Title",

            Self::Artist => "Artist",

            Self::Album => "Album",

            Self::Rating => "Rating",
        };

        f.write_str(display_str)
    }
}

#[derive(SerBytes, Default, Debug, Copy, Clone)]
pub struct SortBy {
    pub sort_by_variant: SortByVariant,
    pub inverted: bool,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum SortableProperty<'s> {
    Str(&'s str),
    U8(u8),
}

/// A list of songs that may at most contain one of each song

#[derive(Debug)]
pub(super) struct SongList {
    songs_vec: SongVec,
    songs_set: HashSet<Arc<Song>>,
}

impl SongList {
    const TITLE_INDEX: usize = 0;
    const ALBUM_INDEX: usize = 1;
    const ARTIST_INDEX: usize = 2;
    const RATING_INDEX: usize = 3;

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
            Self::push_song0(*song_id, songs_set, &mut songs_vec, None)
                .expect("Will not fail if original_name is none");
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

    pub(super) fn push_new_song(&mut self, song_id: Id, original_name: &str) -> Result<(), ()> {
        let songs_set = &mut self.songs_set;
        let mut songs_vec = write_rwlock(&self.songs_vec);

        Self::push_song0(song_id, songs_set, &mut songs_vec, Some(original_name))
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

            let index = match sort_by.sort_by_variant {
                SortByVariant::Title => Self::TITLE_INDEX,
                SortByVariant::Artist => Self::ARTIST_INDEX,
                SortByVariant::Album => Self::ALBUM_INDEX,
                SortByVariant::Rating => Self::RATING_INDEX,
            };

            let a_sort_props = Self::get_sort_properties(&a_song_data, index);
            let b_sort_props = Self::get_sort_properties(&b_song_data, index);

            // for i in 0..a_sort_props.len() {
            //     let sort_prop_a = a_sort_props[i];
            //     let sort_prop_b = b_sort_props[i];
            //
            //     sort_prop_a
            // }

            if !sort_by.inverted {
                a_sort_props.cmp(&b_sort_props)
            } else {
                b_sort_props.cmp(&a_sort_props)
            }
        });
    }

    fn get_sort_properties(song_data: &SongData, swap_index: usize) -> [SortableProperty; 4] {
        let mut sort_properties = [SortableProperty::U8(0); 4];

        sort_properties[Self::TITLE_INDEX] = SortableProperty::Str(&song_data.title);
        sort_properties[Self::ALBUM_INDEX] = SortableProperty::Str(&song_data.album);
        sort_properties[Self::ARTIST_INDEX] =
            SortableProperty::Str(&song_data.artist.artist_string);
        sort_properties[Self::RATING_INDEX] = SortableProperty::U8(MAX_RATING - song_data.rating);

        let temp = sort_properties[swap_index];

        for i in (1..=swap_index).rev() {
            sort_properties[i] = sort_properties[i - 1];
        }

        sort_properties[0] = temp;

        sort_properties
    }

    fn push_song0(
        song_id: Id,
        songs_set: &mut HashSet<Arc<Song>>,
        songs_vec: &mut Vec<Arc<Song>>,
        original_name: Option<&str>,
    ) -> Result<(), ()> {
        if let Some(original_name) = original_name {
            SONG_POOL.register_new_song(song_id, original_name.to_string())?;
        }

        let song = SONG_POOL.get_song_by_id(song_id);

        {
            let mut song_data = song.get_song_data().clone();

            if song_data.title.is_empty() {
                song_data.title = original_name.unwrap_or("Unnamed Song").to_string();
                song.set_song_data_and_save(song_data);
            }
        }

        if !songs_set.contains(&song) {
            songs_set.insert(Arc::clone(&song));
            songs_vec.push(song);
        }

        Ok(())
    }
}
