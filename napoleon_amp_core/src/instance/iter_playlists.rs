use crate::content::folder::Folder;
use crate::content::folder::content::FolderContentVariant;
use crate::content::playlist::PlaylistType;
use std::collections::LinkedList;
use std::rc::Rc;

pub struct IterPlaylists {
    folder_queue: LinkedList<Rc<Folder>>,
    folder: Rc<Folder>,
    index: usize,
}

impl IterPlaylists {
    pub(super) fn new(base_folder: Rc<Folder>) -> Self {
        Self {
            folder_queue: LinkedList::new(),
            folder: base_folder,
            index: 0,
        }
    }
}

impl Iterator for IterPlaylists {
    type Item = Rc<PlaylistType>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let folder_contents = self.folder.get_contents();

            if self.index == folder_contents.len() {
                let new_folder = if let Some(new_folder) = self.folder_queue.pop_front() {
                    new_folder
                } else {
                    return None;
                };

                drop(folder_contents);

                self.index = 0;
                self.folder = new_folder;
            }

            let content = self.folder.get_contents()[self.index].clone();

            self.index += 1;

            match content {
                FolderContentVariant::Folder(folder) => {
                    self.folder_queue.push_back(folder);
                }

                FolderContentVariant::Playlist(playlist) => {
                    return Some(playlist);
                }
            }
        }
    }
}
