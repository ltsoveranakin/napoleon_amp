mod data;
pub mod manager;
mod queue;

use crate::content::playlist::data::{PlaybackMode, PlaylistData};
use crate::content::playlist::manager::{MusicCommand, MusicManager};
use crate::content::song::song_data::get_song_data_from_song_file;
use crate::content::song::Song;
use crate::content::{unwrap_inner_ref, unwrap_inner_ref_mut, NamedPathLike, PathNamed};
use crate::paths::{song_file, songs_dir};
use crate::song_pool::SONG_POOL;
use crate::{read_rwlock, write_rwlock, ReadWrapper};
use rodio::Source;
use serbytes::prelude::SerBytes;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Write};
use std::ops::{Deref, RangeInclusive};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::{fs, io};

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

    pub fn get_selected_songs<'s>(&self, songs: &'s [Arc<Song>]) -> &'s [Arc<Song>] {
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
    path_named: RefCell<PathNamed>,
    songs: Arc<RwLock<Vec<Arc<Song>>>>,
    has_loaded_songs: Cell<bool>,
    music_manager: RefCell<Option<MusicManager>>,
    pub variant: PlaylistVariant,
    songs_filtered: Arc<RwLock<Vec<Arc<Song>>>>,
    selected_songs: RefCell<SelectedSongsVariant>,
    playlist_data: RefCell<Option<PlaylistData>>,
}

impl Playlist {
    fn new(path_named: PathNamed, variant: PlaylistVariant) -> Self {
        Self {
            path_named: RefCell::new(path_named),
            songs: Arc::new(RwLock::new(Vec::new())),
            has_loaded_songs: Cell::new(false),
            music_manager: RefCell::new(None),
            variant,
            songs_filtered: Arc::new(RwLock::new(Vec::new())),
            selected_songs: RefCell::new(SelectedSongsVariant::None),
            playlist_data: RefCell::new(None),
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

    pub fn get_or_load_songs(&self) -> ReadWrapper<'_, Vec<Arc<Song>>> {
        let songs_filtered = read_rwlock(&self.songs_filtered);

        if songs_filtered.is_empty() {
            self.get_or_load_songs_unfiltered()
        } else {
            songs_filtered
        }
    }

    pub fn get_or_load_songs_arc(&self) -> Arc<RwLock<Vec<Arc<Song>>>> {
        let songs_filtered = read_rwlock(&self.songs_filtered);

        if songs_filtered.is_empty() {
            self.get_or_load_songs_unfiltered();

            Arc::clone(&self.songs)
        } else {
            Arc::clone(&self.songs_filtered)
        }
    }

    pub fn get_or_load_songs_unfiltered(&self) -> ReadWrapper<'_, Vec<Arc<Song>>> {
        if self.has_loaded_songs.get() {
            read_rwlock(&self.songs)
        } else {
            let playlist_data = self.get_or_load_playlist_data();

            let mut loaded_song_file_names_backing = Vec::new();

            let song_file_names = match self.variant {
                PlaylistVariant::PlaylistFile => &playlist_data.song_file_names,

                PlaylistVariant::SongFolder => {
                    // TODO: preallocate loaded_song_file_names_backing
                    for song_dir in fs::read_dir(songs_dir()).expect("Song directory to exist") {
                        if let Ok(song_dir) = song_dir {
                            if let Some(ext) = song_dir.path().extension()
                                && ext != "snap"
                            {
                                loaded_song_file_names_backing.push(
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
                    }

                    &loaded_song_file_names_backing
                }
            };

            {
                let mut songs = write_rwlock(&self.songs);

                songs.reserve_exact(song_file_names.len());

                for song_name in song_file_names.iter() {
                    songs.push(SONG_POOL.get_song_by_name(song_name.clone()));
                }
            }

            self.has_loaded_songs.set(true);

            read_rwlock(&self.songs)
        }
    }

    pub fn set_search_query_filter(&self, search_str: &str) {
        let mut filtered_songs = write_rwlock(&self.songs_filtered);
        filtered_songs.clear();

        if search_str.is_empty() {
            return;
        }

        let parsed_search = if let Some(parsed_search) = ParsedSearch::parse_search_str(search_str)
        {
            parsed_search
        } else {
            return;
        };

        for song in self.get_or_load_songs_unfiltered().iter() {
            let song_data = song.get_or_load_song_data();
            let strings_to_search: &[&String] = match parsed_search.search_type {
                ParsedSearchType::Title => &[&song_data.title],

                ParsedSearchType::Album => &[&song_data.album],

                ParsedSearchType::Artist => &[&song_data.artist],

                ParsedSearchType::Any => &[&song_data.title, &song_data.album, &song_data.artist],
            };

            let mut valid_search = false;

            for str_search_to in strings_to_search {
                let search_to_lower = str_search_to.to_lowercase();
                if search_to_lower.contains(&parsed_search.value_lower) {
                    valid_search = !parsed_search.not;
                    break;
                } else if parsed_search.not {
                    valid_search = true;
                    break;
                }
            }

            if valid_search {
                filtered_songs.push(Arc::clone(song));
            }
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

    pub fn import_songs(
        &self,
        song_paths: &[PathBuf],
        delete_original: bool,
    ) -> Result<(), Vec<usize>> {
        let mut already_exists = Vec::new();
        {
            let mut songs = write_rwlock(&self.songs);

            songs.reserve_exact(song_paths.len());

            for (i, original_song_path) in song_paths.iter().enumerate() {
                let file_name = original_song_path
                    .file_name()
                    .expect("Path does not terminate in ..");

                let songs_dir = songs_dir();

                if !fs::exists(&songs_dir).expect("Verified existence of song directory") {
                    fs::create_dir_all(songs_dir).expect("Directories created");
                }

                let new_song_path = song_file(file_name);

                // TODO: handle if new song location already exists, also just handling all the errors here properly. esp invalid format

                if fs::exists(&new_song_path).expect(&format!(
                    "Unable to verify new song path does not exist at path: {:?}",
                    new_song_path
                )) {
                    already_exists.push(i);
                } else {
                    File::create(&new_song_path).expect(&format!(
                        "Unable to create new song file to copy to; path: {:?}",
                        new_song_path
                    ));

                    fs::copy(original_song_path, &new_song_path).expect("Failed copy song to dest");

                    if delete_original {
                        fs::remove_file(original_song_path)
                            .expect("Failed to remove original file");
                    }
                }

                let song =
                    SONG_POOL.get_song_by_name(file_name.to_str().expect("Valid utf8").to_string());

                {
                    let mut song_data = write_rwlock(&song.song_data);

                    get_song_data_from_song_file(&song, &mut song_data);
                }

                **write_rwlock(&song.has_loaded_song_data) = true;

                songs.push(song);
            }
        }

        self.save_contents();

        if !already_exists.is_empty() {
            println!("Imported songs and saved successfully, but some failed to import");
            Err(already_exists)
        } else {
            println!("Imported songs and saved successfully");
            Ok(())
        }
    }

    pub fn get_music_manager(&self) -> Ref<'_, Option<MusicManager>> {
        self.music_manager.borrow()
    }

    pub fn get_path_named_ref(&self) -> Ref<'_, PathNamed> {
        self.path_named.borrow()
    }

    pub fn rename(&self, new_name: String) -> io::Result<()> {
        self.path_named.borrow_mut().rename(new_name)
    }

    pub fn set_playback_mode(&self, playback_mode: PlaybackMode) {
        {
            let mut playlist_data = self.get_or_load_playlist_data_mut();

            playlist_data.playback_mode = playback_mode.into();
        }
        self.save_contents();
    }

    pub fn next_playback_mode(&self) {
        let next_playback_mode = match self.playback_mode() {
            PlaybackMode::Sequential => PlaybackMode::Shuffle,
            PlaybackMode::Shuffle => PlaybackMode::Sequential,
        };

        self.set_playback_mode(next_playback_mode);
    }

    pub fn playback_mode(&self) -> PlaybackMode {
        *self.get_or_load_playlist_data().playback_mode.deref()
    }

    pub fn delete_song(&self, song_index: usize) {
        {
            let mut songs = write_rwlock(&self.songs);
            let songs_filtered = read_rwlock(&self.songs_filtered);

            if songs_filtered.is_empty() {
                songs.remove(song_index);
            } else {
                let mut songs_filtered = write_rwlock(&self.songs_filtered);

                let song_removed = songs_filtered.remove(song_index);

                let mut index_to_remove = None;

                for (i, song) in songs.iter().enumerate() {
                    if song == &song_removed {
                        index_to_remove = Some(i);
                        break;
                    }
                }

                if let Some(index) = index_to_remove {
                    songs.remove(index);
                }
            }
        }

        self.save_contents();
    }

    /// Returns `None` if music manager is `None` (no song is playing) otherwise returns the index
    /// of the next song that will be played (with respect to the queue)

    pub fn get_current_song_playing_index(&self) -> Option<usize> {
        self.get_music_manager()
            .as_ref()
            .map(|manager| manager.queue().get_current_song_index())
    }

    pub(crate) fn start_play_song(&self, song_index: usize, volume: f32) {
        if let Some(music_manager) = self.music_manager.take() {
            let current_handle = music_manager.playing_handle;

            let old_music_command_tx = music_manager.music_command_tx;

            old_music_command_tx
                .send(MusicCommand::Stop)
                .expect("Current playing thread to be alive");

            current_handle.join().expect("Unwrap for panic in thread");
        }

        let playlist_data = self.get_or_load_playlist_data();

        let music_manager = MusicManager::try_create(
            self.get_or_load_songs_arc(),
            song_index,
            volume,
            *playlist_data.playback_mode,
        );

        self.music_manager.replace(music_manager);
    }

    pub(crate) fn stop_music(&self) {
        if let Some(music_manager) = self.music_manager.take() {
            music_manager.send_stop_command();
        }
    }

    pub(crate) fn import_existing_songs(&self, new_songs: &[Arc<Song>]) {
        {
            let mut playlist_songs = write_rwlock(&self.songs);
            playlist_songs.reserve_exact(new_songs.len());

            for new_song in new_songs {
                playlist_songs.push(Arc::clone(new_song));
            }
        }

        self.save_contents();
    }

    fn set_selected_songs(&self, selected_songs: SelectedSongsVariant) {
        *self.selected_songs.borrow_mut() = selected_songs;
    }

    fn get_or_load_playlist_data(&self) -> Ref<'_, PlaylistData> {
        let playlist_data = self.playlist_data.borrow();

        if playlist_data.is_some() {
            unwrap_inner_ref(playlist_data)
        } else {
            drop(playlist_data);

            self.load_playlist_data_from_file();

            unwrap_inner_ref(self.playlist_data.borrow())
        }
    }

    fn get_or_load_playlist_data_mut(&self) -> RefMut<'_, PlaylistData> {
        let playlist_data = self.playlist_data.borrow_mut();

        if playlist_data.is_some() {
            unwrap_inner_ref_mut(playlist_data)
        } else {
            drop(playlist_data);

            self.load_playlist_data_from_file();

            unwrap_inner_ref_mut(self.playlist_data.borrow_mut())
        }
    }

    fn load_playlist_data_from_file(&self) {
        let playlist_data = if let Ok(file_buf) = fs::read(&self.path_named.borrow().path) {
            PlaylistData::from_vec(file_buf).unwrap_or(PlaylistData::new_empty())
        } else {
            PlaylistData::new_empty()
        };

        self.playlist_data.replace(Some(playlist_data));
    }

    /// Saves the list of songs to the file at `self.path_named`
    /// This does nothing if `self.variant` is [`PlaylistVariant::SongFolder`]

    fn save_contents(&self) {
        if matches!(self.variant, PlaylistVariant::SongFolder) {
            return;
        }

        let mut file = File::options()
            .write(true)
            .open(&*self.path_named.borrow())
            .expect("Failed to open file in write mode");

        let songs = self.get_or_load_songs();

        let mut playlist_data = self.get_or_load_playlist_data_mut();

        playlist_data.song_file_names.clear();

        for song in songs.iter() {
            playlist_data.song_file_names.push(song.file_name())
        }

        file.write_all(playlist_data.to_bb().buf())
            .expect("Unable to write playlist to file");
    }
}

#[derive(Debug)]
struct ParsedSearch {
    value_lower: String,
    search_type: ParsedSearchType,
    not: bool,
}

#[derive(Debug)]
enum ParsedSearchType {
    Title,
    Artist,
    Album,
    Any,
}

struct Terms {
    search_type: String,
    search_value: String,
    not: bool,
}

impl ParsedSearch {
    fn parse_search_str(search_str: &str) -> Option<Self> {
        if !search_str.starts_with("$") {
            Some(ParsedSearch {
                value_lower: search_str.to_lowercase(),
                search_type: ParsedSearchType::Any,
                not: false,
            })
        } else if let Some(Terms {
            search_type,
            search_value,
            not,
        }) = Self::try_get_terms(search_str)
        {
            let parsed_search_type = match &*search_type {
                "title" => Some(ParsedSearchType::Title),

                "artist" => Some(ParsedSearchType::Artist),

                "album" => Some(ParsedSearchType::Album),

                "any" => Some(ParsedSearchType::Any),

                _ => None,
            };

            parsed_search_type.map(|search_type| ParsedSearch {
                value_lower: search_value,
                search_type,
                not,
            })
        } else {
            None
        }
    }

    fn try_get_terms(search_str: &str) -> Option<Terms> {
        let search_spl = &mut search_str[1..].split(":");

        let mut search_type = search_spl.next()?.to_lowercase();
        let search_value = search_spl.next()?.to_lowercase();

        let not = if search_type.starts_with("!") {
            search_type.remove(0);
            true
        } else {
            false
        };

        Some(Terms {
            search_type,
            search_value,
            not,
        })
    }
}

// impl NamedPathLike for Playlist {
//     fn get_path_named(&self) -> &PathNamed {
//         &self.path_named
//     }
// }
