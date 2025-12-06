use crate::collection::song::Song;
use serbytes::prelude::{ReadByteBuffer, SerBytes};
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;

pub struct Playlist {
    pub(crate) path: PathBuf,
}

#[derive(SerBytes)]
pub struct PlaylistData {
    songs: Vec<Song>,
}

impl Playlist {
    pub fn get_playlist_data(&self) -> io::Result<PlaylistData> {
        let mut file = File::open(&self.path)?;
        let mut playlist_file_bytes = vec![];

        file.read_to_end(&mut playlist_file_bytes)?;

        let mut bb = ReadByteBuffer::from_vec(playlist_file_bytes);

        let playlist_data = PlaylistData::from_buf(&mut bb)?;

        Ok(playlist_data)
    }
}
