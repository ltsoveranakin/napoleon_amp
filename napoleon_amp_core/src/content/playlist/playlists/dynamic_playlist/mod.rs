mod dynamic_playlist_data;
mod filter;
mod rules;

use crate::content::playlist::playlists::dynamic_playlist::dynamic_playlist_data::VersionedDynamicPlaylistData;
use serbytes::prelude::SerBytes;

pub struct DynamicPlaylist {
    pub dynamic_playlist_data: VersionedDynamicPlaylistData,
}
