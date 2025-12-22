use crate::data::folder::Folder;
use crate::data::PathNamed;
use crate::paths::folders_dir;
use std::rc::Rc;

pub struct NapoleonInstance {
    base_folder: Rc<Folder>,
}

impl NapoleonInstance {
    pub fn new() -> Self {
        Self {
            base_folder: Rc::new(Folder::new(PathNamed::new(folders_dir()), None)),
        }
    }

    pub fn get_base_folder(&self) -> Rc<Folder> {
        Rc::clone(&self.base_folder)
    }
}
