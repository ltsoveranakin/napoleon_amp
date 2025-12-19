use crate::data::song::Song;
use crate::data::{NamedPathLike, PathNamed};
use crate::paths::song_file;
use crate::unwrap_lock;
use rodio::{Decoder, OutputStreamBuilder, Sink};
use serbytes::prelude::SerBytes;
use std::cell::{Cell, OnceCell, RefCell};
use std::fs::File;
use std::io::{Read, Write};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Mutex, RwLock, RwLockReadGuard};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{fs, thread};

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

struct Queue {
    indexes: Vec<usize>,
    index: usize,
}

enum MusicCommand {
    Play,
    Pause,
    Stop,
    Previous,
    Next,
}

pub struct SongStatus {
    pub song: Song,
}

struct MusicManager {
    playing_handle: OnceCell<JoinHandle<()>>,
    music_command_tx: Sender<MusicCommand>,
    song_status: Arc<Mutex<SongStatus>>,
}

pub struct Playlist {
    path_named: PathNamed,
    songs: RwLock<Vec<Song>>,
    has_loaded_songs: Cell<bool>,
    queue: RefCell<Queue>,
    music_manager: RefCell<Option<MusicManager>>,
    is_playing: Cell<bool>,
}

impl Playlist {
    pub(super) fn new(path_named: PathNamed) -> Self {
        Self {
            path_named,
            songs: RwLock::new(Vec::new()),
            queue: RefCell::new(Queue {
                indexes: Vec::new(),
                index: 0,
            }),
            has_loaded_songs: Cell::new(false),
            music_manager: RefCell::new(None),
            is_playing: Cell::new(false),
        }
    }

    pub fn get_or_load_songs(&self) -> RwLockReadGuard<Vec<Song>> {
        if self.has_loaded_songs.get() {
            self.songs.read().unwrap()
        } else {
            let playlist_data = if let Ok(file_buf_str) = fs::read_to_string(&self.path_named.path)
            {
                PlaylistData::from_bytes(file_buf_str.as_bytes())
                    .unwrap_or(PlaylistData::new_empty())
            } else {
                PlaylistData::new_empty()
            };

            let mut songs = self.songs.write().unwrap();

            songs.reserve_exact(playlist_data.songs.len());

            for song_name in playlist_data.songs {
                songs.push(Song::new(
                    PathNamed::new(song_file(song_name)).expect("Unhandled TODO:"),
                ));
            }

            self.update_queue_indexes(songs.len());
            drop(songs);

            self.has_loaded_songs.set(true);

            self.songs.read().unwrap()
        }
    }

    fn update_queue_indexes(&self, song_count: usize) {
        if self.queue.borrow().indexes.is_empty() {
            let indexes = &mut self.queue.borrow_mut().indexes;
            indexes.clear();

            for i in 0..song_count {
                indexes.push(i);
            }
        }
    }

    pub fn current_song_status(&self) -> Option<Arc<Mutex<SongStatus>>> {
        Some(Arc::clone(
            &self.music_manager.borrow().deref().as_ref()?.song_status,
        ))
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

    pub fn play_song(&self, song_index: usize) {
        self.set_queue(song_index);
        self.is_playing.set(true);

        let (music_command_tx, music_command_rx) = mpsc::channel();

        if let Some(mut music_manager) = self.music_manager.take() {
            let current_handle = music_manager
                .playing_handle
                .take()
                .expect("Playing handle to exist; Only taken from here");

            let old_music_command_tx = music_manager.music_command_tx;

            old_music_command_tx
                .send(MusicCommand::Stop)
                .expect("Current playing thread to be alive");

            current_handle.join().expect("Unwrap for panic in thread");
        }

        let builder = thread::Builder::new();

        // TODO: make so no need to clone songs here

        let songs = self.get_or_load_songs().clone();

        let song = songs[song_index].clone();
        let current_song_status = Arc::new(Mutex::new(SongStatus { song }));
        let song_status = Arc::clone(&current_song_status);

        let handle = builder
            .name("Music Manager".to_string())
            .spawn(move || {
                let mut index = song_index;
                let songs_len = songs.len() as i32;

                let new_index = |index: usize, delta: i32| -> usize {
                    ((index as i32) + delta).rem_euclid(songs_len) as usize
                };

                let stream_handle = OutputStreamBuilder::open_default_stream()
                    .expect("Unable to open audio stream to default audio device");
                let sink = Sink::connect_new(&stream_handle.mixer());

                loop {
                    if let Ok(music_command) = music_command_rx.try_recv() {
                        match music_command {
                            MusicCommand::Stop => {
                                sink.stop();
                                break;
                            }

                            MusicCommand::Pause => {
                                sink.pause();
                            }

                            MusicCommand::Play => {
                                sink.play();
                            }

                            MusicCommand::Previous => {
                                sink.clear();
                                index = new_index(index, -1);
                            }

                            MusicCommand::Next => {
                                sink.clear();
                                index = new_index(index, 1);
                            }
                        }
                    }

                    if sink.empty() {
                        let song = songs.get(index).expect("Invalid song index given");

                        let file = File::open(song.path())
                            .expect(&format!("Unable to open song file for: {:?}", song.path()));

                        // TODO: handle this:
                        let source =
                            Decoder::try_from(file).expect("Unable to create decoder from file");

                        sink.append(source);

                        unwrap_lock(&current_song_status).song = song.clone();

                        sink.play();

                        index = new_index(index, 1);
                    }

                    thread::sleep(Duration::from_millis(16))
                }
            })
            .expect("Unable to spawn thread at OS level");

        let playing_handle = OnceCell::new();
        playing_handle.set(handle).expect("Value is uninitialized");

        let music_manager = MusicManager {
            playing_handle,
            music_command_tx,
            song_status,
        };

        self.music_manager.replace(Some(music_manager));
    }

    pub fn play(&self) {
        if self.try_send_command(MusicCommand::Play) {
            self.is_playing.set(true);
        }
    }

    pub fn pause(&self) {
        if self.try_send_command(MusicCommand::Pause) {
            self.is_playing.set(false);
        }
    }

    pub fn previous(&self) {
        if self.try_send_command(MusicCommand::Previous) {
            self.is_playing.set(true);
        }
    }

    pub fn next(&self) {
        if self.try_send_command(MusicCommand::Next) {
            self.is_playing.set(true);
        }
    }

    pub fn toggle_playback(&self) {
        if self.is_playing() {
            self.pause();
        } else {
            self.play()
        }
    }

    pub fn stop(&self) {
        if self.try_send_command(MusicCommand::Stop) {
            self.is_playing.set(false);
            self.music_manager.replace(None);
        }
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.get()
    }

    /// Attempts to send a MusicCommand. Returns true if it was able to send a command, false otherwise.
    ///
    /// It will succeed so long as there is music currently active (returns true)

    fn try_send_command(&self, music_command: MusicCommand) -> bool {
        let music_manager_ref = self.music_manager.borrow();

        if let Some(music_manager) = music_manager_ref.deref().as_ref() {
            music_manager
                .music_command_tx
                .send(music_command)
                .expect(DEAD_MUSIC_THREAD_MESSAGE);

            true
        } else {
            false
        }
    }

    fn set_queue(&self, start_index: usize) {
        let mut queue = self.queue.borrow_mut();

        queue.index = start_index;
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
