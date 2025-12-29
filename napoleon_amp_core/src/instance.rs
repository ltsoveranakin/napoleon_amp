use crate::data::folder::Folder;
use crate::data::playlist::Playlist;
use crate::data::song::Song;
use crate::data::PathNamed;
use crate::paths::folders_dir;
use crate::song_player_provider::rodio_player_provider::RodioPlayerProvider;
use crate::song_player_provider::SongPlayerProvider;
use std::rc::Rc;

pub struct NapoleonInstance<S = RodioPlayerProvider>
where
    S: SongPlayerProvider,
{
    base_folder: Rc<Folder<S>>,
    copied_songs: Option<Vec<Song>>,
}

impl<S: SongPlayerProvider> NapoleonInstance<S> {
    pub fn new() -> Self {
        Self {
            base_folder: Rc::new(Folder::<S>::new(PathNamed::new(folders_dir()), None)),
            copied_songs: None,
        }
    }

    pub fn get_base_folder(&self) -> Rc<Folder<S>> {
        Rc::clone(&self.base_folder)
    }

    pub fn copy_selected_songs(&mut self, playlist: &Playlist<S>) {
        let songs = &playlist.get_or_load_songs();
        let selected_songs_variant = playlist.get_selected_songs_variant();

        let selected_songs = selected_songs_variant.get_selected_songs(songs).to_vec();

        self.copied_songs = Some(selected_songs);
    }

    pub fn paste_copied_songs(&self, playlist: &Playlist<S>) {
        if let Some(ref copied_songs) = self.copied_songs {
            playlist.import_existing_songs(copied_songs);
        }
    }
}
