use crate::content::folder::content_pool::CONTENT_POOL;
use crate::content::playlist::PlaylistData;
use crate::content::playlist::data::{
    PlaylistContentData, PlaylistSongListData, PlaylistUserData, PlaylistUserDataStd,
};
use crate::content::playlist::playlists::dynamic_playlist::rules::Rules;
use crate::content::song::Song;
use crate::content::song::song_pool::SONG_POOL;
use crate::content::{SaveData, map_ids_to_songs};
use crate::paths::content_playlist_user_data_file;
use crate::time_now;
use serbytes::prelude::{
    BBReadResult, CurrentVersion, FromFileResult, ReadByteBufferRefMut, SerBytes, VersioningWrapper,
};
use simple_id::prelude::Id;
use std::cell::Cell;
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
}

#[derive(Default)]
pub(super) struct SongListRes {
    pub(super) songs: Vec<Arc<Song>>,
    pub(super) song_list_data: PlaylistSongListData,
    pub(super) used_cached_songs: bool,
}

impl DynamicPlaylistDataStd {
    pub(crate) fn new(content_data: PlaylistContentData) -> Self {
        Self {
            user_data: PlaylistUserDataStd::new(content_data).into(),
            rules: Rules::new(),
        }
    }

    pub(super) fn get_song_list(&self, self_id: Id) -> FromFileResult<'_, SongListRes> {
        let mut song_ids_checked = HashSet::new();

        let playlist_ids = self.rules.get_playlist_ids();

        let mut can_use_cached_songs = true;

        let mut playlists_song_lists_data = Vec::with_capacity(playlist_ids.len());

        let self_song_list_data = CONTENT_POOL.get_playlist_song_list_data(self_id)?;

        for playlist_id in playlist_ids.iter().copied() {
            let song_list_data = CONTENT_POOL.get_playlist_song_list_data(playlist_id)?;

            // this dynamic playlist was updated before the other one we're trying to load from
            // therefore cant used cached song list
            // so recreate the entire song list
            println!(
                "updated; self: {} < other: {}; now: {}",
                self_song_list_data.last_updated.get(),
                song_list_data.last_updated.get(),
                time_now().as_secs()
            );
            if self_song_list_data.last_updated.get() < song_list_data.last_updated.get() {
                can_use_cached_songs = false;
            }

            playlists_song_lists_data.push(song_list_data);
        }

        if can_use_cached_songs {
            return Ok(SongListRes {
                songs: map_ids_to_songs(&self_song_list_data.song_ids),
                song_list_data: self_song_list_data,
                used_cached_songs: true,
            });
        }

        let mut songs = Vec::new();
        let mut song_ids = Vec::new();

        for song_list_data in playlists_song_lists_data {
            // let song_list_data = CONTENT_POOL.get_playlist_song_list_data(playlist_id)?;

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
                        song_ids.push(song.id);
                        songs.push(song);
                    }
                }
            }
        }

        Ok(SongListRes {
            songs,
            song_list_data: PlaylistSongListData {
                song_ids,
                last_updated: Cell::new(time_now().as_secs()),
            },
            used_cached_songs: false,
        })
    }
}
