use crate::content::song::{Song, UNKNOWN_ALBUM_STR, UNKNOWN_ARTIST_STR};
use crate::content::NamedPathLike;
use serbytes::prelude::SerBytes;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::{MetadataOptions, StandardTagKey, Value};
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;

#[derive(SerBytes, Clone, Debug)]
pub enum SongTagValue {
    String(String),
}

#[derive(SerBytes, Eq, PartialEq, Hash, Clone, Debug)]
pub enum TagType {
    Dynamic(String),
}

#[derive(SerBytes, Clone, Debug)]
pub struct Artist {
    pub artist_string: String,
}

impl Artist {
    fn new(artist_string: impl Into<String>) -> Self {
        Self {
            artist_string: artist_string.into(),
        }
    }

    /// Returns the artist string as it is, if there are multiple artists they will be separated by a "/"
    ///
    /// Ex. Artist0/Artist1/ArtistN

    // pub fn fully_qualified_artist_string(&self) -> &String {
    //     &self.artist_string
    // }

    // pub(crate) fn set_artist(&mut self, artist_string: impl Into<String>) {
    //     self.artist_string = artist_string.into();
    // }

    pub fn get_artist_list(&self) -> Vec<&str> {
        self.artist_string.split("/").collect()
    }

    pub fn main_artist(&self) -> &str {
        self.artist_string
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
    pub custom_song_tags: HashMap<TagType, SongTagValue>,
}

impl Default for SongData {
    fn default() -> Self {
        Self {
            artist: Artist::new(UNKNOWN_ARTIST_STR),
            album: UNKNOWN_ALBUM_STR.to_string(),
            title: String::new(),
            custom_song_tags: HashMap::new(),
        }
    }
}

pub(crate) fn get_song_data_from_song_file(song: &Song, song_data: &mut SongData) {
    get_song_data_from_song_file_with_paths(song.path(), &song.song_data_path, song_data);
}

pub(super) fn get_song_data_from_song_file_with_paths(
    song_path: &PathBuf,
    song_data_path: &PathBuf,
    song_data: &mut SongData,
) {
    let song_file = File::open(&song_path).expect("Open new song file");

    let ext = song_path.extension().unwrap().to_str().unwrap().to_string();

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
                song_path, error
            );
        }
    }

    fs::write(song_data_path, song_data.to_bb().buf()).expect("Clean write to song data file");
}
