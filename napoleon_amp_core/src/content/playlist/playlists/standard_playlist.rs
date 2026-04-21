use crate::content::folder::Folder;
use crate::content::playlist::data::PlaylistUserData;
use crate::content::playlist::playlists::get_user_data_ref_cell;
use crate::content::playlist::{InnerPlaylist, Playlist};
use simple_id::prelude::Id;
use std::cell::{OnceCell, Ref, RefCell, RefMut};
use std::ops::Deref;
use std::rc::Rc;

/// The type of playlist this will attempt to load songs from

#[derive(Debug)]
pub enum StandardPlaylistVariant {
    /// Will attempt to load all songs that have been registered
    AllSongs,
    /// Will attempt to load all songs in the playlist data file that matches the current id
    Normal,
}

#[derive(Debug)]
pub struct StandardPlaylist {
    inner_playlist: InnerPlaylist,
    playlist_user_data: OnceCell<RefCell<PlaylistUserData>>,
}

impl Deref for StandardPlaylist {
    type Target = InnerPlaylist;

    fn deref(&self) -> &Self::Target {
        &self.inner_playlist
    }
}

impl StandardPlaylist {
    pub(crate) fn new(id: Id, parent: &Rc<Folder>) -> Self {
        Self {
            inner_playlist: InnerPlaylist::new(id, parent),
            playlist_user_data: OnceCell::new(),
        }
    }
}

impl Playlist for StandardPlaylist {
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

impl PartialEq for StandardPlaylist {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for StandardPlaylist {}
