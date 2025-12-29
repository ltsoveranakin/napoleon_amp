use crate::data::folder::Folder;
use crate::data::playlist::Playlist;
use crate::song_player_provider::SongPlayerProvider;
use std::rc::Rc;

#[derive(Debug)]
pub enum FolderContentVariant<S: SongPlayerProvider> {
    SubFolder(Rc<Folder<S>>),
    Playlist(Rc<Playlist<S>>),
}

#[derive(Debug)]
pub struct FolderContent<S: SongPlayerProvider> {
    pub variant: FolderContentVariant<S>,
}

impl<S: SongPlayerProvider> FolderContent<S> {
    pub(super) fn new(variant: FolderContentVariant<S>) -> Self {
        Self { variant }
    }
}
