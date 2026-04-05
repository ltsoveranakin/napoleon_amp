use crate::content::folder::Folder;
use crate::content::playlist::data::{PlaybackMode, PlaylistUserData};
use crate::content::playlist::playlists::get_user_data_ref_cell;
use crate::content::playlist::song_list::SongList;
use crate::content::playlist::{InnerPlaylist, Playlist, PlaylistParent, SelectedSongsVariant};
use crate::read_rwlock;
use simple_id::prelude::Id;
use std::cell::{Cell, OnceCell, Ref, RefCell, RefMut};
use std::ops::{Deref, RangeInclusive};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

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
    pub(crate) fn new(id: Id, variant: StandardPlaylistVariant, parent: &Rc<Folder>) -> Self {
        Self {
            inner_playlist: InnerPlaylist {
                id,
                parent: PlaylistParent {
                    id: parent.id,
                    parent: Rc::downgrade(parent),
                },
                songs: RefCell::new(SongList::new()),
                has_loaded_songs: Cell::new(false),
                music_manager: RefCell::new(None),
                variant,
                songs_filtered: Arc::new(RwLock::new(Vec::new())),
                selected_songs: RefCell::new(SelectedSongsVariant::None),
                playlist_song_list_data: OnceCell::new(),
                total_length: RefCell::new(None),
                current_search_str: RefCell::new(String::new()),
            },
            playlist_user_data: OnceCell::new(),
        }
    }

    pub(crate) fn new_file(id: Id, parent: &Rc<Folder>) -> Self {
        Self::new(id, StandardPlaylistVariant::Normal, parent)
    }

    /// Sets the selected range of the playlist
    /// Errors under 3 conditions.
    ///
    /// If end is less than start.
    /// If start is greater than or equal to (potentially filtered songs) length.
    /// If end is greater than or equal to (potentially filtered songs) length.

    pub fn select_range(&self, range: RangeInclusive<usize>) -> Result<(), ()> {
        let songs_lock = self.get_song_vec();
        let songs = read_rwlock(&songs_lock);

        let start = *range.start();
        let end = *range.end();
        let song_len = songs.len();

        if end < start || start >= song_len || end >= song_len {
            Err(())
        } else {
            self.set_selected_songs(SelectedSongsVariant::Range(range));
            Ok(())
        }
    }

    pub fn set_playback_mode(&self, playback_mode: PlaybackMode) {
        {
            let mut playlist_data = self.get_user_data_mut();

            playlist_data.inner_mut().playback_mode = playback_mode.into();
        }
        self.save_user_data();
    }

    pub fn playback_mode(&self) -> PlaybackMode {
        self.get_user_data().inner().playback_mode
    }

    pub fn get_volume(&self) -> f32 {
        self.get_user_data().inner().volume
    }

    pub fn get_name(&self) -> Ref<'_, String> {
        Ref::map(self.get_user_data(), |d| &d.inner().content_data.name)
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
