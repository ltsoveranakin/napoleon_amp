use crate::content::song::UNKNOWN_ARTIST_STR;
use serbytes::prelude::SerBytes;

#[derive(SerBytes, Clone, Debug)]
pub struct Artist {
    /// The full artist string which includes all artists that contributed to the song, separated by slashes (/)
    pub full_artist_string: String,
}

impl Artist {
    pub(super) fn new(artist_string: impl Into<String>) -> Self {
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
