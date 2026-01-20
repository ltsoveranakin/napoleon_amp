pub mod content;

use crate::data::folder::content::{FolderContent, FolderContentVariant};

use crate::data::playlist::{Playlist, PlaylistVariant};
use crate::data::{unwrap_inner_ref, NamedPathLike, PathNamed};
use std::cell::{Ref, RefCell};
use std::fs;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub struct Folder {
    path_named: PathNamed,
    contents: RefCell<Option<Vec<Rc<FolderContent>>>>,
    pub parent: Option<Weak<Self>>,
}

impl Folder {
    pub(crate) fn new(path_named: PathNamed, parent: Option<Weak<Self>>) -> Self {
        Self {
            path_named,
            contents: RefCell::new(None),
            parent,
        }
    }

    pub fn get_or_load_content(this: &Rc<Self>) -> Ref<'_, Vec<Rc<FolderContent>>> {
        let contents = if this.contents.borrow().is_some() {
            unwrap_inner_ref(this.contents.borrow())
        } else {
            let mut contents = vec![];
            for dir in this
                .path_named
                .path
                .read_dir()
                .expect("Unable to read path directory")
            {
                if dir.is_err() {
                    continue;
                }

                // Unwrapping, continues above if dir is err
                let dir = dir.unwrap();

                let content = if let Ok(file_type) = dir.file_type() {
                    if file_type.is_dir() {
                        let path_named = PathNamed::new(dir.path());
                        let sub_folder = Folder::new(path_named, Some(Rc::downgrade(this)));

                        Some(FolderContent::new(FolderContentVariant::SubFolder(
                            Rc::new(sub_folder),
                        )))
                        // can be symlink, check if file to be safe
                    } else if file_type.is_file() {
                        let path_named = PathNamed::new(dir.path());
                        let playlist = Playlist::new_file(path_named);

                        Some(FolderContent::new(FolderContentVariant::Playlist(Rc::new(
                            playlist,
                        ))))
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(content) = content {
                    contents.push(Rc::new(content));
                }
            }

            this.contents.replace(Some(contents));

            unwrap_inner_ref(this.contents.borrow())
        };

        contents
    }

    fn add_content(this: &Rc<Self>, folder_content_variant: FolderContentVariant) {
        if this.contents.borrow().is_none() {
            Self::get_or_load_content(this);
        }

        let mut contents = this.contents.borrow_mut();

        contents
            .as_mut()
            .expect("Loaded contents if none; Guaranteed")
            .push(Rc::new(FolderContent::new(folder_content_variant)))
    }

    pub fn add_folder(this: &Rc<Self>, folder_name: String) {
        let path_named = this.path_named.extend(format!("{}/", folder_name));
        let folder = Folder::new(path_named, Some(Rc::downgrade(this)));

        Self::add_content(this, FolderContentVariant::SubFolder(Rc::new(folder)));
    }

    pub fn add_playlist(this: &Rc<Self>, playlist_name: String) {
        let path_named = this.path_named.extend(format!("{}.pnap", playlist_name));
        let playlist = Playlist::new_file(path_named);

        Self::add_content(this, FolderContentVariant::Playlist(Rc::new(playlist)));
    }

    pub fn delete_content(&self, content_index: usize) {
        if let Some(contents) = &mut *self.contents.borrow_mut() {
            if (0..contents.len()).contains(&content_index) {
                let content = contents.remove(content_index);

                match &content.variant {
                    FolderContentVariant::Playlist(playlist) => {
                        match playlist.variant {
                            PlaylistVariant::PlaylistFile => {
                                fs::remove_file(playlist.path()).expect("Delete file successfully");
                            }

                            PlaylistVariant::SongFolder => {
                                todo!(
                                    "Need to check if current playlist is all songs, dont try and delete an invalid file in that case"
                                )
                            }
                        }

                        // playlist.path()
                    }

                    FolderContentVariant::SubFolder(folder) => {
                        todo!()
                    }
                }
            }
        }
    }
}

impl NamedPathLike for Folder {
    fn get_path_named(&self) -> &PathNamed {
        &self.path_named
    }
}
