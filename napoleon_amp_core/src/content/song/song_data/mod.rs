mod artist;
mod meta;
mod v1;
mod v2;
mod v3;
mod v4;

use crate::content::song::Song;
pub(crate) use crate::content::song::song_data::artist::Artist;
use crate::content::song::song_data::v1::SongDataStdV1;
use crate::content::song::song_data::v2::SongDataStdV2;
use crate::content::song::song_data::v3::SongDataStdV3;
use crate::content::song::song_data::v4::{SongDataMeta2, SongDataStdV4};
use serbytes::prelude::{
    BBReadResult, CurrentVersion, ReadByteBufferRefMut, SerBytes, SizedBlock, VersioningWrapper,
};
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::{MetadataOptions, StandardTagKey, Value};
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;

pub const MAX_RATING: u32 = 5;

pub type SongDataStd = SongDataStdV4;
pub type SongData = VersioningWrapper<SongDataStd, SongDataVersion>;

#[derive(SerBytes, Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum SongDataVersion {
    V1,
    V2,
    V3,
    #[default]
    V4,
}

impl CurrentVersion for SongDataVersion {
    type Output = SongDataStd;

    fn get_data_from_buf(&self, buf: &mut ReadByteBufferRefMut) -> BBReadResult<Self::Output> {
        match self {
            Self::V1 => {
                let sd_v1 = SongDataStdV1::from_buf(buf)?;

                let sd_v4 = SongDataStdV4 {
                    title: sd_v1.title,
                    custom_tags: sd_v1.custom_tags,
                    rating: sd_v1.rating,
                    user_tag: sd_v1.user_tag.inner,
                    meta: SizedBlock::new(Ok(SongDataMeta2 {
                        artist: Artist {
                            full_artist_string: sd_v1.artist.full_artist_string,
                        },
                        album: sd_v1.album,
                        song_length: sd_v1.meta.unwrap_or_default().length,
                    })),
                    times_listened: 0,
                };

                Ok(sd_v4)
            }

            Self::V2 => {
                let sd_v2 = SongDataStdV2::from_buf(buf)?;

                let sd_v4 = SongDataStdV4 {
                    title: sd_v2.title,
                    custom_tags: sd_v2.custom_tags,
                    rating: sd_v2.rating,
                    user_tag: sd_v2.user_tag,
                    meta: SizedBlock::new(Ok(SongDataMeta2 {
                        artist: Artist {
                            full_artist_string: sd_v2.artist.full_artist_string,
                        },
                        album: sd_v2.album,
                        song_length: sd_v2.song_length,
                    })),
                    times_listened: 0,
                };

                Ok(sd_v4)
            }

            Self::V3 => {
                let sd_v3 = SongDataStdV3::from_buf(buf)?;

                let sd_v4 = SongDataStdV4 {
                    title: sd_v3.title,
                    custom_tags: sd_v3.custom_tags,
                    rating: sd_v3.rating,
                    user_tag: sd_v3.user_tag,
                    meta: SizedBlock::new(Ok(SongDataMeta2 {
                        artist: Artist {
                            full_artist_string: sd_v3.artist.full_artist_string,
                        },
                        album: sd_v3.album,
                        song_length: sd_v3.song_length,
                    })),
                    times_listened: sd_v3.times_listened,
                };

                Ok(sd_v4)
            }

            Self::V4 => SongDataStdV4::from_buf(buf),
        }
    }

    fn current_version() -> Self {
        Self::default()
    }
}

pub(crate) fn get_song_data_from_song_file(song: &Song, song_data: &mut SongDataStd) {
    get_song_data_from_song_file_with_paths(&song.song_audio_path, &song.song_data_path, song_data);
}

pub(super) fn get_song_data_from_song_file_with_paths(
    song_audio_path: &PathBuf,
    song_data_path: &PathBuf,
    song_data: &mut SongDataStd,
) -> bool {
    let song_file = File::open(&song_audio_path).expect("Open new song file");

    let ext = song_audio_path
        .extension()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let mss_options = MediaSourceStreamOptions::default();

    let mss = MediaSourceStream::new(Box::new(song_file), mss_options);

    if song_data.meta.inner.is_err() {
        song_data.meta = SizedBlock::new(Ok(SongDataMeta2::default()));
    }

    let mut did_err = false;

    match get_probe().format(
        Hint::new().with_extension(&ext),
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    ) {
        Ok(mut probe_result) => {
            if let Some(track) = probe_result.format.default_track() {
                let params = &track.codec_params;
                if let (Some(sample_rate), Some(total_frames)) =
                    (params.sample_rate, params.n_frames)
                {
                    let duration_seconds = total_frames as f64 / sample_rate as f64;
                    let duration = Duration::from_secs_f64(duration_seconds);
                    song_data.meta_mut().song_length = duration.as_secs() as u32;
                }
            }

            if let Some(meta) = probe_result.metadata.get() {
                if let Some(meta_revision) = meta.current() {
                    for tag in meta_revision.tags() {
                        if let Some(std_key) = tag.std_key {
                            match std_key {
                                StandardTagKey::Artist => match tag.value {
                                    Value::String(ref artist_string) => {
                                        song_data.meta_mut().artist = Artist::new(artist_string);
                                    }

                                    _ => {
                                        unreachable!()
                                    }
                                },

                                StandardTagKey::Album => match tag.value {
                                    Value::String(ref album) => {
                                        song_data.meta_mut().album = album.clone();
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
                "failed getting format for {:?}; The title of this song is: {}; error: {}",
                song_audio_path, song_data.title, error
            );

            did_err = true;
        }
    }

    fs::write(song_data_path, song_data.to_bb().buf()).expect("Clean write to song data file");

    did_err
}
