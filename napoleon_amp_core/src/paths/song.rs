use crate::content::song::song_cover_pool::SongCoverId;
use crate::paths::{DATA_EXT, SONG_DATA_EXT, napoleon_amp_dir};
use simple_id::prelude::Id;
use std::path::PathBuf;

pub(crate) fn songs_blanket_dir_v2() -> PathBuf {
    napoleon_amp_dir().join("songs_v2/")
}

pub(crate) fn songs_data_dir_v2() -> PathBuf {
    songs_blanket_dir_v2().join("data/")
}

pub(crate) fn songs_audio_dir_v2() -> PathBuf {
    songs_blanket_dir_v2().join("audio/")
}

pub(crate) fn songs_cover_dir_v2() -> PathBuf {
    songs_blanket_dir_v2().join("cover/")
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

pub(crate) fn song_cover_file(song_cover_id: &SongCoverId) -> PathBuf {
    songs_cover_dir_v2().join(format!("{}.cov", song_cover_id.to_string()))
}
