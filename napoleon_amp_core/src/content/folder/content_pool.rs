use crate::content::folder::{FolderContentData, FolderData};
use crate::content::playlist::data::{PlaylistContentData, PlaylistSongListData, PlaylistUserData};
use crate::paths::{
    content_folder_file, content_playlist_song_list_file, content_playlist_user_data_file,
};
use crate::song_pool::SONG_POOL;
use crate::{WriteWrapper, write_rwlock};
use serbytes::prelude::{BBReadResult, SerBytes};
use simple_id::prelude::{Id, SmallRngIdGenerator};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};
use std::{fs, io};

pub(crate) static CONTENT_POOL: LazyLock<ContentPool> = LazyLock::new(ContentPool::new);

struct ContentInner {
    id_generator: SmallRngIdGenerator,
    provide_assoc_files: Box<dyn Send + Sync + 'static + Fn(Id) -> Vec<PathBuf>>,
}

impl ContentInner {
    fn new<F>(provide_assoc_files: F) -> Self
    where
        F: Send + Sync + 'static + Fn(Id) -> Vec<PathBuf>,
    {
        Self {
            id_generator: SmallRngIdGenerator::default(),
            provide_assoc_files: Box::new(provide_assoc_files),
        }
    }

    fn get_associated_files(&self, id: Id) -> Vec<PathBuf> {
        (self.provide_assoc_files)(id)
    }

    fn remove_file_assoc(&self, id: Id) -> Result<(), RemoveAssociatedFileError> {
        for file_path in self.get_associated_files(id) {
            if let Err(io_error) = fs::remove_file(&file_path) {
                // Dont care if file doesnt exist... we're deleting it anyways lol
                if io_error.kind() != ErrorKind::NotFound {
                    return Err(RemoveAssociatedFileError {
                        io_error,
                        file_path,
                    });
                }
            }
        }

        Ok(())
    }

    fn exists(&self, id: Id) -> bool {
        for file_path in self.get_associated_files(id) {
            if fs::exists(file_path).unwrap_or(false) {
                return true;
            }
        }

        false
    }
}

#[derive(Debug)]
pub struct RemoveAssociatedFileError {
    pub io_error: io::Error,
    pub file_path: PathBuf,
}

type RmAssocResult = Result<(), RemoveAssociatedFileError>;

pub(crate) struct ContentPool {
    folders: RwLock<ContentInner>,
    playlists: RwLock<ContentInner>,
}

impl ContentPool {
    fn new() -> Self {
        Self {
            folders: RwLock::new(ContentInner::new(|id| vec![content_folder_file(id)])),
            playlists: RwLock::new(ContentInner::new(|id| {
                vec![
                    content_playlist_user_data_file(id),
                    content_playlist_song_list_file(id),
                ]
            })),
        }
    }

    pub(crate) fn get_playlist_user_data(&self, playlist_id: Id) -> BBReadResult<PlaylistUserData> {
        if playlist_id == Id::ZERO {
            let data =
                PlaylistUserData::new(PlaylistContentData::new("Base".to_string(), Id::ZERO));

            Ok(data)
        } else {
            PlaylistUserData::from_file_path(content_playlist_user_data_file(playlist_id))
        }
    }

    pub(crate) fn get_playlist_song_list_data(
        &self,
        playlist_id: Id,
    ) -> BBReadResult<PlaylistSongListData> {
        if playlist_id == Id::ZERO {
            let data = PlaylistSongListData {
                song_ids: SONG_POOL
                    .get_registered_songs()
                    .name_map
                    .values()
                    .copied()
                    .collect(),
            };

            Ok(data)
        } else {
            PlaylistSongListData::from_file_path(content_playlist_song_list_file(playlist_id))
        }
    }

    pub(super) fn delete_playlist(&self, playlist_id: Id) -> RmAssocResult {
        Self::delete_content0(&mut self.playlists_mut(), playlist_id)
    }

    pub(super) fn delete_folder(&self, folder_id: Id) -> RmAssocResult {
        Self::delete_content0(&mut self.folders_mut(), folder_id)
    }

    fn delete_content0(content_inner: &mut ContentInner, content_id: Id) -> RmAssocResult {
        content_inner.remove_file_assoc(content_id)?;

        Ok(())
    }

    pub(super) fn create_new_playlist(
        &self,
        playlist_name: String,
        parent_folder: Id,
    ) -> io::Result<Id> {
        let id = Self::generate_unique_id(&self.playlists);

        let playlist_data =
            PlaylistUserData::new(PlaylistContentData::new(playlist_name, parent_folder));

        playlist_data.save_data(id)?;

        Ok(id)
    }

    pub(super) fn create_new_folder(
        &self,
        folder_name: String,
        parent_folder: Option<Id>,
    ) -> io::Result<Id> {
        let id = Self::generate_unique_id(&self.folders);

        let folder_data = FolderData::new(FolderContentData::new(folder_name, parent_folder));

        folder_data.save_data(id)?;

        Ok(id)
    }

    fn playlists_mut(&self) -> WriteWrapper<'_, ContentInner> {
        write_rwlock(&self.playlists)
    }

    fn folders_mut(&self) -> WriteWrapper<'_, ContentInner> {
        write_rwlock(&self.playlists)
    }

    fn generate_unique_id(content_inner: &RwLock<ContentInner>) -> Id {
        let mut content_inner_mut = write_rwlock(content_inner);

        loop {
            let id = content_inner_mut.id_generator.generate_new_id();

            if !content_inner_mut.exists(id) {
                return id;
            }
        }
    }
}
