use crate::content::song::UNKNOWN_ALBUM_STR;
use crate::content::song::song_data::Artist;
use serbytes::prelude::{BBReadResult, MayNotExistOrDefault, ReadError, SerBytes, SizedBlock};

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
    pub times_skipped: MayNotExistOrDefault<u32>,
}

impl Default for SongDataStdV4 {
    fn default() -> Self {
        Self {
            title: String::new(),
            custom_tags: Vec::new(),
            rating: 0,
            user_tag: String::new().into(),
            meta: SizedBlock::new(Err(ReadError::default())),
            times_listened: 0,
            times_skipped: 0.into(),
        }
    }
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
            song_length: u32::MAX,
        }
    }
}
