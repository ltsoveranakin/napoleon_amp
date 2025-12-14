use std::fs;
use std::path::PathBuf;

pub mod folder;
pub mod playlist;
pub mod song;

pub struct PathNamed {
    /// The absolute path to the target this is in the format:
    ///
    /// parent_directory/{name}(.ext if applicable)
    path: PathBuf,
    pub name: String,
}

impl PathNamed {
    /// Returns None if call to Path::file_name is None (path terminates in ..)
    ///
    ///
    /// Panics if conversion from OsStr to str fails (invalid utf8)
    ///
    /// Panics if unable to create directories
    pub(super) fn new(path: PathBuf) -> Option<Self> {
        if !path.try_exists().expect("TODO: HANDLE ME") {
            println!("create directories: {:?}", path);
            fs::create_dir_all(&path).expect("Failed to create needed directories.");
        }

        let name = path
            .file_name()?
            .to_str()
            .expect("Unable to convert path to valid utf8 string")
            .to_string();
        Some(Self { path, name })
    }

    fn extend(&self, ext: String) -> Option<Self> {
        Self::new(self.path.join(ext))
    }
}
