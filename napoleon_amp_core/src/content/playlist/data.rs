use crate::content::playlist::song_list::SortBy;
use serbytes::prelude::{
    MayNotExistDataProvider, MayNotExistOrDefault, MayNotExistOrElse, SerBytes,
};
use std::fmt::{Display, Formatter};

const DEFAULT_VOLUME: f32 = 1.0;

#[derive(SerBytes, Default, Debug, Copy, Clone)]
pub enum PlaybackMode {
    #[default]
    Sequential,
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

#[derive(SerBytes, Debug)]
pub(super) struct PlaylistData {
    pub(super) song_file_names: Vec<String>,
    pub(super) playback_mode: MayNotExistOrDefault<PlaybackMode>,
    pub(super) volume: MayNotExistOrElse<f32, VolumeDNEDataProvider>,
    pub(super) sort_by: MayNotExistOrDefault<SortBy>,
}

impl PlaylistData {
    pub(super) fn new_empty() -> Self {
        Self::new_capacity(0)
    }

    pub(super) fn new_capacity(cap: usize) -> Self {
        Self {
            song_file_names: Vec::with_capacity(cap),
            playback_mode: MayNotExistOrDefault::default(),
            volume: DEFAULT_VOLUME.into(),
            sort_by: SortBy::default().into(),
        }
    }
}
