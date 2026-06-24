use crate::content::song::song_data::v4::DEFAULT_CUSTOM_VOLUME;
use serbytes::prelude::MayNotExistDataProvider;

pub struct CustomVolumeDataProvider;

impl MayNotExistDataProvider<f32> for CustomVolumeDataProvider {
    fn get_data() -> f32 {
        DEFAULT_CUSTOM_VOLUME
    }
}
