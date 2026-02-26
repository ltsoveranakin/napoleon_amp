use crate::content::song::Song;
use crate::content::{NamedPathLike, PathNamed};
use crate::id_generator::IdGenerator;
use crate::paths::{song_audio_file_v2, song_data_file_v2, songs_dir_v1, SONG_DATA_EXT};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::{fs, io};

pub(super) fn fixup_needed() -> io::Result<()> {
    if songs_dir_v1().try_exists()? {
        println!("Migrating from songsv1 to songsv2");
        fixup_songs_v1_to_v2()?;
    }

    Ok(())
}

fn fixup_songs_v1_to_v2() -> io::Result<()> {
    let mut generator = IdGenerator::new();

    for dir_entry in songs_dir_v1().read_dir()?.flatten() {
        let old_path = dir_entry.path();
        let extension = if let Some(ext) = old_path.extension().and_then(|ext| ext.to_str()) {
            ext
        } else {
            continue;
        };

        if extension != SONG_DATA_EXT {
            fixup_song_audio_file(old_path, &mut generator)?;
        }
    }

    Ok(())
}

fn fixup_song_audio_file(old_path: PathBuf, generator: &mut IdGenerator) -> io::Result<()> {
    let song_id = generator.generate_new_id();

    let old_song = Song::new(PathNamed::new(old_path));

    let new_song_data_path = song_data_file_v2(&song_id);
    let new_song_audio_path = song_audio_file_v2(&song_id);

    let mut old_song_data = old_song.get_song_data().clone();
    old_song_data.audio_file = new_song_audio_path
        .to_str()
        .ok_or(ErrorKind::InvalidFilename)?
        .to_string()
        .into();

    if !new_song_data_path.try_exists()? {
        fs::create_dir_all(
            new_song_data_path
                .parent()
                .ok_or(ErrorKind::InvalidFilename)?,
        )?;
    }

    if !new_song_audio_path.try_exists()? {
        fs::create_dir_all(
            new_song_audio_path
                .parent()
                .ok_or(ErrorKind::InvalidFilename)?,
        )?;
    }

    fs::copy(old_song.path(), new_song_audio_path)?;
    fs::copy(old_song.song_data_path, new_song_data_path)?;

    Ok(())
}
