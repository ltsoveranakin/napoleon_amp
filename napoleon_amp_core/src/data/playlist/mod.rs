pub mod manager;

use crate::data::playlist::manager::{MusicCommand, MusicManager};
use crate::data::song::Song;
use crate::data::{NamedPathLike, PathNamed};
use crate::paths::song_file;
use crate::{read_rwlock, write_rwlock, ReadWrapper};
use rodio::Source;
use serbytes::prelude::SerBytes;
use std::cell::{Cell, Ref, RefCell};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

const DEAD_MUSIC_THREAD_MESSAGE: &'static str =
    "Music thread should be dead, and this should be cleaned up";

#[derive(SerBytes)]
struct PlaylistData {
    songs: Vec<String>,
}

impl PlaylistData {
    fn new_empty() -> Self {
        Self::new_capacity(0)
    }

    fn new_capacity(cap: usize) -> Self {
        Self {
            songs: Vec::with_capacity(cap),
        }
    }
}

#[derive(Clone, Debug)]
struct Queue {
    indexes: Vec<usize>,
    index: usize,
}

impl Queue {
    fn set_starting_index(&mut self, index: usize) {
        self.index = index;
    }

    fn next(&mut self) -> usize {
        self.index += 1;
        self.index
    }

    fn previous(&mut self) -> usize {
        self.index -= 1;
        self.index
    }

    fn get_current(&self) -> usize {
        self.index
    }
}

pub struct Playlist {
    path_named: PathNamed,
    songs: Arc<RwLock<Vec<Song>>>,
    has_loaded_songs: Cell<bool>,
    queue: RefCell<Queue>,
    music_manager: RefCell<Option<MusicManager>>,
}

impl Playlist {
    pub(super) fn new(path_named: PathNamed) -> Self {
        Self {
            path_named,
            songs: Arc::new(RwLock::new(Vec::new())),
            queue: RefCell::new(Queue {
                indexes: Vec::new(),
                index: 0,
            }),
            has_loaded_songs: Cell::new(false),
            music_manager: RefCell::new(None),
        }
    }

    pub fn get_or_load_songs(&self) -> ReadWrapper<Vec<Song>> {
        if self.has_loaded_songs.get() {
            self.songs.read().unwrap().into()
        } else {
            let playlist_data = if let Ok(file_buf_str) = fs::read_to_string(&self.path_named.path)
            {
                PlaylistData::from_bytes(file_buf_str.as_bytes())
                    .unwrap_or(PlaylistData::new_empty())
            } else {
                PlaylistData::new_empty()
            };

            let mut songs = write_rwlock(&self.songs);

            songs.reserve_exact(playlist_data.songs.len());

            for song_name in playlist_data.songs {
                songs.push(Song::new(
                    PathNamed::new(song_file(song_name)).expect("Unhandled TODO:"),
                ));
            }

            self.update_queue_indexes(songs.len());
            drop(songs);

            self.has_loaded_songs.set(true);

            read_rwlock(&self.songs)
        }
    }

    fn update_queue_indexes(&self, song_count: usize) {
        let mut queue = self.queue.borrow_mut();

        if queue.indexes.is_empty() {
            let indexes = &mut queue.indexes;
            indexes.clear();

            for i in 0..song_count {
                indexes.push(i);
            }
        }
    }

    pub fn import_songs(
        &self,
        song_paths: &[PathBuf],
        delete_original: bool,
    ) -> Result<(), Vec<usize>> {
        let mut failed_import = Vec::new();
        for (i, original_song_path) in song_paths.iter().enumerate() {
            let file_name = original_song_path
                .file_name()
                .expect("Path terminates in ..");
            let new_song_path = song_file(file_name);

            if fs::exists(&new_song_path).expect(&format!(
                "Unable to verify new song path does not exist at path: {:?}",
                new_song_path
            )) {
                failed_import.push(i);
            }

            File::create(&new_song_path).expect(&format!(
                "Unable to create new song file to copy to; path: {:?}",
                new_song_path
            ));

            fs::copy(original_song_path, &new_song_path).expect("Failed copy song to dest");

            if delete_original {
                fs::remove_file(original_song_path).expect("Failed to remove original file");
            }

            self.add_song(Song::new(PathNamed::new(new_song_path).expect("song_path")));
        }

        let songs = self.get_or_load_songs();

        self.update_queue_indexes(songs.len());
        self.save_contents();

        if !failed_import.is_empty() {
            Err(failed_import)
        } else {
            Ok(())
        }
    }

    pub fn play_song(&self, song_index: usize, volume: f32) {
        if let Some(music_manager) = self.music_manager.take() {
            let current_handle = music_manager.playing_handle;

            let old_music_command_tx = music_manager.music_command_tx;

            old_music_command_tx
                .send(MusicCommand::Stop)
                .expect("Current playing thread to be alive");

            current_handle.join().expect("Unwrap for panic in thread");
        }

        let mut queue = self.queue.borrow_mut();

        queue.set_starting_index(song_index);

        let music_manager =
            MusicManager::try_create(Arc::clone(&self.songs), queue.clone(), volume);

        self.music_manager.replace(music_manager);
    }

    pub fn get_music_manager(&self) -> Ref<'_, Option<MusicManager>> {
        self.music_manager.borrow()
    }

    pub fn stop_music(&self) {
        if let Some(music_manager) = self.music_manager.take() {
            music_manager.send_stop_command();
        }
    }

    fn add_song(&self, song: Song) {
        if self.has_loaded_songs.get() {
            let songs = self.get_or_load_songs();
            drop(songs);
        }

        self.songs.write().expect("Lock poisoned").push(song);
    }

    fn save_contents(&self) {
        let mut file = File::options()
            .write(true)
            .open(&self.path_named)
            .expect("Failed to open file in write mode");

        let songs = self.get_or_load_songs();

        let mut playlist_data = PlaylistData::new_capacity(songs.len());

        for song in songs.iter() {
            playlist_data.songs.push(song.path_string())
        }

        file.write_all(playlist_data.to_bb().buf())
            .expect("Unable to write playlist to file");
    }
}

impl NamedPathLike for Playlist {
    fn get_path_named(&self) -> &PathNamed {
        &self.path_named
    }
}
