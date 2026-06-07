mod master;
pub mod song_data;
pub(crate) mod song_pool;

use crate::content::song::song_data::{SongData, get_song_data_from_song_file};
use crate::paths::song::{song_audio_file_v2, song_data_file_v2};
use crate::{ReadGuard, WriteGuard, read_rwlock, write_rwlock};
use serbytes::prelude::SerBytesFs;
use simple_id::prelude::Id;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

pub static UNKNOWN_ARTIST_STR: &str = "Unknown Artist";
pub static UNKNOWN_ALBUM_STR: &str = "Unknown Album";

#[derive(Debug)]
pub struct Song {
    pub(crate) id: Id,
    pub song_audio_path: PathBuf,
    pub song_data_path: PathBuf,
    pub(super) song_data: OnceLock<RwLock<SongData>>,
}

impl Song {
    pub(crate) fn new(song_id: Id) -> Self {
        let song_audio_path = song_audio_file_v2(&song_id);
        let song_data_path = song_data_file_v2(&song_id);

        Self {
            song_data: OnceLock::new(),
            song_data_path,
            song_audio_path,
            id: song_id,
        }
    }

    pub fn get_song_data(&self) -> ReadGuard<'_, SongData> {
        let song_data_lock = self.get_song_data_rwlock();

        read_rwlock(song_data_lock)
    }

    pub fn get_song_data_mut(&self) -> WriteGuard<'_, SongData> {
        let song_data_lock = self.get_song_data_rwlock();

        write_rwlock(song_data_lock)
    }

    pub fn get_song_data_rwlock(&self) -> &RwLock<SongData> {
        self.song_data.get_or_init(|| {
            let mut song_data = match SongData::from_file_path(&self.song_data_path) {
                Ok(song_data) => song_data,
                Err(e) => {
                    eprintln!("{}", e);

                    let mut sd = SongData::default();
                    get_song_data_from_song_file(&self, &mut sd);

                    sd
                }
            };

            if song_data.inner.meta.inner.is_err() {
                get_song_data_from_song_file(&self, &mut song_data);
            }

            let meta = song_data.inner.meta_mut();

            if meta.artist.full_artist_string.is_empty() {
                meta.artist.full_artist_string = UNKNOWN_ARTIST_STR.to_string();
            }

            if meta.album.len() == 0 {
                meta.album = UNKNOWN_ALBUM_STR.into();
            }

            if song_data.did_update() {
                println!("updating song data");
                song_data
                    .write_to_file_path(song_data_file_v2(&self.id))
                    .expect("unable to update song data to latest version");
            }

            RwLock::new(song_data)
        })
    }

    pub fn set_song_data_and_save(&self, new_song_data: SongData) {
        let mut song_data = self.get_song_data_mut();

        *song_data = new_song_data;

        drop(song_data);

        self.save_song_data()
    }

    pub fn save_song_data(&self) {
        self.save_song_data_already_borrowed(&self.get_song_data());
    }

    pub fn save_song_data_already_borrowed(&self, song_data: &SongData) {
        song_data
            .write_to_file_path(&self.song_data_path)
            .expect("Write song data to file");
    }
}

impl PartialEq<Song> for Song {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialEq<&Song> for Song {
    fn eq(&self, other: &&Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Song {}

impl Hash for Song {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
