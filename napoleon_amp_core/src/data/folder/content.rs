use crate::data::folder::Folder;
use crate::data::playlist::Playlist;
use std::rc::{Rc, Weak};

pub enum FolderContentVariant {
    SubFolder(Rc<Folder>),
    Playlist(Rc<Playlist>),
}

pub struct FolderContent {
    parent: Weak<Folder>,
    pub variant: FolderContentVariant,
}

impl FolderContent {
    pub(super) fn new(parent: Weak<Folder>, variant: FolderContentVariant) -> Self {
        Self { parent, variant }
    }
}
