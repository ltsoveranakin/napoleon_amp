pub mod folder;
mod playlist;
mod song;

use serbytes::prelude::*;

// trait ParentPath {
//     fn get_full_path(name: &str, parent: Option<&Folder>) -> String {
//         let path = if let Some(parent) = parent {
//             format!("{}/{}", parent.get_path(), name)
//         } else {
//             name.into()
//         };
//
//         path
//     }
//
//     fn get_path(&self) -> &str;
// }
