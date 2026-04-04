use crate::content::folder::ContentData;
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

#[derive(SerBytes, Debug)]
enum PlaylistTypeData {
    Standard,
}

pub(crate) type PlaylistContentData = ContentData<Id>;

pub type PlaylistUserData = VersioningWrapper<PlaylistUserDataStd, PlaylistUserDataVersion>;

#[derive(SerBytes, Debug)]
pub struct PlaylistUserDataStd {
    pub playlist_type_data: PlaylistTypeData,
    pub content_data: PlaylistContentData,
    pub playback_mode: PlaybackMode,
    pub volume: f32,
    pub sort_by: SortBy,
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
            playlist_type_data: PlaylistTypeData::Standard,
            content_data,
            playback_mode: PlaybackMode::default(),
            volume: DEFAULT_VOLUME,
            sort_by: SortBy::default(),
        }
    }

    pub fn get_data_path(id: Id) -> PathBuf {
        content_playlist_user_data_file(id)
    }

    pub fn save_data(&self, id: Id) -> io::Result<()> {
        self.write_to_file_path(Self::get_data_path(id))
    }
}

#[derive(SerBytes, Debug)]
pub struct PlaylistSongListData {
    pub(crate) song_ids: Vec<Id>,
    pub(crate) last_updated: MayNotExistOrDefault<u64>,
}

impl PlaylistSongListData {
    pub fn get_data_path(id: Id) -> PathBuf {
        content_playlist_song_list_file(id)
    }

    pub(crate) fn save_data(&mut self, id: Id) -> io::Result<()> {
        self.last_updated = time_now().as_secs().into();
        self.write_to_file_path(Self::get_data_path(id))
    }
}
