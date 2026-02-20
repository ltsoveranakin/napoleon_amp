mod data;

use crate::content::folder::Folder;
use crate::content::playlist::data::PlaybackMode;
use crate::content::playlist::Playlist;
use crate::content::song::Song;
use crate::content::PathNamed;
use crate::discord_rpc::discord_rpc_thread;
use crate::instance::data::InstanceData;
use crate::paths::folders_dir;
use crate::read_rwlock;
use rand::{rng, RngExt};
use std::cell::LazyCell;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

pub struct NapoleonInstance {
    base_folder: Rc<Folder>,
    copied_songs: Option<Vec<Arc<Song>>>,
    currently_playing_playlist: Option<Rc<Playlist>>,
    instance_data: LazyCell<InstanceData>,
    _discord_rpc_thread: Option<JoinHandle<()>>,
}

impl NapoleonInstance {
    pub fn new() -> Self {
        Self {
            base_folder: Rc::new(Folder::new(PathNamed::new(folders_dir()), None)),
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

    pub fn get_base_folder(&self) -> Rc<Folder> {
        Rc::clone(&self.base_folder)
    }

    pub fn copy_selected_songs(&mut self, playlist: &Playlist) {
        let song_vec = playlist.get_or_load_songs();
        let songs = read_rwlock(&song_vec);
        let selected_songs_variant = playlist.get_selected_songs_variant();

        let selected_songs = selected_songs_variant.get_selected_songs(&*songs).to_vec();

        self.copied_songs = Some(selected_songs);
    }

    pub fn paste_copied_songs(&self, playlist: &Playlist) {
        if let Some(ref copied_songs) = self.copied_songs {
            playlist.import_existing_songs(copied_songs);
        }
    }

    pub fn start_play_song(&mut self, playlist: Rc<Playlist>, song_index: usize) {
        self.stop_music();
        playlist.start_play_song(song_index);
        self.currently_playing_playlist = Some(playlist);
    }

    pub fn start_play_playlist(&mut self, playlist: Rc<Playlist>) {
        let song_index = match playlist.playback_mode() {
            PlaybackMode::Sequential => 0,

            PlaybackMode::Shuffle => {
                let songs_len = read_rwlock(&playlist.get_or_load_songs()).len();
                rng().random_range(0..songs_len)
            }
        };

        self.start_play_song(playlist, song_index);
    }

    pub fn stop_music(&mut self) {
        if let Some(current_playing_playlist) = self.currently_playing_playlist.take() {
            current_playing_playlist.stop_music();
        }
    }
}
