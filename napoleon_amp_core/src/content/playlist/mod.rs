pub mod data;
pub mod manager;
pub mod queue;
mod song_list;

use crate::content::folder::content_pool::CONTENT_POOL;
use crate::content::playlist::data::{PlaybackMode, PlaylistContentData, PlaylistSongListData, PlaylistUserData};
use crate::content::playlist::manager::MusicManager;
use crate::content::playlist::song_list::{SongList, SongVec, SortBy, SortByVariant};
use crate::content::song::song_data::SongData;
use crate::content::song::Song;
use crate::paths::song::{song_audio_file_v2, songs_audio_dir_v2, songs_data_dir_v2};
use crate::paths::SONG_DATA_EXT_NO_PER;
use crate::song_pool::SONG_POOL;
use crate::{read_rwlock, write_rwlock};

use std::cell::{Cell, OnceCell, Ref, RefCell, RefMut};
use std::collections::HashSet;

use std::fs::File;

use crate::content::folder::Folder;
use serbytes::prelude::SerBytes;
use simple_id::prelude::{Id, SmallRngIdGenerator};
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::rc::{Rc, Weak};
use std::sync::{Arc, RwLock};
use std::{fs, io};

/// The type of playlist this will attempt to load songs from

#[derive(Debug)]
pub enum PlaylistVariant {
    /// Will attempt to load all songs that have been registered
    AllSongs,
    /// Will attempt to load all songs in the playlist data file that matches the current id
    Normal,
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
pub(crate) struct PlaylistParent {
    id: Id,
    parent: Weak<Folder>,
}

#[derive(Debug)]
pub struct Playlist {
    pub id: Id,
    parent: PlaylistParent,
    songs: RefCell<SongList>,
    has_loaded_songs: Cell<bool>,
    music_manager: RefCell<Option<MusicManager>>,
    pub variant: PlaylistVariant,
    songs_filtered: SongVec,
    selected_songs: RefCell<SelectedSongsVariant>,
    playlist_user_data: OnceCell<RefCell<PlaylistUserData>>,
    playlist_song_list_data: OnceCell<RefCell<PlaylistSongListData>>,
}

impl Playlist {
    pub(crate) fn new(id: Id, variant: PlaylistVariant, parent: &Rc<Folder>) -> Self {
        Self {
            id,
            parent: PlaylistParent {
                id: parent.id,
                parent: Rc::downgrade(parent),
            },
            songs: RefCell::new(SongList::new()),
            has_loaded_songs: Cell::new(false),
            music_manager: RefCell::new(None),
            variant,
            songs_filtered: Arc::new(RwLock::new(Vec::new())),
            selected_songs: RefCell::new(SelectedSongsVariant::None),
            playlist_user_data: OnceCell::new(),
            playlist_song_list_data: OnceCell::new(),
        }
    }

    pub(super) fn new_file(id: Id, parent: &Rc<Folder>) -> Self {
        Self::new(id, PlaylistVariant::Normal, parent)
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

    pub fn get_or_load_songs_unfiltered(&self) -> SongVec {
        if self.has_loaded_songs.get() {
            self.songs.borrow().songs_arc()
        } else {
            let song_list_data = self.get_song_list_data();

            let loaded_song_ids_backing;

            let (song_ids, should_sort) = match self.variant {
                PlaylistVariant::Normal => (&song_list_data.song_ids, false),

                PlaylistVariant::AllSongs => {
                    loaded_song_ids_backing = SONG_POOL
                        .get_registered_songs()
                        .name_map
                        .values()
                        .copied()
                        .collect();

                    (&loaded_song_ids_backing, true)
                }
            };

            let mut songs = self.songs.borrow_mut();

            songs.push_songs(song_ids);

            if should_sort {
                songs.sort_songs(SortBy::default());
            }

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

                ParsedSearchType::Artist => &[&song_data.artist.full_artist_string],

                ParsedSearchType::UserTag => &[&song_data.user_tag.inner],

                ParsedSearchType::Any => &[
                    &song_data.title,
                    &song_data.album,
                    &song_data.artist.full_artist_string,
                    &song_data.user_tag.inner,
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

            let mut generator = SmallRngIdGenerator::default();

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

                songs
                    .push_new_song(song_id, &original_song_file_name)
                    .expect("Push new song");
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

    // pub fn get_path_named_ref(&self) -> Ref<'_, PathNamed> {
    //     self.path_named.borrow()
    // }

    pub fn rename(&self, new_name: String) -> io::Result<()> {
        let mut pl_data = self.get_or_load_user_data_mut();

        pl_data.content_data.name = new_name;

        pl_data.save_data(self.id)
    }

    pub fn set_playback_mode(&self, playback_mode: PlaybackMode) {
        {
            let mut playlist_data = self.get_or_load_user_data_mut();

            playlist_data.playback_mode = playback_mode.into();
        }
        self.save_user_datab();
    }

    pub fn next_playback_mode(&self) {
        let next_playback_mode = match self.playback_mode() {
            PlaybackMode::Sequential => PlaybackMode::Shuffle,
            PlaybackMode::Shuffle => PlaybackMode::Sequential,
        };

        self.set_playback_mode(next_playback_mode);
    }

    pub fn playback_mode(&self) -> PlaybackMode {
        self.get_or_load_user_data().playback_mode
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

        self.save_song_list();
    }

    /// Returns `None` if music manager is `None` (no song is playing) otherwise returns the index
    /// of the next song that will be played (with respect to the queue)

    pub fn get_current_song_playing(&self) -> Option<Arc<Song>> {
        self.get_music_manager()
            .as_ref()
            .map(|manager| manager.get_song_status().song)
    }

    pub fn get_volume(&self) -> f32 {
        self.get_or_load_user_data().volume
    }

    pub fn set_volume(&self, mut volume: f32) {
        volume = volume.clamp(0.0, 1.0);
        if let Some(manager) = &*self.get_music_manager() {
            manager.set_volume(volume);
        }

        self.get_or_load_user_data_mut().volume = volume;

        self.save_user_datab();
    }

    pub fn sort_by_and_save(&self, sort_by: SortBy) {
        self.get_or_load_user_data_mut().sort_by = sort_by.into();
        self.sort_songs_and_save();
    }

    fn sort_songs_and_save(&self) {
        self.songs
            .borrow_mut()
            .sort_songs(self.get_or_load_user_data().sort_by);

        self.save_song_list();
    }

    pub fn get_sorting_by(&self) -> SortBy {
        self.get_or_load_user_data().sort_by
    }

    pub fn next_sorting_by(&self) {
        let mut sort_by = self.get_or_load_user_data().sort_by;

        let next_sort_by = match sort_by.sort_by_variant {
            SortByVariant::Title => SortByVariant::Artist,

            SortByVariant::Artist => SortByVariant::Album,

            SortByVariant::Album => SortByVariant::Rating,

            SortByVariant::Rating => SortByVariant::Title,
        };

        sort_by.sort_by_variant = next_sort_by;

        self.sort_by_and_save(sort_by);
    }

    pub fn get_artist_list(&self) -> Vec<String> {
        self.get_string_list(|song_data| &song_data.artist.full_artist_string)
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

        let playlist_data = self.get_or_load_user_data();

        let actual_index =

            if !read_rwlock(&self.songs_filtered).is_empty() {
                let songs_vec = self.get_or_load_songs();
                let songs = read_rwlock(&songs_vec);
                let song_to_start_with = &songs[song_index];

                let mut index = None;

                for (i, song) in read_rwlock(&self.get_or_load_songs_unfiltered()).iter().enumerate() {
                    if song == song_to_start_with {
                        index = Some(i);
                        break;
                    }
                }

                index.expect("Song in filtered but now unfiltered (HOW???)")
            } else {
                song_index
            };

        let music_manager = MusicManager::try_create(
            self.get_or_load_songs_unfiltered(),
            actual_index,
            playlist_data.volume,
            playlist_data.playback_mode,
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
            let mut songs = self.songs.borrow_mut();

            songs.push_songs_arc_list(new_songs);
        }

        self.sort_songs_and_save();
    }

    fn set_selected_songs(&self, selected_songs: SelectedSongsVariant) {
        *self.selected_songs.borrow_mut() = selected_songs;
    }

    fn get_or_load_user_data_refcell(&self) -> &RefCell<PlaylistUserData> {
        self.playlist_user_data.get_or_init(|| {
            let playlist_data = CONTENT_POOL
                .get_playlist_user_data(self.id)
                .unwrap_or_else(|_| {
                    PlaylistUserData::new(PlaylistContentData::new("Deleted Playlist".to_string(), self.parent.id))
                });

            RefCell::new(playlist_data)
        })
    }

    fn get_or_load_user_data_mut(&self) -> RefMut<'_, PlaylistUserData> {
        self.get_or_load_user_data_refcell().borrow_mut()
    }

    pub fn get_or_load_user_data(&self) -> Ref<'_, PlaylistUserData> {
        self.get_or_load_user_data_refcell().borrow()
    }

    pub fn get_name(&self) -> Ref<'_, String> {
        Ref::map(self.get_or_load_user_data(), |d| &d.content_data.name)
    }

    fn get_song_list_data_refcell(&self) -> &RefCell<PlaylistSongListData> {
        self.playlist_song_list_data.get_or_init(|| {
            let song_list_data = CONTENT_POOL
                .get_playlist_song_list_data(self.id)
                .unwrap_or_else(|_| {
                    PlaylistSongListData {
                        song_ids: Vec::new()
                    }
                });

            RefCell::new(song_list_data)
        })
    }

    fn get_song_list_data(&self) -> Ref<'_, PlaylistSongListData> {
        self.get_song_list_data_refcell().borrow()
    }

    fn get_song_list_mut(&self) -> RefMut<'_, PlaylistSongListData> {
        self.get_song_list_data_refcell().borrow_mut()
    }

    /// Saves the list of songs to the file at `self.path_named`
    /// This does nothing if `self.variant` is [`PlaylistVariant::AllSongs`] or if this is the 'all songs' playlist

    fn save_user_datab(&self) {
        if matches!(self.variant, PlaylistVariant::AllSongs) || self.id == Id::ZERO {
            return;
        }

        let playlist_data = self.get_or_load_user_data();
        playlist_data
            .save_data(self.id)
            .expect("Write playlist user data to file");
    }

    fn save_song_list(&self) {
        if matches!(self.variant, PlaylistVariant::AllSongs) || self.id == Id::ZERO {
            return;
        }

        let songs_unfiltered = self.get_or_load_songs_unfiltered();

        let songs = read_rwlock(&songs_unfiltered);

        let mut song_list = self.get_song_list_mut();

        song_list.song_ids.clear();
        song_list.song_ids.reserve(songs.len());

        for song in songs.iter() {
            song_list.song_ids.push(song.id);
        }

        song_list.save_data(self.id).expect("Write playlist song list data to file");
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
    UserTag,
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

                "utag" => Some(ParsedSearchType::UserTag),

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
