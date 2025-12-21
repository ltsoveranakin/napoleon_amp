mod master;

use crate::data::{NamedPathLike, PathNamed};

#[derive(Clone, Debug)]
pub struct Song {
    path_named: PathNamed,
}

impl Song {
    pub(super) fn new(path_named: PathNamed) -> Self {
        Self { path_named }
    }
}

impl NamedPathLike for Song {
    fn get_path_named(&self) -> &PathNamed {
        &self.path_named
    }
}
