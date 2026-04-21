use crate::content::SaveData;
use crate::content::playlist::PlaylistData;
use crate::content::playlist::data::{PlaylistContentData, PlaylistUserData, PlaylistUserDataStd};
use crate::content::playlist::playlists::dynamic_playlist::rules::Rules;
use crate::paths::content_playlist_user_data_file;
use crate::time_now;
use serbytes::prelude::{
    BBReadResult, CurrentVersion, ReadByteBufferRefMut, SerBytes, VersioningWrapper,
};
use simple_id::prelude::Id;
use std::path::PathBuf;

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
pub enum ImportFrom {
    AllSongs,
    Playlists(Vec<Id>),
}

#[derive(SerBytes, Debug, Clone)]
pub struct DynamicPlaylistDataStd {
    pub user_data: PlaylistUserData,
    pub rules: Rules,
    last_updated: u64,
}

impl DynamicPlaylistDataStd {
    pub(crate) fn new(content_data: PlaylistContentData) -> Self {
        Self {
            user_data: PlaylistUserDataStd::new(content_data).into(),
            rules: Rules::new(),
            last_updated: time_now().as_secs(),
        }
    }
}
