use crate::content::folder::Folder;
use crate::content::folder::content_pool::CONTENT_POOL;
use crate::content::playlist::data::{
    PlaybackMode, PlaylistContentData, PlaylistSongListData, PlaylistUserData,
};
use crate::content::playlist::manager::MusicManager;
use crate::content::playlist::song_list::{SongList, SongVec, SortBy};
use crate::content::playlist::{
    ParsedSearch, ParsedSearchType, Playlist, PlaylistParent, SelectedSongsVariant,
};
use crate::content::song::Song;
use crate::paths::SONG_DATA_EXT_NO_PER;
use crate::paths::song::{song_audio_file_v2, songs_audio_dir_v2, songs_data_dir_v2};
use crate::song_pool::SONG_POOL;
use crate::{read_rwlock, write_rwlock};
use simple_id::prelude::{Id, SmallRngIdGenerator};
use std::cell::{Cell, OnceCell, Ref, RefCell, RefMut};
use std::fs::File;
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::{fs, io};

/// The type of playlist this will attempt to load songs from

#[derive(Debug)]
pub enum StandardPlaylistVariant {
    /// Will attempt to load all songs that have been registered
    AllSongs,
    /// Will attempt to load all songs in the playlist data file that matches the current id
    Normal,
}

#[derive(Debug)]
pub struct StandardPlaylist {
    pub id: Id,
    parent: PlaylistParent,
    songs: RefCell<SongList>,
    has_loaded_songs: Cell<bool>,
    music_manager: RefCell<Option<MusicManager>>,
    pub variant: StandardPlaylistVariant,
    songs_filtered: SongVec,
    selected_songs: RefCell<SelectedSongsVariant>,
    playlist_user_data: OnceCell<RefCell<PlaylistUserData>>,
    playlist_song_list_data: OnceCell<RefCell<PlaylistSongListData>>,
    total_length: RefCell<Option<u32>>,
    current_search_str: RefCell<String>,
}

impl StandardPlaylist {
    pub(crate) fn new(id: Id, variant: StandardPlaylistVariant, parent: &Rc<Folder>) -> Self {
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
            total_length: RefCell::new(None),
            current_search_str: RefCell::new(String::new()),
        }
    }

    pub(crate) fn new_file(id: Id, parent: &Rc<Folder>) -> Self {
        Self::new(id, StandardPlaylistVariant::Normal, parent)
    }

    /// Sets the selected range of the playlist
    /// Errors under 3 conditions.
    ///
    /// If end is less than start.
    /// If start is greater than or equal to (potentially filtered songs) length.
    /// If end is greater than or equal to (potentially filtered songs) length.

    pub fn select_range(&self, range: RangeInclusive<usize>) -> Result<(), ()> {
        let songs_lock = self.get_song_vec();
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

    pub fn set_playback_mode(&self, playback_mode: PlaybackMode) {
        {
            let mut playlist_data = self.get_user_data_mut();

            playlist_data.playback_mode = playback_mode.into();
        }
        self.save_user_data();
    }

    pub fn playback_mode(&self) -> PlaybackMode {
        self.get_user_data().playback_mode
    }

    pub fn get_volume(&self) -> f32 {
        self.get_user_data().volume
    }

    fn set_selected_songs(&self, selected_songs: SelectedSongsVariant) {
        *self.selected_songs.borrow_mut() = selected_songs;
    }

    pub fn get_name(&self) -> Ref<'_, String> {
        Ref::map(self.get_user_data(), |d| &d.content_data.name)
    }

    fn get_song_list_data_refcell(&self) -> &RefCell<PlaylistSongListData> {
        self.playlist_song_list_data.get_or_init(|| {
            let song_list_data = CONTENT_POOL
                .get_playlist_song_list_data(self.id)
                .unwrap_or_else(|_| PlaylistSongListData {
                    song_ids: Vec::new(),
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
    /// This does nothing if `self.variant` is [`StandardPlaylistVariant::AllSongs`] or if this is the 'all songs' playlist

    fn save_user_data(&self) {
        if matches!(self.variant, StandardPlaylistVariant::AllSongs) || self.id == Id::ZERO {
            return;
        }

        let playlist_data = self.get_user_data();
        playlist_data
            .save_data(self.id)
            .expect("Write playlist user data to file");
    }

    fn save_song_list(&self) {
        if matches!(self.variant, StandardPlaylistVariant::AllSongs) || self.id == Id::ZERO {
            return;
        }

        let songs_unfiltered = self.get_song_vec_unfiltered();

        let songs = read_rwlock(&songs_unfiltered);

        let mut song_list = self.get_song_list_mut();

        song_list.song_ids.clear();
        song_list.song_ids.reserve(songs.len());

        for song in songs.iter() {
            song_list.song_ids.push(song.id);
        }

        song_list
            .save_data(self.id)
            .expect("Write playlist song list data to file");
    }
}

impl Playlist for StandardPlaylist {
    fn id(&self) -> Id {
        self.id
    }

    fn get_song_vec(&self) -> SongVec {
        if self.current_search_str.borrow().is_empty() {
            self.get_song_vec_unfiltered()
        } else {
            Arc::clone(&self.songs_filtered)
        }
    }

    fn get_user_data_ref_cell(&self) -> &RefCell<PlaylistUserData> {
        self.playlist_user_data.get_or_init(|| {
            let playlist_data = CONTENT_POOL
                .get_playlist_user_data(self.id)
                .unwrap_or_else(|_| {
                    PlaylistUserData::new(PlaylistContentData::new(
                        "Deleted Playlist".to_string(),
                        self.parent.id,
                    ))
                });

            RefCell::new(playlist_data)
        })
    }

    fn start_play_song(&self, song_index: usize) {
        if let Some(music_manager) = self.music_manager.take() {
            music_manager.send_stop_command();

            let current_handle = music_manager.playing_handle;

            current_handle.join().expect("Unwrap for panic in thread");
        }

        let playlist_data = self.get_user_data();

        let actual_index = if !read_rwlock(&self.songs_filtered).is_empty() {
            let songs_vec = self.get_song_vec();
            let songs = read_rwlock(&songs_vec);
            let song_to_start_with = &songs[song_index];

            let mut index = None;

            for (i, song) in read_rwlock(&self.get_song_vec_unfiltered())
                .iter()
                .enumerate()
            {
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
            self.get_song_vec_unfiltered(),
            actual_index,
            playlist_data.volume,
            playlist_data.playback_mode,
        );

        self.music_manager.replace(music_manager);
    }

    fn stop_music(&self) {
        if let Some(music_manager) = self.music_manager.take() {
            music_manager.send_stop_command();
        }
    }

    fn get_music_manager(&self) -> Ref<'_, Option<MusicManager>> {
        self.music_manager.borrow()
    }

    fn set_volume(&self, mut volume: f32) {
        volume = volume.clamp(0.0, 1.0);

        if let Some(manager) = &*self.get_music_manager() {
            manager.set_volume(volume);
        }

        self.get_user_data_mut().volume = volume;

        self.save_user_data();
    }

    fn delete_song(&self, song_index: usize) {
        if matches!(self.variant, StandardPlaylistVariant::AllSongs) {
            return;
        }

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

    fn select_all(&self) {
        self.set_selected_songs(SelectedSongsVariant::All);
    }

    fn import_existing_songs(&self, new_songs: &[Arc<Song>]) {
        {
            let mut songs = self.songs.borrow_mut();

            songs.push_songs_arc_list(new_songs);
        }

        self.sort_songs();
    }

    fn get_selected_songs(&self) -> SelectedSongsVariant {
        self.selected_songs.borrow().clone()
    }

    fn get_song_vec_unfiltered(&self) -> SongVec {
        if self.has_loaded_songs.get() {
            self.songs.borrow().songs_arc()
        } else {
            let song_list_data = self.get_song_list_data();

            let loaded_song_ids_backing;

            let (song_ids, should_sort) = match self.variant {
                StandardPlaylistVariant::Normal => (&song_list_data.song_ids, false),

                StandardPlaylistVariant::AllSongs => {
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

    fn get_total_song_duration(&self) -> u32 {
        *self.total_length.borrow_mut().get_or_insert_with(|| {
            let mut total_length = 0;

            for song in read_rwlock(&self.get_song_vec_unfiltered()).iter() {
                total_length += song.get_song_data().meta.as_ref().unwrap().length;
            }

            total_length
        })
    }

    fn import_songs(
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

        self.sort_songs();

        if !already_exists.is_empty() {
            println!("Imported songs and saved successfully, but some failed to import");
            Err(already_exists)
        } else {
            println!("Imported songs and saved successfully");
            Ok(())
        }
    }

    fn set_search_query_filter(&self, search_str: &str) {
        self.set_selected_songs(SelectedSongsVariant::None);

        *self.current_search_str.borrow_mut() = search_str.to_string();

        if search_str.is_empty() {
            return;
        }

        let mut filtered_songs = write_rwlock(&self.songs_filtered);
        filtered_songs.clear();

        let parsed_search = if let Some(parsed_search) = ParsedSearch::parse_search_str(search_str)
        {
            parsed_search
        } else {
            return;
        };

        for song in read_rwlock(&self.get_song_vec_unfiltered()).iter() {
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

    fn get_current_song_playing(&self) -> Option<Arc<Song>> {
        self.get_music_manager()
            .as_ref()
            .map(|manager| manager.get_song_status().song)
    }

    fn select_single(&self, index: usize) {
        if index < read_rwlock(&self.get_song_vec()).len() {
            self.set_selected_songs(SelectedSongsVariant::Single(index));
        }
    }

    fn rename(&self, new_name: String) -> io::Result<()> {
        let mut pl_data = self.get_user_data_mut();

        pl_data.content_data.name = new_name;

        pl_data.save_data(self.id)
    }

    fn sort_songs(&self) {
        self.songs
            .borrow_mut()
            .sort_songs(self.get_user_data().sort_by);

        self.save_song_list();
    }
}

impl PartialEq for StandardPlaylist {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for StandardPlaylist {}
