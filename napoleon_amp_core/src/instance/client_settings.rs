use crate::content::SaveData;
use crate::paths::client_settings_file_path;
use serbytes::prelude::{
    BBReadResult, CurrentVersion, ReadByteBufferRefMut, SerBytes, VersioningWrapper,
};
use std::path::PathBuf;

pub type ClientSettings = VersioningWrapper<ClientSettingsStd, ClientSettingsVers>;

#[derive(SerBytes)]
pub enum ClientSettingsVers {
    V1,
}

impl CurrentVersion for ClientSettingsVers {
    type Output = ClientSettingsStd;

    fn get_data_from_buf(&self, buf: &mut ReadByteBufferRefMut) -> BBReadResult<Self::Output> {
        match self {
            Self::V1 => ClientSettingsStd::from_buf(buf),
        }
    }

    fn current_version() -> Self {
        Self::V1
    }
}

#[derive(SerBytes)]
pub struct ClientSettingsStd {
    pub inactive_render_timeout_ms: u16,
}

impl Default for ClientSettingsStd {
    fn default() -> Self {
        Self {
            inactive_render_timeout_ms: 1000,
        }
    }
}

impl SaveData<()> for ClientSettings {
    fn get_path(_: ()) -> PathBuf {
        client_settings_file_path()
    }
}
