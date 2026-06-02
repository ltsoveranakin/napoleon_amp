use crate::content::playlist::playlists::ALL_SONGS_PLAYLIST_ID;
use crate::content::playlist::playlists::dynamic_playlist::filter::FilterRules;
use serbytes::prelude::SerBytes;
use simple_id::prelude::Id;
use std::borrow::Cow;

#[derive(SerBytes, Debug, Clone)]
pub enum ImportFrom {
    AllSongs,
    PlaylistIds(Vec<Id>),
}

#[derive(SerBytes, Debug, Clone)]
pub struct Rules {
    pub import_from: ImportFrom,
    pub filters: Vec<FilterRules>,
}

impl Rules {
    pub(super) fn new() -> Self {
        Self {
            import_from: ImportFrom::AllSongs,
            filters: Vec::new(),
        }
    }

    pub(super) fn get_playlist_ids(&self) -> Cow<'_, Vec<Id>> {
        let ids = match &self.import_from {
            ImportFrom::AllSongs => Cow::Owned(vec![ALL_SONGS_PLAYLIST_ID]),

            ImportFrom::PlaylistIds(playlist_ids) => Cow::Borrowed(playlist_ids),
        };

        ids
    }
}
