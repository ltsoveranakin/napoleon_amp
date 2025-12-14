use crate::data::folder::Folder;
use crate::data::playlist::Playlist;
use std::rc::Rc;

pub enum FolderContentVariant {
    SubFolder(Rc<Folder>),
    Playlist(Playlist),
}

pub struct FolderContent {
    parent: Rc<Folder>,
    pub variant: FolderContentVariant,
}

impl FolderContent {
    pub(super) fn new(parent: Rc<Folder>, variant: FolderContentVariant) -> Self {
        Self { parent, variant }
    }
}
