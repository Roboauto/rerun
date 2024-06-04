use std::marker::PhantomData;

use arrow2::{
    array::{
        Array as ArrowArray, ListArray as ArrowListArray, PrimitiveArray as ArrowPrimitiveArray,
    },
    Either,
};

use re_log_types::{RowId, TimeInt, Timeline};
use re_types_core::{Component, ComponentName};

use crate::{Chunk, ChunkTimeline};

// ---

impl Chunk {
    // TODO
    pub fn sliced(&self, index: usize, len: usize) -> Self {
        let Self {
            id,
            entity_path,
            is_sorted,
            row_ids,
            timelines,
            components,
        } = self;

        // TODO: it's time to keep rowids and timelines in arrow directly

        Self {
            id: *id,
            entity_path: entity_path.clone(),
            is_sorted: *is_sorted,
            row_ids: row_ids[index..index + len].to_vec(),
            timelines: timelines
                .iter()
                .map(|(timeline, time_chunk)| (*timeline, time_chunk.sliced(index, len)))
                .collect(),
            components: components
                .iter()
                .map(|(component_name, list_array)| {
                    (*component_name, list_array.clone().sliced(index, len))
                })
                .collect(),
        }
    }

    #[inline]
    pub fn row_id_at(&self, at: usize) -> Option<RowId> {
        self.row_ids.get(at).copied()
    }

    #[inline]
    pub fn time_at(&self, timeline: &Timeline, at: usize) -> Option<TimeInt> {
        self.timelines
            .get(timeline)
            .and_then(|time_chunk| time_chunk.time_at(at))
    }

    #[inline]
    pub fn component_batch_at(
        &self,
        component_name: &ComponentName,
        at: usize,
    ) -> Option<Box<dyn ArrowArray>> {
        self.components.get(component_name).and_then(|list_array| {
            // TODO: why the fuck is this unwrap only
            Some(
                list_array
                    .as_any()
                    .downcast_ref::<ArrowListArray<i32>>()?
                    .value(at),
            )
        })
    }
}

// TODO: sorting a chunk by timeline will be hugely important too

impl ChunkTimeline {
    // TODO
    pub fn sliced(&self, index: usize, len: usize) -> Self {
        let Self {
            timeline,
            times,
            is_sorted,
            time_range,
        } = self;

        let is_sorted = (len < 1) || *is_sorted;

        Self::new(
            Some(is_sorted),
            *timeline,
            ArrowPrimitiveArray::sliced(times.clone() /* cheap */, index, len),
        )
        .unwrap()
    }

    #[inline]
    pub fn time_at(&self, at: usize) -> Option<TimeInt> {
        self.times().get(at).copied().map(TimeInt::new_temporal)
    }
}

pub struct ChunkIter<C: Component> {
    _phantom: PhantomData<C>,
}

impl Chunk {
    // TODO: sorting etc, whatever

    pub fn iter(
        &self,
        timeline: &Timeline,
        component_name: &ComponentName,
    ) -> impl Iterator<Item = (RowId, TimeInt, Option<Box<dyn ArrowArray>>)> {
        let Self {
            id,
            entity_path,
            is_sorted,
            row_ids,
            mut timelines,
            mut components,
        } = self.clone(); // TODO: as in -- actual clone

        // TODO: arrow this
        let row_ids = row_ids.clone().into_iter();

        let data_times = timelines
            .remove(timeline)
            .into_iter()
            .flat_map(|time_chunk| {
                time_chunk
                    .times
                    .values_iter()
                    .copied()
                    .map(TimeInt::new_temporal)
                    .collect::<Vec<_>>()
            })
            // TODO: explain
            .chain(std::iter::repeat(TimeInt::STATIC));

        let arrays = components
            .remove(component_name)
            .into_iter()
            .flat_map(|list_array| list_array.into_iter().collect::<Vec<_>>()); // TODO

        itertools::izip!(row_ids, data_times, arrays)
    }

    pub fn iter_typed<C: Component>(
        &self,
        timeline: &Timeline,
    ) -> impl Iterator<Item = (RowId, TimeInt, Vec<C>)> + '_ {
        let Self {
            id,
            entity_path,
            is_sorted,
            row_ids,
            timelines,
            components,
        } = self;

        let Some(time_chunk) = timelines.get(timeline) else {
            return Either::Left(std::iter::empty());
        };

        let Some(all_values) = components.get(&C::name()) else {
            return Either::Left(std::iter::empty());
        };
        let Some(values) = C::from_arrow(&**all_values.values()).ok() else {
            return Either::Left(std::iter::empty());
        };

        Either::Right(itertools::izip!(
            row_ids.iter().copied(),
            time_chunk
                .times
                .values_iter()
                .copied()
                .map(TimeInt::new_temporal),
            all_values
                .offsets()
                .iter()
                .zip(all_values.offsets().lengths())
                .map(
                    move |(&index, length)| values[index as usize..index as usize + length]
                        .to_vec()
                ),
        ))
    }
}
