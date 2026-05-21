use crate::content::song::{Song, UNKNOWN_ALBUM_STR, UNKNOWN_ARTIST_STR};
use serbytes::prelude::{
    BBReadResult, CurrentVersion, MayNotExistOrDefault, ReadByteBufferRefMut, ReadError, SerBytes,
    SizedBlock, VersioningWrapper,
};
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::{MetadataOptions, StandardTagKey, Value};
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;

pub const MAX_RATING: u32 = 5;

pub type SongDataStd = SongDataStdV4;
pub type SongData = VersioningWrapper<SongDataStd, SongDataVersion>;

#[derive(SerBytes, Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum SongDataVersion {
    V1,
    V2,
    V3,
    #[default]
    V4,
}

impl CurrentVersion for SongDataVersion {
    type Output = SongDataStd;

    fn get_data_from_buf(&self, buf: &mut ReadByteBufferRefMut) -> BBReadResult<Self::Output> {
        match self {
            Self::V1 => {
                let sd_v1 = SongDataStdV1::from_buf(buf)?;

                let sd_v4 = SongDataStdV4 {
                    title: sd_v1.title,
                    custom_tags: sd_v1.custom_tags,
                    rating: sd_v1.rating,
                    user_tag: sd_v1.user_tag.inner,
                    meta: SizedBlock::new(Ok(SongDataMeta2 {
                        artist: Artist {
                            full_artist_string: sd_v1.artist.full_artist_string,
                        },
                        album: sd_v1.album,
                        song_length: sd_v1.meta.unwrap_or_default().length,
                    })),
                    times_listened: 0,
                };

                Ok(sd_v4)
            }

            Self::V2 => {
                let sd_v2 = SongDataStdV2::from_buf(buf)?;

                let sd_v4 = SongDataStdV4 {
                    title: sd_v2.title,
                    custom_tags: sd_v2.custom_tags,
                    rating: sd_v2.rating,
                    user_tag: sd_v2.user_tag,
                    meta: SizedBlock::new(Ok(SongDataMeta2 {
                        artist: Artist {
                            full_artist_string: sd_v2.artist.full_artist_string,
                        },
                        album: sd_v2.album,
                        song_length: sd_v2.song_length,
                    })),
                    times_listened: 0,
                };

                Ok(sd_v4)
            }

            Self::V3 => {
                let sd_v3 = SongDataStdV3::from_buf(buf)?;

                let sd_v4 = SongDataStdV4 {
                    title: sd_v3.title,
                    custom_tags: sd_v3.custom_tags,
                    rating: sd_v3.rating,
                    user_tag: sd_v3.user_tag,
                    meta: SizedBlock::new(Ok(SongDataMeta2 {
                        artist: Artist {
                            full_artist_string: sd_v3.artist.full_artist_string,
                        },
                        album: sd_v3.album,
                        song_length: sd_v3.song_length,
                    })),
                    times_listened: sd_v3.times_listened,
                };

                Ok(sd_v4)
            }

            Self::V4 => SongDataStdV4::from_buf(buf),
        }
    }

    fn current_version() -> Self {
        Self::default()
    }
}

/// Data stored for each song which has been registered, contains metadata which is commonly used

#[derive(SerBytes, Clone, Debug)]
pub struct SongDataStdV4 {
    pub title: String,
    pub custom_tags: Vec<String>,
    /// A rating of the song from 0 to 5
    /// where 0 represents unrated and 1-5 represent a rating
    pub rating: u8,
    pub user_tag: String,
    /// Metadata related to a song, this is never of type Err, and can be unwrapped with no issue. Use the helper function [`SongDataStdV4::meta`] where appropriate
    pub meta: SizedBlock<BBReadResult<SongDataMeta2>>,
    pub times_listened: u32,
}

impl SongDataStdV4 {
    /// Helper function that unwraps and retrieves the song metadata
    pub fn meta(&self) -> &SongDataMeta2 {
        self.meta.inner.as_ref().unwrap()
    }

    pub fn meta_mut(&mut self) -> &mut SongDataMeta2 {
        self.meta.inner.as_mut().unwrap()
    }
}

/// Any part of song data that can be retrieved with parsing the audio file

#[derive(SerBytes, Clone, Debug)]
pub struct SongDataMeta2 {
    pub artist: Artist,
    pub album: String,
    pub song_length: u32,
}

impl Default for SongDataMeta2 {
    fn default() -> Self {
        Self {
            artist: Artist::default(),
            album: UNKNOWN_ALBUM_STR.to_string(),
            song_length: 0,
        }
    }
}

#[derive(SerBytes, Clone, Debug)]
pub struct SongDataStdV3 {
    pub artist: Artist,
    pub album: String,
    pub title: String,
    pub custom_tags: Vec<String>,
    /// A rating of the song from 0 to 5
    /// where 0 represents unrated and 1-5 represent a rating
    pub rating: u8,
    pub user_tag: String,
    pub song_length: u32,
    pub times_listened: u32,
}

#[derive(SerBytes, Clone, Debug)]
pub struct SongDataStdV2 {
    pub artist: Artist,
    pub album: String,
    pub title: String,
    pub custom_tags: Vec<String>,
    pub(crate) audio_file: String,
    /// A rating of the song from 0 to 5
    /// where 0 represents unrated and 1-5 represent a rating
    pub rating: u8,
    pub user_tag: String,
    pub song_length: u32,
}

#[derive(SerBytes, Clone, Debug)]
pub struct SongDataStdV1 {
    pub artist: Artist,
    pub album: String,
    pub title: String,
    pub custom_tags: Vec<String>,
    pub(crate) audio_file: String,
    /// A rating of the song from 0 to 5
    /// where 0 represents unrated and 1-5 represent a rating
    pub rating: u8,
    pub user_tag: MayNotExistOrDefault<String>,
    pub meta: BBReadResult<SongDataMeta>,
}

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

impl Default for Artist {
    fn default() -> Self {
        Self::new(UNKNOWN_ARTIST_STR)
    }
}

#[derive(SerBytes, Clone, Debug)]
pub struct SongDataMeta {
    pub length: u32,
}

impl Default for SongDataMeta {
    fn default() -> Self {
        Self { length: 0 }
    }
}

impl Default for SongDataStd {
    fn default() -> Self {
        Self {
            title: String::new(),
            custom_tags: Vec::new(),
            rating: 0,
            user_tag: String::new().into(),
            meta: SizedBlock::new(Err(ReadError::default())),
            times_listened: 0,
        }
    }
}

pub(crate) fn get_song_data_from_song_file(song: &Song, song_data: &mut SongDataStd) {
    get_song_data_from_song_file_with_paths(&song.song_audio_path, &song.song_data_path, song_data);
}

pub(super) fn get_song_data_from_song_file_with_paths(
    song_audio_path: &PathBuf,
    song_data_path: &PathBuf,
    song_data: &mut SongDataStd,
) -> bool {
    let song_file = File::open(&song_audio_path).expect("Open new song file");

    let ext = song_audio_path
        .extension()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let mss_options = MediaSourceStreamOptions::default();

    let mss = MediaSourceStream::new(Box::new(song_file), mss_options);

    if song_data.meta.inner.is_err() {
        song_data.meta = SizedBlock::new(Ok(SongDataMeta2::default()));
    }

    let mut did_err = false;

    match get_probe().format(
        Hint::new().with_extension(&ext),
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    ) {
        Ok(mut probe_result) => {
            if let Some(track) = probe_result.format.default_track() {
                let params = &track.codec_params;
                if let (Some(sample_rate), Some(total_frames)) =
                    (params.sample_rate, params.n_frames)
                {
                    let duration_seconds = total_frames as f64 / sample_rate as f64;
                    let duration = Duration::from_secs_f64(duration_seconds);
                    song_data.meta_mut().song_length = duration.as_secs() as u32;
                }
            }

            if let Some(meta) = probe_result.metadata.get() {
                if let Some(meta_revision) = meta.current() {
                    for tag in meta_revision.tags() {
                        if let Some(std_key) = tag.std_key {
                            match std_key {
                                StandardTagKey::Artist => match tag.value {
                                    Value::String(ref artist_string) => {
                                        song_data.meta_mut().artist = Artist::new(artist_string);
                                    }

                                    _ => {
                                        unreachable!()
                                    }
                                },

                                StandardTagKey::Album => match tag.value {
                                    Value::String(ref album) => {
                                        song_data.meta_mut().album = album.clone();
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
                "failed getting format for {:?}; The title of this song is: {}; error: {}",
                song_audio_path, song_data.title, error
            );

            did_err = true;
        }
    }

    fs::write(song_data_path, song_data.to_bb().buf()).expect("Clean write to song data file");

    did_err
}
