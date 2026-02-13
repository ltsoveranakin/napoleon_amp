use chrono::{Datelike, Local, Timelike};
use std::path::{Path, PathBuf};

fn home_dir() -> PathBuf {
    dirs_next::home_dir().expect("Forced home directory")
}

fn napoleon_amp_dir() -> PathBuf {
    home_dir().join("/napoleon_amp/")
}

pub(super) fn instance_data_file_path() -> PathBuf {
    napoleon_amp_dir().join("instance_data.dnap")
}

pub(super) fn folders_dir() -> PathBuf {
    napoleon_amp_dir().join("folders/")
}

pub(super) fn songs_dir() -> PathBuf {
    napoleon_amp_dir().join("songs/")
}

pub(super) fn song_file<P: AsRef<Path>>(song_name: P) -> PathBuf {
    songs_dir().join(song_name)
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
