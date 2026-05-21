use crate::content::song::song_data::Artist;
use serbytes::prelude::SerBytes;

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
