mod dynamic_playlist_data;
mod filter;
mod rules;

use crate::content::playlist::playlists::dynamic_playlist::dynamic_playlist_data::VersionedDynamicPlaylistData;
use crate::content::playlist::{InnerPlaylist, Playlist};
use serbytes::prelude::SerBytes;

#[derive(Debug)]
pub struct DynamicPlaylist {
    pub inner_playlist: InnerPlaylist,
    pub dynamic_playlist_data: VersionedDynamicPlaylistData,
}

impl Playlist for DynamicPlaylist {
    fn get_inner(&self) -> &InnerPlaylist {
        &self.inner_playlist
    }
}

impl PartialEq for DynamicPlaylist {
    fn eq(&self, other: &Self) -> bool {
        self.inner_playlist == other.inner_playlist
    }
}

impl Eq for DynamicPlaylist {}
