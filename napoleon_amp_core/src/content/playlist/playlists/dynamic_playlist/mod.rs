pub mod dynamic_playlist_data;
pub mod filter;
mod rules;

use crate::content::SaveData;
use crate::content::folder::Folder;
use crate::content::folder::content_pool::CONTENT_POOL;
use crate::content::playlist::data::{PlaylistContentData, PlaylistSongListData, PlaylistUserData};
use crate::content::playlist::playlists::dynamic_playlist::dynamic_playlist_data::{
    DynamicPlaylistData, DynamicPlaylistDataStd,
};
use crate::content::playlist::{ClearSongsCache, InnerPlaylist, Playlist};
use crate::content::song::Song;
use serbytes::prelude::SerBytes;
use simple_id::prelude::Id;
use std::cell::{Cell, OnceCell, Ref, RefCell, RefMut};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug)]
pub struct DynamicPlaylist {
    temp_pinned_songs: RefCell<Option<Vec<Arc<Song>>>>,
    inner_playlist: InnerPlaylist,
    dynamic_playlist_data: OnceCell<RefCell<DynamicPlaylistData>>,
}

impl DynamicPlaylist {
    pub(crate) fn new(id: Id, parent: &Rc<Folder>) -> Self {
        Self {
            temp_pinned_songs: RefCell::new(None),
            inner_playlist: InnerPlaylist::new(id, parent),
            dynamic_playlist_data: OnceCell::new(),
        }
    }

    fn get_data_ref_cell(&self) -> &RefCell<DynamicPlaylistData> {
        self.dynamic_playlist_data.get_or_init(|| {
            let inner = self.get_inner();

            let playlist_data = CONTENT_POOL
                .get_dynamic_playlist_user_data(inner.id)
                .unwrap_or_else(|_| {
                    DynamicPlaylistDataStd::new(PlaylistContentData::new(
                        "Deleted Playlist".to_string(),
                        inner.parent.id,
                    ))
                    .into()
                });

            RefCell::new(playlist_data)
        })
    }

    pub fn get_dyn_user_data(&self) -> Ref<'_, DynamicPlaylistData> {
        self.get_data_ref_cell().borrow()
    }

    pub fn get_dyn_user_data_mut(&self) -> RefMut<'_, DynamicPlaylistData> {
        self.get_data_ref_cell().borrow_mut()
    }
}

impl Playlist for DynamicPlaylist {
    fn get_inner(&self) -> &InnerPlaylist {
        &self.inner_playlist
    }

    fn get_user_data(&self) -> Ref<'_, PlaylistUserData> {
        Ref::map(self.get_data_ref_cell().borrow(), |dynamic_data| {
            &dynamic_data.inner.user_data
        })
    }

    fn get_user_data_mut(&self) -> RefMut<'_, PlaylistUserData> {
        RefMut::map(self.get_data_ref_cell().borrow_mut(), |dynamic_data| {
            &mut dynamic_data.inner.user_data
        })
    }

    fn save_user_data(&self) -> std::io::Result<()> {
        let playlist_data = self.get_dyn_user_data();

        playlist_data.save_data(self.id())
    }

    fn get_icon(&self) -> &'static str {
        "dyn_playlist_icon.png"
    }

    fn load_song_list_data_refcell(&self) -> PlaylistSongListData {
        *self.temp_pinned_songs.borrow_mut() = None;

        let songs = self
            .get_dyn_user_data()
            .inner
            .get_song_list()
            .unwrap_or_default();

        let song_ids = songs.iter().map(|song| song.id).collect();

        // Little optimization so songs (and their loaded data when checking if they work with the filter) don't get dropped
        *self.temp_pinned_songs.borrow_mut() = Some(songs);

        PlaylistSongListData {
            song_ids,
            last_updated: Cell::new(self.get_dyn_user_data().inner.last_updated),
        }
    }
}

impl PartialEq for DynamicPlaylist {
    fn eq(&self, other: &Self) -> bool {
        self.inner_playlist == other.inner_playlist
    }
}

impl Eq for DynamicPlaylist {}

impl Deref for DynamicPlaylist {
    type Target = InnerPlaylist;

    fn deref(&self) -> &Self::Target {
        &self.inner_playlist
    }
}
