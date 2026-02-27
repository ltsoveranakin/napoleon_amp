use crate::id_generator::Id;
use crate::paths::{napoleon_amp_dir, DATA_EXT, SONG_DATA_EXT};
use std::path::{Path, PathBuf};

pub(crate) fn songs_dir_v1() -> PathBuf {
    napoleon_amp_dir().join("songs/")
}

pub(crate) fn song_file_v1<P: AsRef<Path>>(song_name: P) -> PathBuf {
    songs_dir_v1().join(song_name)
}

pub(crate) fn songs_blanket_dir_v2() -> PathBuf {
    napoleon_amp_dir().join("songs_v2/")
}

pub(crate) fn songs_data_dir_v2() -> PathBuf {
    songs_blanket_dir_v2().join("data/")
}

pub(crate) fn songs_audio_dir_v2() -> PathBuf {
    songs_blanket_dir_v2().join("audio/")
}

pub(crate) fn registered_songs_data_file_v2() -> PathBuf {
    songs_blanket_dir_v2().join(format!("song_set{}", DATA_EXT))
}

pub(crate) fn song_data_file_v2(song_id: &Id) -> PathBuf {
    songs_data_dir_v2().join(format!("{}{}", song_id.to_string(), SONG_DATA_EXT))
}

pub(crate) fn song_audio_file_v2(song_id: &Id) -> PathBuf {
    // TODO: work with any audio type
    songs_audio_dir_v2().join(format!("{}.mp3", song_id.to_string()))
}
