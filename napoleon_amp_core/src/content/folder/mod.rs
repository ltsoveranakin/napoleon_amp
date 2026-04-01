pub mod content;
pub(crate) mod content_pool;

use crate::content::folder::content::FolderContentVariant;
use crate::content::folder::content_pool::{CONTENT_POOL, RemoveAssociatedFileError};
use crate::content::playlist::{PlaylistType, StandardPlaylist};
use crate::paths::content_folder_file;
use serbytes::prelude::{MayNotExistOrDefault, SerBytes};
use simple_id::prelude::Id;
use std::cell::{OnceCell, Ref, RefCell, RefMut};
use std::fmt::Debug;
use std::io;
use std::path::PathBuf;
use std::rc::{Rc, Weak};

#[derive(SerBytes, Debug, Copy, Clone)]
pub enum FolderDataContentVariant {
    Folder,
    Playlist,
}

#[derive(SerBytes, Debug)]
pub struct ContentData<P> {
    pub name: String,
    pub parent: P,
}

impl<P> ContentData<P> {
    pub(super) fn new(name: String, parent: P) -> Self {
        Self { name, parent }
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
    pub expanded: MayNotExistOrDefault<bool>,
}

impl FolderData {
    pub(super) fn new(content_data: FolderContentData) -> Self {
        Self {
            content_data,
            contents: Vec::new(),
            expanded: true.into(),
        }
    }

    pub fn get_folder_data_path(id: Id) -> PathBuf {
        content_folder_file(id)
    }

    pub(crate) fn save_data(&self, id: Id) -> io::Result<()> {
        self.write_to_file_path(Self::get_folder_data_path(id))
    }
}

#[derive(Debug)]
pub enum DeleteContentError {
    Io(io::Error),
    RemoveAssoc(RemoveAssociatedFileError),
    IndexOutOfBounds,
}

impl From<io::Error> for DeleteContentError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<RemoveAssociatedFileError> for DeleteContentError {
    fn from(value: RemoveAssociatedFileError) -> Self {
        Self::RemoveAssoc(value)
    }
}

#[derive(Debug)]
pub struct Folder {
    pub id: Id,
    pub parent: Option<Weak<Folder>>,
    folder_data: OnceCell<RefCell<FolderData>>,
    contents: OnceCell<RefCell<Vec<FolderContentVariant>>>,
}

impl Folder {
    pub(crate) fn new(folder_id: Id, parent: Option<Weak<Folder>>) -> Self {
        Self {
            id: folder_id,
            parent,
            folder_data: OnceCell::new(),
            contents: OnceCell::new(),
        }
    }

    pub fn create_folder(this: &Rc<Self>, folder_name: String) -> io::Result<()> {
        let folder_id = CONTENT_POOL.create_new_folder(folder_name, Some(this.id))?;

        Self::create_content(this, FolderDataContentVariant::Folder, folder_id);

        Ok(())
    }

    pub fn create_playlist(this: &Rc<Self>, playlist_name: String) -> io::Result<()> {
        let playlist_id = CONTENT_POOL.create_new_playlist(playlist_name, this.id)?;

        Self::create_content(this, FolderDataContentVariant::Playlist, playlist_id);

        Ok(())
    }

    pub fn delete_content(this: &Rc<Self>, content_index: usize) -> Result<(), DeleteContentError> {
        let mut folder_data = this.get_folder_data_mut();
        let folder_data_contents = &mut folder_data.contents;

        if content_index < folder_data_contents.len() {
            folder_data_contents.remove(content_index);
            let content = Self::get_contents_mut(this).remove(content_index);

            match content {
                FolderContentVariant::Playlist(playlist) => {
                    CONTENT_POOL.delete_playlist(playlist.id())?
                }
                FolderContentVariant::Folder(folder) => {
                    Folder::delete_self(&folder)?;
                }
            }

            folder_data.save_data(this.id)?;

            Ok(())
        } else {
            Err(DeleteContentError::IndexOutOfBounds)
        }
    }

    fn delete_self(this: &Rc<Self>) -> Result<(), DeleteContentError> {
        let contents_len = Folder::get_contents(this).len();

        for content_index in (0..contents_len).rev() {
            Folder::delete_content(this, content_index)?;
        }

        CONTENT_POOL.delete_folder(this.id)?;

        Ok(())
    }

    fn get_folder_data_refcell(&self) -> &RefCell<FolderData> {
        self.folder_data.get_or_init(|| {
            let folder_path = content_folder_file(self.id);

            let data = FolderData::from_file_path(folder_path).unwrap_or_else(|_| {
                assert_eq!(self.id, Id::ZERO, "Temp fix for base folder");
                let data = FolderData::new(FolderContentData::new("Base".to_string(), None));

                data.save_data(self.id).expect("write folder data to disk");

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

            FolderDataContentVariant::Playlist => FolderContentVariant::Playlist(Rc::new(
                PlaylistType::Standard(StandardPlaylist::new_file(id, this)),
            )),
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
        {
            let mut contents = Self::get_contents_mut(this);

            contents.push(Self::get_folder_content_variant(this, variant, id));
        }

        let mut folder_data = this.get_folder_data_mut();

        folder_data
            .contents
            .push(ContentsListElements { id, variant });

        folder_data
            .write_to_file_path(content_folder_file(this.id))
            .expect("Write folder data to file");
    }
}

// impl NamedPathLike for Folder {
//     fn get_path_named(&self) -> &PathNamed {
//         &self.path_named
//     }
// }
