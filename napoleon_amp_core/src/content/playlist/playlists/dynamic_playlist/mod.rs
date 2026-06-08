pub mod dynamic_playlist_data;
pub mod filter;
pub mod rules;

use crate::content::SaveData;
use crate::content::folder::Folder;
use crate::content::folder::content_pool::CONTENT_POOL;
use crate::content::playlist::data::{PlaylistContentData, PlaylistSongListData, PlaylistUserData};
use crate::content::playlist::playlists::dynamic_playlist::dynamic_playlist_data::{
    DynamicPlaylistData, DynamicPlaylistDataStd,
};
use crate::content::playlist::{ClearSongsCache, InnerPlaylist, Playlist, PlaylistData};
use crate::content::song::Song;
use simple_id::prelude::Id;
use std::cell::{OnceCell, Ref, RefCell, RefMut};
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
                    DynamicPlaylistDataStd::new(PlaylistContentData::new_deleted(inner.parent.id))
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

    fn get_icon_str(&self) -> &'static str {
        "dyn_playlist_icon.png"
    }

    fn load_song_list_data(&self) -> PlaylistSongListData {
        *self.temp_pinned_songs.borrow_mut() = None;

        let dyn_user_data = self.get_dyn_user_data();

        let song_list_res = dyn_user_data
            .inner
            .get_song_list(self.id)
            .unwrap_or_default();

        // Little optimization so songs (and their loaded data when checking if they work with the filter) don't get dropped
        *self.temp_pinned_songs.borrow_mut() = Some(song_list_res.songs);

        if song_list_res.used_cached_songs {
            println!("Using cached song list");
        } else {
            println!("Recreating song list");

            // TODO: handle failure? doesnt break if it fails to save since its a dyn playlist that will just recreate itself
            let _ = song_list_res.song_list_data.save_data(self.id);
        }

        song_list_res.song_list_data
    }
}

impl ClearSongsCache for DynamicPlaylist {
    fn clear_songs_cache(&self) {
        {
            let mut song_list = self.get_song_list_mut();
            song_list.song_ids.clear();
            song_list.last_updated.set(0);
            song_list.save_data(self.id).expect("Unable to clear cache");
        }

        self.inner_playlist.clear_songs_cache();
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
