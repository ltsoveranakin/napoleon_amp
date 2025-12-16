use std::cell::{Ref, RefMut};
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};

pub mod folder;
pub mod playlist;
pub mod song;

pub struct PathNamed {
    /// The absolute path to the target this is in the format:
    ///
    /// parent_directory/{name}(.ext if applicable)
    path: PathBuf,
    name: String,
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
            if path
                .to_str()
                .expect("Invalid string from path")
                .ends_with("/")
            {
                create_dir_all(&path).expect("Failed to create needed directories.");
            } else {
                File::create(&path).expect(&format!("Unable to create file at path: {:?}", path));
            }
        }

        let mut ext_free_path = path.clone();

        ext_free_path.set_extension("");

        let name = ext_free_path
            .file_name()?
            .to_str()
            .expect("Unable to convert path to valid utf8 string")
            .to_string();

        Some(Self { path, name })
    }

    fn extend<P: AsRef<Path>>(&self, ext: P) -> Option<Self> {
        Self::new(self.path.join(ext))
    }
}

pub trait NamedPathLike {
    fn name(&self) -> &str;

    fn path(&self) -> &PathBuf;

    fn path_string(&self) -> String {
        self.path()
            .to_str()
            .expect("Unable to convert path to str")
            .to_string()
    }
}

impl NamedPathLike for PathNamed {
    fn name(&self) -> &str {
        &*self.name
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl AsRef<Path> for PathNamed {
    fn as_ref(&self) -> &Path {
        &*self.path
    }
}

/// Panics if Option\<T\> is None

pub(super) fn unwrap_inner_ref<T>(r: Ref<Option<T>>) -> Ref<T> {
    Ref::map(r, |opt| opt.as_ref().expect("Failed unwrap inner Ref"))
}

pub(super) fn unwrap_inner_ref_mut<T>(r: RefMut<Option<T>>) -> RefMut<T> {
    RefMut::map(r, |opt| opt.as_mut().expect("Failed unwrap inner RefMut"))
}
