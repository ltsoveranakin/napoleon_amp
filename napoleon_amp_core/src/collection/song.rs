use serbytes::prelude::SerBytes;

#[derive(SerBytes)]
pub struct Song {
    song_id: u16
}

impl Song {
    fn get_song_file(&self) {

    }
}