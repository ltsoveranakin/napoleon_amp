use crate::content::folder::{FolderContentData, FolderData};
use crate::content::playlist::data::{PlaylistContentData, PlaylistSongListData, PlaylistUserData};
use crate::paths::{content_folder_file, content_folders_index_file, content_playlist_song_list_file, content_playlist_user_data_file, content_playlists_index_file};
use crate::song_pool::SONG_POOL;
use crate::{write_rwlock, WriteWrapper};
use serbytes::prelude::{BBReadResult, ReadError, SerBytes};
use simple_id::prelude::{Id, SmallRngIdGenerator};
use std::collections::HashSet;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};
use std::{fs, io};

pub(crate) static CONTENT_POOL: LazyLock<ContentPool> = LazyLock::new(ContentPool::new);

struct ContentInner {
    index_data_path: PathBuf,
    id_generator: SmallRngIdGenerator,
    index_data: Option<HashSet<Id>>,
    provide_assoc_files: Box<dyn Send + Sync + 'static + Fn(Id) -> Vec<PathBuf>>,
}

impl ContentInner

{
    fn new<F>(index_path: PathBuf, provide_assoc_files: F) -> Self
    where
        F: Send + Sync + 'static + Fn(Id) -> Vec<PathBuf>,
    {
        Self {
            index_data_path: index_path,
            id_generator: SmallRngIdGenerator::default(),
            index_data: None,
            provide_assoc_files: Box::new(provide_assoc_files),
        }
    }

    fn get_index_data(&mut self) -> &mut HashSet<Id> {
        self.index_data.get_or_insert_with(|| {
            HashSet::from_file_path(&self.index_data_path).unwrap_or_default()
        })
    }

    fn save_index_data(&mut self) {
        self.get_index_data()
            .write_to_file_path(content_playlists_index_file())
            .expect("Write index data to file");
    }

    fn remove_file_assoc(&self, id: Id) -> Result<(), RemoveAssociatedFileError> {
        for file_path in (self.provide_assoc_files)(id) {
            if let Err(io_error) =
                fs::remove_file(&file_path) {
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
}

#[derive(Debug)]
pub struct RemoveAssociatedFileError {
    io_error: io::Error,
    file_path: PathBuf,
}

type RmAssocResult = Result<(), RemoveAssociatedFileError>;

pub(crate) struct ContentPool {
    folders: RwLock<ContentInner>,
    playlists: RwLock<ContentInner>,
}

impl ContentPool {
    fn new() -> Self {
        Self {
            folders: RwLock::new(ContentInner::new(content_folders_index_file(), |id| {
                vec![content_folder_file(id)]
            })),
            playlists: RwLock::new(ContentInner::new(content_playlists_index_file(), |id| {
                vec![content_playlist_song_list_file(id), content_playlist_user_data_file(id)]
            })),
        }
    }

    pub(crate) fn get_playlist_user_data(&self, playlist_id: Id) -> BBReadResult<PlaylistUserData> {
        if playlist_id == Id::ZERO {
            let data = PlaylistUserData::new(PlaylistContentData::new(
                "Base".to_string(),
                Id::ZERO,
            ));

            Ok(data)
        } else if self.playlists_mut().get_index_data().contains(&playlist_id) {
            PlaylistUserData::from_file_path(content_playlist_user_data_file(playlist_id))
        } else {
            Err(ReadError::new(
                "Playlist doesn't exist in index".to_string(),
            ))
        }
    }

    pub(crate) fn get_playlist_song_list_data(&self, playlist_id: Id) -> BBReadResult<PlaylistSongListData> {
        if playlist_id == Id::ZERO {
            let data = PlaylistSongListData {
                song_ids: SONG_POOL
                    .get_registered_songs()
                    .name_map
                    .values()
                    .copied()
                    .collect()
            };

            Ok(data)
        } else if self.playlists_mut().get_index_data().contains(&playlist_id) {
            PlaylistSongListData::from_file_path(content_playlist_song_list_file(playlist_id))
        } else {
            Err(ReadError::new(
                "Playlist doesn't exist in index".to_string(),
            ))
        }
    }

    pub(super) fn check_folder_exists(&self, folder_id: &Id) -> bool {
        write_rwlock(&self.folders)
            .get_index_data()
            .contains(folder_id)
    }

    pub(super) fn check_playlist_exists(&self, playlist_id: &Id) -> bool {
        write_rwlock(&self.playlists)
            .get_index_data()
            .contains(playlist_id)
    }

    pub(super) fn delete_playlist(&self, playlist_id: &Id) -> RmAssocResult {
        Self::delete_content0(&mut self.playlists_mut(), playlist_id)
    }

    pub(super) fn delete_folder(&self, folder_id: &Id) -> RmAssocResult {
        Self::delete_content0(&mut self.folders_mut(), folder_id)
    }

    fn delete_content0(content_inner: &mut ContentInner, content_id: &Id) -> RmAssocResult {
        let index_data = content_inner.get_index_data();

        if index_data.remove(content_id) {
            content_inner.save_index_data();
        }

        content_inner.remove_file_assoc(*content_id)?;

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

            let index_data = content_inner_mut.get_index_data();

            if !index_data.contains(&id) {
                index_data.insert(id);
                content_inner_mut.save_index_data();
                return id;
            }
        }
    }
}
