use serbytes::prelude::SerBytes;
use simple_id::prelude::Id;
use std::cell::{Ref, RefMut};
use std::io;
use std::path::PathBuf;

pub mod folder;
pub mod playlist;
pub mod song;

/// Panics if Option\<T\> is None

pub(super) fn unwrap_inner_ref<T>(r: Ref<Option<T>>) -> Ref<T> {
    Ref::map(r, |opt| opt.as_ref().expect("Failed unwrap inner Ref"))
}

pub(super) fn unwrap_inner_ref_mut<T>(r: RefMut<Option<T>>) -> RefMut<T> {
    RefMut::map(r, |opt| opt.as_mut().expect("Failed unwrap inner RefMut"))
}

pub trait SaveData: SerBytes {
    fn get_path(id: Id) -> PathBuf;

    fn save_data(&mut self, id: Id) -> io::Result<()> {
        self.write_to_file_path(Self::get_path(id))
    }
}
