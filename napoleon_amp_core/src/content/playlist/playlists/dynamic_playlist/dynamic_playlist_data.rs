use crate::content::SaveData;
use crate::content::folder::content_pool::CONTENT_POOL;
use crate::content::playlist::PlaylistData;
use crate::content::playlist::data::{PlaylistContentData, PlaylistUserData, PlaylistUserDataStd};
use crate::content::playlist::playlists::dynamic_playlist::rules::Rules;
use crate::content::song::Song;
use crate::paths::content_playlist_user_data_file;
use crate::song_pool::SONG_POOL;
use crate::time_now;
use serbytes::prelude::{
    BBReadResult, CurrentVersion, FromFileResult, ReadByteBufferRefMut, SerBytes, VersioningWrapper,
};
use simple_id::prelude::Id;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

pub type DynamicPlaylistData =
    VersioningWrapper<DynamicPlaylistDataStd, DynamicPlaylistDataVersion>;

impl PlaylistData for DynamicPlaylistData {
    fn new_all_songs() -> Self {
        DynamicPlaylistDataStd::new(PlaylistContentData::new_all_songs()).into()
    }

    fn new_deleted_with_data(content_data: PlaylistContentData) -> Self {
        DynamicPlaylistDataStd::new(content_data).into()
    }
}

impl SaveData for DynamicPlaylistData {
    fn get_path(id: Id) -> PathBuf {
        content_playlist_user_data_file(id)
    }
}

#[derive(SerBytes, Debug)]
pub enum DynamicPlaylistDataVersion {
    V1,
}

impl CurrentVersion for DynamicPlaylistDataVersion {
    type Output = DynamicPlaylistDataStd;

    fn get_data_from_buf(&self, buf: &mut ReadByteBufferRefMut) -> BBReadResult<Self::Output> {
        match self {
            Self::V1 => DynamicPlaylistDataStd::from_buf(buf),
        }
    }

    fn current_version() -> Self {
        Self::V1
    }
}

#[derive(SerBytes, Debug, Clone)]
pub struct DynamicPlaylistDataStd {
    pub user_data: PlaylistUserData,
    pub rules: Rules,
    pub(super) last_updated: u64,
}

impl DynamicPlaylistDataStd {
    pub(crate) fn new(content_data: PlaylistContentData) -> Self {
        Self {
            user_data: PlaylistUserDataStd::new(content_data).into(),
            rules: Rules::new(),
            last_updated: time_now().as_secs(),
        }
    }

    pub(super) fn get_song_list(&self) -> FromFileResult<'_, Vec<Arc<Song>>> {
        let mut song_ids_checked = HashSet::new();
        let mut songs = Vec::new();

        let playlist_ids = self.rules.get_playlist_ids();

        for playlist_id in playlist_ids.iter().copied() {
            let song_list_data = CONTENT_POOL.get_playlist_song_list_data(playlist_id)?;
            if song_list_data.last_updated.get() < self.last_updated {
                continue;
            }

            for song_id in song_list_data.song_ids {
                if song_ids_checked.insert(song_id) {
                    let song = SONG_POOL.get_song_by_id(song_id);

                    let mut failed = false;

                    for filter in &self.rules.filters {
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
}
