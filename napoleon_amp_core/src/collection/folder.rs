use crate::collection::playlist::Playlist;
use std::path::PathBuf;
use std::{fs, io};

pub type FolderID = u32;

#[derive(Clone)]
pub struct Folder {
    pub name: String,
    pub(crate) path: PathBuf,
}

pub enum FolderContent {
    Folder(Folder),
    Playlist(Playlist),
}

impl Folder {
    pub fn add_folder(&mut self, folder_name: String) -> io::Result<Folder> {
        let new_path = self.path.join(format!("{}/", folder_name));

        fs::create_dir_all(&new_path)?;

        let folder = Folder {
            path: new_path,
            name: folder_name,
        };

        Ok(folder)
    }

    pub fn get_contents(&self) -> io::Result<Vec<FolderContent>> {
        let read_dir = fs::read_dir(&self.path)?;

        let mut contents = vec![];

        for dir_res in read_dir {
            let dir = dir_res?;
            let path_buf = dir.path();

            let name = path_buf.file_name().unwrap().to_str().unwrap().to_string();

            let content = if path_buf.is_file() {
                FolderContent::Playlist(Playlist { path: path_buf })
            } else {
                FolderContent::Folder(Folder {
                    path: path_buf,
                    name,
                })
            };

            contents.push(content);
        }

        Ok(contents)
    }
}
