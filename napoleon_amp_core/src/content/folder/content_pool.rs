use crate::content::folder::{FolderContentData, FolderData};
use crate::content::playlist::data::{PlaylistContentData, PlaylistData};
use crate::id_generator::{Id, SmallRngIdGenerator};
use crate::paths::{
    content_folders_index_file, content_playlist_file, content_playlists_index_file,
};
use crate::song_pool::SONG_POOL;
use crate::{write_rwlock, WriteWrapper};
use serbytes::prelude::{BBReadResult, ReadError, SerBytes};
use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};

pub(crate) static CONTENT_POOL: LazyLock<ContentPool> = LazyLock::new(ContentPool::new);

struct ContentInner {
    index_data_path: PathBuf,
    id_generator: SmallRngIdGenerator,
    index_data: Option<HashSet<Id>>,
}

impl ContentInner {
    fn new(index_path: PathBuf) -> Self {
        Self {
            index_data_path: index_path,
            id_generator: SmallRngIdGenerator::new(),
            index_data: None,
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
}

pub(crate) struct ContentPool {
    folders: RwLock<ContentInner>,
    playlists: RwLock<ContentInner>,
}

impl ContentPool {
    fn new() -> Self {
        Self {
            folders: RwLock::new(ContentInner::new(content_folders_index_file())),
            playlists: RwLock::new(ContentInner::new(content_playlists_index_file())),
        }
    }

    pub(crate) fn get_playlist_data(&self, playlist_id: Id) -> BBReadResult<PlaylistData> {
        if playlist_id == Id::ZERO {
            // TODO: parent id should be option here or not option on folders
            let mut data = PlaylistData::new(PlaylistContentData::new(
                playlist_id,
                "Base".to_string(),
                Id::ZERO,
            ));

            data.song_ids = SONG_POOL
                .get_registered_songs()
                .name_map
                .values()
                .copied()
                .collect();

            Ok(data)
        } else if self.playlists_mut().get_index_data().contains(&playlist_id) {
            PlaylistData::from_file_path(content_playlist_file(playlist_id))
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

    pub(super) fn delete_playlist(&self, playlist_id: &Id) {
        Self::delete_content(&mut self.playlists_mut(), playlist_id)
    }

    pub(super) fn delete_folder(&self, folder_id: &Id) {
        Self::delete_content(&mut self.folders_mut(), folder_id)
    }

    fn delete_content(content_inner: &mut ContentInner, content_id: &Id) {
        let index_data = content_inner.get_index_data();

        if index_data.remove(content_id) {
            content_inner.save_index_data();
        }
    }

    pub(super) fn create_new_playlist(
        &self,
        playlist_name: String,
        parent_folder: Id,
    ) -> io::Result<Id> {
        let id = Self::generate_unique_id(&self.playlists);

        let playlist_data =
            PlaylistData::new(PlaylistContentData::new(id, playlist_name, parent_folder));

        playlist_data.save_data()?;

        Ok(id)
    }

    pub(super) fn create_new_folder(
        &self,
        folder_name: String,
        parent_folder: Option<Id>,
    ) -> io::Result<Id> {
        let id = Self::generate_unique_id(&self.folders);

        let folder_data = FolderData::new(FolderContentData::new(id, folder_name, parent_folder));

        folder_data.save_data()?;

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
