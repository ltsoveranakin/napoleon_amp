use crate::data::folder::Folder;
use crate::data::playlist::Playlist;
use std::rc::Rc;

#[derive(Debug)]
pub enum FolderContentVariant {
    SubFolder(Rc<Folder>),
    Playlist(Rc<Playlist>),
}

#[derive(Debug)]
pub struct FolderContent {
    pub variant: FolderContentVariant,
}

impl FolderContent {
    pub(super) fn new(variant: FolderContentVariant) -> Self {
        Self { variant }
    }
}
