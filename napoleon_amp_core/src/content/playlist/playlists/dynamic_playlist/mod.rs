mod dynamic_playlist_data;
mod filter;
mod rules;

use crate::content::folder::content_pool::CONTENT_POOL;
use crate::content::playlist::data::{PlaylistContentData, PlaylistUserData};
use crate::content::playlist::playlists::dynamic_playlist::dynamic_playlist_data::{
    DynamicPlaylistData, DynamicPlaylistDataStd,
};
use crate::content::playlist::{InnerPlaylist, Playlist};
use serbytes::prelude::SerBytes;
use std::cell::{OnceCell, Ref, RefCell, RefMut};

#[derive(Debug)]
pub struct DynamicPlaylist {
    inner_playlist: InnerPlaylist,
    dynamic_playlist_data: OnceCell<RefCell<DynamicPlaylistData>>,
}

impl DynamicPlaylist {
    fn get_data_ref_cell(&self) -> &RefCell<DynamicPlaylistData> {
        self.dynamic_playlist_data.get_or_init(|| {
            let inner = self.get_inner();

            let playlist_data = CONTENT_POOL
                .get_playlist_user_data(inner.id)
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
}

impl PartialEq for DynamicPlaylist {
    fn eq(&self, other: &Self) -> bool {
        self.inner_playlist == other.inner_playlist
    }
}

impl Eq for DynamicPlaylist {}
