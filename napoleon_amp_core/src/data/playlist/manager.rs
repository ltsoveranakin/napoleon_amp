use crate::data::playlist::{Queue, DEAD_MUSIC_THREAD_MESSAGE};
use crate::data::song::Song;
use crate::data::NamedPathLike;
use crate::{read_rwlock, write_rwlock, ReadWrapper};
use rodio::source::SeekError;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::fs::File;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub(super) enum MusicCommand {
    Play,
    Pause,
    Stop,
    Previous,
    Next,
    SetVolume(f32),
}

#[derive(Clone, Debug)]
pub struct SongStatus {
    pub(super) song: Song,
    pub(super) total_duration: Option<Duration>,
}

impl SongStatus {
    pub fn song(&self) -> &Song {
        &self.song
    }

    pub fn total_duration(&self) -> Option<Duration> {
        self.total_duration
    }
}

pub struct MusicManager {
    pub(super) playing_handle: JoinHandle<()>,
    pub(super) music_command_tx: Sender<MusicCommand>,
    pub(super) sink: Arc<Sink>,
    queue: Arc<RwLock<Queue>>,
    song_status: Arc<RwLock<SongStatus>>,
    /// Not currently used, but must not be dropped in order to keep audio stream alive
    pub(super) _output_stream: OutputStream,
}

impl MusicManager {
    pub(super) fn try_create(
        songs: Arc<RwLock<Vec<Song>>>,
        queue: Queue,
        volume: f32,
    ) -> Option<Self> {
        if queue.indexes.is_empty() {
            return None;
        }

        let (music_command_tx, music_command_rx) = mpsc::channel();

        let song_status = Arc::new(RwLock::new(SongStatus {
            song: read_rwlock(&songs)[queue.indexes[0]].clone(),
            total_duration: None,
        }));
        let song_status_thread = Arc::clone(&song_status);

        let output_stream = OutputStreamBuilder::open_default_stream()
            .expect("Unable to open audio stream to default audio device");

        let sink = Arc::new(Sink::connect_new(&output_stream.mixer()));
        let sink_thread = Arc::clone(&sink);

        let songs_len = queue.indexes.len() as i32;

        let queue = Arc::new(RwLock::new(queue));
        let queue_thread = Arc::clone(&queue);

        let playing_handle = thread::Builder::new()
            .name("Music Manager".to_string())
            .spawn(move || {
                let sink = sink_thread;
                let queue = queue_thread;
                let song_status = song_status_thread;

                let new_index = |index: usize, delta: i32| -> usize {
                    ((index as i32) + delta).rem_euclid(songs_len) as usize
                };

                sink.set_volume(volume);

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
                                write_rwlock(&queue).previous();
                            }

                            MusicCommand::Next => {
                                sink.clear();
                                write_rwlock(&queue).next();
                            }

                            MusicCommand::SetVolume(volume) => {
                                sink.set_volume(volume);
                            }
                        }
                    }

                    if sink.empty() {
                        println!("sink empty, next song");
                        let songs = read_rwlock(&songs);
                        let song_index = read_rwlock(&queue).get_current();
                        let song = songs.get(song_index).expect("Invalid song index given");

                        let file = File::open(song.path())
                            .expect(&format!("Unable to open song file for: {:?}", song.path()));

                        // Skip if invalid file
                        if let Ok(source) = Decoder::try_from(file) {
                            let mut song_status = write_rwlock(&song_status);

                            song_status.song = song.clone();

                            song_status.total_duration =
                                if let Some(total_duration) = source.total_duration() {
                                    Some(total_duration)
                                } else {
                                    None
                                };

                            sink.append(source);

                            sink.play();
                        } else {
                            println!("Invalid or corrupted audio file detected, skipping")
                        }

                        write_rwlock(&queue).next();
                    }

                    thread::sleep(Duration::from_millis(16))
                }
            })
            .expect("Unable to spawn thread at OS level");

        // let playing_handle = OnceCell::new();
        // playing_handle.set(handle).unwrap();

        Some(Self {
            playing_handle,
            music_command_tx,
            sink,
            queue,
            song_status,
            _output_stream: output_stream,
        })
    }

    pub fn set_volume(&self, volume: f32) {
        self.send_command(MusicCommand::SetVolume(volume));
    }

    /// Gets the current playhead position in the song.
    ///
    /// Returns None if there is no music active

    pub fn get_song_pos(&self) -> Duration {
        self.sink.get_pos()
    }

    pub fn try_seek(&self, pos: Duration) -> Result<(), SeekError> {
        self.sink.try_seek(pos)
    }

    /// Returns the current song status, this can deadlock if the song changes at the same time, instead use [`Self::get_song_status_cloned`] if cloning is acceptable

    pub fn get_song_status_ref(&self) -> ReadWrapper<SongStatus> {
        read_rwlock(&self.song_status)
    }

    pub fn get_song_status_cloned(&self) -> SongStatus {
        self.get_song_status_ref().clone()
    }

    pub fn is_playing(&self) -> bool {
        !self.sink.is_paused()
    }

    pub fn toggle_playback(&self) {
        if self.is_playing() {
            self.pause();
        } else {
            self.play()
        }
    }

    pub fn play(&self) {
        self.send_command(MusicCommand::Play)
    }

    pub fn pause(&self) {
        self.send_command(MusicCommand::Pause)
    }

    pub fn previous(&self) {
        self.send_command(MusicCommand::Previous)
    }

    pub fn next(&self) {
        self.send_command(MusicCommand::Next)
    }

    pub(super) fn send_stop_command(&self) {
        self.send_command(MusicCommand::Stop);
    }

    /// Attempts to send a MusicCommand. Returns true if it was able to send a command, false otherwise.
    ///
    /// It will succeed so long as there is music currently active (returns true)

    fn send_command(&self, music_command: MusicCommand) {
        self.music_command_tx
            .send(music_command)
            .expect(DEAD_MUSIC_THREAD_MESSAGE);
    }
}
