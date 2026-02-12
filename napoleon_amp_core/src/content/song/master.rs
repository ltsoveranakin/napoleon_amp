use serbytes::prelude::SerBytes;
use std::path::PathBuf;

#[derive(SerBytes)]
struct SongMasterFileIndexData {
    song_path: String,
    // usages:
}

struct SongMasterFileData {
    indexes: Vec<SongMasterFileIndexData>,
}

pub struct SongMaster {
    path: PathBuf,
    songs: Vec<String>,
}
