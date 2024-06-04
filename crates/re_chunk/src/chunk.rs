use std::collections::BTreeMap;

use arrow2::{
    array::{
        Array as ArrowArray, ListArray as ArrowListArray, PrimitiveArray as ArrowPrimitiveArray,
    },
    chunk::Chunk as ArrowChunk,
    datatypes::{
        DataType as ArrowDatatype, Field as ArrowField, Metadata as ArrowMetadata,
        Schema as ArrowSchema, TimeUnit as ArrowTimeUnit,
    },
};

use re_log_types::{EntityPath, ResolvedTimeRange, RowId, TimeInt, TimePoint, Timeline};
use re_types_core::{ComponentName, SerializationError};

// TODO: we're going to need a chunk iterator for sure, where the cost of downcasting etc is only
// paid when creating the iterator itself.

// TODO: would be nice to offer a helper to merge N chunks into a pure arrow chunk, that doesnt
// need to respect the usual split conditions (e.g. to print a giant dataframe of the entire
// store).

// ---

/// Errors that can occur when creating/manipulating a [`Chunk`]s, directly or indirectly through
/// the use of a [`crate::ChunkBatcher`].
#[derive(thiserror::Error, Debug)]
pub enum ChunkError {
    #[error("Detected malformed Chunk: {reason}")]
    Malformed { reason: String },

    #[error(transparent)]
    Serialization(#[from] SerializationError),

    #[error("Chunks cannot be empty")]
    Empty,
}

pub type ChunkResult<T> = Result<T, ChunkError>;

// ---

// TODO: the store ID should be in the metadata here so we can remove the layer on top

/// Unique identifier for a [`Chunk`], using a [`re_tuid::Tuid`].
pub type ChunkId = re_tuid::Tuid;

/// Dense arrow-based storage of N rows of multi-component multi-temporal data for a specific entity.
///
/// This is our core datastructure for logging, storing, querying and transporting data around.
///
/// The chunk as a whole is always ascendingly sorted by [`RowId`] before it gets manipulated in any way.
/// Its time columns might or might not be ascendingly sorted, depending on how the data was logged.
///
/// This is the in-memory representation of a chunk, optimized for efficient manipulation of the
/// data within. For transport, see [`crate::TransportChunk`] instead.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub(crate) id: ChunkId,

    pub(crate) entity_path: EntityPath,

    /// Is the chunk as a whole sorted by [`RowId`]?
    pub(crate) is_sorted: bool,

    /// The respective [`RowId`]s for each row of data.
    // TODO: get rid of deser?
    pub(crate) row_ids: Vec<RowId>,

    /// The time columns.
    ///
    /// Each column must be the same length as `row_ids`.
    ///
    /// Empty if this is a static chunk.
    pub(crate) timelines: BTreeMap<Timeline, ChunkTimeline>,

    /// A sparse `ListArray` for each component.
    ///
    /// Each `ListArray` must be the same length as `row_ids`.
    ///
    /// Sparse so that we can e.g. log a `Position` at one timestamp but not a `Color`.
    // TODO: well these should be listarrays then
    pub(crate) components: BTreeMap<ComponentName, ArrowListArray<i32>>,
}

// ---

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkTimeline {
    pub(crate) timeline: Timeline,

    /// Every single timestamp for this timeline.
    ///
    /// * This might or might not be sorted, depending on how the data was logged.
    /// * This is guaranteed to always be dense, because chunks are split anytime a timeline is
    ///   added or removed.
    /// * This cannot ever contain `TimeInt::STATIC`, since static data doesn't even have timelines.
    pub(crate) times: ArrowPrimitiveArray<i64>,

    /// Is [`Self::times`] sorted?
    ///
    /// This is completely independent of [`Chunk::is_sorted`]: a timeline doesn't necessarily
    /// follow the global [`RowId`]-based order, although it does in most cases (happy path).
    pub(crate) is_sorted: bool,

    /// The time range covered by [`Self::times`].
    ///
    /// Not necessarily contiguous! Just the min and max value found in [`Self::times`].
    pub(crate) time_range: ResolvedTimeRange,
}

// TODO
// #[cfg(test)] // do not ever use this outside internal testing, it's extremely slow and hackish
// impl PartialEq for Chunk {
//     #[inline]
//     fn eq(&self, rhs: &Self) -> bool {
//         let Self {
//             id: _, // we're comparing the contents
//             entity_path,
//             is_sorted,
//             row_ids,
//             timelines,
//             components,
//         } = self;
//
//         use itertools::Itertools as _;
//
//         *entity_path == rhs.entity_path
//             && *is_sorted == rhs.is_sorted
//             && *row_ids == rhs.row_ids
//             && *timelines == rhs.timelines
//             && components.keys().collect_vec() == rhs.components.keys().collect_vec()
//             && components.iter().all(|(component_name, list_array)| {
//                 let Some(rhs_list_array) = rhs.components.get(component_name) else {
//                     return false;
//                 };
//
//                 // `arrow2::compute::comparison` has very limited support for the different arrow
//                 // types, so we just do our best here.
//                 // This is just a testing/debugging tool.
//                 if arrow2::compute::comparison::can_eq(list_array.data_type()) {
//                     arrow2::compute::comparison::eq(list_array, rhs_list_array)
//                         .values_iter()
//                         .all(|v| v)
//                 } else {
//                     list_array.data_type() == rhs_list_array.data_type()
//                         && list_array.len() == rhs_list_array.len()
//                 }
//             })
//     }
// }
//
// #[cfg(test)] // do not ever use this outside internal testing, it's extremely slow and hackish
// impl Eq for Chunk {}
//
// #[cfg(test)] // do not ever use this outside internal testing, it's extremely slow and hackish
// impl PartialEq for ChunkTimeline {
//     #[inline]
//     fn eq(&self, rhs: &Self) -> bool {
//         let Self {
//             timeline,
//             times,
//             is_sorted,
//             time_range,
//         } = self;
//
//         *timeline == rhs.timeline
//             && *is_sorted == rhs.is_sorted
//             && *time_range == rhs.time_range
//             && {
//                 arrow2::compute::comparison::eq(&*times, &rhs.times)
//                     .values_iter()
//                     .all(|v| v)
//             }
//     }
// }
//
// #[cfg(test)] // do not ever use this outside internal testing, it's extremely slow and hackish
// impl Eq for ChunkTimeline {}

impl Chunk {
    /// Creates a new [`Chunk`].
    ///
    /// This will fail if the passed in data is malformed in any way -- see [`Self::sanity_check`]
    /// for details.
    ///
    /// Iff you know for sure whether the data is already appropriately sorted or not, specify `is_sorted`.
    /// When left unspecified (`None`), it will be computed in O(n) time.
    pub fn new(
        id: ChunkId,
        entity_path: EntityPath,
        mut is_sorted: Option<bool>,
        mut row_ids: Vec<RowId>,
        timelines: BTreeMap<Timeline, ChunkTimeline>,
        mut components: BTreeMap<ComponentName, ArrowListArray<i32>>,
    ) -> ChunkResult<Self> {
        if row_ids.is_empty() {
            return Err(ChunkError::Empty);
        }

        if timelines.is_empty() {
            for list_array in components.values_mut() {
                list_array.slice(row_ids.len() - 1, 1);
            }
            row_ids = vec![row_ids[row_ids.len() - 1]];
            is_sorted = Some(true);
        }

        let mut chunk = Self {
            id,
            entity_path,
            is_sorted: false,
            row_ids,
            timelines: timelines
                .into_iter()
                .filter(|(_, time_chunk)| !time_chunk.times.is_empty())
                .collect(),
            components,
        };

        chunk.is_sorted = is_sorted.unwrap_or_else(|| chunk.is_sorted_uncached());

        chunk.sanity_check()?;

        Ok(chunk)
    }

    /// Simple helper for [`Self::new`] for static data.
    #[inline]
    pub fn new_static(
        id: ChunkId,
        entity_path: EntityPath,
        is_sorted: Option<bool>,
        row_ids: Vec<RowId>,
        components: BTreeMap<ComponentName, ArrowListArray<i32>>,
    ) -> ChunkResult<Self> {
        Self::new(
            id,
            entity_path,
            is_sorted,
            row_ids,
            Default::default(),
            components,
        )
    }
}

impl ChunkTimeline {
    /// Creates a new [`ChunkTimeline`].
    ///
    /// Returns `None` if `times` is empty.
    ///
    /// Iff you know for sure whether the data is already appropriately sorted or not, specify `is_sorted`.
    /// When left unspecified (`None`), it will be computed in O(n) time.
    pub fn new(
        is_sorted: Option<bool>,
        timeline: Timeline,
        times: ArrowPrimitiveArray<i64>,
    ) -> Option<Self> {
        re_tracing::profile_function!(format!("{} times", times.len()));

        let times = times.to(timeline.datatype()); // TODO: document + sanity
        let time_slice = times.values().as_slice();
        if time_slice.is_empty() {
            return None;
        }

        let is_sorted =
            is_sorted.unwrap_or_else(|| time_slice.windows(2).all(|times| times[0] <= times[1]));

        let time_range = if is_sorted {
            // NOTE: The 'or' in 'unwrap_or' is never hit, but better safe than sorry.
            let min_time = time_slice
                .first()
                .copied()
                .map_or(TimeInt::MIN, TimeInt::new_temporal);
            let max_time = time_slice
                .last()
                .copied()
                .map_or(TimeInt::MAX, TimeInt::new_temporal);
            ResolvedTimeRange::new(min_time, max_time)
        } else {
            // NOTE: Do the iteration multiple times in a cache-friendly way rather than the opposite.
            // NOTE: The 'or' in 'unwrap_or' is never hit, but better safe than sorry.
            let min_time = time_slice
                .iter()
                .min()
                .copied()
                .map_or(TimeInt::MIN, TimeInt::new_temporal);
            let max_time = time_slice
                .iter()
                .max()
                .copied()
                .map_or(TimeInt::MAX, TimeInt::new_temporal);
            ResolvedTimeRange::new(min_time, max_time)
        };

        Some(Self {
            timeline,
            times,
            is_sorted,
            time_range,
        })
    }
}

// ---

impl Chunk {
    #[inline]
    pub fn id(&self) -> ChunkId {
        self.id
    }

    #[inline]
    pub fn entity_path(&self) -> &EntityPath {
        &self.entity_path
    }

    /// How many columns in total? Includes control, time, and component columns.
    #[inline]
    pub fn num_columns(&self) -> usize {
        let Self {
            id: _,
            entity_path: _, // not an actual column
            is_sorted: _,
            row_ids: _,
            timelines,
            components,
        } = self;

        1 /* row_ids */ + timelines.len() + components.len()
    }

    #[inline]
    pub fn num_controls(&self) -> usize {
        _ = self;
        1 /* row_ids */
    }

    #[inline]
    pub fn num_timelines(&self) -> usize {
        self.timelines.len()
    }

    #[inline]
    pub fn num_components(&self) -> usize {
        self.components.len()
    }

    #[inline]
    pub fn num_rows(&self) -> usize {
        self.row_ids.len()
    }

    pub fn row_ids(&self) -> &[RowId] {
        &self.row_ids
    }

    /// Returns the [`RowId`]-range in this [`Chunk`].
    ///
    /// This is O(1) if the chunk is sorted, O(n) otherwise.
    #[inline]
    pub fn row_id_range(&self) -> (RowId, RowId) {
        #[allow(clippy::unwrap_used)] // cannot create empty chunks
        if self.is_sorted() {
            (
                self.row_ids.first().copied().unwrap(),
                self.row_ids.last().copied().unwrap(),
            )
        } else {
            (
                self.row_ids.iter().min().copied().unwrap(),
                self.row_ids.iter().max().copied().unwrap(),
            )
        }
    }

    #[inline]
    pub fn is_static(&self) -> bool {
        self.timelines.is_empty()
    }

    #[inline]
    pub fn timelines(&self) -> &BTreeMap<Timeline, ChunkTimeline> {
        &self.timelines
    }

    #[inline]
    pub fn component_names(&self) -> impl Iterator<Item = ComponentName> + '_ {
        self.components.keys().copied()
    }

    #[inline]
    pub fn components(&self) -> &BTreeMap<ComponentName, ArrowListArray<i32>> {
        &self.components
    }

    /// Computes the maximum value for each and every timeline present across this entire chunk,
    /// and returns the corresponding [`TimePoint`].
    #[inline]
    pub fn timepoint_max(&self) -> TimePoint {
        self.timelines
            .iter()
            .map(|(timeline, info)| (*timeline, info.time_range.max()))
            .collect()
    }
}

impl std::fmt::Display for Chunk {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let chunk = self.to_transport().map_err(|err| {
            re_log::error_once!("couldn't display Chunk: {err}");
            std::fmt::Error
        })?;
        chunk.fmt(f)
    }
}

impl ChunkTimeline {
    #[inline]
    pub fn time_range(&self) -> ResolvedTimeRange {
        self.time_range
    }

    // TODO
    #[inline]
    pub fn times(&self) -> &[i64] {
        // Unwrap: sanity checked
        let times = self
            .times
            .as_any()
            .downcast_ref::<ArrowPrimitiveArray<i64>>()
            .unwrap();
        times.values().as_slice()
    }
}

// TODO: sizebytes impl + sizebytes caching + sizebytes in transport metadata

impl re_types_core::SizeBytes for Chunk {
    #[inline]
    fn heap_size_bytes(&self) -> u64 {
        let Self {
            id,
            entity_path,
            is_sorted,
            row_ids,
            timelines,
            components,
        } = self;

        id.heap_size_bytes()
            + entity_path.heap_size_bytes()
            + is_sorted.heap_size_bytes()
            + row_ids.heap_size_bytes()
            + timelines.heap_size_bytes()
            + components.heap_size_bytes()
    }
}

impl re_types_core::SizeBytes for ChunkTimeline {
    #[inline]
    fn heap_size_bytes(&self) -> u64 {
        let Self {
            timeline,
            times,
            is_sorted,
            time_range,
        } = self;

        timeline.heap_size_bytes()
            + times.heap_size_bytes() // cheap
            + is_sorted.heap_size_bytes()
            + time_range.heap_size_bytes()
    }
}

// TODO(cmc): methods to merge chunks (compaction).

// --- Sanity checks ---

impl Chunk {
    /// Returns an error if the Chunk's invariants are not upheld.
    ///
    /// Costly checks are only run in debug builds.
    pub fn sanity_check(&self) -> ChunkResult<()> {
        re_tracing::profile_function!();

        let Self {
            id: _,
            entity_path: _,
            is_sorted,
            row_ids,
            timelines,
            components,
        } = self;

        if row_ids.is_empty() || components.is_empty() {
            return Err(ChunkError::Empty);
        }

        // Row IDs
        #[allow(clippy::collapsible_if)] // readability
        if cfg!(debug_assertions) {
            if *is_sorted != self.is_sorted_uncached() {
                return Err(ChunkError::Malformed {
                    reason: format!(
                        "Chunk is marked as {}sorted but isn't: {row_ids:?}",
                        if *is_sorted { "" } else { "un" },
                    ),
                });
            }
        }

        // Timelines
        for (timeline, time_chunk) in timelines {
            if time_chunk.times.len() != row_ids.len() {
                return Err(ChunkError::Malformed {
                    reason: format!(
                        "All timelines in a chunk must have the same number of timestamps, matching the number of row IDs.\
                         Found {} row IDs but {} timestamps for timeline {:?}",
                        row_ids.len(), time_chunk.times.len(), timeline.name(),
                    ),
                });
            }

            time_chunk.sanity_check()?;
        }

        // Components
        for (component_name, list_array) in components {
            if !matches!(list_array.data_type(), arrow2::datatypes::DataType::List(_)) {
                return Err(ChunkError::Malformed {
                    reason: format!(
                        "The outer array in a chunked component batch must be a sparse list, got {:?}",
                        list_array.data_type(),
                    ),
                });
            }
            if let arrow2::datatypes::DataType::List(field) = list_array.data_type() {
                if !field.is_nullable {
                    return Err(ChunkError::Malformed {
                        reason: format!(
                            "The outer array in chunked component batch must be a sparse list, got {:?}",
                            list_array.data_type(),
                        ),
                    });
                }
            }
            if list_array.len() != row_ids.len() {
                return Err(ChunkError::Malformed {
                    reason: format!(
                        "All component batches in a chunk must have the same number of rows, matching the number of row IDs.\
                         Found {} row IDs but {} rows for component batch {component_name}",
                        row_ids.len(), list_array.len(),
                    ),
                });
            }
        }

        Ok(())
    }
}

impl ChunkTimeline {
    /// Returns an error if the Chunk's invariants are not upheld.
    ///
    /// Costly checks are only run in debug builds.
    pub fn sanity_check(&self) -> ChunkResult<()> {
        let Self {
            timeline,
            times,
            is_sorted,
            time_range,
        } = self;

        // TODO: sanity check timeline datatype and subdatatype

        let Some(times) = times.as_any().downcast_ref::<ArrowPrimitiveArray<i64>>() else {
            return Err(ChunkError::Malformed {
                reason: format!(
                    "Chunk timeline must be backed by an array of i64s, got {:?} instead",
                    times.data_type(),
                ),
            });
        };
        let times = times.values().as_slice();

        #[allow(clippy::collapsible_if)] // readability
        if cfg!(debug_assertions) {
            if *is_sorted != times.windows(2).all(|times| times[0] <= times[1]) {
                return Err(ChunkError::Malformed {
                    reason: format!(
                        "Chunk timeline is marked as {}sorted but isn't: {times:?}",
                        if *is_sorted { "" } else { "un" },
                    ),
                });
            }
        }

        #[allow(clippy::collapsible_if)] // readability
        if cfg!(debug_assertions) {
            let is_tight_lower_bound = times.iter().any(|&time| time == time_range.min().as_i64());
            let is_tight_upper_bound = times.iter().any(|&time| time == time_range.max().as_i64());
            let is_tight_bound = is_tight_lower_bound && is_tight_upper_bound;

            if !is_tight_bound {
                return Err(ChunkError::Malformed {
                    reason: "Chunk timeline's cached time range isn't a tight bound.".to_owned(),
                });
            }

            for &time in times {
                if time < time_range.min().as_i64() || time > time_range.max().as_i64() {
                    return Err(ChunkError::Malformed {
                        reason: format!(
                            "Chunk timeline's cached time range is wrong.\
                             Found a time value of {} while its time range is {time_range:?}",
                            time,
                        ),
                    });
                }

                if time == TimeInt::STATIC.as_i64() {
                    return Err(ChunkError::Malformed {
                        reason: "A chunk's timeline should never contain a static time value."
                            .to_owned(),
                    });
                }
            }
        }

        Ok(())
    }
}
