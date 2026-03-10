pub mod content;
pub(crate) mod content_pool;

use crate::content::folder::content::FolderContentVariant;
use crate::content::folder::content_pool::CONTENT_POOL;
use crate::content::playlist::Playlist;
use crate::id_generator::Id;
use crate::paths::content_folder_file;
use serbytes::prelude::SerBytes;
use std::cell::{OnceCell, Ref, RefCell, RefMut};
use std::fmt::Debug;
use std::io;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::rc::{Rc, Weak};

#[derive(SerBytes, Debug, Copy, Clone)]
pub enum FolderDataContentVariant {
    Folder,
    Playlist,
}

#[derive(SerBytes, Debug)]
pub struct ContentData<P> {
    pub id: Id,
    pub name: String,
    pub parent: P,
}

impl<P> ContentData<P> {
    pub(super) fn new(id: Id, name: String, parent: P) -> Self {
        Self { id, name, parent }
    }
}

#[derive(SerBytes, Debug)]
pub struct ContentsListElements {
    pub variant: FolderDataContentVariant,
    pub id: Id,
}

pub type FolderContentData = ContentData<Option<Id>>;

#[derive(SerBytes, Debug)]
pub struct FolderData {
    pub content_data: FolderContentData,
    pub contents: Vec<ContentsListElements>,
}

impl FolderData {
    pub(super) fn new(content_data: FolderContentData) -> Self {
        Self {
            content_data,
            contents: Vec::new(),
        }
    }

    pub fn get_folder_data_path(&self) -> PathBuf {
        content_folder_file(self.content_data.id)
    }

    pub(crate) fn save_data(&self) -> io::Result<()> {
        self.write_to_file_path(self.get_folder_data_path())
    }
}

// #[derive(SerBytes, Debug)]
// struct PlaylistData {
//     content_data: ContentData<Id>,
//     contents: Vec<FolderDataContent>,
// }

#[derive(Debug)]
pub struct Folder {
    folder_id: Id,
    pub parent: Option<Weak<Folder>>,
    folder_data: OnceCell<RefCell<FolderData>>,
    contents: OnceCell<RefCell<Vec<FolderContentVariant>>>,
}

impl Folder {
    pub(crate) fn new(folder_id: Id, parent: Option<Weak<Folder>>) -> Self {
        Self {
            folder_id,
            parent,
            folder_data: OnceCell::new(),
            contents: OnceCell::new(),
        }
    }

    // pub fn get_or_load_content(this: &Rc<Self>) -> Ref<'_, Vec<Rc<FolderContent>>> {
    //     let contents = if this.contents.borrow().is_some() {
    //         unwrap_inner_ref(this.contents.borrow())
    //     } else {
    //         let mut contents = vec![];
    //         for dir in this
    //             .path_named
    //             .path
    //             .read_dir()
    //             .expect("Unable to read path directory")
    //         {
    //             if dir.is_err() {
    //                 continue;
    //             }
    //
    //             // Unwrapping, continues above if dir is err
    //             let dir = dir.unwrap();
    //
    //             let content = if let Ok(file_type) = dir.file_type() {
    //                 if file_type.is_dir() {
    //                     let path_named = PathNamed::new(dir.path());
    //                     let sub_folder = Folder::new(path_named, Some(Rc::downgrade(this)));
    //
    //                     Some(FolderContent::new(FolderContentVariant::SubFolder(
    //                         Rc::new(sub_folder),
    //                     )))
    //                     // can be symlink, check if file to be safe
    //                 } else if file_type.is_file() {
    //                     let path_named = PathNamed::new(dir.path());
    //                     let playlist = Playlist::new_file(path_named);
    //
    //                     Some(FolderContent::new(FolderContentVariant::Playlist(Rc::new(
    //                         playlist,
    //                     ))))
    //                 } else {
    //                     None
    //                 }
    //             } else {
    //                 None
    //             };
    //
    //             if let Some(content) = content {
    //                 contents.push(Rc::new(content));
    //             }
    //         }
    //
    //         this.contents.replace(Some(contents));
    //
    //         unwrap_inner_ref(this.contents.borrow())
    //     };
    //
    //     contents
    // }

    pub fn create_folder(this: &Rc<Self>, folder_name: String) -> io::Result<()> {
        let folder_id = CONTENT_POOL.create_new_folder(folder_name, Some(this.folder_id))?;

        Self::create_content(this, FolderDataContentVariant::Folder, folder_id);

        Ok(())
    }

    pub fn create_playlist(this: &Rc<Self>, playlist_name: String) -> io::Result<()> {
        let playlist_id = CONTENT_POOL.create_new_playlist(playlist_name, this.folder_id)?;

        Self::create_content(this, FolderDataContentVariant::Playlist, playlist_id);

        Ok(())
    }

    pub fn delete_content(this: &Rc<Self>, content_index: usize) -> io::Result<()> {
        let mut folder_data = this.get_folder_data_mut();
        let folder_data_contents = &mut folder_data.contents;

        if content_index < folder_data_contents.len() {
            let content = folder_data_contents.remove(content_index);
            Self::get_contents_mut(this).remove(content_index);

            match content.variant {
                FolderDataContentVariant::Playlist => CONTENT_POOL.delete_playlist(&content.id),
                FolderDataContentVariant::Folder => CONTENT_POOL.delete_folder(&content.id),
            }

            folder_data.save_data()
        } else {
            Err(ErrorKind::InvalidInput.into())
        }
    }

    fn get_folder_data_refcell(&self) -> &RefCell<FolderData> {
        self.folder_data.get_or_init(|| {
            let folder_path = content_folder_file(self.folder_id);

            let data = FolderData::from_file_path(folder_path).unwrap_or_else(|_| {
                assert_eq!(self.folder_id, Id::ZERO, "Temp fix for base folder");
                let data = FolderData::new(FolderContentData::new(
                    self.folder_id,
                    "Base".to_string(),
                    None,
                ));

                data.save_data().expect("write folder data to disk");

                data
            });

            RefCell::new(data)
        })
    }

    pub fn get_folder_data(&self) -> Ref<'_, FolderData> {
        self.get_folder_data_refcell().borrow()
    }

    fn get_folder_data_mut(&self) -> RefMut<'_, FolderData> {
        self.get_folder_data_refcell().borrow_mut()
    }

    fn get_folder_content_variant(
        this: &Rc<Self>,
        variant: FolderDataContentVariant,
        id: Id,
    ) -> FolderContentVariant {
        let parent = Rc::downgrade(this);

        match variant {
            FolderDataContentVariant::Folder => {
                FolderContentVariant::Folder(Rc::new(Folder::new(id, Some(parent))))
            }

            FolderDataContentVariant::Playlist => {
                FolderContentVariant::Playlist(Rc::new(Playlist::new_file(id)))
            }
        }
    }

    fn get_contents_refcell(this: &Rc<Self>) -> &RefCell<Vec<FolderContentVariant>> {
        this.contents.get_or_init(|| {
            let data_contents = &this.get_folder_data().contents;

            let mut contents = Vec::with_capacity(data_contents.len());

            for ContentsListElements { id, variant } in data_contents {
                contents.push(Self::get_folder_content_variant(this, *variant, *id));
            }

            RefCell::new(contents)
        })
    }

    pub fn get_contents(this: &Rc<Self>) -> Ref<'_, Vec<FolderContentVariant>> {
        Self::get_contents_refcell(this).borrow()
    }

    fn get_contents_mut(this: &Rc<Self>) -> RefMut<'_, Vec<FolderContentVariant>> {
        Self::get_contents_refcell(this).borrow_mut()
    }

    fn create_content(this: &Rc<Self>, variant: FolderDataContentVariant, id: Id) {
        let mut folder_data = this.get_folder_data_mut();
        let mut contents = Self::get_contents_mut(this);

        contents.push(Self::get_folder_content_variant(this, variant, id));

        folder_data
            .contents
            .push(ContentsListElements { id, variant });

        folder_data
            .write_to_file_path(content_folder_file(this.folder_id))
            .expect("Write folder data to file");
    }
}

// impl NamedPathLike for Folder {
//     fn get_path_named(&self) -> &PathNamed {
//         &self.path_named
//     }
// }
