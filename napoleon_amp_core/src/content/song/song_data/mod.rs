mod artist;
pub mod meta;
mod util;
mod v1;
mod v2;
mod v3;
pub(super) mod v4;
pub mod v5;

use crate::content::song::song_cover_pool::SONG_COVER_POOL;
pub(crate) use crate::content::song::song_data::artist::Artist;
use crate::content::song::song_data::meta::{AssignIfError, SongDataMetaV2};
use crate::content::song::song_data::v1::SongDataStdV1;
use crate::content::song::song_data::v2::SongDataStdV2;
use crate::content::song::song_data::v3::SongDataStdV3;
use crate::content::song::song_data::v4::SongDataStdV4;
use crate::content::song::song_data::v5::SongDataStdV5;
use crate::content::song::{Song, UNKNOWN_ALBUM_STR};
use serbytes::prelude::{
    BBReadResult, CurrentVersion, ReadByteBufferRefMut, SerBytes, SerBytesFs, SizedBlock,
    VersioningWrapper,
};
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::{MetadataOptions, StandardTagKey, StandardVisualKey, Value, Visual};
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;

pub const MAX_RATING: u32 = 5;

pub type SongDataStd = SongDataStdV5;
pub type SongData = VersioningWrapper<SongDataStd, SongDataVersion>;

#[derive(SerBytes, Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum SongDataVersion {
    V1,
    V2,
    V3,

    V4,
    #[default]
    V5,
}

impl CurrentVersion for SongDataVersion {
    type Output = SongDataStd;

    fn get_data_from_buf(&self, buf: &mut ReadByteBufferRefMut) -> BBReadResult<Self::Output> {
        match self {
            Self::V1 => {
                let sd_v1 = SongDataStdV1::from_buf(buf)?;

                let sd_v5 = SongDataStdV5 {
                    title: sd_v1.title.clone(),
                    original_title: sd_v1.title,
                    custom_tags: sd_v1.custom_tags,
                    rating: sd_v1.rating,
                    user_tag: sd_v1.user_tag.inner,
                    meta: SizedBlock::new(SongDataMetaV2 {
                        artist: Artist {
                            full_artist_string: sd_v1.artist.full_artist_string,
                        }
                        .into(),
                        album: sd_v1.album.into(),
                        song_length: sd_v1.meta.unwrap_or_default().length.into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                };

                Ok(sd_v5)
            }

            Self::V2 => {
                let sd_v2 = SongDataStdV2::from_buf(buf)?;

                let sd_v5 = SongDataStdV5 {
                    title: sd_v2.title.clone(),
                    original_title: sd_v2.title,
                    custom_tags: sd_v2.custom_tags,
                    rating: sd_v2.rating,
                    user_tag: sd_v2.user_tag,
                    meta: SizedBlock::new(SongDataMetaV2 {
                        artist: Artist {
                            full_artist_string: sd_v2.artist.full_artist_string,
                        }
                        .into(),
                        album: sd_v2.album.into(),
                        song_length: sd_v2.song_length.into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                };

                Ok(sd_v5)
            }

            Self::V3 => {
                let sd_v3 = SongDataStdV3::from_buf(buf)?;

                let sd_v5 = SongDataStdV5 {
                    title: sd_v3.title.clone(),
                    original_title: sd_v3.title,
                    custom_tags: sd_v3.custom_tags,
                    rating: sd_v3.rating,
                    user_tag: sd_v3.user_tag,
                    meta: SizedBlock::new(SongDataMetaV2 {
                        artist: Artist {
                            full_artist_string: sd_v3.artist.full_artist_string,
                        }
                        .into(),
                        album: sd_v3.album.into(),
                        song_length: sd_v3.song_length.into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                };

                Ok(sd_v5)
            }

            Self::V4 => {
                let sd_v4 = SongDataStdV4::from_buf(buf)?;

                Ok(SongDataStdV5 {
                    title: sd_v4.title.clone(),
                    original_title: sd_v4.title,
                    custom_tags: sd_v4.custom_tags,
                    rating: sd_v4.rating,
                    user_tag: sd_v4.user_tag,
                    meta: sd_v4.meta,
                    ..Default::default()
                })
            }

            Self::V5 => SongDataStdV5::from_buf(buf),
        }
    }

    fn current_version() -> Self {
        Self::default()
    }
}

pub(crate) fn get_song_data_from_song_file(song: &Song, song_data: &mut SongData) {
    get_song_data_from_song_file_with_paths(&song.song_audio_path, &song.song_data_path, song_data);
}

pub(super) fn get_song_data_from_song_file_with_paths(
    song_audio_path: &PathBuf,
    song_data_path: &PathBuf,
    song_data: &mut SongData,
) -> bool {
    let song_data_std = &mut song_data.inner;
    let song_file = File::open(&song_audio_path).expect("Open new song file");

    let ext = song_audio_path
        .extension()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let mss_options = MediaSourceStreamOptions::default();

    let mss = MediaSourceStream::new(Box::new(song_file), mss_options);

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
                    song_data_std.meta.inner.song_length.inner = Ok(duration.as_secs() as u32);
                }
            }

            if let Some(meta) = probe_result.metadata.get() {
                if let Some(meta_revision) = meta.current() {
                    if song_data_std.meta.inner.cover.inner.is_err() {
                        let visuals = meta_revision.visuals();
                        let cover = match visuals.len() {
                            0 => None,

                            1 => {
                                let visual = &visuals[0];

                                Some(visual)
                            }

                            _ => {
                                let mut pref_visual = (get_visual_score(&visuals[0]), &visuals[0]);

                                for visual in visuals.iter().skip(1) {
                                    let score = get_visual_score(visual);

                                    if score < pref_visual.0 {
                                        pref_visual.0 = score;
                                        pref_visual.1 = visual;
                                    }
                                }

                                Some(pref_visual.1)
                            }
                        };

                        song_data_std.meta.inner.cover.inner = Ok(cover.map(|visual| {
                            SONG_COVER_POOL
                                .get_or_create_cover_id_from_bytes(&visual.media_type, &visual.data)
                        }));
                    }

                    for tag in meta_revision.tags() {
                        if let Some(std_key) = tag.std_key {
                            match std_key {
                                StandardTagKey::Artist => match &tag.value {
                                    Value::String(artist_string) => {
                                        song_data_std
                                            .meta
                                            .inner
                                            .artist
                                            .inner
                                            .assign_if_err_callback(|| Artist::new(artist_string));
                                    }

                                    _ => {
                                        unreachable!()
                                    }
                                },

                                StandardTagKey::Album => match &tag.value {
                                    Value::String(album) => song_data_std
                                        .meta
                                        .inner
                                        .album
                                        .inner
                                        .assign_if_err_callback(|| album.clone()),

                                    _ => {
                                        unreachable!()
                                    }
                                },

                                StandardTagKey::TrackTitle => match &tag.value {
                                    Value::String(title) => {
                                        if song_data_std.title != "" {
                                            song_data_std.title = title.clone();
                                        }
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
                song_audio_path, song_data_std.title, error
            );

            // panic!("uhh here");

            did_err = true;
        }
    }

    let meta = &mut song_data.inner.meta.inner;

    if meta.song_length.inner.is_err() {
        meta.song_length.inner = Ok(0);
    }

    if meta.artist.inner.is_err() {
        meta.artist.inner = Ok(Artist::default());
    }

    if meta.album.inner.is_err() {
        meta.album.inner = Ok(UNKNOWN_ALBUM_STR.to_string());
    }

    if meta.cover.inner.is_err() {
        meta.cover.inner = Ok(None);
    }

    // let cover_id = song_data.inner.meta.inner.cover.unwrapped_ref().unwrap();
    // println!("sdat: song: {} data: {:?}", song_data.inner.title, cover_id);
    // SONG_COVER_POOL.get_or_load_value(cover_id, |song_cover_data| {
    //     println!("byte len: {:?}", song_cover_data.inner.bytes.inner.len());
    // }, || {
    //     panic!("No cover");
    // });

    song_data
        .write_to_file_path(song_data_path)
        .expect("Clean write to song data file");

    did_err
}

fn get_visual_score(visual: &Visual) -> u8 {
    use StandardVisualKey::*;

    const USAGE_ORDER: &[StandardVisualKey] = &[
        FrontCover,
        Illustration,
        BackCover,
        BandArtistLogo,
        ScreenCapture,
        Media,
        FileIcon,
        OtherIcon,
        Leaflet,
        LeadArtistPerformerSoloist,
        ArtistPerformer,
        Conductor,
        BandOrchestra,
        Composer,
        Lyricist,
        RecordingLocation,
        RecordingSession,
        Performance,
        PublisherStudioLogo,
    ];

    if let Some(usage) = visual.usage {
        let usage_score = USAGE_ORDER
            .iter()
            .enumerate()
            .find_map(
                |(i, usage_o)| {
                    if usage == *usage_o { Some(i) } else { None }
                },
            )
            .unwrap();

        usage_score as u8
    } else {
        0
    }
}
