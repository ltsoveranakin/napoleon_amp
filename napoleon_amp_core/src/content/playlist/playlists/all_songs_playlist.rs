use crate::content::folder::Folder;
use crate::content::playlist::data::PlaylistUserData;
use crate::content::playlist::playlists::get_user_data_ref_cell;
use crate::content::playlist::{InnerPlaylist, Playlist, default_save_user_data};
use simple_id::prelude::Id;
use std::cell::{OnceCell, Ref, RefCell, RefMut};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Debug)]
pub struct AllSongsPlaylist {
    inner_playlist: InnerPlaylist,
    playlist_user_data: OnceCell<RefCell<PlaylistUserData>>,
}

impl AllSongsPlaylist {
    pub fn new(base_folder: &Rc<Folder>) -> Self {
        Self {
            inner_playlist: InnerPlaylist::new(Id::ZERO, base_folder),
            playlist_user_data: OnceCell::new(),
        }
    }
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

    fn save_user_data(&self) -> std::io::Result<()> {
        default_save_user_data(&self.get_user_data(), self.id)
    }

    /// Does nothing since "All Songs" playlist doesn't have a specified file location instead it just loads all the registered songs

    fn save_song_list(&self) {}

    /// Does nothing (currently)

    fn delete_song(&self, _: usize)
    where
        Self: Sized,
    {
        todo!()
    }
}

impl PartialEq for AllSongsPlaylist {
    fn eq(&self, other: &Self) -> bool {
        self.inner_playlist == other.inner_playlist
    }
}

impl Eq for AllSongsPlaylist {}

impl Deref for AllSongsPlaylist {
    type Target = InnerPlaylist;

    fn deref(&self) -> &Self::Target {
        &self.inner_playlist
    }
}
