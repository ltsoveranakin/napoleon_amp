use crate::data::playlist::Playlist;
use crate::data::PathNamed;
use std::cell::RefCell;
use std::rc::Rc;

pub enum FolderContentVariant {
    SubFolder(Folder),
    Playlist(Playlist),
}

struct FolderContent {
    parent: Rc<Folder>,
    variant: FolderContentVariant,
}

impl FolderContent {
    fn new(parent: Rc<Folder>, variant: FolderContentVariant) -> Self {
        Self { parent, variant }
    }
}

pub struct Folder {
    path_named: PathNamed,
    contents: RefCell<Option<Vec<FolderContent>>>,
}

impl Folder {
    fn new(path_named: PathNamed) -> Self {
        Self {
            path_named,
            contents: RefCell::new(None),
        }
    }
}

pub trait GetOrLoadContent {
    fn get_or_load_content(&mut self) -> &Vec<FolderContent>;
}

impl GetOrLoadContent for Rc<Folder> {
    fn get_or_load_content(&mut self) -> &Vec<FolderContent> {
        let c = self.contents.borrow();

        let contents = if c.is_some() {
            // Check: above check if is_some, cannot be None
            self.contents.borrow()
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
                            Rc::clone(self),
                            FolderContentVariant::SubFolder(sub_folder),
                        ))
                        // can be symlink, check if file to be safe
                    } else if file_type.is_file() {
                        let path_named =
                            PathNamed::new(dir.path()).expect("Unable to create PathNamed");
                        let playlist = Playlist::new(path_named);

                        Some(FolderContent::new(
                            Rc::clone(self),
                            FolderContentVariant::Playlist(playlist),
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
            let contents_ref = &contents;
            self.contents = Some(contents);
            contents_ref
        };

        contents
    }
}
