use crate::data::song::Song;
use crate::data::{unwrap_inner_ref, unwrap_inner_ref_mut, NamedPathLike, PathNamed};
use crate::paths::song_file;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use serbytes::prelude::SerBytes;
use std::cell::{Ref, RefCell};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

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

struct Playback {
    stream_handle: OutputStream,
    sink: Sink,
}

pub struct Playlist {
    path_named: PathNamed,
    songs: RefCell<Option<Vec<Song>>>,
    playback: RefCell<Option<Playback>>,
}

impl Playlist {
    pub(super) fn new(path_named: PathNamed) -> Self {
        Self {
            path_named,
            songs: RefCell::new(None),
            playback: RefCell::new(None),
        }
    }

    pub fn get_or_load_songs(&self) -> Ref<Vec<Song>> {
        let songs = if self.songs.borrow().is_some() {
            unwrap_inner_ref(self.songs.borrow())
        } else {
            let playlist_data = if let Ok(file_buf_str) = fs::read_to_string(&self.path_named.path)
            {
                PlaylistData::from_bytes(file_buf_str.as_bytes())
                    .unwrap_or(PlaylistData::new_empty())
            } else {
                PlaylistData::new_empty()
            };

            self.songs.replace(Some(
                playlist_data
                    .songs
                    .iter()
                    .map(|song_name| {
                        Song::new(PathNamed::new(song_file(song_name)).expect("Unhandled TODO:"))
                    })
                    .collect(),
            ));

            unwrap_inner_ref(self.songs.borrow())
        };

        songs
    }

    pub fn import_songs(&self, song_paths: &[PathBuf], delete_original: bool) {
        for original_song_path in song_paths {
            let file_name = original_song_path
                .file_name()
                .expect("Path terminates in ..");
            let new_song_path = song_file(file_name);

            if fs::exists(&new_song_path).expect(&format!(
                "Unable to verify new song path does not exist at path: {:?}",
                new_song_path
            )) {
                panic!("Song path already exists at this location");
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

        self.save_contents();
    }

    pub fn set_playing_song(&self, song: &Song) {
        let playback = if self.playback.borrow().is_some() {
            // let playback = unwrap_inner_ref_mut(self.playback.borrow_mut());

            let playback_ref = unwrap_inner_ref(self.playback.borrow());

            playback_ref.sink.stop();

            playback_ref
        } else {
            let stream_handle = OutputStreamBuilder::open_default_stream()
                .expect("Unable to open audio stream to default audio device");
            let sink = Sink::connect_new(&stream_handle.mixer());

            self.playback.replace(Some(Playback {
                stream_handle,
                sink,
            }));

            unwrap_inner_ref(self.playback.borrow())
        };

        let file = File::open(song.path())
            .expect(&format!("Unable to open song file for: {:?}", song.path()));

        let source = Decoder::try_from(file).expect("Unable to create decoder from file");

        playback.sink.append(source);
    }

    fn add_song(&self, song: Song) {
        if self.songs.borrow().is_none() {
            self.get_or_load_songs();
        }

        let mut songs = unwrap_inner_ref_mut(self.songs.borrow_mut());

        songs.push(song);
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
