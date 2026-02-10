use crate::data::folder::Folder;
use crate::data::playlist::Playlist;
use crate::data::song::Song;
use crate::data::PathNamed;
use crate::paths::folders_dir;
use std::rc::Rc;
use crate::net::server::NapoleonServer;

pub struct NapoleonInstance {
    base_folder: Rc<Folder>,
    copied_songs: Option<Vec<Song>>,
    server: Option<NapoleonServer>
}

impl NapoleonInstance {
    pub fn new() -> Self {
        Self {
            base_folder: Rc::new(Folder::new(PathNamed::new(folders_dir()), None)),
            copied_songs: None,
            server: None
        }
    }

    pub fn get_base_folder(&self) -> Rc<Folder> {
        Rc::clone(&self.base_folder)
    }

    pub fn copy_selected_songs(&mut self, playlist: &Playlist) {
        let songs = &playlist.get_or_load_songs();
        let selected_songs_variant = playlist.get_selected_songs_variant();

        let selected_songs = selected_songs_variant.get_selected_songs(songs).to_vec();

        self.copied_songs = Some(selected_songs);
    }

    pub fn paste_copied_songs(&self, playlist: &Playlist) {
        if let Some(ref copied_songs) = self.copied_songs {
            playlist.import_existing_songs(copied_songs);
        }
    }
}
