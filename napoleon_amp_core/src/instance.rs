use crate::collection::folder::Folder;
use std::io;

use crate::config::NapoleonConfig;
use serbytes::prelude::ReadError;
use std::path::{Path, PathBuf};

pub struct NapoleonInstance {
    pub config: NapoleonConfig,
    pub home_folder: Folder,
    config_path: String,
}

#[derive(Debug)]
pub enum InitError {
    IOError(io::Error),
    ReadError(ReadError),
}

impl From<io::Error> for InitError {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<ReadError> for InitError {
    fn from(value: ReadError) -> Self {
        Self::ReadError(value)
    }
}

impl NapoleonInstance {
    pub fn init() -> Result<Self, InitError> {
        let napoleon_cfg_dir = dirs_next::home_dir()
            .expect("Unhandled")
            .join("napoleon_amp/");
        Self::init_from_config_path(napoleon_cfg_dir)
    }

    pub fn init_from_config_path<P>(config_path: P) -> Result<Self, InitError>
    where
        P: AsRef<Path> + Clone,
    {
        let path_str = config_path.as_ref().to_str().expect("Shouldn't fail");

        let path_buf = PathBuf::from(path_str);

        let config = NapoleonConfig::load_from_path(path_str)?;
        let home_folder = Folder {
            name: "Home".to_string(),
            path: path_buf.join("config/"),
        };

        Ok(Self {
            config,
            home_folder,
            config_path: path_str.into(),
        })
    }

    pub fn save_config(&self) -> io::Result<()> {
        self.config.write_to_disk(&self.config_path)
    }
}
