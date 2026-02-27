mod master;
pub mod song_data;

use crate::content::song::song_data::{get_song_data_from_song_file, SongData};
use crate::id_generator::Id;
use crate::paths::song::{song_audio_file_v2, song_data_file_v2};
use crate::{read_rwlock, write_rwlock, ReadWrapper, WriteWrapper};
use serbytes::prelude::SerBytes;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

pub static UNKNOWN_ARTIST_STR: &str = "Unknown Artist";
pub static UNKNOWN_ALBUM_STR: &str = "Unknown Album";

#[derive(Debug)]
pub struct Song {
    pub(crate) id: Id,
    pub(crate) song_audio_path: PathBuf,
    pub(crate) song_data_path: PathBuf,
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

    pub fn get_song_data(&self) -> ReadWrapper<'_, SongData> {
        let song_data_lock = self.get_song_data_rwlock();

        read_rwlock(song_data_lock)
    }

    pub fn get_or_load_song_data_mut(&self) -> WriteWrapper<'_, SongData> {
        let song_data_lock = self.get_song_data_rwlock();

        write_rwlock(song_data_lock)
    }

    fn get_song_data_rwlock(&self) -> &RwLock<SongData> {
        self.song_data.get_or_init(|| {
            let mut song_data =
                SongData::from_file_path(&self.song_data_path).unwrap_or_else(|_| {
                    let mut sd = SongData::default();
                    get_song_data_from_song_file(&self, &mut sd);

                    sd
                });

            if song_data.artist.artist_string.len() == 0 {
                song_data.artist.artist_string = UNKNOWN_ARTIST_STR.to_string();
            }

            if song_data.album.len() == 0 {
                song_data.album = UNKNOWN_ALBUM_STR.into();
            }

            RwLock::new(song_data)
        })
    }

    pub fn set_song_data(&self, new_song_data: SongData) {
        let mut song_data = self.get_or_load_song_data_mut();

        **song_data = new_song_data;

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
