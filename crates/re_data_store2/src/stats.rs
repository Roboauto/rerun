use std::sync::Arc;

use re_chunk::Chunk;
use re_types_core::SizeBytes;

use crate::DataStore2;

// ---

#[derive(Default, Debug, Clone, Copy)]
pub struct DataStoreChunkStats2 {
    pub num_chunks: u64,

    pub avg_size_bytes: u64,
    pub min_size_bytes: u64,
    pub max_size_bytes: u64,
    pub total_size_bytes: u64,

    pub avg_num_rows: u64,
    pub min_num_rows: u64,
    pub max_num_rows: u64,
    pub total_num_rows: u64,

    pub avg_num_components: u64,
    pub min_num_components: u64,
    pub max_num_components: u64,
    pub total_num_components: u64,

    pub avg_num_timelines: u64,
    pub min_num_timelines: u64,
    pub max_num_timelines: u64,
    pub total_num_timelines: u64,
}

impl std::ops::Add for DataStoreChunkStats2 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            num_chunks: self.num_chunks + rhs.num_chunks,

            avg_size_bytes: ((self.avg_size_bytes + rhs.avg_size_bytes) as f64 / 2.0) as u64,
            min_size_bytes: u64::min(self.min_size_bytes, rhs.min_size_bytes),
            max_size_bytes: u64::max(self.max_size_bytes, rhs.max_size_bytes),
            total_size_bytes: self.total_size_bytes + rhs.total_size_bytes,

            avg_num_rows: ((self.avg_num_rows + rhs.avg_num_rows) as f64 / 2.0) as u64,
            min_num_rows: u64::min(self.min_num_rows, rhs.min_num_rows),
            max_num_rows: u64::max(self.max_num_rows, rhs.max_num_rows),
            total_num_rows: self.total_num_rows + rhs.total_num_rows,

            avg_num_components: ((self.avg_num_components + rhs.avg_num_components) as f64 / 2.0)
                as u64,
            min_num_components: u64::min(self.min_num_components, rhs.min_num_components),
            max_num_components: u64::max(self.max_num_components, rhs.max_num_components),
            total_num_components: self.total_num_components + rhs.total_num_components,

            avg_num_timelines: ((self.avg_num_timelines + rhs.avg_num_timelines) as f64 / 2.0)
                as u64,
            min_num_timelines: u64::min(self.min_num_timelines, rhs.min_num_timelines),
            max_num_timelines: u64::max(self.max_num_timelines, rhs.max_num_timelines),
            total_num_timelines: self.total_num_timelines + rhs.total_num_timelines,
        }
    }
}

impl DataStoreChunkStats2 {
    #[inline]
    pub fn from_chunk(chunk: &Arc<Chunk>) -> Self {
        let size_bytes = <Chunk as SizeBytes>::total_size_bytes(&**chunk);
        let num_rows = chunk.num_rows() as u64;
        let num_components = chunk.num_components() as u64;
        let num_timelines = chunk.num_timelines() as u64;

        Self {
            num_chunks: 1,

            avg_size_bytes: size_bytes,
            min_size_bytes: size_bytes,
            max_size_bytes: size_bytes,
            total_size_bytes: size_bytes,

            avg_num_rows: num_rows,
            min_num_rows: num_rows,
            max_num_rows: num_rows,
            total_num_rows: num_rows,

            avg_num_components: num_components,
            min_num_components: num_components,
            max_num_components: num_components,
            total_num_components: num_components,

            avg_num_timelines: num_timelines,
            min_num_timelines: num_timelines,
            max_num_timelines: num_timelines,
            total_num_timelines: num_timelines,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct DataStoreStats2 {
    pub static_chunks: DataStoreChunkStats2,
    pub temporal_chunks: DataStoreChunkStats2,
}

impl DataStore2 {
    pub fn stats(&self) -> DataStoreStats2 {
        re_tracing::profile_function!();

        let static_chunks = {
            re_tracing::profile_scope!("static");
            self.iter_chunks()
                .filter(|(_, chunk)| chunk.is_static())
                .fold(DataStoreChunkStats2::default(), |stats, (_, chunk)| {
                    stats + DataStoreChunkStats2::from_chunk(chunk)
                })
        };

        let temporal_chunks = {
            re_tracing::profile_scope!("temporal");
            self.iter_chunks()
                .filter(|(_, chunk)| !chunk.is_static())
                .fold(DataStoreChunkStats2::default(), |stats, (_, chunk)| {
                    stats + DataStoreChunkStats2::from_chunk(chunk)
                })
        };

        DataStoreStats2 {
            static_chunks,
            temporal_chunks,
        }
    }
}
