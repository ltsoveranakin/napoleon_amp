use std::path::{Path, PathBuf};

fn home_dir() -> PathBuf {
    dirs_next::home_dir().expect("Forced home directory")
}

fn nap_amp_dir() -> PathBuf {
    home_dir().join("/napoleon_amp/")
}

pub(super) fn folders_dir() -> PathBuf {
    nap_amp_dir().join("folders/")
}

pub(super) fn songs_dir() -> PathBuf {
    nap_amp_dir().join("songs/")
}

pub(super) fn song_file<P: AsRef<Path>>(song_name: P) -> PathBuf {
    songs_dir().join(song_name)
}
