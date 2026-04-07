pub mod all_songs_playlist;
pub mod dynamic_playlist;
pub mod standard_playlist;

use crate::content::folder::Folder;
use crate::content::folder::content_pool::CONTENT_POOL;
use crate::content::playlist::data::PlaylistSongListData;
use crate::content::playlist::manager::MusicManager;
use crate::content::playlist::song_list::{SongList, SongVec};
use crate::content::playlist::{PlaylistData, PlaylistParent, SelectedSongsVariant};
pub use dynamic_playlist::*;
use serbytes::prelude::SerBytes;
use simple_id::prelude::Id;
pub use standard_playlist::*;
use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

const ALL_SONGS_PLAYLIST_ID: Id = Id::ZERO;

#[derive(Debug)]
pub struct InnerPlaylist {
    pub(crate) id: Id,
    pub(crate) parent: PlaylistParent,
    pub(crate) songs: RefCell<SongList>,
    pub(crate) has_loaded_songs: Cell<bool>,
    pub(crate) music_manager: RefCell<Option<MusicManager>>,
    pub(crate) songs_filtered: SongVec,
    pub(crate) selected_songs: RefCell<SelectedSongsVariant>,
    pub(crate) playlist_song_list_data: OnceCell<RefCell<PlaylistSongListData>>,
    pub(crate) total_length: RefCell<Option<u32>>,
    pub(crate) current_search_str: RefCell<String>,
}

impl InnerPlaylist {
    fn new(id: Id, parent: &Rc<Folder>) -> Self {
        Self {
            id,
            parent: PlaylistParent {
                id: parent.id,
                parent: Rc::downgrade(parent),
            },
            songs: RefCell::new(SongList::new()),
            has_loaded_songs: Cell::new(false),
            music_manager: RefCell::new(None),
            songs_filtered: Arc::new(RwLock::new(Vec::new())),
            selected_songs: RefCell::new(SelectedSongsVariant::None),
            playlist_song_list_data: OnceCell::new(),
            total_length: RefCell::new(None),
            current_search_str: RefCell::new(String::new()),
        }
    }
}

impl PartialEq for InnerPlaylist {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for InnerPlaylist {}

fn get_user_data_ref_cell<'a, D>(
    playlist_user_data: &'a OnceCell<RefCell<D>>,
    inner: &InnerPlaylist,
) -> &'a RefCell<D>
where
    D: SerBytes + PlaylistData,
{
    playlist_user_data.get_or_init(|| {
        let playlist_data = CONTENT_POOL
            .get_playlist_user_data(inner.id)
            .unwrap_or_else(|_| D::new_deleted(inner.parent.id));

        RefCell::new(playlist_data)
    })
}
