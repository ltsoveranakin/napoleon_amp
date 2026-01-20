mod master;

use crate::data::{NamedPathLike, PathNamed};
use crate::{read_rwlock, write_rwlock, ReadWrapper};
use serbytes::prelude::SerBytes;
use std::collections::HashMap;
use std::fs;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(SerBytes, Clone, Debug)]
enum SongTagValue {
    String(String),
}

#[derive(SerBytes, Eq, PartialEq, Hash, Clone, Debug)]
enum TagType {
    Dynamic(String),
}

#[derive(SerBytes, Clone, Debug, Default)]
pub struct SongData {
    pub artist: String,
    pub album: String,
    pub title: String,
    custom_song_tags: HashMap<TagType, SongTagValue>,
}

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
            let song_data_loaded =
                SongData::from_vec(fs::read(&self.song_data_path).expect("Read song data file"))
                    .unwrap_or(SongData::default());

            **write_rwlock(&self.song_data) = song_data_loaded;

            **write_rwlock(&self.has_loaded_song_data) = true;

            read_rwlock(&self.song_data)
        }
    }
}

impl Clone for Song {
    fn clone(&self) -> Self {
        Self {
            path_named: self.path_named.clone(),
            song_data_path: self.song_data_path.clone(),
            // The RwLock is only ever written to once, when lazily loaded at Song::get_or_load_song_data.
            // The loading operation will never panic while holding a lock to the data, so this call to expect is okay
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

impl NamedPathLike for Song {
    fn get_path_named(&self) -> &PathNamed {
        &self.path_named
    }
}
