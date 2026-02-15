use crate::content::playlist::queue::Queue;
use crate::content::playlist::PlaybackMode;
use crate::content::song::Song;
use crate::content::NamedPathLike;
use crate::discord_rpc::{send_rpc_action, RPCAction, SetSongData};
use crate::{read_rwlock, write_rwlock, ReadWrapper};
use rodio::cpal::traits::HostTrait;
use rodio::source::SeekError;
use rodio::{cpal, Decoder, DeviceTrait, OutputStream, OutputStreamBuilder, Sink, Source};
use std::any::Any;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{BufReader, ErrorKind};
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{io, thread};

static DEAD_MUSIC_THREAD_MESSAGE: &'static str =
    "Music thread should be dead, and this should be cleaned up";

pub(super) enum SwitchSongMusicCommand {
    Previous,
    Next,
    SkipToQueueIndex(usize),
}

pub(super) enum MusicCommand {
    Play,
    Pause,
    Stop { send_stop_action: bool },
    SwitchSong(SwitchSongMusicCommand),
    SetVolume(f32),
}

#[derive(Clone, Debug)]
pub struct SongStatus {
    pub(super) song: Arc<Song>,
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

pub(super) struct DebugWrapper<T>(T);

impl<T: 'static> Debug for DebugWrapper<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "No debug information for type: {:?}",
            self.type_id()
        ))
    }
}

impl<T> Deref for DebugWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for DebugWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub struct MusicManager {
    pub(super) playing_handle: JoinHandle<()>,
    pub(super) music_command_tx: Sender<MusicCommand>,
    pub(super) sink: DebugWrapper<Arc<RwLock<Sink>>>,
    pub(super) queue: Arc<RwLock<Queue>>,
    song_status: Arc<RwLock<SongStatus>>,
}

impl MusicManager {
    pub(super) fn try_create(
        songs_arc: Arc<RwLock<Vec<Arc<Song>>>>,
        start_index: usize,
        volume: f32,
        playback_mode: PlaybackMode,
    ) -> Option<Self> {
        // TODO: return result instead of option
        let songs = read_rwlock(&songs_arc);

        if songs.is_empty() {
            return None;
        }

        let queue = Queue::new(start_index, &songs, playback_mode);

        let (music_command_tx, music_command_rx) = mpsc::channel();

        let song_status = Arc::new(RwLock::new(SongStatus {
            song: Arc::clone(&songs[queue.indexes[0]]),
            total_duration: None,
        }));
        let song_status_thread = Arc::clone(&song_status);

        let (sink, output_stream) = create_sink(volume);

        let stream = RwLock::new(output_stream);

        let sink = Arc::new(RwLock::new(sink));
        let sink_thread = Arc::clone(&sink);

        let queue = Arc::new(RwLock::new(queue));
        let queue_thread = Arc::clone(&queue);

        let songs_thread = Arc::clone(&songs_arc);

        let playing_handle = thread::Builder::new()
            .name("Music Manager".to_string())
            .spawn(move || {
                let sink_arc = sink_thread;
                let queue = queue_thread;
                let song_status = song_status_thread;
                let songs = songs_thread;

                let mut audio_device_in_use = cpal::default_host().default_output_device();
                let mut change_audio_device = false;

                let mut is_playing = true;

                let mut should_send_stop_action = true;

                loop {
                    if change_audio_device {
                        println!("Changing audio device, replacing sink");

                        audio_device_in_use = cpal::default_host().default_output_device();

                        let mut sink = write_rwlock(&sink_arc);
                        let song_pos = sink.get_pos();

                        let (new_sink, new_stream) = create_sink(volume);

                        **sink = new_sink;

                        **write_rwlock(&stream) = new_stream;

                        if let Ok(source) = get_decoder_for_song(&read_rwlock(&song_status).song) {
                            sink.append(source);
                            sink.try_seek(song_pos).ok();
                        }

                        change_audio_device = false;
                    }

                    let sink = read_rwlock(&sink_arc);

                    if let Ok(music_command) = music_command_rx.try_recv() {
                        match music_command {
                            MusicCommand::Stop { send_stop_action } => {
                                should_send_stop_action = send_stop_action;
                                sink.stop();
                                break;
                            }

                            MusicCommand::Pause => {
                                is_playing = false;
                                sink.pause();
                                send_rpc_action(RPCAction::StopMusic);
                            }

                            MusicCommand::Play => {
                                is_playing = true;
                                sink.play();
                                send_rpc_action(RPCAction::Resume);
                            }

                            MusicCommand::SwitchSong(switch_song_command) => {
                                let mut queue = write_rwlock(&queue);

                                match switch_song_command {
                                    SwitchSongMusicCommand::Previous => {
                                        if sink.get_pos().as_secs() > 3 {
                                            queue.restart_song();
                                        } else {
                                            queue.previous();
                                        }
                                    }

                                    SwitchSongMusicCommand::Next => {
                                        // Queue has already incremented
                                    }

                                    SwitchSongMusicCommand::SkipToQueueIndex(index) => {
                                        queue.set_index_from_queue(index);
                                    }
                                }

                                sink.clear();
                            }

                            MusicCommand::SetVolume(volume) => {
                                sink.set_volume(volume);
                            }
                        }
                    }

                    if sink.empty() {
                        let songs = read_rwlock(&songs);
                        let song_index = read_rwlock(&queue).get_next_song_index();

                        let song = if let Some(song) = songs.get(song_index) {
                            song
                        } else {
                            write_rwlock(&queue).reset_queue();
                            continue;
                        };

                        // Skip if invalid file
                        if let Ok(source) = get_decoder_for_song(song) {
                            let mut song_status = write_rwlock(&song_status);

                            song_status.song = song.clone();

                            let total_song_duration =
                                if let Some(total_duration) = source.total_duration() {
                                    Some(total_duration)
                                } else {
                                    None
                                };

                            song_status.total_duration = total_song_duration;

                            let song_data = song.get_or_load_song_data();

                            send_rpc_action(RPCAction::SetSong(SetSongData {
                                song_title: song_data.title.clone(),
                                song_artist: song_data.artist.to_string(),
                                song_duration: total_song_duration,
                            }));

                            sink.append(source);

                            sink.play();
                        } else {
                            println!("Invalid or corrupted audio file detected, skipping")
                        }

                        write_rwlock(&queue).next();
                    }

                    if is_playing {
                        println!("poll");
                        let just_polled_device = cpal::default_host().default_output_device();

                        if let Some(just_polled_device) = just_polled_device {
                            let audio_device_in_use = if let Some(ad) = &audio_device_in_use {
                                ad
                            } else {
                                change_audio_device = true;
                                continue;
                            };

                            if just_polled_device.name() != audio_device_in_use.name() {
                                change_audio_device = true;
                            }
                        }
                    }

                    thread::sleep(Duration::from_millis(100))
                }

                // End of music thread... cleanup
                if should_send_stop_action {
                    send_rpc_action(RPCAction::StopMusic);
                }
            })
            .expect("Unable to spawn thread at OS level");

        Some(Self {
            playing_handle,
            music_command_tx,
            sink: DebugWrapper(sink),
            queue,
            song_status,
        })
    }

    pub fn queue(&self) -> ReadWrapper<'_, Queue> {
        read_rwlock(&self.queue)
    }

    pub(super) fn set_volume(&self, volume: f32) {
        self.send_command(MusicCommand::SetVolume(volume));
    }

    /// Gets the current playhead position in the song.

    pub fn get_song_pos(&self) -> Duration {
        self.get_sink().get_pos()
    }

    pub fn try_seek(&self, pos: Duration) -> Result<(), SeekError> {
        self.get_sink().try_seek(pos)
    }

    pub fn get_song_status(&self) -> SongStatus {
        read_rwlock(&self.song_status).clone()
    }

    pub fn is_playing(&self) -> bool {
        !self.get_sink().is_paused()
    }

    pub fn toggle_playback(&self) {
        if self.is_playing() {
            self.pause();
        } else {
            self.play();
        }
    }

    pub fn play(&self) {
        self.send_command(MusicCommand::Play);
    }

    pub fn pause(&self) {
        self.send_command(MusicCommand::Pause);
    }

    pub fn previous(&self) {
        self.switch_song_command(SwitchSongMusicCommand::Previous);
    }

    pub fn next(&self) {
        self.switch_song_command(SwitchSongMusicCommand::Next);
    }

    pub fn set_queue_index(&self, index: usize) {
        self.switch_song_command(SwitchSongMusicCommand::SkipToQueueIndex(index));
    }

    pub(super) fn send_stop_command(&self, send_stop_action: bool) {
        self.send_command(MusicCommand::Stop { send_stop_action });
    }

    /// Attempts to send a MusicCommand.
    ///
    /// It will succeed so long as there is music currently active (returns true)

    fn send_command(&self, music_command: MusicCommand) {
        self.music_command_tx
            .send(music_command)
            .expect(DEAD_MUSIC_THREAD_MESSAGE);
    }

    fn switch_song_command(&self, switch_song_music_command: SwitchSongMusicCommand) {
        self.send_command(MusicCommand::SwitchSong(switch_song_music_command));
    }

    fn get_sink(&self) -> ReadWrapper<'_, Sink> {
        read_rwlock(&self.sink)
    }
}

fn create_sink(volume: f32) -> (Sink, OutputStream) {
    let output_stream = OutputStreamBuilder::open_default_stream()
        .expect("Unable to open audio stream to default audio device");

    let sink = Sink::connect_new(&output_stream.mixer());

    sink.set_volume(volume);

    (sink, output_stream)
}

fn get_decoder_for_song(song: &Song) -> io::Result<Decoder<BufReader<File>>> {
    let file =
        File::open(song.path()).expect(&format!("Unable to open song file for: {:?}", song.path()));

    Decoder::try_from(file).map_err(|_| ErrorKind::InvalidData.into())
}
