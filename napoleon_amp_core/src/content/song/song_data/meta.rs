use crate::content::song::UNKNOWN_ALBUM_STR;
use crate::content::song::song_data::Artist;
use serbytes::prelude::{BBReadResult, ReadError, SerBytes, U8Vec};
use symphonia::core::meta::Visual;

#[derive(SerBytes, Clone, Debug)]
pub struct SongDataMetaV1 {
    pub length: u32,
}

impl Default for SongDataMetaV1 {
    fn default() -> Self {
        Self { length: 0 }
    }
}

/// Any part of song data that can be retrieved with parsing the audio file

#[derive(SerBytes, Clone, Debug)]
pub struct SongDataMetaV2 {
    pub artist: BBReadResult<Artist>,
    pub album: BBReadResult<String>,
    pub song_length: BBReadResult<u32>,
    pub cover: BBReadResult<Option<ImageData>>,
}

impl SongDataMetaV2 {
    pub(super) fn default_ok() -> Self {
        Self {
            artist: Ok(Artist::default()),
            album: Ok(UNKNOWN_ALBUM_STR.to_string()),
            song_length: Ok(0),
            cover: Ok(None),
        }
    }

    pub(crate) fn has_err(&self) -> bool {
        self.artist.is_err()
            || self.album.is_err()
            || self.song_length.is_err()
            || self.cover.is_err()
    }
}

impl Default for SongDataMetaV2 {
    fn default() -> Self {
        Self {
            artist: Err(ReadError::default()),
            album: Err(ReadError::default()),
            song_length: Err(ReadError::default()),
            cover: Err(ReadError::default()),
        }
    }
}

#[derive(SerBytes, Clone, Debug)]
pub struct ImageData {
    pub media_mime_type: String,
    pub data: U8Vec<u32>,
}

impl ImageData {
    pub(super) fn from_visual(visual: &Visual) -> Self {
        Self {
            media_mime_type: visual.media_type.clone(),
            data: visual.data.to_vec().into(),
        }
    }
}

pub(crate) trait AssignIfError<T> {
    fn assign_if_err(&mut self, value: T) {
        self.assign_if_err_callback(|| value);
    }

    fn assign_if_err_callback<F>(&mut self, f: F)
    where
        F: FnOnce() -> T;
}

impl<T> AssignIfError<T> for BBReadResult<T> {
    fn assign_if_err_callback<F>(&mut self, f: F)
    where
        F: FnOnce() -> T,
    {
        if self.is_err() {
            *self = Ok(f());
        }
    }
}
