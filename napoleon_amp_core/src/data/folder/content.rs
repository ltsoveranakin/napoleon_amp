use crate::data::folder::Folder;
use crate::data::playlist::Playlist;
use std::rc::Rc;

pub enum FolderContentVariant {
    SubFolder(Rc<Folder>),
    Playlist(Rc<Playlist>),
}

pub struct FolderContent {
    pub variant: FolderContentVariant,
}

impl FolderContent {
    pub(super) fn new(variant: FolderContentVariant) -> Self {
        Self { variant }
    }
}
