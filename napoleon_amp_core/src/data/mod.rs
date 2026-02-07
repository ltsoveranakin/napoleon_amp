use std::cell::{Ref, RefMut};
use std::fs::{create_dir_all, File};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub mod folder;
pub mod playlist;
pub mod song;

#[derive(Clone, Debug)]
pub struct PathNamed {
    /// The absolute path to the target this is in the format:
    ///
    /// parent_directory/{name}(.ext if applicable)
    path: PathBuf,
    name: String,
}

impl PathNamed {
    /// Panics if conversion from OsStr to str fails (invalid utf8)
    ///
    /// Panics if unable to create directories at the system level
    pub(super) fn new(path: PathBuf) -> Self {
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
            .file_name()
            .expect("Path to not terminate in '..'")
            .to_str()
            .expect("Unable to convert path to valid utf8 string")
            .to_string();

        Self { path, name }
    }

    fn extend<P: AsRef<Path>>(&self, ext: P) -> Self {
        Self::new(self.path.join(ext))
    }

    /// Renames both the name and the path
    ///
    /// Returns Err if a path with that name already exists

    pub(crate) fn rename(&mut self, new_name: String) -> io::Result<()> {
        let path_str = self
            .path
            .to_str()
            .expect("Unable to convert path to valid utf8 string");
        let new_path = PathBuf::new().join(path_str.replace(&self.name, &new_name));

        if fs::exists(&new_path)? {
            return Err(ErrorKind::InvalidInput.into());
        }

        fs::rename(&self.path, &new_path)?;

        self.name = new_name;
        self.path = new_path;

        Ok(())
    }
}

pub trait NamedPathLike {
    fn get_path_named(&self) -> &PathNamed;

    fn name(&self) -> &str {
        &*self.get_path_named().name
    }

    fn path(&self) -> &PathBuf {
        &self.get_path_named().path
    }

    fn path_string(&self) -> String {
        self.path()
            .to_str()
            .expect("Unable to convert path to str")
            .to_string()
    }

    fn file_name(&self) -> String {
        self.path()
            .file_name()
            .expect("Should have valid filename")
            .to_str()
            .unwrap()
            .to_string()
    }
}

impl NamedPathLike for PathNamed {
    fn get_path_named(&self) -> &PathNamed {
        self
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
