use crate::instance::InitError;
use serbytes::prelude::SerBytes;
use std::fmt::Debug;
use std::fs::{create_dir_all, File};
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// The config file, this is always stored in the same place regardless of device.

#[derive(SerBytes)]
pub struct NapoleonConfig {
    core_path: String,
}

impl Default for NapoleonConfig {
    fn default() -> Self {
        Self {
            core_path: dirs_next::home_dir()
                .expect("Unhandled")
                .join("napoleon_amp/")
                .to_str()
                .unwrap()
                .to_string(),
        }
    }
}

impl NapoleonConfig {
    pub fn base_playlist_folder_path(&self) -> PathBuf {
        let mut buf = PathBuf::from(&self.core_path);
        buf.join("config")
    }

    pub(super) fn load_from_path(config_path: &str) -> Result<Self, InitError> {
        let directory_path = Path::new(config_path);

        let mut file_path = directory_path.to_path_buf();
        file_path.push("config.cfg");

        if !file_path.exists() {
            create_dir_all(directory_path)?;
            File::create(&file_path)?;
            return Ok(NapoleonConfig::default());
        }

        let mut file =
            File::open(&file_path).expect(&format!("unable to open config file {:?}", file_path));

        let mut buf = vec![];

        file.read_to_end(&mut buf)
            .expect("Unable to read config file");

        let config_result = NapoleonConfig::from_bytes(&buf);

        if config_result.is_err() {
            let config = NapoleonConfig::default();
            config
                .write_to_disk(file_path)
                .expect("Unable to write to disk");
            Ok(config)
        } else {
            Ok(config_result.expect("Not err"))
        }
    }

    pub(crate) fn write_to_disk<P: AsRef<Path> + Debug>(&self, config_path: P) -> io::Result<()> {
        let mut file = File::options()
            .write(true)
            .open(&config_path)
            .expect(&format!("unable to open file {:?}", config_path));

        let mut byte_buf = self.to_bb().into_vec();

        file.write_all(&mut byte_buf)
            .expect("Unable to write to bytes");

        Ok(())
    }
}
