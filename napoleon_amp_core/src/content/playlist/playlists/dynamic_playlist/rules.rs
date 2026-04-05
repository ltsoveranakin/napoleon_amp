use crate::content::folder::content_pool::CONTENT_POOL;
use crate::content::playlist::playlists::ALL_SONGS_PLAYLIST_ID;
use crate::content::playlist::playlists::dynamic_playlist::dynamic_playlist_data::ImportFrom;
use crate::content::playlist::playlists::dynamic_playlist::filter::FilterRules;
use crate::content::song::Song;
use crate::song_pool::SONG_POOL;
use serbytes::prelude::{FromFileResult, SerBytes};
use simple_id::prelude::Id;
use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(SerBytes, Debug)]
pub struct Rules {
    import_from: ImportFrom,
    filters: Vec<FilterRules>,
}

impl Rules {
    pub(super) fn new() -> Self {
        Self {
            import_from: ImportFrom::AllSongs,
            filters: Vec::new(),
        }
    }

    fn get_song_list(&self, self_last_updated: u64) -> FromFileResult<'_, Vec<Arc<Song>>> {
        let mut song_ids_checked = HashSet::new();
        let mut songs = Vec::new();

        let playlist_ids = self.get_playlist_ids();

        for playlist_id in playlist_ids.iter().copied() {
            let song_list_data = CONTENT_POOL.get_playlist_song_list_data(playlist_id)?;
            if song_list_data.last_updated.inner < self_last_updated {
                continue;
            }

            for song_id in song_list_data.song_ids {
                if song_ids_checked.insert(song_id) {
                    let song = SONG_POOL.get_song_by_id(song_id);

                    let mut failed = false;

                    for filter in &self.filters {
                        if !filter.does_song_pass(&song) {
                            failed = true;
                            break;
                        }
                    }

                    if !failed {
                        songs.push(song);
                    }
                }
            }
        }

        Ok(songs)
    }

    fn get_playlist_ids(&self) -> Cow<'_, Vec<Id>> {
        let ids = match &self.import_from {
            ImportFrom::AllSongs => Cow::Owned(vec![ALL_SONGS_PLAYLIST_ID]),

            ImportFrom::Playlists(playlist_ids) => Cow::Borrowed(playlist_ids),
        };

        ids
    }
}
