use crate::content::SaveData;
use crate::content::folder::ContentData;
use crate::content::playlist::PlaylistData;
use crate::content::playlist::song_list::SortBy;
use crate::paths::{content_playlist_song_list_file, content_playlist_user_data_file};
use crate::{Next, time_now};
use serbytes::prelude::{
    BBReadResult, CurrentVersion, MayNotExistOrDefault, ReadByteBufferRefMut, SerBytes,
    VersioningWrapper,
};
use simple_id::prelude::Id;
use std::fmt::{Display, Formatter};
use std::io;
use std::path::PathBuf;

const DEFAULT_VOLUME: f32 = 1.0;

#[derive(SerBytes, Default, Debug, Copy, Clone)]
pub enum PlaybackMode {
    Sequential,
    #[default]
    Shuffle,
}

impl Next for PlaybackMode {
    fn get_next(&self) -> Self {
        match self {
            PlaybackMode::Sequential => PlaybackMode::Shuffle,
            PlaybackMode::Shuffle => PlaybackMode::Sequential,
        }
    }
}

impl Display for PlaybackMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sequential => f.write_str("Sequential"),
            Self::Shuffle => f.write_str("Shuffle"),
        }
    }
}

pub(crate) type PlaylistContentData = ContentData<Id>;

pub type PlaylistUserData = VersioningWrapper<PlaylistUserDataStd, PlaylistUserDataVersion>;

impl PlaylistData for PlaylistUserData {
    fn new_all_songs() -> Self {
        PlaylistUserDataStd::new(PlaylistContentData::new_all_songs()).into()
    }

    fn new_deleted_with_data(content_data: PlaylistContentData) -> Self {
        PlaylistUserDataStd::new(content_data).into()
    }
}

impl SaveData for PlaylistUserData {
    fn get_path(id: Id) -> PathBuf {
        content_playlist_user_data_file(id)
    }
}

#[derive(SerBytes, Debug)]
pub struct PlaylistUserDataStd {
    pub content_data: PlaylistContentData,
    pub playback_mode: PlaybackMode,
    pub volume: f32,
    pub sort_by: SortBy,
}

impl PlaylistData for PlaylistContentData {
    fn new_all_songs() -> Self {
        Self::new("All Songs".to_string(), Id::ZERO)
    }

    fn new_deleted_with_data(content_data: PlaylistContentData) -> Self {
        Self::new("Deleted Playlist".to_string(), content_data.parent)
    }
}

#[derive(SerBytes, Debug)]
pub enum PlaylistUserDataVersion {
    V1,
}

impl CurrentVersion for PlaylistUserDataVersion {
    type Output = PlaylistUserDataStd;

    fn get_data_from_buf(&self, buf: &mut ReadByteBufferRefMut) -> BBReadResult<Self::Output> {
        match self {
            Self::V1 => PlaylistUserDataStd::from_buf(buf),
        }
    }

    fn current_version() -> Self {
        Self::V1
    }
}

impl PlaylistUserDataStd {
    pub(crate) fn new(content_data: PlaylistContentData) -> Self {
        Self {
            content_data,
            playback_mode: PlaybackMode::default(),
            volume: DEFAULT_VOLUME,
            sort_by: SortBy::default(),
        }
    }
}

#[derive(SerBytes, Debug)]
pub struct PlaylistSongListData {
    pub(crate) song_ids: Vec<Id>,
    pub(crate) last_updated: MayNotExistOrDefault<u64>,
}

impl SaveData for PlaylistSongListData {
    fn get_path(id: Id) -> PathBuf {
        content_playlist_song_list_file(id)
    }

    fn save_data(&mut self, id: Id) -> io::Result<()> {
        self.last_updated = time_now().as_secs().into();
        self.write_to_file_path(Self::get_path(id))
    }
}
