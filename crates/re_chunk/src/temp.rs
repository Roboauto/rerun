// TODO: remove all of this crap.

use arrow2::array::PrimitiveArray as ArrowPrimitiveArray;
use re_log_types::{DataRow, ResolvedTimeRange};

use crate::{arrays_to_list_array, Chunk, ChunkId, ChunkTimeline};

impl Chunk {
    pub fn from_data_row(row: DataRow) -> Self {
        let DataRow {
            row_id,
            timepoint,
            entity_path,
            cells,
        } = row;

        Self {
            id: ChunkId::from_u128(row_id.as_u128()),
            entity_path,
            is_sorted: true,
            row_ids: vec![row_id],
            timelines: timepoint
                .iter()
                .map(|(&timeline, time)| {
                    (
                        timeline,
                        ChunkTimeline {
                            timeline,
                            times: ArrowPrimitiveArray::<i64>::from_vec(vec![time.as_i64()]),
                            is_sorted: true,
                            time_range: ResolvedTimeRange::new(*time, *time),
                        },
                    )
                })
                .collect(),
            components: cells
                .iter()
                .filter_map(|cell| {
                    arrays_to_list_array(&[Some(cell.as_arrow_ref())])
                        .map(|list_array| (cell.component_name(), list_array))
                })
                .collect(),
        }
    }
}
