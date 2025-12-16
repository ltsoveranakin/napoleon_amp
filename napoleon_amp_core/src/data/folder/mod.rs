pub mod content;

use crate::data::folder::content::{FolderContent, FolderContentVariant};
use crate::data::playlist::Playlist;
use crate::data::{unwrap_inner_ref, NamedPathLike, PathNamed};
use std::cell::{Ref, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
// TODO: fix cyclic rc memory leak
pub struct Folder {
    path_named: PathNamed,
    contents: RefCell<Option<Vec<FolderContent>>>,
}

impl Folder {
    pub(crate) fn new(path_named: PathNamed) -> Self {
        Self {
            path_named,
            contents: RefCell::new(None),
        }
    }

    pub fn name(&self) -> &str {
        &self.path_named.name
    }
}

pub trait FolderImpl {
    fn add_content(&self, folder_content_variant: FolderContentVariant);

    fn add_folder(&self, folder_name: String);

    fn add_playlist(&self, playlist_name: String);
}

pub trait GetOrLoadContent {
    fn get_or_load_content(&self) -> Ref<Vec<FolderContent>>;
}

impl GetOrLoadContent for Rc<Folder> {
    fn get_or_load_content(&self) -> Ref<Vec<FolderContent>> {
        let contents = if self.contents.borrow().is_some() {
            unwrap_inner_ref(self.contents.borrow())
        } else {
            let mut contents = vec![];
            for dir in self
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
                        let path_named =
                            PathNamed::new(dir.path()).expect("Unable to create PathNamed");
                        let sub_folder = Folder::new(path_named);

                        Some(FolderContent::new(
                            Rc::downgrade(self),
                            FolderContentVariant::SubFolder(Rc::new(sub_folder)),
                        ))
                        // can be symlink, check if file to be safe
                    } else if file_type.is_file() {
                        let path_named =
                            PathNamed::new(dir.path()).expect("Unable to create PathNamed");
                        let playlist = Playlist::new(path_named);

                        Some(FolderContent::new(
                            Rc::downgrade(self),
                            FolderContentVariant::Playlist(Rc::new(playlist)),
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(content) = content {
                    contents.push(content);
                }
            }

            self.contents.replace(Some(contents));

            unwrap_inner_ref(self.contents.borrow())
        };

        contents
    }
}

impl FolderImpl for Rc<Folder> {
    fn add_content(&self, folder_content_variant: FolderContentVariant) {
        if self.contents.borrow().is_none() {
            self.get_or_load_content();
        }

        let mut contents = self.contents.borrow_mut();

        contents
            .as_mut()
            .expect("Loaded contents if none; Guaranteed")
            .push(FolderContent::new(
                Rc::downgrade(&self),
                folder_content_variant,
            ))
    }

    fn add_folder(&self, folder_name: String) {
        if let Some(path_named) = self.path_named.extend(format!("{}/", folder_name)) {
            let folder = Folder::new(path_named);
            self.add_content(FolderContentVariant::SubFolder(Rc::new(folder)));
        }
    }

    fn add_playlist(&self, playlist_name: String) {
        if let Some(path_named) = self.path_named.extend(format!("{}.pnap", playlist_name)) {
            let playlist = Playlist::new(path_named);

            self.add_content(FolderContentVariant::Playlist(Rc::new(playlist)));
        }
    }
}

impl NamedPathLike for Folder {
    fn name(&self) -> &str {
        self.path_named.name()
    }

    fn path(&self) -> &PathBuf {
        self.path_named.path()
    }
}
