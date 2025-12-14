use crate::data::folder::Folder;
use crate::data::PathNamed;
use std::rc::Rc;

pub struct NapoleonInstance {
    base_folder: Rc<Folder>,
}

impl NapoleonInstance {
    pub fn new() -> Self {
        Self {
            base_folder: Rc::new(Folder::new(
                PathNamed::new(
                    dirs_next::home_dir()
                        .expect("Forced home directory")
                        .join("/napoleon_amp/songs/"),
                )
                .unwrap(),
            )),
        }
    }

    pub fn get_base_folder(&self) -> Rc<Folder> {
        Rc::clone(&self.base_folder)
    }
}

