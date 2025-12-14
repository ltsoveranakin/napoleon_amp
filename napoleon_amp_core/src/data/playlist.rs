use crate::data::song::Song;
use crate::data::PathNamed;

pub struct Playlist {
    path_named: PathNamed,
    songs: Option<Vec<Song>>,
}

impl Playlist {
    pub(super) fn new(path_named: PathNamed) -> Self {
        Self {
            path_named,
            songs: None,
        }
    }
}
