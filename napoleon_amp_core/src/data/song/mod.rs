mod master;

use crate::data::{NamedPathLike, PathNamed};
use std::path::PathBuf;

pub struct Song {
    path_named: PathNamed,
}

impl Song {
    pub(super) fn new(path_named: PathNamed) -> Self {
        Self { path_named }
    }
}

impl NamedPathLike for Song {
    fn name(&self) -> &str {
        self.path_named.name()
    }

    fn path(&self) -> &PathBuf {
        self.path_named.path()
    }
}
