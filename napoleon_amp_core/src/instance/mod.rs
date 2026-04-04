mod data;
mod fixup;

use crate::content::folder::Folder;
use crate::content::playlist::data::PlaybackMode;
use crate::content::playlist::{Playlist, PlaylistType, StandardPlaylist, StandardPlaylistVariant};
use crate::content::song::Song;
use crate::discord_rpc::discord_rpc_thread;
use crate::instance::data::InstanceData;
use crate::read_rwlock;
use rand::{RngExt, rng};
use simple_id::prelude::Id;
use std::cell::LazyCell;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

pub struct NapoleonInstance {
    pub base_folder: Rc<Folder>,
    all_songs: Weak<PlaylistType>,
    copied_songs: Option<Vec<Arc<Song>>>,
    currently_playing_playlist: Option<Rc<PlaylistType>>,
    instance_data: LazyCell<InstanceData>,
    _discord_rpc_thread: Option<JoinHandle<()>>,
}

impl NapoleonInstance {
    pub fn new() -> Self {
        Self {
            // TODO: initialize thru content_pool
            base_folder: Rc::new(Folder::new(Id::ZERO, None)),
            all_songs: Weak::new(),
            copied_songs: None,
            currently_playing_playlist: None,
            instance_data: LazyCell::new(InstanceData::init_self),
            _discord_rpc_thread: Some(thread::spawn(|| {
                if discord_rpc_thread().is_ok() {
                    println!("rpc thread fin ok");
                } else {
                    println!("rpc thread err");
                }
            })),
        }
    }

    pub fn copy_selected_songs(&mut self, playlist: &PlaylistType) {
        let song_vec = playlist.get_song_vec();
        let songs = read_rwlock(&song_vec);
        let selected_songs_variant = playlist.get_selected_songs();

        let selected_songs = selected_songs_variant.get_selected_songs(&*songs).to_vec();

        self.copied_songs = Some(selected_songs);
    }

    pub fn paste_copied_songs(&self, playlist: &PlaylistType) {
        if let Some(ref copied_songs) = self.copied_songs {
            playlist.import_existing_songs(copied_songs);
        }
    }

    pub fn has_copied_songs(&self) -> bool {
        self.copied_songs.is_some()
    }

    pub fn start_play_song(&mut self, playlist: Rc<PlaylistType>, song_index: usize) {
        self.stop_music();
        playlist.start_play_song(song_index);
        self.currently_playing_playlist = Some(playlist);
    }

    pub fn start_play_playlist(&mut self, playlist: Rc<PlaylistType>) {
        let songs_len = read_rwlock(&playlist.get_song_vec()).len();
        if songs_len == 0 {
            return;
        }

        let song_index = match playlist.get_user_data().inner().playback_mode {
            PlaybackMode::Sequential => 0,

            PlaybackMode::Shuffle => rng().random_range(0..songs_len),
        };

        self.start_play_song(playlist, song_index);
    }

    pub fn stop_music(&mut self) {
        if let Some(current_playing_playlist) = self.currently_playing_playlist.take() {
            current_playing_playlist.stop_music();
        }
    }

    pub fn can_queue_song(&self) -> bool {
        self.currently_playing_playlist.is_some()
    }

    pub fn try_queue_song(&self, song: Arc<Song>) -> Result<(), ()> {
        let current_playing_playlist = self.currently_playing_playlist.as_ref().ok_or(())?;
        let manager = current_playing_playlist.get_music_manager();

        manager
            .as_ref()
            .ok_or(())?
            .queue_mut()
            .push_temporary_queue(song);

        Ok(())
    }

    pub fn get_all_songs_playlist(&mut self) -> Rc<PlaylistType> {
        let upgraded_opt = Weak::upgrade(&self.all_songs);

        if let Some(upgraded) = upgraded_opt {
            upgraded
        } else {
            let playlist = Rc::new(PlaylistType::Standard(StandardPlaylist::new(
                Id::ZERO,
                StandardPlaylistVariant::AllSongs,
                &self.base_folder,
            )));

            self.all_songs = Rc::downgrade(&playlist);

            playlist
        }
    }
}
