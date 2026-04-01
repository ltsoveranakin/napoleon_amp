use crate::content::folder::Folder;
use crate::content::playlist::PlaylistType;
use std::rc::Rc;

#[derive(Debug)]
pub enum FolderContentVariant {
    Folder(Rc<Folder>),
    Playlist(Rc<PlaylistType>),
}
