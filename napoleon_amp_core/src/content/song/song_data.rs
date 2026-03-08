use crate::content::song::{Song, UNKNOWN_ALBUM_STR, UNKNOWN_ARTIST_STR};
use serbytes::prelude::SerBytes;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::{MetadataOptions, StandardTagKey, Value};
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;

pub const MAX_RATING: u8 = 5;

#[derive(SerBytes, Clone, Debug)]
pub struct Artist {
    /// The full artist string which includes all artists that contributed to the song, separated by slashes (/)
    pub full_artist_string: String,
}

impl Artist {
    fn new(artist_string: impl Into<String>) -> Self {
        Self {
            full_artist_string: artist_string.into(),
        }
    }

    pub fn main_artist(&self) -> &str {
        self.full_artist_string
            .split("/")
            .next()
            .unwrap_or(UNKNOWN_ARTIST_STR)
    }
}

#[derive(SerBytes, Clone, Debug)]
pub struct SongData {
    pub artist: Artist,
    pub album: String,
    pub title: String,
    pub custom_tags: Vec<String>,
    pub(crate) audio_file: String,
    /// A rating of the song from 0 to 5
    /// where 0 represents unrated and 1-5 represent a rating
    pub rating: u8,
}

impl Default for SongData {
    fn default() -> Self {
        Self {
            artist: Artist::new(UNKNOWN_ARTIST_STR),
            album: UNKNOWN_ALBUM_STR.to_string(),
            title: String::new(),
            custom_tags: Vec::new(),
            audio_file: String::new().into(),
            rating: 0,
        }
    }
}

pub(crate) fn get_song_data_from_song_file(song: &Song, song_data: &mut SongData) {
    get_song_data_from_song_file_with_paths(&song.song_audio_path, &song.song_data_path, song_data);
}

pub(super) fn get_song_data_from_song_file_with_paths(
    song_audio_path: &PathBuf,
    song_data_path: &PathBuf,
    song_data: &mut SongData,
) {
    let song_file = File::open(&song_audio_path).expect("Open new song file");

    let ext = song_audio_path
        .extension()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let mss_options = MediaSourceStreamOptions::default();

    let mss = MediaSourceStream::new(Box::new(song_file), mss_options);

    match get_probe().format(
        Hint::new().with_extension(&ext),
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    ) {
        Ok(mut probe_result) => {
            if let Some(meta) = probe_result.metadata.get() {
                if let Some(meta_revision) = meta.current() {
                    for tag in meta_revision.tags() {
                        if let Some(std_key) = tag.std_key {
                            match std_key {
                                StandardTagKey::Artist => match tag.value {
                                    Value::String(ref artist_string) => {
                                        song_data.artist = Artist::new(artist_string);
                                    }

                                    _ => {
                                        unreachable!()
                                    }
                                },

                                StandardTagKey::Album => match tag.value {
                                    Value::String(ref album) => {
                                        song_data.album = album.clone();
                                    }

                                    _ => {
                                        unreachable!()
                                    }
                                },

                                StandardTagKey::TrackTitle => match tag.value {
                                    Value::String(ref title) => {
                                        song_data.title = title.clone();
                                    }

                                    _ => {
                                        unreachable!()
                                    }
                                },

                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        Err(error) => {
            println!(
                "failed getting format for {:?}; error: {}",
                song_audio_path, error
            );
        }
    }

    // TODO: why is this here?
    fs::write(song_data_path, song_data.to_bb().buf()).expect("Clean write to song data file");
}
