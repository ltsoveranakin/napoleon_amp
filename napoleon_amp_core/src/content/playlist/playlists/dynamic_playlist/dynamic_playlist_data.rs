use crate::content::playlist::AllSongsValue;
use crate::content::playlist::data::{PlaylistContentData, PlaylistUserData, PlaylistUserDataStd};
use crate::content::playlist::playlists::dynamic_playlist::rules::Rules;
use crate::time_now;
use serbytes::prelude::{
    BBReadResult, CurrentVersion, ReadByteBufferRefMut, SerBytes, VersioningWrapper,
};
use simple_id::prelude::Id;

pub type DynamicPlaylistData =
    VersioningWrapper<DynamicPlaylistDataStd, DynamicPlaylistDataVersion>;

impl AllSongsValue for DynamicPlaylistData {
    fn new_all_songs() -> Self {
        DynamicPlaylistDataStd::new(PlaylistContentData::new_all_songs()).into()
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

#[derive(SerBytes, Debug)]
pub enum ImportFrom {
    AllSongs,
    Playlists(Vec<Id>),
}

#[derive(SerBytes, Debug)]
pub struct DynamicPlaylistDataStd {
    pub(crate) user_data: PlaylistUserData,
    rules: Rules,
    last_updated: u64,
}

impl DynamicPlaylistDataStd {
    pub(super) fn new(content_data: PlaylistContentData) -> Self {
        Self {
            user_data: PlaylistUserDataStd::new(content_data).into(),
            rules: Rules::new(),
            last_updated: time_now().as_secs(),
        }
    }
}
