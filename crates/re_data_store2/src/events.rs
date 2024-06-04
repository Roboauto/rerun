use std::sync::Arc;

use re_chunk::Chunk;
use re_log_types::StoreId;

use crate::StoreGeneration2;

// Used all over in docstrings.
#[allow(unused_imports)]
use crate::{DataStore2, StoreSubscriber2};

// ---

/// The atomic unit of change in the Rerun [`DataStore`].
///
/// A [`StoreEvent`] describes the changes caused by the addition or deletion of a
/// [`re_log_types::DataRow`] in the store.
///
/// Methods that mutate the [`DataStore`], such as [`DataStore::insert_row`] and [`DataStore::gc`],
/// return [`StoreEvent`]s that describe the changes.
/// You can also register your own [`StoreSubscriber`] in order to be notified of changes as soon as they
/// happen.
///
/// Refer to field-level documentation for more details and check out [`StoreDiff`] for a precise
/// definition of what an event involves.
//
// TODO: docs
// TODO: (Partial)Eq?
#[derive(Debug, Clone)]
pub struct StoreEvent2 {
    /// Which [`DataStore`] sent this event?
    pub store_id: StoreId,

    /// What was the store's generation when it sent that event?
    pub store_generation: StoreGeneration2,

    /// Monotonically increasing ID of the event.
    ///
    /// This is on a per-store basis.
    ///
    /// When handling a [`StoreEvent`], if this is the first time you process this [`StoreId`] and
    /// the associated `event_id` is not `1`, it means you registered late and missed some updates.
    pub event_id: u64,

    /// What actually changed?
    ///
    /// Refer to [`StoreDiff`] for more information.
    pub diff: StoreDiff2,
}

impl std::ops::Deref for StoreEvent2 {
    type Target = StoreDiff2;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.diff
    }
}

/// Is it an addition or a deletion?
///
/// Reminder: ⚠ Do not confuse _a deletion_ and _a clear_ ⚠.
///
/// A deletion is the result of a row being completely removed from the store as part of the
/// garbage collection process.
///
/// A clear, on the other hand, is the act of logging an empty [`re_types_core::ComponentBatch`],
/// either directly using the logging APIs, or indirectly through the use of a
/// [`re_types_core::archetypes::Clear`] archetype.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreDiffKind2 {
    Addition,
    Deletion,
}

impl StoreDiffKind2 {
    #[inline]
    pub fn delta(&self) -> i64 {
        match self {
            Self::Addition => 1,
            Self::Deletion => -1,
        }
    }
}

/// Describes an atomic change in the Rerun [`DataStore`]: a row has been added or deleted.
///
/// From a query model standpoint, the [`DataStore`] _always_ operates one row at a time:
/// - The contents of a row (i.e. its columns) are immutable past insertion, by virtue of
///   [`RowId`]s being unique and non-reusable.
/// - Similarly, garbage collection always removes _all the data_ associated with a row in one go:
///   there cannot be orphaned columns. When a row is gone, all data associated with it is gone too.
///
/// Refer to field-level documentation for more information.
//
// TODO: docs
// TODO: (Partial)Eq?
#[derive(Debug, Clone)]
pub struct StoreDiff2 {
    /// Addition or deletion?
    ///
    /// The store's internals are opaque and don't necessarily reflect the query model (e.g. there
    /// might be data in the store that cannot by reached by any query).
    ///
    /// A [`StoreDiff`] answers a logical question: "does there exist a query path which can return
    /// data from that row?".
    ///
    /// An event of kind deletion only tells you that, from this point on, no query can return data from that row.
    /// That doesn't necessarily mean that the data is actually gone, i.e. don't make assumptions of e.g. the size
    /// in bytes of the store based on these events.
    /// They are in "query-model space" and are not an accurate representation of what happens in storage space.
    pub kind: StoreDiffKind2,

    // TODO: not an ID cause we want to make sure that systems receiving the event actually get a
    // chance to see the associated data.
    /// The chunk that was added or removed.
    pub chunk: Arc<Chunk>,
}

impl StoreDiff2 {
    #[inline]
    pub fn addition(chunk: Arc<Chunk>) -> Self {
        Self {
            kind: StoreDiffKind2::Addition,
            chunk,
        }
    }

    #[inline]
    pub fn deletion(chunk: Arc<Chunk>) -> Self {
        Self {
            kind: StoreDiffKind2::Deletion,
            chunk,
        }
    }

    #[inline]
    pub fn is_static(&self) -> bool {
        self.chunk.is_static()
    }

    /// `-1` for deletions, `+1` for additions.
    #[inline]
    pub fn delta(&self) -> i64 {
        self.kind.delta()
    }

    #[inline]
    pub fn num_components(&self) -> usize {
        self.chunk.num_components()
    }
}
