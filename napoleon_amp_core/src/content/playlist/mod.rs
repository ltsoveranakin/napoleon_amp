pub mod data;
pub mod manager;
mod queue;
mod song_list;

use crate::content::playlist::data::{PlaybackMode, PlaylistData};
use crate::content::playlist::manager::MusicManager;
use crate::content::playlist::song_list::{SongList, SongVec, SortBy, SortByVariant};
use crate::content::song::song_data::SongData;
use crate::content::song::Song;
use crate::content::{unwrap_inner_ref, unwrap_inner_ref_mut, NamedPathLike, PathNamed};
use crate::id_generator::{Id, IdGenerator};
use crate::paths::{
    song_audio_file_v2, songs_audio_dir_v2, songs_data_dir_v2, songs_dir_v1, SONG_DATA_EXT_NO_PER,
};
use crate::song_pool::SONG_POOL;
use crate::{read_rwlock, write_rwlock};
use rodio::Source;
use serbytes::prelude::SerBytes;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::HashSet;
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
    songs: RefCell<SongList>,
    has_loaded_songs: Cell<bool>,
    music_manager: RefCell<Option<MusicManager>>,
    pub variant: PlaylistVariant,
    songs_filtered: SongVec,
    selected_songs: RefCell<SelectedSongsVariant>,
    playlist_data: RefCell<Option<PlaylistData>>,
}

impl Playlist {
    fn new(path_named: PathNamed, variant: PlaylistVariant) -> Self {
        Self {
            path_named: RefCell::new(path_named),
            songs: RefCell::new(SongList::new()),
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
        Self::new_folder(PathNamed::new(songs_dir_v1()))
    }

    /// Gets the songs in the current playlist, with the filter if one is enabled

    pub fn get_or_load_songs(&self) -> SongVec {
        let songs_filtered = read_rwlock(&self.songs_filtered);

        if songs_filtered.is_empty() {
            self.get_or_load_songs_unfiltered()
        } else {
            Arc::clone(&self.songs_filtered)
        }
    }

    // fn get_or_load_songs_arc(&self) -> Arc<RwLock<Vec<Arc<Song>>>> {
    //     let songs_filtered = read_rwlock(&self.songs_filtered);
    //
    //     if songs_filtered.is_empty() {
    //         self.get_or_load_songs_unfiltered();
    //
    //         self.songs.borrow().songs_arc()
    //     } else {
    //         Arc::clone(&self.songs_filtered)
    //     }
    // }

    pub fn get_or_load_songs_unfiltered(&self) -> SongVec {
        if self.has_loaded_songs.get() {
            self.songs.borrow().songs_arc()
        } else {
            let playlist_data = self.get_or_load_playlist_data();

            let mut loaded_song_ids_backing = Vec::new();

            let song_ids = match self.variant {
                PlaylistVariant::PlaylistFile => &playlist_data.song_ids,

                PlaylistVariant::SongFolder => {
                    fs::create_dir_all(songs_data_dir_v2()).expect("Create songs_data_dir_v2");
                    // TODO: preallocate loaded_song_file_names_backing
                    for song_dir in fs::read_dir(songs_data_dir_v2())
                        .expect("Song directory to exist")
                        .flatten()
                    {
                        let mut song_file_path = song_dir.path();

                        song_file_path.set_extension("");

                        let song_file_name = song_file_path
                            .file_name()
                            .expect("Get song file name")
                            .to_str()
                            .expect("Valid utf8 for song file");

                        let id = Id::try_from_str(song_file_name).expect("Parse valid id");

                        loaded_song_ids_backing.push(id);
                    }

                    &loaded_song_ids_backing
                }
            };

            let mut songs = self.songs.borrow_mut();

            songs.push_songs(song_ids);

            self.has_loaded_songs.set(true);

            songs.songs_arc()
        }
    }

    pub fn set_search_query_filter(&self, search_str: &str) {
        self.set_selected_songs(SelectedSongsVariant::None);

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

        for song in read_rwlock(&self.get_or_load_songs_unfiltered()).iter() {
            let song_data = song.get_song_data();
            let strings_to_search: &[&String] = match parsed_search.search_type {
                ParsedSearchType::Title => &[&song_data.title],

                ParsedSearchType::Album => &[&song_data.album],

                ParsedSearchType::Artist => &[&song_data.artist.artist_string],

                ParsedSearchType::Any => &[
                    &song_data.title,
                    &song_data.album,
                    &song_data.artist.artist_string,
                ],
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
        let songs_lock = self.get_or_load_songs();
        let songs = read_rwlock(&songs_lock);

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
        if index < read_rwlock(&self.get_or_load_songs()).len() {
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
            let mut songs = self.songs.borrow_mut();

            songs.reserve(song_paths.len());

            let dirs_to_create = [songs_audio_dir_v2(), songs_data_dir_v2()];

            for dir in dirs_to_create {
                if !fs::exists(&dir).expect("Verified existence of song directory") {
                    fs::create_dir_all(dir).expect("Directories created");
                }
            }

            let mut generator = IdGenerator::new();

            for (i, original_song_path) in song_paths.iter().enumerate() {
                if original_song_path.extension().unwrap() == SONG_DATA_EXT_NO_PER {
                    continue;
                }

                let mut original_song_path1 = original_song_path.clone();

                original_song_path1.set_extension("");

                let original_song_file_name = original_song_path1
                    .file_name()
                    .expect("Valid filename")
                    .to_str()
                    .expect("Valid osstr")
                    .to_string();

                let song_id = generator.generate_new_id();

                let new_song_audio_path = song_audio_file_v2(&song_id);

                // TODO: handle if new song location already exists, also just handling all the errors here properly. esp invalid format

                if fs::exists(&new_song_audio_path).expect(&format!(
                    "Unable to verify new song path does not exist at path: {:?}",
                    new_song_audio_path
                )) {
                    already_exists.push(i);
                } else {
                    File::create(&new_song_audio_path).expect(&format!(
                        "Unable to create new song file to copy to; path: {:?}",
                        new_song_audio_path
                    ));

                    fs::copy(original_song_path, &new_song_audio_path)
                        .expect("Failed copy song to dest");

                    if delete_original {
                        fs::remove_file(original_song_path)
                            .expect("Failed to remove original file");
                    }
                }

                songs.push_new_song(song_id, &original_song_file_name);
            }
        }

        SONG_POOL
            .save_registered_songs()
            .expect("save registered songs");

        self.sort_songs_and_save();

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
        self.get_or_load_playlist_data().playback_mode
    }

    pub fn delete_song(&self, song_index: usize) {
        {
            let mut song_list = self.songs.borrow_mut();
            let songs_filtered = read_rwlock(&self.songs_filtered);

            if songs_filtered.is_empty() {
                song_list.remove_song_at(song_index);
            } else {
                let mut songs_filtered = write_rwlock(&self.songs_filtered);

                let song_removed = songs_filtered.remove(song_index);

                let mut index_to_remove = None;

                for (i, song) in song_list.songs().iter().enumerate() {
                    if song == &song_removed {
                        index_to_remove = Some(i);
                        break;
                    }
                }

                if let Some(index) = index_to_remove {
                    song_list.remove_song_at(index);
                }
            }
        }

        self.save_contents();
    }

    /// Returns `None` if music manager is `None` (no song is playing) otherwise returns the index
    /// of the next song that will be played (with respect to the queue)

    pub fn get_current_song_playing(&self) -> Option<Arc<Song>> {
        self.get_music_manager()
            .as_ref()
            .map(|manager| manager.get_song_status().song)
    }

    pub fn get_volume(&self) -> f32 {
        self.get_or_load_playlist_data().volume
    }

    pub fn set_volume(&self, mut volume: f32) {
        volume = volume.clamp(0.0, 1.0);
        if let Some(manager) = &*self.get_music_manager() {
            manager.set_volume(volume);
        }

        self.get_or_load_playlist_data_mut().volume = volume;

        self.save_contents();
    }

    pub fn sort_by_and_save(&self, sort_by: SortBy) {
        self.get_or_load_playlist_data_mut().sort_by = sort_by.into();
        self.songs.borrow_mut().sort_songs(sort_by);

        self.save_contents();
    }

    fn sort_songs_and_save(&self) {
        self.songs
            .borrow_mut()
            .sort_songs(self.get_or_load_playlist_data().sort_by);

        self.save_contents();
    }

    pub fn get_sorting_by(&self) -> SortBy {
        self.get_or_load_playlist_data().sort_by
    }

    pub fn next_sorting_by(&self) {
        let mut sort_by = self.get_or_load_playlist_data().sort_by;

        let next_sort_by = match sort_by.sort_by_variant {
            SortByVariant::Title => SortByVariant::Artist,

            SortByVariant::Artist => SortByVariant::Album,

            SortByVariant::Album => SortByVariant::Title,
        };

        sort_by.sort_by_variant = next_sort_by;

        self.sort_by_and_save(sort_by);
    }

    pub fn get_artist_list(&self) -> Vec<String> {
        self.get_string_list(|song_data| &song_data.artist.artist_string)
    }

    pub fn get_album_list(&self) -> Vec<String> {
        self.get_string_list(|song_data| &song_data.album)
    }

    fn get_string_list(&self, f: impl Fn(&SongData) -> &String) -> Vec<String> {
        let mut string_set = HashSet::new();

        for song in read_rwlock(&self.get_or_load_songs_unfiltered()).iter() {
            let song_data = song.get_song_data();
            let string_ref = f(&song_data);

            if !string_set.contains(string_ref) {
                string_set.insert(string_ref.clone());
            }
        }

        string_set.into_iter().collect()
    }

    pub(crate) fn start_play_song(&self, song_index: usize) {
        if let Some(music_manager) = self.music_manager.take() {
            music_manager.send_stop_command();

            let current_handle = music_manager.playing_handle;

            current_handle.join().expect("Unwrap for panic in thread");
        }

        let playlist_data = self.get_or_load_playlist_data();

        let music_manager = MusicManager::try_create(
            self.get_or_load_songs_unfiltered(),
            song_index,
            playlist_data.volume,
            playlist_data.playback_mode,
        );

        self.music_manager.replace(music_manager);
    }

    // pub(crate) fn start_play_playlist(&self, ) {
    //
    // }

    pub(crate) fn stop_music(&self) {
        if let Some(music_manager) = self.music_manager.take() {
            music_manager.send_stop_command();
        }
    }

    pub(crate) fn import_existing_songs(&self, new_songs: &[Arc<Song>]) {
        {
            let mut songs = self.songs.borrow_mut();

            songs.push_songs_arc_list(new_songs);
        }

        self.sort_songs_and_save();
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

        let songs_unfiltered = self.get_or_load_songs_unfiltered();

        let songs = read_rwlock(&songs_unfiltered);

        let mut playlist_data = self.get_or_load_playlist_data_mut();

        playlist_data.song_ids.clear();
        playlist_data.song_ids.reserve(songs.len());

        for song in songs.iter() {
            playlist_data.song_ids.push(song.id);
        }

        playlist_data
            .write_to_file_path(&*self.path_named.borrow())
            .expect("Write playlist data to file");
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
