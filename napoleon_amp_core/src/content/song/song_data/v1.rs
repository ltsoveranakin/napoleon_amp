use crate::content::song::song_data::Artist;
use serbytes::prelude::{BBReadResult, MayNotExistOrDefault, SerBytes};

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
pub struct SongDataMeta {
    pub length: u32,
}

impl Default for SongDataMeta {
    fn default() -> Self {
        Self { length: 0 }
    }
}
