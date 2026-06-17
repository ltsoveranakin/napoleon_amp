use crate::content::folder::FolderData;
use crate::content::playlist::{PlaylistDataTypeVariant, PlaylistTypeVariant};
use crate::content::song::song_data::SongDataStd;
use serbytes::prelude::{ReadError, SerBytes};
use simple_id::prelude::Id;

#[derive(SerBytes)]
pub(super) enum NapoleonPacket {
    Init {
        version: String,
    },
    Disconnect {
        reason: String,
    },
    RequestFolderData {
        folder_id: Id,
    },
    SendFolderData {
        folder_id: Id,
        folder_data: FolderData,
    },
    RequestPlaylistData {
        playlist_id: Id,
        playlist_type: PlaylistTypeVariant,
    },
    SendPlaylistData {
        playlist_id: Id,
        playlist_data: PlaylistDataTypeVariant,
    },
    RequestSongBatch {
        song_ids: Vec<Id>,
    },
    RequestAllSongs,
    SendSongBatch {
        songs: Vec<SongDataStd>,
    },
    InvalidPacket {
        err: ReadError<'static>,
    },
}
