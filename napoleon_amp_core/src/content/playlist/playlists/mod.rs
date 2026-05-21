pub mod all_songs_playlist;
pub mod dynamic_playlist;
pub mod standard_playlist;

use crate::content::folder::Folder;
use crate::content::folder::content_pool::CONTENT_POOL;
use crate::content::playlist::data::{PlaylistSongListData, PlaylistUserData};
use crate::content::playlist::manager::MusicManager;
use crate::content::playlist::song_list::{SongList, SongVec};
use crate::content::playlist::{
    ClearSongsCache, ClearSongsCacheMut, PlaylistData, PlaylistParent, SelectedSongsVariant,
};
pub use dynamic_playlist::*;
use simple_id::prelude::Id;
pub use standard_playlist::*;
use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;
use std::sync::RwLock;

const ALL_SONGS_PLAYLIST_ID: Id = Id::ZERO;

// TODO: remove all refcells here and replace with normal mutability
#[derive(Debug)]
pub struct InnerPlaylist {
    pub(crate) id: Id,
    pub(crate) parent: PlaylistParent,
    pub(super) songs: RefCell<SongList>,
    pub(crate) has_loaded_songs: Cell<bool>,
    pub(crate) music_manager: RefCell<Option<MusicManager>>,
    pub(crate) songs_filtered: SongVec,
    pub(crate) selected_songs: RefCell<SelectedSongsVariant>,
    pub(crate) playlist_song_list_data: RefCell<Option<PlaylistSongListData>>,
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
            songs_filtered: SongVec::new(RwLock::new(Vec::new())),
            selected_songs: RefCell::new(SelectedSongsVariant::None),
            playlist_song_list_data: RefCell::new(None),
            total_length: RefCell::new(None),
            current_search_str: RefCell::new(String::new()),
        }
    }
}

impl ClearSongsCache for InnerPlaylist {
    fn clear_songs_cache(&self) {
        // println!("clear song cache")
        self.has_loaded_songs.set(false);
        self.songs.borrow_mut().clear_songs_cache_mut();
        self.playlist_song_list_data.borrow_mut().take();
    }
}

impl PartialEq for InnerPlaylist {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for InnerPlaylist {}

fn get_user_data_ref_cell<'a>(
    playlist_user_data: &'a OnceCell<RefCell<PlaylistUserData>>,
    inner: &InnerPlaylist,
) -> &'a RefCell<PlaylistUserData> {
    playlist_user_data.get_or_init(|| {
        let playlist_data = CONTENT_POOL
            .get_standard_playlist_user_data(inner.id)
            .unwrap_or_else(|_| PlaylistUserData::new_deleted(inner.parent.id));

        RefCell::new(playlist_data)
    })
}
