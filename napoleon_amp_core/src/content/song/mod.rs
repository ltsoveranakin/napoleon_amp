mod master;
pub(crate) mod song_data;

use crate::content::song::song_data::{get_song_data_from_song_file, SongData};
use crate::content::{NamedPathLike, PathNamed};
use crate::{read_rwlock, write_rwlock, ReadWrapper, WriteWrapper};
use serbytes::prelude::SerBytes;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::RwLock;

pub static UNKNOWN_ARTIST_STR: &str = "Unknown Artist";
pub static UNKNOWN_ALBUM_STR: &str = "Unknown Album";

#[derive(Debug)]
pub struct Song {
    path_named: PathNamed,
    pub(super) song_data_path: PathBuf,
    pub(super) song_data: RwLock<SongData>,
    pub(super) has_loaded_song_data: RwLock<bool>,
}

impl Song {
    pub(crate) fn new(path_named: PathNamed) -> Self {
        Self {
            song_data: RwLock::new(SongData {
                title: path_named.name.clone(),
                ..Default::default()
            }),
            song_data_path: path_named
                .path
                .parent()
                .expect("Valid song path")
                .join(format!("{}.snap", path_named.name)),
            path_named,
            has_loaded_song_data: RwLock::new(false),
        }
    }

    pub fn get_or_load_song_data(&self) -> ReadWrapper<'_, SongData> {
        if **read_rwlock(&self.has_loaded_song_data) {
            read_rwlock(&self.song_data)
        } else {
            self.load_song_data();

            read_rwlock(&self.song_data)
        }
    }

    pub fn get_or_load_song_data_mut(&self) -> WriteWrapper<'_, SongData> {
        if **read_rwlock(&self.has_loaded_song_data) {
            write_rwlock(&self.song_data)
        } else {
            self.load_song_data();

            write_rwlock(&self.song_data)
        }
    }

    fn load_song_data(&self) {
        let mut song_data = SongData::from_file_path(&self.song_data_path).unwrap_or_else(|_| {
            let mut sd = SongData::default();
            get_song_data_from_song_file(self, &mut sd);

            sd
        });

        if song_data.artist.len() == 0 {
            song_data.artist = UNKNOWN_ARTIST_STR.into();
        }

        if song_data.album.len() == 0 {
            song_data.album = UNKNOWN_ALBUM_STR.into();
        }

        **write_rwlock(&self.song_data) = song_data;

        **write_rwlock(&self.has_loaded_song_data) = true;
    }

    pub fn set_song_data(&self, new_song_data: SongData) {
        let mut song_data = self.get_or_load_song_data_mut();

        **song_data = new_song_data;

        song_data
            .write_to_file_path(&self.song_data_path)
            .expect("Write song data to file");
    }
}

impl Clone for Song {
    fn clone(&self) -> Self {
        Self {
            path_named: self.path_named.clone(),
            song_data_path: self.song_data_path.clone(),
            // The RwLock is only ever written to once, when lazily loaded at Song::get_or_load_song_data.
            // The loading operation will never panic while holding a lock to the data, so this call to expect is okay

            // No clue why I said that above, its completely redundant since if it did panic it would panic on the main thread anyway.
            song_data: RwLock::new(self.song_data.read().expect("Writer to not panic").clone()),
            has_loaded_song_data: RwLock::new(
                *self
                    .has_loaded_song_data
                    .read()
                    .expect("Writer to not panic"),
            ),
        }
    }
}

impl PartialEq<Song> for Song {
    fn eq(&self, other: &Self) -> bool {
        self.path_named == other.path_named
    }
}

impl PartialEq<&Song> for Song {
    fn eq(&self, other: &&Self) -> bool {
        self.path_named == other.path_named
    }
}

impl Eq for Song {}

impl Hash for Song {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path_named.name.hash(state);
    }
}

impl NamedPathLike for Song {
    fn get_path_named(&self) -> &PathNamed {
        &self.path_named
    }
}
