pub mod manager;
mod queue;

use crate::data::playlist::manager::{MusicCommand, MusicManager};
use crate::data::song::Song;
use crate::data::{unwrap_inner_ref, NamedPathLike, PathNamed};
use crate::paths::{song_file, songs_dir};
use crate::{read_rwlock, write_rwlock, ReadWrapper};
use rodio::Source;
use serbytes::prelude::SerBytes;
use std::cell::{Cell, Ref, RefCell};
use std::collections::LinkedList;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::ops::{Deref, RangeInclusive};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[derive(SerBytes)]
struct PlaylistData {
    songs_file_names: Vec<String>,
}

impl PlaylistData {
    fn new_empty() -> Self {
        Self::new_capacity(0)
    }

    fn new_capacity(cap: usize) -> Self {
        Self {
            songs_file_names: Vec::with_capacity(cap),
        }
    }
}

/// The type of playlist this will attempt to load songs from

#[derive(Debug)]
pub enum PlaylistVariant {
    /// Will attempt to load all songs in the current folder
    SongFolder,
    /// Will attempt to load all songs in the supplied file
    PlaylistFile,
}

pub enum SongList<'s> {
    Filtered(Ref<'s, Vec<Song>>),
    Unfiltered(ReadWrapper<'s, Vec<Song>>),
}

impl<'s> Deref for SongList<'s> {
    type Target = [Song];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Filtered(ref_songs) => &***ref_songs,
            Self::Unfiltered(rw_songs) => &***rw_songs,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SelectedSongsVariant {
    None,
    Range(RangeInclusive<usize>),
    Single(usize),
    All,
}

impl SelectedSongsVariant {
    pub fn is_selected(&self, index: usize) -> bool {
        match self {
            SelectedSongsVariant::All => true,

            SelectedSongsVariant::Range(selected_range) => selected_range.contains(&index),

            SelectedSongsVariant::Single(selected_index) => index == *selected_index,

            SelectedSongsVariant::None => false,
        }
    }

    pub fn get_selected_songs<'s>(&self, songs: &'s [Song]) -> &'s [Song] {
        match self {
            SelectedSongsVariant::All => songs,

            SelectedSongsVariant::Range(selected_range) => {
                let selected_range = selected_range.clone();
                &songs[selected_range]
            }

            SelectedSongsVariant::Single(selected_index) => {
                let selected_index = *selected_index;
                &songs[selected_index..=selected_index]
            }

            SelectedSongsVariant::None => &[],
        }
    }
}

#[derive(Debug)]
pub struct Playlist {
    path_named: PathNamed,
    songs: Arc<RwLock<Vec<Song>>>,
    has_loaded_songs: Cell<bool>,
    music_manager: RefCell<Option<MusicManager>>,
    pub variant: PlaylistVariant,
    songs_filtered: RefCell<Option<Vec<Song>>>,
    selected_songs: RefCell<SelectedSongsVariant>,
}

impl Playlist {
    fn new(path_named: PathNamed, variant: PlaylistVariant) -> Self {
        Self {
            path_named,
            songs: Arc::new(RwLock::new(Vec::new())),
            has_loaded_songs: Cell::new(false),
            music_manager: RefCell::new(None),
            variant,
            songs_filtered: RefCell::new(None),
            selected_songs: RefCell::new(SelectedSongsVariant::None),
        }
    }

    pub(super) fn new_file(path_named: PathNamed) -> Self {
        Self::new(path_named, PlaylistVariant::PlaylistFile)
    }

    fn new_folder(path_named: PathNamed) -> Self {
        Self::new(path_named, PlaylistVariant::SongFolder)
    }

    pub fn all_songs() -> Self {
        Self::new_folder(PathNamed::new(songs_dir()))
    }

    /// Gets the songs in the current playlist, with the filter if one is enabled

    pub fn get_or_load_songs(&self) -> SongList {
        let songs_filtered = self.songs_filtered.borrow();

        if songs_filtered.is_some() {
            SongList::Filtered(unwrap_inner_ref(songs_filtered))
        } else {
            SongList::Unfiltered(self.get_or_load_songs_unfiltered())
        }
    }

    pub fn get_or_load_songs_unfiltered(&self) -> ReadWrapper<Vec<Song>> {
        if self.has_loaded_songs.get() {
            read_rwlock(&self.songs)
        } else {
            let loaded_song_file_names = match self.variant {
                PlaylistVariant::PlaylistFile => {
                    let playlist_data =
                        if let Ok(file_buf_str) = fs::read_to_string(&self.path_named.path) {
                            PlaylistData::from_bytes(file_buf_str.as_bytes())
                                .unwrap_or(PlaylistData::new_empty())
                        } else {
                            PlaylistData::new_empty()
                        };

                    playlist_data.songs_file_names
                }

                PlaylistVariant::SongFolder => {
                    let mut song_file_names = LinkedList::new();
                    for song_dir in fs::read_dir(songs_dir()).expect("Song directory to exist") {
                        if let Ok(song_dir) = song_dir {
                            song_file_names.push_back(
                                song_dir
                                    .path()
                                    .file_name()
                                    .unwrap()
                                    .to_str()
                                    .unwrap()
                                    .to_string(),
                            );
                        }
                    }

                    song_file_names.into_iter().collect()
                }
            };

            {
                let mut songs = write_rwlock(&self.songs);

                songs.reserve_exact(loaded_song_file_names.len());

                for song_name in loaded_song_file_names {
                    let song_path = song_file(&song_name);
                    // println!("song_name {:?} sp {:?}", song_name, song_path);

                    songs.push(Song::new(PathNamed::new(song_path)));
                }
            }

            self.has_loaded_songs.set(true);

            read_rwlock(&self.songs)
        }
    }

    pub fn set_search_query(&self, search_query: Option<&str>) {
        if let Some(search_str) = search_query {
            let search_str_lower = search_str.to_lowercase();

            let mut songs_filtered_ll = LinkedList::new();

            for song in self.get_or_load_songs().iter() {
                if song.name().to_lowercase().contains(&search_str_lower) {
                    songs_filtered_ll.push_back(song.clone());
                }
            }

            *self.songs_filtered.borrow_mut() = Some(songs_filtered_ll.into_iter().collect());
        } else {
            *self.songs_filtered.borrow_mut() = None;
        }
    }

    /// Sets the selected range of the playlist
    /// Errors under 3 conditions.
    ///
    /// If end is less than start.
    /// If start is greater than or equal to (potentially filtered songs) length.
    /// If end is greater than or equal to (potentially filtered songs) length.

    pub fn select_range(&self, range: RangeInclusive<usize>) -> Result<(), ()> {
        let songs = &*self.get_or_load_songs();

        let start = *range.start();
        let end = *range.end();
        let song_len = songs.len();

        if end < start || start >= song_len || end >= song_len {
            Err(())
        } else {
            self.set_selected_songs(SelectedSongsVariant::Range(range));
            Ok(())
        }
    }

    pub fn select_single(&self, index: usize) {
        if index < self.get_or_load_songs().len() {
            self.set_selected_songs(SelectedSongsVariant::Single(index));
        }
    }

    pub fn select_all(&self) {
        self.set_selected_songs(SelectedSongsVariant::All);
    }

    pub fn get_selected_songs_variant(&self) -> SelectedSongsVariant {
        self.selected_songs.borrow().clone()
    }

    fn set_selected_songs(&self, selected_songs: SelectedSongsVariant) {
        *self.selected_songs.borrow_mut() = selected_songs;
    }

    pub fn import_songs(
        &self,
        song_paths: &[PathBuf],
        delete_original: bool,
    ) -> Result<(), Vec<usize>> {
        let mut songs = write_rwlock(&self.songs);
        let mut failed_import = Vec::new();

        songs.reserve_exact(song_paths.len());

        for (i, original_song_path) in song_paths.iter().enumerate() {
            let file_name = original_song_path
                .file_name()
                .expect("Path does not terminate in ..");
            let new_song_path = song_file(file_name);

            if fs::exists(&new_song_path).expect(&format!(
                "Unable to verify new song path does not exist at path: {:?}",
                new_song_path
            )) {
                failed_import.push(i);
            }

            File::create(&new_song_path).expect(&format!(
                "Unable to create new song file to copy to; path: {:?}",
                new_song_path
            ));

            fs::copy(original_song_path, &new_song_path).expect("Failed copy song to dest");

            if delete_original {
                fs::remove_file(original_song_path).expect("Failed to remove original file");
            }

            songs.push(Song::new(PathNamed::new(new_song_path)));
        }

        self.save_contents();

        if !failed_import.is_empty() {
            Err(failed_import)
        } else {
            Ok(())
        }
    }

    pub fn start_play_song(&self, song_index: usize, volume: f32) {
        if let Some(music_manager) = self.music_manager.take() {
            let current_handle = music_manager.playing_handle;

            let old_music_command_tx = music_manager.music_command_tx;

            old_music_command_tx
                .send(MusicCommand::Stop)
                .expect("Current playing thread to be alive");

            current_handle.join().expect("Unwrap for panic in thread");
        }

        // write_rwlock(&self.queue).set_starting_index(song_index);

        let music_manager = MusicManager::try_create(Arc::clone(&self.songs), song_index, volume);

        self.music_manager.replace(music_manager);
    }

    pub fn get_music_manager(&self) -> Ref<'_, Option<MusicManager>> {
        self.music_manager.borrow()
    }

    pub fn stop_music(&self) {
        if let Some(music_manager) = self.music_manager.take() {
            music_manager.send_stop_command();
        }
    }

    pub(crate) fn import_existing_songs(&self, new_songs: &[Song]) {
        {
            let mut playlist_songs = write_rwlock(&self.songs);
            playlist_songs.reserve_exact(new_songs.len());

            for new_song in new_songs {
                playlist_songs.push(new_song.clone());
            }
        }

        self.save_contents();
    }

    /// Saves the list of songs to the file at `self.path_named`
    /// This does nothing if `self.variant` is [`PlaylistVariant::SongFolder`]

    fn save_contents(&self) {
        if matches!(self.variant, PlaylistVariant::SongFolder) {
            return;
        }

        let mut file = File::options()
            .write(true)
            .open(&self.path_named)
            .expect("Failed to open file in write mode");

        let songs = self.get_or_load_songs();

        let mut playlist_data = PlaylistData::new_capacity(songs.len());

        for song in songs.iter() {
            playlist_data.songs_file_names.push(song.file_name())
        }

        file.write_all(playlist_data.to_bb().buf())
            .expect("Unable to write playlist to file");
    }
}

impl NamedPathLike for Playlist {
    fn get_path_named(&self) -> &PathNamed {
        &self.path_named
    }
}
