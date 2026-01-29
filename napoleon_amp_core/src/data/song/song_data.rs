use crate::data::song::{Song, SongData};
use crate::data::NamedPathLike;
use serbytes::prelude::SerBytes;
use std::fs;
use std::fs::File;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::{MetadataOptions, StandardTagKey, Value};
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;

pub(crate) fn get_song_data_from_song_file(song: &Song, song_data: &mut SongData) {
    let song_path = song.path();
    let song_file = File::open(&song_path).expect("Open new song file");

    let ext = song_path.extension().unwrap().to_str().unwrap().to_string();

    let mss_options = MediaSourceStreamOptions::default();

    let mss = MediaSourceStream::new(Box::new(song_file), mss_options);

    match get_probe().format(
        Hint::new().with_extension(&ext),
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    ) {
        Ok(mut probe_result) => {
            if let Some(meta) = probe_result.metadata.get() {
                if let Some(meta_revision) = meta.current() {
                    for tag in meta_revision.tags() {
                        if let Some(std_key) = tag.std_key {
                            match std_key {
                                StandardTagKey::Artist => match tag.value {
                                    Value::String(ref artist) => {
                                        song_data.artist = artist.clone();
                                    }

                                    _ => {
                                        unreachable!()
                                    }
                                },

                                StandardTagKey::Album => match tag.value {
                                    Value::String(ref album) => {
                                        song_data.album = album.clone();
                                    }

                                    _ => {
                                        unreachable!()
                                    }
                                },

                                StandardTagKey::TrackTitle => match tag.value {
                                    Value::String(ref title) => {
                                        song_data.title = title.clone();
                                    }

                                    _ => {
                                        unreachable!()
                                    }
                                },

                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        Err(error) => {
            println!(
                "failed getting format for {:?}; error: {}",
                song_path, error
            );
        }
    }

    fs::write(&song.song_data_path, song_data.to_bb().buf())
        .expect("Clean write to song data file");
}
