use std::{collections::BTreeMap, time::Duration};

use ahash::HashSet;
use nohash_hasher::IntSet;
use re_chunk::ChunkId;
use web_time::Instant;

use re_log_types::{
    EntityPath, EntityPathHash, ResolvedTimeRange, RowId, TimeInt, TimePoint, Timeline,
    VecDequeRemovalExt as _,
};
use re_types_core::{ComponentName, SizeBytes as _};

use crate::{DataStore2, DataStoreStats2, StoreDiffKind2, StoreEvent2};

// ---

#[derive(Debug, Clone, Copy)]
pub enum GarbageCollectionTarget {
    /// Try to drop _at least_ the given fraction.
    ///
    /// The fraction must be a float in the range [0.0 : 1.0].
    DropAtLeastFraction(f64),

    /// GC Everything that isn't protected
    Everything,
}

#[derive(Debug, Clone)]
pub struct GarbageCollectionOptions {
    /// What target threshold should the GC try to meet.
    pub target: GarbageCollectionTarget,

    /// How long the garbage collection in allowed to run for.
    ///
    /// Trades off latency for throughput:
    /// - A smaller `time_budget` will clear less data in a shorter amount of time, allowing for a
    ///   more responsive UI at the cost of more GC overhead and more frequent runs.
    /// - A larger `time_budget` will clear more data in a longer amount of time, increasing the
    ///   chance of UI freeze frames but decreasing GC overhead and running less often.
    ///
    /// The default is an unbounded time budget (i.e. throughput only).
    pub time_budget: Duration,

    /// How many component revisions to preserve on each timeline.
    pub protect_latest: usize,

    /// Components which should not be protected from GC when using `protect_latest`
    pub dont_protect: IntSet<ComponentName>,

    /// Whether to enable batched bucket drops.
    ///
    /// Disabled by default as it is currently slower in most cases (somehow).
    pub enable_batching: bool,
}

impl GarbageCollectionOptions {
    pub fn gc_everything() -> Self {
        Self {
            target: GarbageCollectionTarget::Everything,
            time_budget: std::time::Duration::MAX,
            protect_latest: 0,
            dont_protect: Default::default(),
            enable_batching: false,
        }
    }
}

impl std::fmt::Display for GarbageCollectionTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DropAtLeastFraction(p) => {
                write!(f, "DropAtLeast({:.3}%)", *p * 100.0)
            }
            Self::Everything => write!(f, "Everything"),
        }
    }
}

impl DataStore2 {
    /// Triggers a garbage collection according to the desired `target`.
    ///
    /// Garbage collection's performance is bounded by the number of buckets in each table (for
    /// each `RowId`, we have to find the corresponding bucket, which is roughly `O(log(n))`) as
    /// well as the number of rows in each of those buckets (for each `RowId`, we have to sort the
    /// corresponding bucket (roughly `O(n*log(n))`) and then find the corresponding row (roughly
    /// `O(log(n))`.
    /// The size of the data itself has no impact on performance.
    ///
    /// Returns the list of `RowId`s that were purged from the store.
    ///
    /// ## Semantics
    ///
    /// Garbage collection works on a row-level basis and is driven by [`RowId`] order,
    /// i.e. the order defined by the clients' wall-clocks, allowing it to drop data across
    /// the different timelines in a fair, deterministic manner.
    /// Similarly, out-of-order data is supported out of the box.
    ///
    /// The garbage collector doesn't deallocate data in and of itself: all it does is drop the
    /// store's internal references to that data (the `DataCell`s), which will be deallocated once
    /// their reference count reaches 0.
    ///
    /// ## Limitations
    ///
    /// The garbage collector has limited support for latest-at semantics. The configuration option:
    /// [`GarbageCollectionOptions::protect_latest`] will protect the N latest values of each
    /// component on each timeline. The only practical guarantee this gives is that a latest-at query
    /// with a value of max-int will be unchanged. However, latest-at queries from other arbitrary
    /// points in time may provide different results pre- and post- GC.
    pub fn gc(
        &mut self,
        options: &GarbageCollectionOptions,
    ) -> (Vec<StoreEvent2>, DataStoreStats2) {
        re_tracing::profile_function!();

        self.gc_id += 1;

        // TODO: so, turns out this shit is actually kinda on on the hot path ish, eh.
        let stats_before = self.stats();

        let total_size_bytes_before = (stats_before.static_chunks.total_size_bytes
            + stats_before.temporal_chunks.total_size_bytes)
            as f64;
        let total_num_rows_before =
            stats_before.static_chunks.total_num_rows + stats_before.temporal_chunks.total_num_rows;

        // TODO: we dont care about protected rows, we care about protected chunks, which is
        // trivial now.
        // TODO: that for sure will be weird
        let protected_chunk_ids =
            self.find_all_protected_rows(options.protect_latest, &options.dont_protect);

        let mut diffs = match options.target {
            GarbageCollectionTarget::DropAtLeastFraction(p) => {
                assert!((0.0..=1.0).contains(&p));

                let num_bytes_to_drop = total_size_bytes_before * p;
                let target_num_bytes = total_size_bytes_before - num_bytes_to_drop;

                re_log::trace!(
                    kind = "gc",
                    id = self.gc_id,
                    %options.target,
                    total_num_rows_before = re_format::format_uint(total_num_rows_before),
                    total_size_bytes_before = re_format::format_bytes(total_size_bytes_before),
                    target_num_bytes = re_format::format_bytes(target_num_bytes),
                    drop_at_least_num_bytes = re_format::format_bytes(num_bytes_to_drop),
                    "starting GC"
                );

                self.gc_drop_at_least_num_bytes(options, num_bytes_to_drop, &protected_chunk_ids)
            }
            GarbageCollectionTarget::Everything => {
                re_log::trace!(
                    kind = "gc",
                    id = self.gc_id,
                    %options.target,
                    total_num_rows_before = re_format::format_uint(total_num_rows_before),
                    total_size_bytes_before = re_format::format_bytes(total_size_bytes_before),
                    "starting GC"
                );

                self.gc_drop_at_least_num_bytes(options, f64::INFINITY, &protected_chunk_ids)
            }
        };

        // TODO: collect the ChunkIds first, then do all the post-proc

        // NOTE: only temporal data and row metadata get purged!
        let stats_after = self.stats();
        let total_size_bytes_after = (stats_after.static_chunks.total_size_bytes
            + stats_after.temporal_chunks.total_size_bytes)
            as f64;
        let total_num_rows_after =
            stats_after.static_chunks.total_num_rows + stats_after.temporal_chunks.total_num_rows;

        re_log::trace!(
            kind = "gc",
            id = self.gc_id,
            %options.target,
            total_num_rows_before = re_format::format_uint(total_num_rows_before),
            total_size_bytes_before = re_format::format_bytes(total_size_bytes_before),
            total_num_rows_after = re_format::format_uint(total_num_rows_after),
            total_size_bytes_after = re_format::format_bytes(total_size_bytes_after),
            "GC done"
        );

        let stats_diff = stats_before - stats_after;

        let events: Vec<_> = diffs
            .into_iter()
            .map(|diff| StoreEvent2 {
                store_id: self.id.clone(),
                store_generation: self.generation(),
                event_id: self
                    .event_id
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                diff,
            })
            .collect();

        {
            if cfg!(debug_assertions) {
                let any_event_other_than_deletion =
                    events.iter().any(|e| e.kind != StoreDiffKind2::Deletion);
                assert!(!any_event_other_than_deletion);
            }

            Self::on_events(&events);
        }

        (events, stats_diff)
    }

    /// For each `EntityPath`, `Timeline`, `Component` find the N latest [`ChunkId`]s.
    //
    // TODO(jleibs): More complex functionality might required expanding this to also
    // *ignore* specific entities, components, timelines, etc. for this protection.
    fn find_all_protected_rows(
        &self,
        target_count: usize,
        dont_protect: &HashSet<ComponentName>,
    ) -> HashSet<ChunkId> {
        re_tracing::profile_function!();

        if target_count == 0 {
            return Default::default();
        }

        self.temporal_chunk_ids_per_entity
            .values()
            .flat_map(|temporal_chunk_ids_per_component| {
                temporal_chunk_ids_per_component
                    .into_iter()
                    .filter_map(|(component_name, temporal_chunk_ids_per_timeline)| {
                        (!dont_protect.contains(component_name))
                            .then_some(temporal_chunk_ids_per_timeline)
                    })
                    .flat_map(|temporal_chunk_ids_per_timeline| {
                        temporal_chunk_ids_per_timeline.values().flat_map(
                            |temporal_chunk_ids_per_time| {
                                temporal_chunk_ids_per_time
                                    .per_start_time
                                    .last_key_value()
                                    .map(|(_, chunk_ids)| chunk_ids.iter().copied())
                                    .into_iter()
                                    .flatten()
                                // TODO: end time too
                                // TODO: target count
                            },
                        )
                    })
            })
            .collect()
    }
}
