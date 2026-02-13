use serbytes::prelude::{MayNotExistOrDefault, SerBytes};
use std::fmt::{Display, Formatter};

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

#[derive(SerBytes, Debug)]
pub(super) struct PlaylistData {
    pub(super) song_file_names: Vec<String>,
    pub(super) playback_mode: MayNotExistOrDefault<PlaybackMode>,
}

impl PlaylistData {
    pub(super) fn new_empty() -> Self {
        Self::new_capacity(0)
    }

    pub(super) fn new_capacity(cap: usize) -> Self {
        Self {
            song_file_names: Vec::with_capacity(cap),
            playback_mode: MayNotExistOrDefault::default(),
        }
    }
}
