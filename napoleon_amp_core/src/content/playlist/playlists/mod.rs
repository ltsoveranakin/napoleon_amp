mod dynamic_playlist;
mod standard_playlist;

use crate::content::playlist::data::{PlaylistSongListData, PlaylistUserData};
use crate::content::playlist::manager::MusicManager;
use crate::content::playlist::song_list::{SongList, SongVec};
use crate::content::playlist::{PlaylistParent, SelectedSongsVariant};
pub use dynamic_playlist::*;
use simple_id::prelude::Id;
pub use standard_playlist::*;
use std::cell::{Cell, OnceCell, RefCell};

const ALL_SONGS_PLAYLIST_ID: Id = Id::ZERO;

#[derive(Debug)]
pub struct InnerPlaylist {
    pub(crate) id: Id,
    pub(crate) parent: PlaylistParent,
    pub(crate) songs: RefCell<SongList>,
    pub(crate) has_loaded_songs: Cell<bool>,
    pub(crate) music_manager: RefCell<Option<MusicManager>>,
    pub(crate) songs_filtered: SongVec,
    pub(crate) variant: StandardPlaylistVariant,
    pub(crate) selected_songs: RefCell<SelectedSongsVariant>,
    pub(crate) playlist_user_data: OnceCell<RefCell<PlaylistUserData>>,
    pub(crate) playlist_song_list_data: OnceCell<RefCell<PlaylistSongListData>>,
    pub(crate) total_length: RefCell<Option<u32>>,
    pub(crate) current_search_str: RefCell<String>,
}

impl PartialEq for InnerPlaylist {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for InnerPlaylist {}
