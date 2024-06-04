use std::{
    collections::VecDeque,
    ops::RangeBounds,
    sync::{atomic::Ordering, Arc},
};

use arrow2::array::{Array as ArrowArray, ListArray as ArrowListArray};

use re_log_types::{ResolvedTimeRange, RowId, TimeInt, Timeline};
use re_types_core::ComponentName;

use crate::Chunk;

// --- LatestAt ---

// TODO: add examples similar to re_query

/// A query at a given time, for a given timeline.
///
/// Get the latest version of the data available at this time.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LatestAtQuery {
    timeline: Timeline,
    at: TimeInt,
}

impl std::fmt::Debug for LatestAtQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "<latest at {} on {:?}>",
            self.timeline.typ().format_utc(self.at),
            self.timeline.name(),
        ))
    }
}

impl LatestAtQuery {
    /// The returned query is guaranteed to never include [`TimeInt::STATIC`].
    #[inline]
    pub fn new(timeline: Timeline, at: impl TryInto<TimeInt>) -> Self {
        let at = at.try_into().unwrap_or(TimeInt::MIN);
        Self { timeline, at }
    }

    #[inline]
    pub const fn latest(timeline: Timeline) -> Self {
        Self {
            timeline,
            at: TimeInt::MAX,
        }
    }

    #[inline]
    pub fn timeline(&self) -> Timeline {
        self.timeline
    }

    #[inline]
    pub fn at(&self) -> TimeInt {
        self.at
    }
}

impl Chunk {
    // TODO: tests
    pub fn latest_at(&self, query: &LatestAtQuery, component_name: ComponentName) -> Option<Self> {
        re_tracing::profile_function!(format!("{query:?}"));

        assert!(self.is_sorted()); // TODO
        if !self.is_sorted() {
            re_log::error_once!("querying an unsorted Chunk will always return None");
            return None;
        }

        let component_list_array = self.components.get(&component_name)?;

        let times = self
            .timelines
            .get(&query.timeline())
            .map(|time_chunk| time_chunk.times());

        let mut index = None;

        // TODO: optimizations if sorted
        // if false && time_chunk.is_sorted() {
        if false {
            //
        } else if let Some(times) = times {
            // Temporal chunk

            let mut closest_data_time = TimeInt::MIN;
            let mut closest_row_id = RowId::ZERO;

            for i in 0..self.num_rows() {
                if !component_list_array.is_valid(i) {
                    continue;
                }

                let data_time = TimeInt::new_temporal(times[i]);
                let row_id = self.row_ids[i];

                let is_closer_time = data_time > closest_data_time && data_time <= query.at();
                let is_same_time_but_closer_row_id =
                    data_time == closest_data_time && row_id > closest_row_id;

                if is_closer_time || is_same_time_but_closer_row_id {
                    closest_data_time = data_time;
                    closest_row_id = row_id;
                    index = Some(i);
                }
            }
        } else {
            // Static chunk

            let mut closest_row_id = RowId::ZERO;

            for i in 0..self.num_rows() {
                if !component_list_array.is_valid(i) {
                    continue;
                }

                let row_id = self.row_ids[i];
                let is_closer_row_id = row_id > closest_row_id;

                if is_closer_row_id {
                    closest_row_id = row_id;
                    index = Some(i);
                }
            }
        }

        // TODO: shouldn't we slice the columns too then...?
        index.map(|i| self.sliced(i, 1))
    }
}

// --- Range ---

/// A query over a time range, for a given timeline.
///
/// Get all the data within this time interval, plus the latest one before the start of the
/// interval.
///
/// Motivation: all data is considered alive until the next logging to the same component path.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RangeQuery {
    pub timeline: Timeline,
    pub range: ResolvedTimeRange,
}

impl std::fmt::Debug for RangeQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "<ranging from {} to {} (all inclusive) on {:?}",
            self.timeline.typ().format_utc(self.range.min()),
            self.timeline.typ().format_utc(self.range.max()),
            self.timeline.name(),
        ))
    }
}

impl RangeQuery {
    /// The returned query is guaranteed to never include [`TimeInt::STATIC`].
    #[inline]
    pub const fn new(timeline: Timeline, range: ResolvedTimeRange) -> Self {
        Self { timeline, range }
    }

    #[inline]
    pub const fn everything(timeline: Timeline) -> Self {
        Self {
            timeline,
            range: ResolvedTimeRange::EVERYTHING,
        }
    }

    #[inline]
    pub fn timeline(&self) -> Timeline {
        self.timeline
    }

    #[inline]
    pub fn range(&self) -> ResolvedTimeRange {
        self.range
    }
}
