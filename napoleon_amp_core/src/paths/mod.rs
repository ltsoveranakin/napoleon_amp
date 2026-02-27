pub(super) mod song;

use chrono::{Datelike, Local, Timelike};
use std::path::PathBuf;

const DATA_EXT: &str = ".dnap";
pub(crate) const SONG_DATA_EXT: &str = ".snap";
pub(crate) const SONG_DATA_EXT_NO_PER: &str = "snap";

fn home_dir() -> PathBuf {
    dirs_next::home_dir().expect("Forced home directory")
}

fn napoleon_amp_dir() -> PathBuf {
    home_dir().join("/napoleon_amp/")
}

pub(crate) fn instance_data_file_path() -> PathBuf {
    napoleon_amp_dir().join("instance_data").join(DATA_EXT)
}

pub(crate) fn folders_dir() -> PathBuf {
    napoleon_amp_dir().join("folders/")
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
