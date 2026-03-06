use crate::content::folder::ContentData;
use crate::content::playlist::song_list::SortBy;
use crate::id_generator::Id;
use crate::paths::content_playlist_file;
use serbytes::prelude::{MayNotExistDataProvider, SerBytes};
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

impl Display for PlaybackMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sequential => f.write_str("Sequential"),
            Self::Shuffle => f.write_str("Shuffle"),
        }
    }
}

#[derive(Debug)]
pub(super) struct VolumeDNEDataProvider;

impl MayNotExistDataProvider<f32> for VolumeDNEDataProvider {
    fn get_data() -> f32 {
        DEFAULT_VOLUME
    }
}

pub(crate) type PlaylistContentData = ContentData<Id>;

#[derive(SerBytes, Debug)]
pub struct PlaylistData {
    pub content_data: PlaylistContentData,
    pub(crate) song_ids: Vec<Id>,
    pub(super) playback_mode: PlaybackMode,
    pub(super) volume: f32,
    pub(super) sort_by: SortBy,
}

impl PlaylistData {
    pub(crate) fn new(content_data: PlaylistContentData) -> Self {
        Self {
            content_data,
            song_ids: Vec::new(),
            playback_mode: PlaybackMode::default(),
            volume: DEFAULT_VOLUME,
            sort_by: SortBy::default(),
        }
    }

    pub fn get_data_path(&self) -> PathBuf {
        content_playlist_file(self.content_data.id)
    }

    pub(crate) fn save_data(&self) -> io::Result<()> {
        self.write_to_file_path(self.get_data_path())
    }
}
