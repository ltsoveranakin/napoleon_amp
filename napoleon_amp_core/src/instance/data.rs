use crate::paths::instance_data_file_path;
use serbytes::prelude::SerBytes;

#[derive(SerBytes, Default)]
pub(super) struct InstanceData {
    /// Vector of the fully qualified pathname of folders or playlists
    folder_content_order: Vec<String>,
}

impl InstanceData {
    pub(super) fn init_self() -> InstanceData {
        Self::from_file_path(instance_data_file_path()).unwrap_or(InstanceData::default())
    }
}
