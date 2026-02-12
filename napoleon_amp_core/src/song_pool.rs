use crate::content::song::Song;
use crate::content::PathNamed;
use crate::paths::song_file;
use crate::{read_rwlock, write_rwlock};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

pub(super) static SONG_POOL: LazyLock<SongPool> = LazyLock::new(SongPool::new);

pub(super) type WeakArc<T> = std::sync::Weak<T>;

pub(super) struct SongPool {
    songs: RwLock<HashMap<String, WeakArc<Song>>>,
}

impl SongPool {
    pub(super) fn new() -> Self {
        Self {
            songs: RwLock::new(HashMap::new()),
        }
    }

    pub(super) fn get_song_by_name(&self, song_name: String) -> Arc<Song> {
        let songs = read_rwlock(&self.songs);

        let song = if let Some(song) = songs.get(&song_name) {
            let song_upgraded = song.upgrade();

            song_upgraded.unwrap_or_else(|| {
                drop(songs);

                self.load_song(song_name)
            })
        } else {
            drop(songs);

            self.load_song(song_name)
        };

        song
    }

    fn load_song(&self, song_name: String) -> Arc<Song> {
        let song_path = song_file(&song_name);

        let song = Arc::new(Song::new(PathNamed::new(song_path)));

        let mut songs = write_rwlock(&self.songs);

        songs.insert(song_name, Arc::downgrade(&song));

        song
    }
}
