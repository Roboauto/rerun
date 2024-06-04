use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use re_chunk::{Chunk, ChunkId};
use re_log_types::{EntityPath, RowId, StoreId, TimeInt, Timeline};
use re_types_core::{ComponentName, ComponentNameSet, SizeBytes};

// TODO: should a chunk be identified by its own ID, or the min/max row IDs within?
// I would think the latter makes more sense

// ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataStoreConfig2 {}

impl Default for DataStoreConfig2 {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl DataStoreConfig2 {
    pub const DEFAULT: Self = Self {};
}

// ---

// // TODO: you cannot impl Copy for anything that embeds an arc, so very little point in having a
// // dedicated wrapper type in this instancek
// pub type SharedChunk = Arc<Chunk>;

// TODO: a set of `ChunkId`s, for when chunks overlap
pub type ChunkIdSet = BTreeSet<ChunkId>;

// TODO: `ChunkIdSet`s organized by the start time of their associated Chunk's time range.
// pub type ChunkIdSetPerTime = BTreeMap<TimeInt, ChunkIdSet>;
#[derive(Default, Debug, Clone)]
pub struct ChunkIdSetPerTime {
    pub(crate) per_start_time: BTreeMap<TimeInt, ChunkIdSet>,
    pub(crate) per_end_time: BTreeMap<TimeInt, ChunkIdSet>,
}

pub type ChunkIdSetPerTimePerTimeline = BTreeMap<Timeline, ChunkIdSetPerTime>;

pub type ChunkIdSetPerTimePerTimelinePerComponent =
    BTreeMap<ComponentName, ChunkIdSetPerTimePerTimeline>;

pub type ChunkIdSetPerTimePerTimelinePerComponentPerEntity =
    BTreeMap<EntityPath, ChunkIdSetPerTimePerTimelinePerComponent>;

pub type ChunkIdPerComponent = BTreeMap<ComponentName, ChunkId>;

pub type ChunkIdPerComponentPerEntity = BTreeMap<EntityPath, ChunkIdPerComponent>;

// TODO: `ChunkIdSet`s organized by the minimum `RowId` of their associated Chunk's `RowId` range.
pub type ChunkIdSetPerMinRowId = BTreeMap<RowId, ChunkIdSet>;

pub type ChunkPerChunkId = BTreeMap<ChunkId, Arc<Chunk>>;

// TODO: gonna be a weird one once we got stuff on disk.
/// Incremented on each edit.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StoreGeneration2 {
    insert_id: u64,
    gc_id: u64,
}

/// A complete data store: covers all timelines, all entities, everything.
///
/// ## Debugging
///
/// `DataStore` provides a very thorough `Display` implementation that makes it manageable to
/// know what's going on internally.
/// For even more information, you can set `RERUN_DATA_STORE_DISPLAY_SCHEMAS=1` in your
/// environment, which will result in additional schema information being printed out.
//
// TODO: docs and nice display impl?
#[derive(Debug)]
pub struct DataStore2 {
    pub(crate) id: StoreId,

    /// The configuration of the data store (e.g. bucket sizes).
    pub(crate) config: DataStoreConfig2,

    // TODO: this makes no sense in a new world where each chunk might use a different datatype for
    // a given component (although yes, not right now).
    // /// Keeps track of datatype information for all component types that have been written to
    // /// the store so far.
    // ///
    // /// See also [`Self::lookup_datatype`].
    // //
    // // TODO(#1809): replace this with a centralized Arrow registry.
    // pub(crate) type_registry: DataTypeRegistry,

    // TODO: this is gonna be messy isn't it
    // /// Keeps track of arbitrary per-row metadata.
    // pub(crate) metadata_registry: MetadataRegistry<(TimePoint, EntityPathHash)>,
    pub(crate) chunks_per_chunk_id: ChunkPerChunkId,

    /// TODO
    // TODO: this is the global order, and therefore our GC order (at least for now)
    pub(crate) chunks_per_min_row_id: ChunkIdSetPerMinRowId,

    // TODO: map chunkid to (EntityPath, Option<Timeline>) for GC?

    // TODO: havign component before timeline is probably an issue long term
    // TODO
    // /// All temporal [`IndexedTable`]s for all entities on all timelines.
    // ///
    // /// See also [`Self::static_tables`].
    pub(crate) temporal_chunk_ids_per_entity: ChunkIdSetPerTimePerTimelinePerComponentPerEntity,

    // pub(crate) tables: BTreeMap<(EntityPathHash, Timeline), IndexedTable>,
    //
    /// Static data. Never garbage collected.
    ///
    /// Static data unconditionally shadows temporal data at query time.
    ///
    /// Existing temporal will not be removed. Events won't be fired.
    ///
    /// See also [`Self::tables`].
    //
    // TODO: we actually have a nasty issue here -- we must make sure that a given component only
    // live in one of these chunks max.
    // I.e., on insert, we have we look if the column exists elsewhere, and manually remove it if
    // that's the case.
    // -> sanity check
    pub(crate) static_chunks_per_entity: ChunkIdPerComponentPerEntity,

    // pub(crate) static_tables: BTreeMap<EntityPathHash, StaticTable>,
    /// Monotonically increasing ID for insertions.
    pub(crate) insert_id: u64,

    /// Monotonically increasing ID for queries.
    pub(crate) query_id: AtomicU64,

    /// Monotonically increasing ID for GCs.
    pub(crate) gc_id: u64,

    /// Monotonically increasing ID for store events.
    pub(crate) event_id: AtomicU64,
}

// TODO: type Arc<Chunk>?
// TODO: the big problem is that you can log the exact same interval twice (or more)

impl Clone for DataStore2 {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            config: self.config.clone(),
            // type_registry: self.type_registry.clone(),
            // metadata_registry: self.metadata_registry.clone(),
            chunks_per_chunk_id: self.chunks_per_chunk_id.clone(),
            chunks_per_min_row_id: self.chunks_per_min_row_id.clone(),
            temporal_chunk_ids_per_entity: self.temporal_chunk_ids_per_entity.clone(),
            static_chunks_per_entity: self.static_chunks_per_entity.clone(),
            insert_id: Default::default(),
            query_id: Default::default(),
            gc_id: Default::default(),
            event_id: Default::default(),
        }
    }
}

impl DataStore2 {
    pub fn new(id: StoreId, config: DataStoreConfig2) -> Self {
        Self {
            id,
            config,
            // type_registry: Default::default(),
            // metadata_registry: Default::default(),
            chunks_per_min_row_id: Default::default(),
            chunks_per_chunk_id: Default::default(),
            temporal_chunk_ids_per_entity: Default::default(),
            static_chunks_per_entity: Default::default(),
            insert_id: 0,
            query_id: AtomicU64::new(0),
            gc_id: 0,
            event_id: AtomicU64::new(0),
        }
    }

    #[inline]
    pub fn id(&self) -> &StoreId {
        &self.id
    }

    /// Return the current `StoreGeneration`. This can be used to determine whether the
    /// database has been modified since the last time it was queried.
    #[inline]
    pub fn generation(&self) -> StoreGeneration2 {
        StoreGeneration2 {
            insert_id: self.insert_id,
            gc_id: self.gc_id,
        }
    }

    // TODO
    #[inline]
    pub fn timelines(
        &self,
        entity_path: &EntityPath,
        component_name: &ComponentName,
    ) -> impl Iterator<Item = &Timeline> {
        self.temporal_chunk_ids_per_entity
            .get(entity_path)
            .and_then(|temporal_chunk_ids_per_component_name| {
                temporal_chunk_ids_per_component_name.get(component_name)
            })
            .into_iter()
            .flat_map(|temporal_chunk_ids_per_timeline| temporal_chunk_ids_per_timeline.keys())
    }

    /// See [`DataStoreConfig`] for more information about configuration.
    #[inline]
    pub fn config(&self) -> &DataStoreConfig2 {
        &self.config
    }

    /// Iterate over all chunks in the store, in ascending row-id order.
    // TODO: ye that makes no sense lul
    #[inline]
    pub fn iter_chunks(&self) -> impl Iterator<Item = (RowId, &Arc<Chunk>)> + '_ {
        self.chunks_per_min_row_id
            .iter()
            .flat_map(move |(&row_id, chunk_ids)| {
                chunk_ids.iter().filter_map(move |chunk_id| {
                    self.chunks_per_chunk_id
                        .get(chunk_id)
                        .map(|chunk| (row_id, chunk))
                })
            })
    }

    // TODO
    // /// Lookup the arrow [`DataType`] of a [`re_types_core::Component`] in the internal
    // /// `DataTypeRegistry`.
    // pub fn lookup_datatype(&self, component: &ComponentName) -> Option<&DataType> {
    //     self.type_registry.get(component)
    // }

    // TODO
    // /// The oldest time for which we have any data.
    // ///
    // /// Ignores static data.
    // ///
    // /// Useful to call after a gc.
    // pub fn oldest_time_per_timeline(&self) -> BTreeMap<Timeline, TimeInt> {
    //     re_tracing::profile_function!();
    //
    //     let mut oldest_time_per_timeline = BTreeMap::default();
    //
    //     for index in self.tables.values() {
    //         if let Some(bucket) = index.buckets.values().next() {
    //             let entry = oldest_time_per_timeline
    //                 .entry(bucket.timeline)
    //                 .or_insert(TimeInt::MAX);
    //             if let Some(&time) = bucket.inner.read().col_time.front() {
    //                 *entry = TimeInt::min(*entry, TimeInt::new_temporal(time));
    //             }
    //         }
    //     }
    //
    //     oldest_time_per_timeline
    // }
}
