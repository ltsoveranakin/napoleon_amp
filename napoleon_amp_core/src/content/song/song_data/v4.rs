use crate::content::song::song_data::meta::SongDataMetaV2;
use serbytes::prelude::{
    MayNotExistDataProvider, MayNotExistOrDefault, MayNotExistOrElse, SerBytes, SizedBlock,
};
use std::time::Duration;

pub(crate) const DEFAULT_CUSTOM_VOLUME: f32 = 0.75;

/// Data stored for each song which has been registered, contains metadata which is commonly used

#[derive(SerBytes, Clone, Debug)]
pub struct SongDataStdV4 {
    /// Track title of the song
    pub title: String,
    pub custom_tags: Vec<String>,
    /// A rating of the song from 0 to 5
    /// where 0 represents unrated and 1-5 represent a rating
    pub rating: u8,
    /// A custom user defined tag
    pub user_tag: String,
    /// Metadata related to a song, this is never of type Err, and can be unwrapped with no issue. Use the helper function [`SongDataStdV4::meta`] where appropriate
    pub meta: SizedBlock<SongDataMetaV2>,
    pub times_listened: u32,
    pub times_skipped: MayNotExistOrDefault<u32>,
    pub start_offset: MayNotExistOrDefault<Option<Duration>>,
    pub end_time: MayNotExistOrDefault<Option<Duration>>,
    pub custom_volume: MayNotExistOrElse<f32, CustomVolumeDataProvider>,
}

pub struct CustomVolumeDataProvider;

impl MayNotExistDataProvider<f32> for CustomVolumeDataProvider {
    fn get_data() -> f32 {
        DEFAULT_CUSTOM_VOLUME
    }
}

impl Default for SongDataStdV4 {
    fn default() -> Self {
        Self {
            title: String::new(),
            custom_tags: Vec::new(),
            rating: 0,
            user_tag: String::new().into(),
            meta: SizedBlock::new(SongDataMetaV2::default()),
            times_listened: 0,
            times_skipped: 0.into(),
            start_offset: None.into(),
            end_time: None.into(),
            custom_volume: DEFAULT_CUSTOM_VOLUME.into(),
        }
    }
}
