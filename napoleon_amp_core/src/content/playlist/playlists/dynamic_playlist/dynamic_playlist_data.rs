use crate::content::playlist::playlists::dynamic_playlist::rules::Rules;
use serbytes::prelude::{
    BBReadResult, CurrentVersion, ReadByteBufferRefMut, SerBytes, VersioningWrapper,
};
use simple_id::prelude::Id;

pub type VersionedDynamicPlaylistData =
    VersioningWrapper<DynamicPlaylistData, DynamicPlaylistDataVersion>;

#[derive(SerBytes)]
pub enum DynamicPlaylistDataVersion {
    V1,
}

impl CurrentVersion for DynamicPlaylistDataVersion {
    type Output = DynamicPlaylistData;

    fn get_data_from_buf(&self, buf: &mut ReadByteBufferRefMut) -> BBReadResult<Self::Output> {
        match self {
            Self::V1 => DynamicPlaylistData::from_buf(buf),
        }
    }

    fn current_version() -> Self {
        Self::V1
    }
}

#[derive(SerBytes)]
pub enum ImportFrom {
    AllSongs,
    Playlists(Vec<Id>),
}

#[derive(SerBytes)]
pub struct DynamicPlaylistData {
    rules: Rules,
    last_updated: u64,
}
