use crate::content::song::UNKNOWN_ALBUM_STR;
use crate::content::song::song_cover_pool::SongCoverId;
use crate::content::song::song_data::Artist;
use serbytes::prelude::{BBReadResult, ReadError, ResultBlock, SerBytes};

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
    pub artist: ResultBlock<Artist>,
    pub album: ResultBlock<String>,
    pub song_length: ResultBlock<u32>,
    pub cover: ResultBlock<Option<SongCoverId>>,
}

impl SongDataMetaV2 {
    pub(super) fn default_ok() -> Self {
        Self {
            artist: Artist::default().into(),
            album: UNKNOWN_ALBUM_STR.to_string().into(),
            song_length: 0.into(),
            cover: None.into(),
        }
    }

    pub(crate) fn has_err(&self) -> bool {
        self.artist.inner.is_err()
            || self.album.inner.is_err()
            || self.song_length.inner.is_err()
            || self.cover.inner.is_err()
    }
}

impl Default for SongDataMetaV2 {
    fn default() -> Self {
        Self {
            artist: Err(ReadError::default()).into(),
            album: Err(ReadError::default()).into(),
            song_length: Err(ReadError::default()).into(),
            cover: Err(ReadError::default()).into(),
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
