use crate::content::folder::Folder;
use crate::content::playlist::StaticPlaylist;
use std::rc::Rc;

// use crate::content::folder::Folder;
// use crate::content::playlist::Playlist;
// use std::rc::Rc;
//
#[derive(Debug)]
pub enum FolderContentVariant {
    Folder(Rc<Folder>),
    Playlist(Rc<StaticPlaylist>),
}

// #[derive(Debug)]
// pub struct FolderContent {
//     pub variant: FolderContentVariant,
// }
//
// impl FolderContent {
//     pub(super) fn new(variant: FolderContentVariant) -> Self {
//         Self { variant }
//     }
// }
