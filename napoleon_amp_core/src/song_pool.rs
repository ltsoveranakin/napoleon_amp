use crate::content::song::Song;
use crate::paths::song::registered_songs_data_file_v2;
use crate::{ReadWrapper, read_rwlock, time_now, write_rwlock};
use serbytes::prelude::{ReadError, SerBytes};
use simple_id::prelude::Id;
use std::collections::HashMap;
use std::io;
use std::sync::{Arc, LazyLock, RwLock};

pub(super) static SONG_POOL: LazyLock<SongPool> = LazyLock::new(SongPool::new);

type WeakArc<T> = std::sync::Weak<T>;

#[derive(SerBytes)]
pub(crate) struct RegisteredSongs {
    pub(crate) name_map: HashMap<String, Id>,
    pub(crate) last_updated: Result<u64, ReadError<'static>>,
}

impl Default for RegisteredSongs {
    fn default() -> Self {
        Self {
            name_map: HashMap::default(),
            last_updated: Err(ReadError::default()),
        }
    }
}

impl RegisteredSongs {
    fn save_registered_songs(&mut self) -> io::Result<()> {
        self.last_updated = Ok(time_now().as_secs());
        self.write_to_file_path(registered_songs_data_file_v2())
    }
}

pub(super) struct SongPool {
    songs: RwLock<HashMap<Id, WeakArc<Song>>>,
    registered_songs: LazyLock<RwLock<RegisteredSongs>>,
}

impl SongPool {
    fn new() -> Self {
        Self {
            songs: RwLock::new(HashMap::new()),
            registered_songs: LazyLock::new(Self::load_registered_songs),
        }
    }

    fn load_registered_songs() -> RwLock<RegisteredSongs> {
        let mut registered_songs =
            RegisteredSongs::from_file_path(registered_songs_data_file_v2()).unwrap_or_default();

        if registered_songs.last_updated.is_err() {
            registered_songs.last_updated = Ok(time_now().as_secs());
        }

        // Doesn't matter if we cant save registered songs immediately after loading since it will just reload all the dynamic playlists
        let _ = registered_songs.save_registered_songs();

        RwLock::new(registered_songs)
    }

    pub(super) fn get_song_by_id(&self, song_id: Id) -> Arc<Song> {
        let songs = read_rwlock(&self.songs);

        let song = if let Some(song) = songs.get(&song_id) {
            let song_upgraded = song.upgrade();

            song_upgraded.unwrap_or_else(|| {
                drop(songs);

                self.load_song(song_id)
            })
        } else {
            drop(songs);

            self.load_song(song_id)
        };

        song
    }

    pub(super) fn register_new_song(&self, song_id: Id, name: String) -> Result<(), ()> {
        let name_map = &mut write_rwlock(&self.registered_songs).name_map;

        if name_map.contains_key(&name) {
            return Err(());
        }

        name_map.insert(name, song_id);

        Ok(())
    }

    pub(crate) fn get_registered_songs(&self) -> ReadWrapper<'_, RegisteredSongs> {
        read_rwlock(&self.registered_songs)
    }

    pub(super) fn save_registered_songs(&self) -> io::Result<()> {
        write_rwlock(&self.registered_songs).save_registered_songs()
    }

    fn load_song(&self, song_id: Id) -> Arc<Song> {
        let song = Arc::new(Song::new(song_id));

        let mut songs = write_rwlock(&self.songs);

        songs.insert(song_id, Arc::downgrade(&song));

        song
    }
}
