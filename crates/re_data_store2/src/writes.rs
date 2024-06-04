use std::sync::Arc;

use arrow2::datatypes::DataType;
use itertools::Itertools as _;
use nohash_hasher::IntMap;
use parking_lot::RwLock;

use re_chunk::Chunk;
use re_log::{debug, trace};
use re_types_core::{ComponentName, ComponentNameSet, SizeBytes as _};

use crate::{DataStore2, DataStoreConfig2, StoreDiff2, StoreDiffKind2, StoreEvent2};

// ---

#[derive(thiserror::Error, Debug)]
pub enum WriteError {
    #[error("Chunks must be sorted before insertion in the data store")]
    UnsortedChunk,
}

pub type WriteResult<T> = ::std::result::Result<T, WriteError>;

// ---

impl DataStore2 {
    pub fn insert_chunk(&mut self, chunk: &Arc<Chunk>) -> WriteResult<Option<StoreEvent2>> {
        if self.chunks_per_chunk_id.contains_key(&chunk.id()) {
            // We assume that chunk IDs are unique, and that reinserting a chunk has no effect.
            re_log::warn_once!(
                "Chunk #{} was inserted more than once (this has no effect)",
                chunk.id()
            );
            return Ok(None);
        }

        if !chunk.is_sorted() {
            return Err(WriteError::UnsortedChunk);
        }

        re_tracing::profile_function!(format!("{}", chunk.row_id_range().0));

        self.insert_id += 1;

        let row_id_range = chunk.row_id_range();
        let row_id_min = row_id_range.0;
        let row_id_max = row_id_range.1;

        self.chunks_per_chunk_id.insert(chunk.id(), chunk.clone());
        self.chunks_per_min_row_id
            .entry(row_id_min)
            .or_default()
            .insert(chunk.id());

        if chunk.is_static() {
            for component_name in chunk.component_names() {
                // TODO: probably need some form of compaction to really handle this properly...
                // in the meantime maybe we just rely on the chunkid and w/e
                self.static_chunks_per_entity
                    .entry(chunk.entity_path().clone())
                    .or_default()
                    // TODO: there lies the issue -> we need to keep the chunk with the highest rowid
                    .insert(component_name, chunk.id());
            }
        } else {
            // TODO: it's fine, really -- just index on everything, who cares

            let temporal_chunk_ids_per_component = self
                .temporal_chunk_ids_per_entity
                .entry(chunk.entity_path().clone())
                .or_default();

            for component_name in chunk.component_names() {
                let temporal_chunk_ids_per_timeline = temporal_chunk_ids_per_component
                    .entry(component_name)
                    .or_default();

                for (&timeline, time_chunk) in chunk.timelines() {
                    let temporal_chunk_ids_per_time =
                        temporal_chunk_ids_per_timeline.entry(timeline).or_default();

                    let time_range = time_chunk.time_range();
                    temporal_chunk_ids_per_time
                        .per_start_time
                        .entry(time_range.min())
                        .or_default()
                        .insert(chunk.id());
                    temporal_chunk_ids_per_time
                        .per_end_time
                        .entry(time_range.min())
                        .or_default()
                        .insert(chunk.id());
                }
            }
        }

        let event = StoreEvent2 {
            store_id: self.id.clone(),
            store_generation: self.generation(),
            event_id: self
                .event_id
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            diff: StoreDiff2::addition(chunk.clone()),
        };

        {
            let events = &[event.clone()];

            if cfg!(debug_assertions) {
                let any_event_other_than_addition =
                    events.iter().any(|e| e.kind != StoreDiffKind2::Addition);
                assert!(!any_event_other_than_addition);
            }

            Self::on_events(events);
        }

        Ok(Some(event))
    }
}
