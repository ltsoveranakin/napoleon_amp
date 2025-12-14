use std::path::PathBuf;

mod folder;
mod playlist;
mod song;

pub(crate) struct PathNamed {
    /// The absolute path to the target this is in the format:
    ///
    /// parent_directory/{name}(.ext if applicable)
    path: PathBuf,
    name: String,
}

impl PathNamed {
    fn new(path: PathBuf) -> Option<Self> {
        let name = path
            .file_name()?
            .to_str()
            .expect("Unable to convert path to valid utf8 string")
            .to_string();
        Some(Self { path, name })
    }
}
