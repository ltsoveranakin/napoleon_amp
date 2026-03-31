use crate::content::folder::Folder;
use crate::content::playlist::StandardPlaylist;
use std::rc::Rc;

#[derive(Debug)]
pub enum FolderContentVariant {
    Folder(Rc<Folder>),
    Playlist(Rc<StandardPlaylist>),
}
