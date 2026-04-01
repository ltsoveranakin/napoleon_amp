pub mod data;
pub mod manager;
pub mod playlists;
pub mod queue;
mod song_list;

use crate::content::folder::Folder;
use crate::content::playlist::data::{PlaybackMode, PlaylistUserData};
use crate::content::playlist::manager::MusicManager;
use crate::content::playlist::song_list::SongVec;
use crate::content::song::Song;
use crate::content::song::song_data::SongData;
use crate::read_rwlock;
pub use playlists::*;
use simple_id::prelude::Id;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashSet;
use std::io;
use std::ops::{Deref, RangeInclusive};
use std::path::PathBuf;
use std::rc::Weak;
use std::sync::Arc;

struct SharedPlaylistData {}

pub trait Playlist {
    fn id(&self) -> Id;

    /// Gets the songs in the current playlist, with the filter if one is enabled
    fn get_song_vec(&self) -> SongVec;

    fn get_song_vec_unfiltered(&self) -> SongVec;

    fn get_user_data_ref_cell(&self) -> &RefCell<PlaylistUserData>;

    fn get_user_data(&self) -> Ref<'_, PlaylistUserData> {
        self.get_user_data_ref_cell().borrow()
    }

    fn get_user_data_mut(&self) -> RefMut<'_, PlaylistUserData> {
        self.get_user_data_ref_cell().borrow_mut()
    }

    fn start_play_song(&self, song_index: usize);

    fn stop_music(&self);

    fn get_music_manager(&self) -> Ref<'_, Option<MusicManager>>;

    fn set_volume(&self, volume: f32);

    fn get_volume(&self) -> f32 {
        self.get_user_data().volume
    }

    fn delete_song(&self, index: usize);

    fn get_string_list(&self, filter: &dyn Fn(&SongData) -> &String) -> Vec<String> {
        let mut string_set = HashSet::new();

        for song in read_rwlock(&self.get_song_vec()).iter() {
            let song_data = song.get_song_data();
            let string_ref = filter(&song_data);

            if !string_set.contains(string_ref) {
                string_set.insert(string_ref.clone());
            }
        }

        string_set.into_iter().collect()
    }

    fn get_artist_list(&self) -> Vec<String> {
        self.get_string_list(&|song_data| &song_data.artist.full_artist_string)
    }

    fn get_album_list(&self) -> Vec<String> {
        self.get_string_list(&|song_data| &song_data.album)
    }

    fn select_all(&self);

    fn import_existing_songs(&self, new_songs: &[Arc<Song>]);

    fn get_selected_songs(&self) -> SelectedSongsVariant;

    fn get_total_song_duration(&self) -> u32;

    fn import_songs(&self, song_paths: &[PathBuf], delete_original: bool)
    -> Result<(), Vec<usize>>;

    fn set_search_query_filter(&self, search_str: &str);

    /// Returns `None` if music manager is `None` (no song is playing) otherwise returns the index
    /// of the next song that will be played (with respect to the queue)

    fn get_current_song_playing(&self) -> Option<Arc<Song>>;

    fn select_single(&self, index: usize);

    fn rename(&self, new_name: String) -> io::Result<()>;
}

#[derive(Debug, Eq, PartialEq)]
pub enum PlaylistType {
    Standard(StandardPlaylist),
}

impl PartialEq for dyn Playlist {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for dyn Playlist {}

impl Deref for PlaylistType {
    type Target = dyn Playlist;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Standard(standard_playlist) => standard_playlist,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SelectedSongsVariant {
    None,
    Range(RangeInclusive<usize>),
    Single(usize),
    All,
}

impl SelectedSongsVariant {
    pub fn is_selected(&self, index: usize) -> bool {
        match self {
            SelectedSongsVariant::All => true,

            SelectedSongsVariant::Range(selected_range) => selected_range.contains(&index),

            SelectedSongsVariant::Single(selected_index) => index == *selected_index,

            SelectedSongsVariant::None => false,
        }
    }

    pub fn get_selected_songs<'s>(&self, songs: &'s [Arc<Song>]) -> &'s [Arc<Song>] {
        match self {
            SelectedSongsVariant::All => songs,

            SelectedSongsVariant::Range(selected_range) => {
                let selected_range = selected_range.clone();
                &songs[selected_range]
            }

            SelectedSongsVariant::Single(selected_index) => {
                let selected_index = *selected_index;
                &songs[selected_index..=selected_index]
            }

            SelectedSongsVariant::None => &[],
        }
    }
}

#[derive(Debug)]
pub(crate) struct PlaylistParent {
    id: Id,
    parent: Weak<Folder>,
}

#[derive(Debug)]
struct ParsedSearch {
    value_lower: String,
    search_type: ParsedSearchType,
    not: bool,
}

#[derive(Debug)]
enum ParsedSearchType {
    Title,
    Artist,
    Album,
    UserTag,
    Any,
}

struct Terms {
    search_type: String,
    search_value: String,
    not: bool,
}

impl ParsedSearch {
    fn parse_search_str(search_str: &str) -> Option<Self> {
        if !search_str.starts_with("$") {
            Some(ParsedSearch {
                value_lower: search_str.to_lowercase(),
                search_type: ParsedSearchType::Any,
                not: false,
            })
        } else if let Some(Terms {
            search_type,
            search_value,
            not,
        }) = Self::try_get_terms(search_str)
        {
            let parsed_search_type = match &*search_type {
                "title" => Some(ParsedSearchType::Title),

                "artist" => Some(ParsedSearchType::Artist),

                "album" => Some(ParsedSearchType::Album),

                "utag" => Some(ParsedSearchType::UserTag),

                "any" => Some(ParsedSearchType::Any),

                _ => None,
            };

            parsed_search_type.map(|search_type| ParsedSearch {
                value_lower: search_value,
                search_type,
                not,
            })
        } else {
            None
        }
    }

    fn try_get_terms(search_str: &str) -> Option<Terms> {
        let search_spl = &mut search_str[1..].split(":");

        let mut search_type = search_spl.next()?.to_lowercase();
        let search_value = search_spl.next()?.to_lowercase();

        let not = if search_type.starts_with("!") {
            search_type.remove(0);
            true
        } else {
            false
        };

        Some(Terms {
            search_type,
            search_value,
            not,
        })
    }
}
