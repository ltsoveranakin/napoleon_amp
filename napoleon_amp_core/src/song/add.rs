use std::fs::File;
use std::path::PathBuf;
use std::{fs, io};

use crate::song_tags::SongTags;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::{MetadataOptions, StandardTagKey};
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;

#[derive(Debug)]
pub enum AddSongErr {
    File(io::Error),
    Symphonia(symphonia::core::errors::Error),
}

pub fn add_song_from_path(song_path: PathBuf) -> Result<(), AddSongErr> {
    let buf = song_path.to_path_buf();
    match File::open(song_path.clone()) {
        Ok(file) => {
            let mss = MediaSourceStream::new(Box::new(file), MediaSourceStreamOptions::default());

            match get_probe().format(
                &Hint::default(),
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            ) {
                Ok(mut probe_result) => {
                    let mut format = probe_result.format;

                    // println!("fmt: {:?}", probe.metadata.get());

                    let mut song_tags = SongTags::new();
                    if let Some(meta) = probe_result.metadata.get() {
                        if let Some(rev) = meta.current() {
                            for tag in rev.tags() {
                                song_tags.insert_tag(tag.clone())
                            }
                        }
                    }

                    let name = song_tags
                        .get_std(&StandardTagKey::TrackTitle)
                        .expect("Unhandled");

                    let mut file_path = dirs_next::home_dir().expect("Unhandled");
                    file_path.push(format!(
                        "/napoleon_amp/songs/{}.{}",
                        name,
                        buf.extension()
                            .expect("Unhandled")
                            .to_str()
                            .expect("Shouldn't fail")
                    ));
                    println!("fp: {:?}", file_path);
                    fs::create_dir_all(file_path.parent().expect("Has parent"))
                        .expect("Couldnt create directories");
                    fs::copy(song_path, file_path).expect("Unhandled");
                    Ok(())
                }
                Err(e) => Err(AddSongErr::Symphonia(e)),
            }
        }
        Err(_) => unimplemented!(),
    }
}
