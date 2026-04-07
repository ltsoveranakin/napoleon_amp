use crate::content::folder::Folder;
use crate::content::folder::content_pool::CONTENT_POOL;
use crate::content::playlist::data::{PlaylistSongListData, PlaylistUserData};
use crate::content::playlist::playlists::get_user_data_ref_cell;
use crate::content::playlist::{InnerPlaylist, Playlist};
use crate::time_now;
use simple_id::prelude::Id;
use std::cell::{OnceCell, Ref, RefCell, RefMut};
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

    fn get_song_list_data_refcell(&self) -> &RefCell<PlaylistSongListData> {
        self.inner_playlist.playlist_song_list_data.get_or_init(|| {
            let song_list_data = CONTENT_POOL
                .get_playlist_song_list_data(Id::ZERO)
                .unwrap_or_else(|_| PlaylistSongListData {
                    song_ids: Vec::new(),
                    last_updated: time_now().as_secs().into(),
                });

            RefCell::new(song_list_data)
        })
    }

    /// Does nothing since "All Songs" playlist doesn't have a specified file location instead it just loads all the registered songs

    fn save_song_list(&self) {}

    /// Does nothing (currently)

    fn delete_song(&self, _: usize)
    where
        Self: Sized,
    {
    }
}

impl PartialEq for AllSongsPlaylist {
    fn eq(&self, other: &Self) -> bool {
        self.inner_playlist.id == other.inner_playlist.id
    }
}

impl Eq for AllSongsPlaylist {}
