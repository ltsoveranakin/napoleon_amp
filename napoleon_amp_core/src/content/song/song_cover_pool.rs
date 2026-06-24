use crate::content::SaveData;
use crate::paths::song::song_cover_file;
use crate::pool::DataPool;
use rand::prelude::SmallRng;
use rand::{RngExt, SeedableRng};
use rustc_hash::FxHasher;
use serbytes::prelude::{
    BBReadResult, CurrentVersion, Mapped, MappedDataProvider, ReadByteBufferRefMut, SerBytes,
    U8Vec, VersioningWrapper, WriteByteBufferOwned, u8_slice_to_buf,
};
use simple_id::prelude::{Data, Id, IdDataProvider, IdGenerator};
use std::fmt::{Display, Formatter};
use std::fs;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};

pub type SongCoverData = VersioningWrapper<SongCoverDataStd, SongCoverDataVersion>;

pub(crate) static SONG_COVER_POOL: SongCoverPool = SongCoverPool::new();

struct CoverIdProvider {
    hash_output: u64,
}

impl IdDataProvider for CoverIdProvider {
    fn get_data(&mut self) -> Data {
        let mut rng = SmallRng::seed_from_u64(self.hash_output);

        rng.random()
    }
}

impl SaveData<&SongCoverId> for SongCoverData {
    fn get_path(input: &SongCoverId) -> PathBuf {
        song_cover_file(input)
    }
}

pub(crate) struct SongCoverPool {
    data_pool: LazyLock<DataPool<SongCoverId, SongCoverData>>,
}

impl SongCoverPool {
    const fn new() -> Self {
        Self {
            data_pool: LazyLock::new(|| DataPool::new(song_cover_file)),
        }
    }

    pub(crate) fn get_or_create_cover_id_from_bytes(
        &self,
        mime_type: &str,
        bytes: &[u8],
    ) -> SongCoverId {
        let mut hasher = FxHasher::default();

        bytes.hash(&mut hasher);

        let hash_output = hasher.finish();

        let mut id_gen = IdGenerator::new(CoverIdProvider { hash_output });

        let mut cover_id = id_gen.generate_new_id();

        cover_id.time = 0;

        let cover_id = cover_id.into();

        self.data_pool.get_or_load_value_arc(cover_id, || {
            let bytes_arc: Arc<[u8]> = Arc::from(bytes);

            SongCoverData::new(SongCoverDataStd {
                id: cover_id,
                mime_type: mime_type.to_string(),
                bytes: bytes_arc.into(),
            })
        });

        fs::write(
            format!(
                "C:/Users/ferri/dbg_out-len-{}-{}.jpg",
                bytes.len(),
                cover_id
            ),
            bytes,
        )
        .unwrap();

        cover_id
    }
}

impl Deref for SongCoverPool {
    type Target = DataPool<SongCoverId, SongCoverData>;

    fn deref(&self) -> &Self::Target {
        &self.data_pool
    }
}

#[derive(SerBytes, Copy, Clone, Hash, Eq, PartialEq, Default, Debug)]
pub struct SongCoverId(pub Id);

impl Display for SongCoverId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Id> for SongCoverId {
    fn from(value: Id) -> Self {
        Self(value)
    }
}

#[derive(SerBytes, Clone, Debug)]
pub struct SongCoverDataStd {
    pub id: SongCoverId,
    pub mime_type: String,
    pub bytes: Mapped<Arc<[u8]>, MappedArcBuf>,
}

impl Default for SongCoverDataStd {
    fn default() -> Self {
        Self {
            id: SongCoverId::default(),
            mime_type: "Unknown Image".to_string(),
            bytes: Mapped::default(),
        }
    }
}

#[derive(SerBytes, Default, Copy, Clone, Debug)]
pub enum SongCoverDataVersion {
    #[default]
    V1,
}

impl CurrentVersion for SongCoverDataVersion {
    type Output = SongCoverDataStd;

    fn get_data_from_buf(&self, buf: &mut ReadByteBufferRefMut) -> BBReadResult<Self::Output> {
        SongCoverDataStd::from_buf(buf)
    }

    fn current_version() -> Self {
        Self::default()
    }
}

#[derive(Debug)]
pub struct MappedArcBuf;

impl MappedDataProvider<Arc<[u8]>> for MappedArcBuf {
    fn value_from_buf(buf: &mut ReadByteBufferRefMut) -> BBReadResult<Arc<[u8]>> {
        let vec = U8Vec::<u32>::from_buf(buf)?.vec;

        Ok(vec.into())
    }

    fn value_to_buf(value: &Arc<[u8]>, buf: &mut WriteByteBufferOwned) {
        u8_slice_to_buf::<u32>(buf, value)
    }
}
