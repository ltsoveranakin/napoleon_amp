use crate::id_generator::Id;
use chrono::{Datelike, Local, Timelike};
use std::path::{Path, PathBuf};

const DATA_EXT: &str = ".dnap";
pub(super) const SONG_DATA_EXT: &str = ".snap";
pub(super) const SONG_DATA_EXT_NO_PER: &str = "snap";

fn home_dir() -> PathBuf {
    dirs_next::home_dir().expect("Forced home directory")
}

fn napoleon_amp_dir() -> PathBuf {
    home_dir().join("/napoleon_amp/")
}

pub(super) fn instance_data_file_path() -> PathBuf {
    napoleon_amp_dir().join("instance_data").join(DATA_EXT)
}

pub(super) fn folders_dir() -> PathBuf {
    napoleon_amp_dir().join("folders/")
}

pub(super) fn songs_dir_v1() -> PathBuf {
    napoleon_amp_dir().join("songs/")
}

pub(super) fn song_file_v1<P: AsRef<Path>>(song_name: P) -> PathBuf {
    songs_dir_v1().join(song_name)
}

pub(super) fn songs_blanket_dir_v2() -> PathBuf {
    napoleon_amp_dir().join("songs_v2/")
}

pub(super) fn songs_data_dir_v2() -> PathBuf {
    songs_blanket_dir_v2().join("data/")
}

pub(super) fn songs_audio_dir_v2() -> PathBuf {
    songs_blanket_dir_v2().join("audio/")
}

pub(super) fn registered_songs_data_file_v2() -> PathBuf {
    songs_blanket_dir_v2().join(format!("song_set{}", DATA_EXT))
}

pub(super) fn song_data_file_v2(song_id: &Id) -> PathBuf {
    songs_data_dir_v2().join(format!("{}{}", song_id.to_string(), SONG_DATA_EXT))
}

pub(super) fn song_audio_file_v2(song_id: &Id) -> PathBuf {
    // TODO: work with any audio type
    songs_audio_dir_v2().join(format!("{}.mp3", song_id.to_string()))
}

pub fn log_dir() -> PathBuf {
    napoleon_amp_dir().join("logs/")
}

pub fn crash_file_time_now() -> PathBuf {
    let date_time = Local::now();

    log_dir().join(format!(
        "YMD-{}-{}-{}-HMS-{}-{}-{}_crash.txt",
        date_time.year(),
        date_time.month(),
        date_time.day(),
        date_time.hour(),
        date_time.minute(),
        date_time.second()
    ))
}
