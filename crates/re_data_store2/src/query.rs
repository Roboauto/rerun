use std::{
    collections::{BTreeMap, VecDeque},
    ops::RangeBounds,
    sync::{atomic::Ordering, Arc},
};

use arrow2::array::Array as ArrowArray;
use itertools::Itertools as _;
use nohash_hasher::IntSet;

use re_chunk::{Chunk, LatestAtQuery, RangeQuery};
use re_log::trace;
use re_log_types::{
    EntityPath, EntityPathHash, ResolvedTimeRange, RowId, TimeInt, TimePoint, Timeline,
};
use re_types_core::{ComponentName, ComponentNameSet};

use crate::DataStore2;

// TODO: we will want to test issues introduced by this new model, e.g. crazily overlapping
// chunks (VRS is a good real-world example of that).

// ---

#[derive(Debug, Clone)]
pub struct RangeResult {
    // pub inner: re_chunk::RangeResult,
    pub chunk: Arc<Chunk>,
}

// TODO
// impl std::ops::Deref for RangeResult {
//     type Target = re_chunk::RangeResult;
//
//     #[inline]
//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }

// ---

impl DataStore2 {
    /// Retrieve all the [`ComponentName`]s that have been written to for a given [`EntityPath`] on
    /// the specified [`Timeline`].
    ///
    /// Static components are always included in the results.
    ///
    /// Returns `None` if the entity doesn't exist at all on this `timeline`.
    pub fn all_components(
        &self,
        timeline: &Timeline,
        entity_path: &EntityPath,
    ) -> Option<ComponentNameSet> {
        re_tracing::profile_function!();

        self.query_id.fetch_add(1, Ordering::Relaxed);

        let static_components: Option<ComponentNameSet> = self
            .static_chunks_per_entity
            .get(entity_path)
            .map(|static_chunks_per_component| {
                static_chunks_per_component.keys().copied().collect()
            });

        let temporal_components: Option<ComponentNameSet> = self
            .temporal_chunk_ids_per_entity
            .get(entity_path)
            .map(|temporal_chunk_ids_per_component| {
                temporal_chunk_ids_per_component
                    .iter()
                    .filter_map(|(component_name, temporal_chunk_ids_per_timeline)| {
                        temporal_chunk_ids_per_timeline
                            .contains_key(timeline)
                            .then_some(component_name)
                    })
                    .copied()
                    .collect()
            });

        match (static_components, temporal_components) {
            (None, None) => None,
            (None, comps @ Some(_)) | (comps @ Some(_), None) => comps,
            (Some(static_comps), Some(temporal_comps)) => {
                Some(static_comps.into_iter().chain(temporal_comps).collect())
            }
        }
    }

    /// Check whether a given entity has a specific [`ComponentName`] either on the specified
    /// timeline, or in its static data.
    #[inline]
    pub fn entity_has_component(
        &self,
        timeline: &Timeline,
        entity_path: &EntityPath,
        component_name: &ComponentName,
    ) -> bool {
        re_tracing::profile_function!();
        self.all_components(timeline, entity_path)
            .map_or(false, |components| components.contains(component_name))
    }

    /// Find the earliest time at which something was logged for a given entity on the specified
    /// timeline.
    ///
    /// Ignores static data.
    #[inline]
    pub fn entity_min_time(
        &self,
        timeline: &Timeline,
        entity_path: &EntityPath,
    ) -> Option<TimeInt> {
        let temporal_chunks_per_component = self.temporal_chunk_ids_per_entity.get(entity_path)?;

        let mut time_min = TimeInt::MAX;
        for temporal_chunk_ids_per_timeline in temporal_chunks_per_component.values() {
            if let Some(time) = temporal_chunk_ids_per_timeline.get(timeline).and_then(
                |temporal_chunk_ids_per_time| {
                    temporal_chunk_ids_per_time
                        .per_start_time
                        .first_key_value()
                        .map(|(time, _)| *time)
                },
            ) {
                time_min = TimeInt::min(time_min, time);
            }
        }

        (time_min != TimeInt::MAX).then_some(time_min)
    }

    // TODO: the doc should say black on white that all components must share the same row
    // TODO: the notion of a primary is irrelevant without instance keys -- get rid of it.
    // TODO: or rather, specifying any other com
    //
    /// Queries the datastore for the cells of the specified `component_names`, as seen from the point
    /// of view of the so-called `primary` component.
    ///
    /// Returns an array of `DataCell`s (as well as the associated _data_ time and [`RowId`], if
    /// the data is temporal) on success.
    ///
    /// Success is defined by one thing and one thing only: whether a cell could be found for the
    /// `primary` component.
    /// The presence or absence of secondary components has no effect on the success criteria.
    ///
    /// If the entity has static component data associated with it, it will unconditionally
    /// override any temporal component data.
    //
    // TODO: that's just wrong: it needs to be indexed by max_time in order to be correct.
    // and also by rowid range
    pub fn latest_at(
        &self,
        query: &LatestAtQuery,
        entity_path: &EntityPath,
        component_name: ComponentName,
    ) -> Vec<Arc<Chunk>> {
        re_tracing::profile_function!(format!("{query:?}"));

        self.query_id.fetch_add(1, Ordering::Relaxed);

        if let Some(static_chunk) = self
            .static_chunks_per_entity
            .get(entity_path)
            .and_then(|static_chunks_per_component| {
                static_chunks_per_component.get(&component_name)
            })
            .and_then(|chunk_id| self.chunks_per_chunk_id.get(chunk_id))
        {
            return vec![Arc::clone(static_chunk)];
        }

        if let Some(temporal_chunk_ids) = self
            .temporal_chunk_ids_per_entity
            .get(entity_path)
            .and_then(|temporal_chunk_ids_per_component| {
                temporal_chunk_ids_per_component.get(&component_name)
            })
            .and_then(|temporal_chunk_ids_per_timeline| {
                temporal_chunk_ids_per_timeline.get(&query.timeline())
            })
            .and_then(|temporal_chunk_ids_per_time| {
                temporal_chunk_ids_per_time
                    .per_end_time
                    .range(..=query.at())
                    .next_back()
                    .map(|(_time, chunk_ids)| chunk_ids)
            })
        {
            return temporal_chunk_ids
                .iter()
                .filter_map(|chunk_id| self.chunks_per_chunk_id.get(chunk_id).cloned())
                .collect();
        }

        Vec::new()
    }

    /// Iterates the datastore in order to return the cells of the specified `component_names` for
    /// the given time range.
    ///
    /// For each and every relevant row that is found, the returned iterator will yield an array
    /// that is filled with the cells of each and every component in `component_names`, or `None` if
    /// said component is not available in that row.
    ///
    /// This method cannot fail! If there's no data to return, an empty iterator is returned.
    ///
    /// ⚠ Contrary to latest-at queries, range queries can and will yield multiple rows for a
    /// single timestamp if it happens to hold multiple entries.
    ///
    /// If the entity has static component data associated with it, it will unconditionally
    /// override any temporal component data.
    pub fn range(
        &self,
        query: &RangeQuery,
        entity_path: &EntityPath,
        component_name: ComponentName,
    ) -> Vec<Arc<Chunk>> {
        // Beware! This merely measures the time it takes to gather all the necessary metadata
        // for building the returned iterator.
        re_tracing::profile_function!(format!("{query:?}"));

        self.query_id.fetch_add(1, Ordering::Relaxed);

        if let Some(static_chunk) = self
            .static_chunks_per_entity
            .get(entity_path)
            .and_then(|static_chunks_per_component| {
                static_chunks_per_component.get(&component_name)
            })
            .and_then(|chunk_id| self.chunks_per_chunk_id.get(chunk_id))
        {
            return vec![Arc::clone(static_chunk)];
        }

        self.temporal_chunk_ids_per_entity
            .get(entity_path)
            .and_then(|temporal_chunk_ids_per_component| {
                temporal_chunk_ids_per_component.get(&component_name)
            })
            .and_then(|temporal_chunk_ids_per_timeline| {
                temporal_chunk_ids_per_timeline.get(&query.timeline())
            })
            .into_iter()
            .flat_map(|temporal_chunk_ids_per_time| {
                temporal_chunk_ids_per_time
                    .per_start_time
                    .range(query.range().min()..=query.range().max())
                    .map(|(_time, chunk_ids)| chunk_ids)
            })
            .flat_map(|temporal_chunk_ids| {
                temporal_chunk_ids
                    .iter()
                    .filter_map(|chunk_id| self.chunks_per_chunk_id.get(chunk_id).cloned())
            })
            .collect()
    }

    #[inline]
    pub fn row_metadata(&self, row_id: &RowId) -> Option<&(TimePoint, EntityPathHash)> {
        // TODO: do we even need this one?
        None
    }
}
