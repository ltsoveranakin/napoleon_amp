use crate::content::song::Song;
use crate::id_generator::Id;

use crate::paths::song::registered_songs_data_file_v2;
use crate::{read_rwlock, write_rwlock};
use serbytes::prelude::SerBytes;
use std::collections::HashMap;
use std::io;
use std::sync::{Arc, LazyLock, RwLock};

pub(super) static SONG_POOL: LazyLock<SongPool> = LazyLock::new(SongPool::new);

pub(super) type WeakArc<T> = std::sync::Weak<T>;

#[derive(SerBytes, Default)]
struct RegisteredSongs {
    // songs_set: HashSet<Id>,
    name_map: HashMap<String, Id>,
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
        let registered_songs =
            RegisteredSongs::from_file_path(registered_songs_data_file_v2()).unwrap_or_default();

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

    pub(super) fn register_song(&self, song_id: Id, name: String) {
        write_rwlock(&self.registered_songs)
            .name_map
            .insert(name, song_id);
    }

    pub(super) fn save_registered_songs(&self) -> io::Result<()> {
        read_rwlock(&self.registered_songs).write_to_file_path(registered_songs_data_file_v2())
    }

    fn load_song(&self, song_id: Id) -> Arc<Song> {
        let song = Arc::new(Song::new(song_id));

        let mut songs = write_rwlock(&self.songs);

        songs.insert(song_id, Arc::downgrade(&song));

        song
    }
}
