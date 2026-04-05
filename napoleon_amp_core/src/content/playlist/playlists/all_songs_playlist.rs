use crate::content::playlist::data::PlaylistUserData;
use crate::content::playlist::playlists::get_user_data_ref_cell;
use crate::content::playlist::{InnerPlaylist, Playlist};
use std::cell::{OnceCell, Ref, RefCell, RefMut};

pub struct AllSongsPlaylist {
    inner_playlist: InnerPlaylist,
    playlist_user_data: OnceCell<RefCell<PlaylistUserData>>,
}

impl Playlist for AllSongsPlaylist {
    fn get_inner(&self) -> &InnerPlaylist {
        &self.inner_playlist
    }

    fn get_user_data(&self) -> Ref<'_, PlaylistUserData> {
        get_user_data_ref_cell(&self.playlist_user_data, &self.inner_playlist).borrow()
    }

    fn get_user_data_mut(&self) -> RefMut<'_, PlaylistUserData> {
        get_user_data_ref_cell(&self.playlist_user_data, &self.inner_playlist).borrow_mut()
    }
}
