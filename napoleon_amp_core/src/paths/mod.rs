pub(super) mod song;

use crate::id_generator::Id;
use chrono::{Datelike, Local, Timelike};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::{fs, io};

const DATA_EXT: &str = ".dnap";
const INDEX_EXT: &str = ".inap";
const FOLDER_EXT: &str = ".fnap";
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

pub(crate) fn content_blanket_path() -> PathBuf {
    napoleon_amp_dir().join("content/")
}

pub(crate) fn content_folder_path() -> PathBuf {
    content_blanket_path().join("folders/")
}

pub(crate) fn content_folder_file(id: Id) -> PathBuf {
    content_folder_path().join(format!("{}{}", id, FOLDER_EXT))
}

pub(crate) fn content_playlist_path() -> PathBuf {
    content_blanket_path().join("playlists/")
}

pub(crate) fn content_playlist_file(id: Id) -> PathBuf {
    content_playlist_path().join(format!("{}{}", id, FOLDER_EXT))
}

pub(crate) fn content_folders_index_file() -> PathBuf {
    content_blanket_path().join(format!("folders{}", INDEX_EXT))
}

pub(crate) fn content_playlists_index_file() -> PathBuf {
    content_blanket_path().join(format!("playlists{}", INDEX_EXT))
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

pub fn show_file_in_explorer(path: impl AsRef<Path>) -> io::Result<()> {
    let path_buf = fs::canonicalize(path)?;
    let parent = path_buf.parent().ok_or(ErrorKind::InvalidFilename)?;
    open::that_detached(parent)

    // let path_str = path_buf.to_string_lossy();

    // let path_str = if path_str.starts_with(r"\\?\") {
    //     path_str.replace(r"\\?\", "")
    // } else {
    //     path_str.to_string()
    // };
    //
    // // let arg = format!(r#"/select,"{}""#, path_str);
    // // println!("arg: {}", arg);
    // Command::new("explorer")
    //     .arg(r#"/select,""#)
    //     // .arg("/select,")
    //     // .arg(format!("\"{}\"", path_str))
    //     .spawn()?;

    // Ok(())
}
