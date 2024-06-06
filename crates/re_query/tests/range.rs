// https://github.com/rust-lang/rust-clippy/issues/10011
#![cfg(test)]

use std::sync::Arc;

use itertools::Itertools as _;

use re_data_store2::{
    external::re_chunk::Chunk, DataStore2, RangeQuery, ResolvedTimeRange, StoreSubscriber2 as _,
    TimeInt,
};
use re_log_types::{
    build_frame_nr,
    example_components::{MyColor, MyPoint, MyPoints},
    EntityPath, RowId, TimePoint, Timeline,
};
use re_query::{Caches, PromiseResolver, PromiseResult};
use re_types::{Archetype, ComponentBatch};
use re_types_core::Loggable as _;

// ---

#[test]
fn simple_range() -> anyhow::Result<()> {
    let mut store = DataStore2::new(
        re_log_types::StoreId::random(re_log_types::StoreKind::Recording),
        Default::default(),
    );
    let mut caches = Caches::new(&store);

    let entity_path: EntityPath = "point".into();

    let timepoint1 = [build_frame_nr(123)];
    let row_id1_1 = RowId::new();
    let points1_1 = vec![MyPoint::new(1.0, 2.0), MyPoint::new(3.0, 4.0)];
    let row_id1_2 = RowId::new();
    let colors1_2 = vec![MyColor::from_rgb(255, 0, 0)];
    let chunk = Chunk::builder(entity_path.clone())
        .with_component_batch(row_id1_1, timepoint1, &points1_1)
        .with_component_batch(row_id1_2, timepoint1, &colors1_2)
        .build()?;
    insert_and_react(&mut store, &mut caches, &Arc::new(chunk));

    let timepoint2 = [build_frame_nr(223)];
    let row_id2 = RowId::new();
    let colors2 = vec![MyColor::from_rgb(255, 0, 0)];
    let chunk = Chunk::builder(entity_path.clone())
        .with_component_batch(row_id2, timepoint2, &colors2)
        .build()?;
    insert_and_react(&mut store, &mut caches, &Arc::new(chunk));

    let timepoint3 = [build_frame_nr(323)];
    let row_id3 = RowId::new();
    let points3 = vec![MyPoint::new(10.0, 20.0), MyPoint::new(30.0, 40.0)];
    let chunk = Chunk::builder(entity_path.clone())
        .with_component_batch(row_id3, timepoint3, &points3)
        .build()?;
    insert_and_react(&mut store, &mut caches, &Arc::new(chunk));

    // --- First test: `(timepoint1, timepoint3]` ---

    let query = RangeQuery::new(
        timepoint1[0].0,
        ResolvedTimeRange::new(timepoint1[0].1.as_i64() + 1, timepoint3[0].1),
    );

    let expected_points = &[
        ((TimeInt::new_temporal(323), row_id3), points3.as_slice()), //
    ];
    let expected_colors = &[
        ((TimeInt::new_temporal(223), row_id2), colors2.as_slice()), //
    ];
    query_and_compare(
        &caches,
        &store,
        &query,
        &entity_path,
        expected_points,
        expected_colors,
    );

    // --- Second test: `[timepoint1, timepoint3]` ---

    let query = RangeQuery::new(
        timepoint1[0].0,
        ResolvedTimeRange::new(timepoint1[0].1, timepoint3[0].1),
    );

    let expected_points = &[
        (
            (TimeInt::new_temporal(123), row_id1_1),
            points1_1.as_slice(),
        ), //
        ((TimeInt::new_temporal(323), row_id3), points3.as_slice()), //
    ];
    let expected_colors = &[
        (
            (TimeInt::new_temporal(123), row_id1_2),
            colors1_2.as_slice(),
        ), //
        ((TimeInt::new_temporal(223), row_id2), colors2.as_slice()), //
    ];
    query_and_compare(
        &caches,
        &store,
        &query,
        &entity_path,
        expected_points,
        expected_colors,
    );

    Ok(())
}

#[test]
fn static_range() -> anyhow::Result<()> {
    let mut store = DataStore2::new(
        re_log_types::StoreId::random(re_log_types::StoreKind::Recording),
        Default::default(),
    );
    let mut caches = Caches::new(&store);

    let entity_path: EntityPath = "point".into();

    let timepoint1 = [build_frame_nr(123)];
    let row_id1_1 = RowId::new();
    let points1_1 = vec![MyPoint::new(1.0, 2.0), MyPoint::new(3.0, 4.0)];
    let row_id1_2 = RowId::new();
    let colors1_2 = vec![MyColor::from_rgb(255, 0, 0)];
    let chunk = Chunk::builder(entity_path.clone())
        .with_component_batch(row_id1_1, timepoint1, &points1_1)
        .with_component_batch(row_id1_2, timepoint1, &colors1_2)
        .build()?;
    insert_and_react(&mut store, &mut caches, &Arc::new(chunk));
    // Insert statically too!
    let row_id1_3 = RowId::new();
    let chunk = Chunk::builder(entity_path.clone())
        .with_component_batch(row_id1_3, TimePoint::default(), &colors1_2)
        .build()?;
    insert_and_react(&mut store, &mut caches, &Arc::new(chunk));

    let timepoint2 = [build_frame_nr(223)];
    let row_id2_1 = RowId::new();
    let colors2_1 = vec![MyColor::from_rgb(255, 0, 0)];
    let chunk = Chunk::builder(entity_path.clone())
        .with_component_batch(row_id2_1, timepoint2, &colors2_1)
        .build()?;
    insert_and_react(&mut store, &mut caches, &Arc::new(chunk));
    // Insert statically too!
    let row_id2_2 = RowId::new();
    let chunk = Chunk::builder(entity_path.clone())
        .with_component_batch(row_id2_2, TimePoint::default(), &colors2_1)
        .build()?;
    insert_and_react(&mut store, &mut caches, &Arc::new(chunk));

    let timepoint3 = [build_frame_nr(323)];
    // Create some Positions with implicit instances
    let row_id3 = RowId::new();
    let points3 = vec![MyPoint::new(10.0, 20.0), MyPoint::new(30.0, 40.0)];
    let chunk = Chunk::builder(entity_path.clone())
        .with_component_batch(row_id3, timepoint3, &points3)
        .build()?;
    insert_and_react(&mut store, &mut caches, &Arc::new(chunk));

    // --- First test: `(timepoint1, timepoint3]` ---

    let query = RangeQuery::new(
        timepoint1[0].0,
        ResolvedTimeRange::new(timepoint1[0].1.as_i64() + 1, timepoint3[0].1),
    );

    let expected_points = &[
        ((TimeInt::new_temporal(323), row_id3), points3.as_slice()), //
    ];
    let expected_colors = &[
        ((TimeInt::STATIC, row_id2_2), colors2_1.as_slice()), //
    ];
    query_and_compare(
        &caches,
        &store,
        &query,
        &entity_path,
        expected_points,
        expected_colors,
    );

    // --- Second test: `[timepoint1, timepoint3]` ---

    // The inclusion of `timepoint1` means latest-at semantics will fall back to timeless data!

    let query = RangeQuery::new(
        timepoint1[0].0,
        ResolvedTimeRange::new(timepoint1[0].1, timepoint3[0].1),
    );

    let expected_points = &[
        (
            (TimeInt::new_temporal(123), row_id1_1),
            points1_1.as_slice(),
        ), //
        ((TimeInt::new_temporal(323), row_id3), points3.as_slice()), //
    ];
    let expected_colors = &[
        ((TimeInt::STATIC, row_id2_2), colors2_1.as_slice()), //
    ];
    query_and_compare(
        &caches,
        &store,
        &query,
        &entity_path,
        expected_points,
        expected_colors,
    );

    // --- Third test: `[-inf, +inf]` ---

    let query = RangeQuery::new(
        timepoint1[0].0,
        ResolvedTimeRange::new(TimeInt::MIN, TimeInt::MAX),
    );

    // same expectations
    query_and_compare(
        &caches,
        &store,
        &query,
        &entity_path,
        expected_points,
        expected_colors,
    );

    Ok(())
}
//
// // Test the case where the user loads a piece of data at the end of the time range, then a piece at
// // the beginning of the range, and finally a piece right in the middle.
// //
// // DATA = ###################################################
// //          |      |     |       |            \_____/
// //          \______/     |       |            query #1
// //          query #2     \_______/
// //                       query #3
// //
// // There is no data invalidation involved, which is what makes this case tricky: the cache must
// // properly keep track of the fact that there are holes in the data -- on purpose.
// #[test]
// fn time_back_and_forth() {
//     let mut store = DataStore2::new(
//         re_log_types::StoreId::random(re_log_types::StoreKind::Recording),
//         Default::default(),
//     );
//     let mut caches = Caches::new(&store);
//
//     let entity_path: EntityPath = "point".into();
//
//     let (rows, points): (Vec<_>, Vec<_>) = (0..10)
//         .map(|i| {
//             let timepoint = [build_frame_nr(i)];
//             let points = vec![MyPoint::new(1.0, 2.0), MyPoint::new(3.0, 4.0)];
//             let row = DataRow::from_cells1_sized(
//                 RowId::new(),
//                 entity_path.clone(),
//                 timepoint,
//                 points.clone(),
//             )
//             .unwrap();
//
//             insert_and_react(&mut store, &mut caches, &row);
//
//             (row, points)
//         })
//         .unzip();
//
//     // --- Query #1: `[8, 10]` ---
//
//     let query = RangeQuery::new(
//         Timeline::new_sequence("frame_nr"),
//         ResolvedTimeRange::new(8, 10),
//     );
//
//     let expected_points = &[
//         (
//             (TimeInt::new_temporal(8), rows[8].row_id()), //
//             points[8].as_slice(),
//         ), //
//         (
//             (TimeInt::new_temporal(9), rows[9].row_id()), //
//             points[9].as_slice(),
//         ), //
//     ];
//     query_and_compare(&caches, &store, &query, &entity_path, expected_points, &[]);
//
//     // --- Query #2: `[1, 3]` ---
//
//     let query = RangeQuery::new(
//         Timeline::new_sequence("frame_nr"),
//         ResolvedTimeRange::new(1, 3),
//     );
//
//     let expected_points = &[
//         (
//             (TimeInt::new_temporal(1), rows[1].row_id()), //
//             points[1].as_slice(),
//         ), //
//         (
//             (TimeInt::new_temporal(2), rows[2].row_id()), //
//             points[2].as_slice(),
//         ), //
//         (
//             (TimeInt::new_temporal(3), rows[3].row_id()), //
//             points[3].as_slice(),
//         ), //
//     ];
//     query_and_compare(&caches, &store, &query, &entity_path, expected_points, &[]);
//
//     // --- Query #3: `[5, 7]` ---
//
//     let query = RangeQuery::new(
//         Timeline::new_sequence("frame_nr"),
//         ResolvedTimeRange::new(5, 7),
//     );
//
//     let expected_points = &[
//         (
//             (TimeInt::new_temporal(5), rows[5].row_id()), //
//             points[5].as_slice(),
//         ), //
//         (
//             (TimeInt::new_temporal(6), rows[6].row_id()), //
//             points[6].as_slice(),
//         ), //
//         (
//             (TimeInt::new_temporal(7), rows[7].row_id()), //
//             points[7].as_slice(),
//         ), //
//     ];
//     query_and_compare(&caches, &store, &query, &entity_path, expected_points, &[]);
// }
//
// #[test]
// fn invalidation() {
//     let entity_path = "point";
//
//     let test_invalidation = |query: RangeQuery,
//                              present_data_timepoint: TimePoint,
//                              past_data_timepoint: TimePoint,
//                              future_data_timepoint: TimePoint| {
//         let past_timestamp = past_data_timepoint
//             .get(&query.timeline())
//             .copied()
//             .unwrap_or(TimeInt::STATIC);
//         let present_timestamp = present_data_timepoint
//             .get(&query.timeline())
//             .copied()
//             .unwrap_or(TimeInt::STATIC);
//         let future_timestamp = future_data_timepoint
//             .get(&query.timeline())
//             .copied()
//             .unwrap_or(TimeInt::STATIC);
//
//         let mut store = DataStore2::new(
//             re_log_types::StoreId::random(re_log_types::StoreKind::Recording),
//             Default::default(),
//         );
//         let mut caches = Caches::new(&store);
//
//         let points1 = vec![MyPoint::new(1.0, 2.0), MyPoint::new(3.0, 4.0)];
//         let row1 = DataRow::from_cells1_sized(
//             RowId::new(),
//             entity_path,
//             present_data_timepoint.clone(),
//             points1.clone(),
//         )
//         .unwrap();
//         insert_and_react(&mut store, &mut caches, &row1);
//
//         let colors2 = vec![MyColor::from_rgb(1, 2, 3)];
//         let row2 = DataRow::from_cells1_sized(
//             RowId::new(),
//             entity_path,
//             present_data_timepoint.clone(),
//             colors2.clone(),
//         )
//         .unwrap();
//         insert_and_react(&mut store, &mut caches, &row2);
//
//         let expected_points = &[
//             ((present_timestamp, row1.row_id()), points1.as_slice()), //
//         ];
//         let expected_colors = &[
//             ((present_timestamp, row2.row_id()), colors2.as_slice()), //
//         ];
//         query_and_compare(
//             &caches,
//             &store,
//             &query,
//             &entity_path.into(),
//             expected_points,
//             expected_colors,
//         );
//
//         // --- Modify present ---
//
//         // Modify the PoV component
//         let points3 = vec![MyPoint::new(10.0, 20.0), MyPoint::new(30.0, 40.0)];
//         let row3 = DataRow::from_cells1_sized(
//             RowId::new(),
//             entity_path,
//             present_data_timepoint.clone(),
//             points3.clone(),
//         )
//         .unwrap();
//         insert_and_react(&mut store, &mut caches, &row3);
//
//         let expected_points = &[
//             ((present_timestamp, row1.row_id()), points1.as_slice()), //
//             ((present_timestamp, row3.row_id()), points3.as_slice()), //
//         ];
//         let expected_colors = &[
//             ((present_timestamp, row2.row_id()), colors2.as_slice()), //
//         ];
//         query_and_compare(
//             &caches,
//             &store,
//             &query,
//             &entity_path.into(),
//             expected_points,
//             expected_colors,
//         );
//
//         // Modify the optional component
//         let colors4 = vec![MyColor::from_rgb(4, 5, 6), MyColor::from_rgb(7, 8, 9)];
//         let row4 = DataRow::from_cells1_sized(
//             RowId::new(),
//             entity_path,
//             present_data_timepoint,
//             colors4.clone(),
//         )
//         .unwrap();
//         insert_and_react(&mut store, &mut caches, &row4);
//
//         let expected_points = &[
//             ((present_timestamp, row1.row_id()), points1.as_slice()), //
//             ((present_timestamp, row3.row_id()), points3.as_slice()), //
//         ];
//         let expected_colors = &[
//             ((present_timestamp, row2.row_id()), colors2.as_slice()), //
//             ((present_timestamp, row4.row_id()), colors4.as_slice()), //
//         ];
//         query_and_compare(
//             &caches,
//             &store,
//             &query,
//             &entity_path.into(),
//             expected_points,
//             expected_colors,
//         );
//
//         // --- Modify past ---
//
//         // Modify the PoV component
//         let points5 = vec![MyPoint::new(100.0, 200.0), MyPoint::new(300.0, 400.0)];
//         let row5 = DataRow::from_cells1_sized(
//             RowId::new(),
//             entity_path,
//             past_data_timepoint.clone(),
//             points5.clone(),
//         )
//         .unwrap();
//         insert_and_react(&mut store, &mut caches, &row5);
//
//         let expected_points1 = &[
//             ((past_timestamp, row5.row_id()), points5.as_slice()), //
//         ] as &[_];
//         let expected_points2 = &[
//             ((past_timestamp, row5.row_id()), points5.as_slice()), //
//             ((present_timestamp, row1.row_id()), points1.as_slice()), //
//             ((present_timestamp, row3.row_id()), points3.as_slice()), //
//         ] as &[_];
//         let expected_points = if past_data_timepoint.is_static() {
//             expected_points1
//         } else {
//             expected_points2
//         };
//         let expected_colors = &[
//             ((present_timestamp, row2.row_id()), colors2.as_slice()), //
//             ((present_timestamp, row4.row_id()), colors4.as_slice()), //
//         ];
//         query_and_compare(
//             &caches,
//             &store,
//             &query,
//             &entity_path.into(),
//             expected_points,
//             expected_colors,
//         );
//
//         // Modify the optional component
//         let colors6 = vec![MyColor::from_rgb(10, 11, 12), MyColor::from_rgb(13, 14, 15)];
//         let row6 = DataRow::from_cells1_sized(
//             RowId::new(),
//             entity_path,
//             past_data_timepoint.clone(),
//             colors6.clone(),
//         )
//         .unwrap();
//         insert_and_react(&mut store, &mut caches, &row6);
//
//         let expected_colors1 = &[
//             ((past_timestamp, row6.row_id()), colors6.as_slice()), //
//         ] as &[_];
//         let expected_colors2 = &[
//             ((past_timestamp, row6.row_id()), colors6.as_slice()), //
//             ((present_timestamp, row2.row_id()), colors2.as_slice()), //
//             ((present_timestamp, row4.row_id()), colors4.as_slice()), //
//         ] as &[_];
//         let expected_colors = if past_data_timepoint.is_static() {
//             expected_colors1
//         } else {
//             expected_colors2
//         };
//         query_and_compare(
//             &caches,
//             &store,
//             &query,
//             &entity_path.into(),
//             expected_points,
//             expected_colors,
//         );
//
//         // --- Modify future ---
//
//         // Modify the PoV component
//         let points7 = vec![MyPoint::new(1000.0, 2000.0), MyPoint::new(3000.0, 4000.0)];
//         let row7 = DataRow::from_cells1_sized(
//             RowId::new(),
//             entity_path,
//             future_data_timepoint.clone(),
//             points7.clone(),
//         )
//         .unwrap();
//         insert_and_react(&mut store, &mut caches, &row7);
//
//         let expected_points1 = &[
//             ((past_timestamp, row5.row_id()), points5.as_slice()), //
//         ] as &[_];
//         let expected_points2 = &[
//             ((past_timestamp, row5.row_id()), points5.as_slice()), //
//             ((present_timestamp, row1.row_id()), points1.as_slice()), //
//             ((present_timestamp, row3.row_id()), points3.as_slice()), //
//             ((future_timestamp, row7.row_id()), points7.as_slice()), //
//         ] as &[_];
//         let expected_points = if past_data_timepoint.is_static() {
//             expected_points1
//         } else {
//             expected_points2
//         };
//         query_and_compare(
//             &caches,
//             &store,
//             &query,
//             &entity_path.into(),
//             expected_points,
//             expected_colors,
//         );
//
//         // Modify the optional component
//         let colors8 = vec![MyColor::from_rgb(16, 17, 18)];
//         let row8 = DataRow::from_cells1_sized(
//             RowId::new(),
//             entity_path,
//             future_data_timepoint,
//             colors8.clone(),
//         )
//         .unwrap();
//         insert_and_react(&mut store, &mut caches, &row8);
//
//         let expected_colors1 = &[
//             ((past_timestamp, row6.row_id()), colors6.as_slice()), //
//         ] as &[_];
//         let expected_colors2 = &[
//             ((past_timestamp, row6.row_id()), colors6.as_slice()), //
//             ((present_timestamp, row2.row_id()), colors2.as_slice()), //
//             ((present_timestamp, row4.row_id()), colors4.as_slice()), //
//             ((future_timestamp, row8.row_id()), colors8.as_slice()), //
//         ] as &[_];
//         let expected_colors = if past_data_timepoint.is_static() {
//             expected_colors1
//         } else {
//             expected_colors2
//         };
//         query_and_compare(
//             &caches,
//             &store,
//             &query,
//             &entity_path.into(),
//             expected_points,
//             expected_colors,
//         );
//     };
//
//     let timeless = TimePoint::default();
//     let frame_122 = build_frame_nr(122);
//     let frame_123 = build_frame_nr(123);
//     let frame_124 = build_frame_nr(124);
//
//     test_invalidation(
//         RangeQuery::new(frame_123.0, ResolvedTimeRange::EVERYTHING),
//         [frame_123].into(),
//         [frame_122].into(),
//         [frame_124].into(),
//     );
//
//     test_invalidation(
//         RangeQuery::new(frame_123.0, ResolvedTimeRange::EVERYTHING),
//         [frame_123].into(),
//         timeless,
//         [frame_124].into(),
//     );
// }
//
// // Test the following scenario:
// // ```py
// // rr.log("points", rr.Points3D([1, 2, 3]), static=True)
// //
// // # Do first query here: LatestAt(+inf)
// // # Expected: points=[[1,2,3]] colors=[]
// //
// // rr.set_time(2)
// // rr.log_components("points", rr.components.MyColor(0xFF0000))
// //
// // # Do second query here: LatestAt(+inf)
// // # Expected: points=[[1,2,3]] colors=[0xFF0000]
// //
// // rr.set_time(3)
// // rr.log_components("points", rr.components.MyColor(0x0000FF))
// //
// // # Do third query here: LatestAt(+inf)
// // # Expected: points=[[1,2,3]] colors=[0x0000FF]
// //
// // rr.set_time(3)
// // rr.log_components("points", rr.components.MyColor(0x00FF00))
// //
// // # Do fourth query here: LatestAt(+inf)
// // # Expected: points=[[1,2,3]] colors=[0x00FF00]
// // ```
// #[test]
// fn invalidation_of_future_optionals() {
//     let mut store = DataStore2::new(
//         re_log_types::StoreId::random(re_log_types::StoreKind::Recording),
//         Default::default(),
//     );
//     let mut caches = Caches::new(&store);
//
//     let entity_path = "points";
//
//     let timeless = TimePoint::default();
//     let frame2 = [build_frame_nr(2)];
//     let frame3 = [build_frame_nr(3)];
//
//     let query = RangeQuery::new(frame2[0].0, ResolvedTimeRange::EVERYTHING);
//
//     let points1 = vec![MyPoint::new(1.0, 2.0), MyPoint::new(3.0, 4.0)];
//     let row1 =
//         DataRow::from_cells1_sized(RowId::new(), entity_path, timeless, points1.clone()).unwrap();
//     insert_and_react(&mut store, &mut caches, &row1);
//
//     let expected_points = &[
//         ((TimeInt::STATIC, row1.row_id()), points1.as_slice()), //
//     ];
//     let expected_colors = &[];
//     query_and_compare(
//         &caches,
//         &store,
//         &query,
//         &entity_path.into(),
//         expected_points,
//         expected_colors,
//     );
//
//     let colors2 = vec![MyColor::from_rgb(255, 0, 0)];
//     let row2 =
//         DataRow::from_cells1_sized(RowId::new(), entity_path, frame2, colors2.clone()).unwrap();
//     insert_and_react(&mut store, &mut caches, &row2);
//
//     let expected_colors = &[
//         (
//             (TimeInt::new_temporal(2), row2.row_id()),
//             colors2.as_slice(),
//         ), //
//     ];
//     query_and_compare(
//         &caches,
//         &store,
//         &query,
//         &entity_path.into(),
//         expected_points,
//         expected_colors,
//     );
//
//     let colors3 = vec![MyColor::from_rgb(0, 0, 255)];
//     let row3 =
//         DataRow::from_cells1_sized(RowId::new(), entity_path, frame3, colors3.clone()).unwrap();
//     insert_and_react(&mut store, &mut caches, &row3);
//
//     let expected_colors = &[
//         (
//             (TimeInt::new_temporal(2), row2.row_id()),
//             colors2.as_slice(),
//         ), //
//         (
//             (TimeInt::new_temporal(3), row3.row_id()),
//             colors3.as_slice(),
//         ), //
//     ];
//     query_and_compare(
//         &caches,
//         &store,
//         &query,
//         &entity_path.into(),
//         expected_points,
//         expected_colors,
//     );
//
//     let colors4 = vec![MyColor::from_rgb(0, 255, 0)];
//     let row4 =
//         DataRow::from_cells1_sized(RowId::new(), entity_path, frame3, colors4.clone()).unwrap();
//     insert_and_react(&mut store, &mut caches, &row4);
//
//     let expected_colors = &[
//         (
//             (TimeInt::new_temporal(2), row2.row_id()),
//             colors2.as_slice(),
//         ), //
//         (
//             (TimeInt::new_temporal(3), row3.row_id()),
//             colors3.as_slice(),
//         ), //
//         (
//             (TimeInt::new_temporal(3), row4.row_id()),
//             colors4.as_slice(),
//         ), //
//     ];
//     query_and_compare(
//         &caches,
//         &store,
//         &query,
//         &entity_path.into(),
//         expected_points,
//         expected_colors,
//     );
// }
//
// #[test]
// fn invalidation_static() {
//     let mut store = DataStore2::new(
//         re_log_types::StoreId::random(re_log_types::StoreKind::Recording),
//         Default::default(),
//     );
//     let mut caches = Caches::new(&store);
//
//     let entity_path = "points";
//
//     let timeless = TimePoint::default();
//
//     let frame0 = [build_frame_nr(TimeInt::ZERO)];
//     let query = RangeQuery::new(frame0[0].0, ResolvedTimeRange::EVERYTHING);
//
//     let points1 = vec![MyPoint::new(1.0, 2.0), MyPoint::new(3.0, 4.0)];
//     let row1 =
//         DataRow::from_cells1_sized(RowId::new(), entity_path, timeless.clone(), points1.clone())
//             .unwrap();
//     insert_and_react(&mut store, &mut caches, &row1);
//
//     let expected_points = &[
//         ((TimeInt::STATIC, row1.row_id()), points1.as_slice()), //
//     ];
//     let expected_colors = &[];
//     query_and_compare(
//         &caches,
//         &store,
//         &query,
//         &entity_path.into(),
//         expected_points,
//         expected_colors,
//     );
//
//     let colors2 = vec![MyColor::from_rgb(255, 0, 0)];
//     let row2 =
//         DataRow::from_cells1_sized(RowId::new(), entity_path, timeless.clone(), colors2.clone())
//             .unwrap();
//     insert_and_react(&mut store, &mut caches, &row2);
//
//     let expected_colors = &[
//         ((TimeInt::STATIC, row2.row_id()), colors2.as_slice()), //
//     ];
//     query_and_compare(
//         &caches,
//         &store,
//         &query,
//         &entity_path.into(),
//         expected_points,
//         expected_colors,
//     );
//
//     let colors3 = vec![MyColor::from_rgb(0, 0, 255)];
//     let row3 =
//         DataRow::from_cells1_sized(RowId::new(), entity_path, timeless, colors3.clone()).unwrap();
//     insert_and_react(&mut store, &mut caches, &row3);
//
//     let expected_colors = &[
//         ((TimeInt::STATIC, row3.row_id()), colors3.as_slice()), //
//     ];
//     query_and_compare(
//         &caches,
//         &store,
//         &query,
//         &entity_path.into(),
//         expected_points,
//         expected_colors,
//     );
// }
//
// // See <https://github.com/rerun-io/rerun/pull/6214>.
// #[test]
// fn concurrent_multitenant_edge_case() -> anyhow::Result<()> {
//     let mut store = DataStore2::new(
//         re_log_types::StoreId::random(re_log_types::StoreKind::Recording),
//         Default::default(),
//     );
//     let mut caches = Caches::new(&store);
//
//     let entity_path: EntityPath = "point".into();
//
//     let add_points = |time: i64, point_value: f32| {
//         let timepoint = [build_frame_nr(time)];
//         let points = vec![
//             MyPoint::new(point_value, point_value + 1.0),
//             MyPoint::new(point_value + 2.0, point_value + 3.0),
//         ];
//         let row = DataRow::from_cells1_sized(
//             RowId::new(),
//             entity_path.clone(),
//             timepoint,
//             points.clone(),
//         )?;
//         Ok::<_, DataReadError>((timepoint, points, row))
//     };
//
//     let (timepoint1, points1, row1) = add_points(123, 1.0)?;
//     insert_and_react(&mut store, &mut caches, &row1);
//     let (_timepoint2, points2, row2) = add_points(223, 2.0)?;
//     insert_and_react(&mut store, &mut caches, &row2);
//     let (_timepoint3, points3, row3) = add_points(323, 3.0)?;
//     insert_and_react(&mut store, &mut caches, &row3);
//
//     // --- Tenant #1 queries the data, but doesn't cache the result in the deserialization cache ---
//
//     let query = RangeQuery::new(timepoint1[0].0, ResolvedTimeRange::EVERYTHING);
//
//     eprintln!("{}", store.to_data_table().unwrap());
//
//     {
//         let cached = caches.range(
//             &store,
//             &query,
//             &entity_path,
//             MyPoints::all_components().iter().copied(),
//         );
//
//         let _cached_all_points = cached.get_required(MyPoint::name()).unwrap();
//     }
//
//     // --- Meanwhile, tenant #2 queries and deserializes the data ---
//
//     let query = RangeQuery::new(timepoint1[0].0, ResolvedTimeRange::EVERYTHING);
//
//     let expected_points = &[
//         (
//             (TimeInt::new_temporal(123), row1.row_id()),
//             points1.as_slice(),
//         ), //
//         (
//             (TimeInt::new_temporal(223), row2.row_id()),
//             points2.as_slice(),
//         ), //
//         (
//             (TimeInt::new_temporal(323), row3.row_id()),
//             points3.as_slice(),
//         ), //
//     ];
//     query_and_compare(&caches, &store, &query, &entity_path, expected_points, &[]);
//
//     Ok(())
// }
//
// // See <https://github.com/rerun-io/rerun/issues/6279>.
// #[test]
// fn concurrent_multitenant_edge_case2() -> anyhow::Result<()> {
//     let mut store = DataStore2::new(
//         re_log_types::StoreId::random(re_log_types::StoreKind::Recording),
//         Default::default(),
//     );
//     let mut caches = Caches::new(&store);
//
//     let entity_path: EntityPath = "point".into();
//
//     let add_points = |time: i64, point_value: f32| {
//         let timepoint = [build_frame_nr(time)];
//         let points = vec![
//             MyPoint::new(point_value, point_value + 1.0),
//             MyPoint::new(point_value + 2.0, point_value + 3.0),
//         ];
//         let row = DataRow::from_cells1_sized(
//             RowId::new(),
//             entity_path.clone(),
//             timepoint,
//             points.clone(),
//         )?;
//         Ok::<_, DataReadError>((timepoint, points, row))
//     };
//
//     let (timepoint1, points1, row1) = add_points(123, 1.0)?;
//     insert_and_react(&mut store, &mut caches, &row1);
//     let (_timepoint2, points2, row2) = add_points(223, 2.0)?;
//     insert_and_react(&mut store, &mut caches, &row2);
//     let (_timepoint3, points3, row3) = add_points(323, 3.0)?;
//     insert_and_react(&mut store, &mut caches, &row3);
//     let (_timepoint4, points4, row4) = add_points(423, 4.0)?;
//     insert_and_react(&mut store, &mut caches, &row4);
//     let (_timepoint5, points5, row5) = add_points(523, 5.0)?;
//     insert_and_react(&mut store, &mut caches, &row5);
//
//     // --- Tenant #1 queries the data at (123, 223), but doesn't cache the result in the deserialization cache ---
//
//     let query1 = RangeQuery::new(timepoint1[0].0, ResolvedTimeRange::new(123, 223));
//     {
//         let cached = caches.range(
//             &store,
//             &query1,
//             &entity_path,
//             MyPoints::all_components().iter().copied(),
//         );
//
//         let _cached_all_points = cached.get_required(MyPoint::name()).unwrap();
//     }
//
//     // --- Tenant #2 queries the data at (423, 523), but doesn't cache the result in the deserialization cache ---
//
//     let query2 = RangeQuery::new(timepoint1[0].0, ResolvedTimeRange::new(423, 523));
//     {
//         let cached = caches.range(
//             &store,
//             &query2,
//             &entity_path,
//             MyPoints::all_components().iter().copied(),
//         );
//
//         let _cached_all_points = cached.get_required(MyPoint::name()).unwrap();
//     }
//
//     // --- Tenant #2 queries the data at (223, 423) and deserializes it ---
//
//     let query3 = RangeQuery::new(timepoint1[0].0, ResolvedTimeRange::new(223, 423));
//     let expected_points = &[
//         (
//             (TimeInt::new_temporal(223), row2.row_id()),
//             points2.as_slice(),
//         ), //
//         (
//             (TimeInt::new_temporal(323), row3.row_id()),
//             points3.as_slice(),
//         ), //
//         (
//             (TimeInt::new_temporal(423), row4.row_id()),
//             points4.as_slice(),
//         ), //
//     ];
//     query_and_compare(&caches, &store, &query3, &entity_path, expected_points, &[]);
//
//     // --- Tenant #1 finally deserializes its data ---
//
//     let expected_points = &[
//         (
//             (TimeInt::new_temporal(123), row1.row_id()),
//             points1.as_slice(),
//         ), //
//         (
//             (TimeInt::new_temporal(223), row2.row_id()),
//             points2.as_slice(),
//         ), //
//     ];
//     query_and_compare(&caches, &store, &query1, &entity_path, expected_points, &[]);
//
//     // --- Tenant #2 finally deserializes its data ---
//
//     let expected_points = &[
//         (
//             (TimeInt::new_temporal(423), row4.row_id()),
//             points4.as_slice(),
//         ), //
//         (
//             (TimeInt::new_temporal(523), row5.row_id()),
//             points5.as_slice(),
//         ), //
//     ];
//     query_and_compare(&caches, &store, &query2, &entity_path, expected_points, &[]);
//
//     Ok(())
// }

// // ---

fn insert_and_react(store: &mut DataStore2, caches: &mut Caches, chunk: &Arc<Chunk>) {
    caches.on_events(&[store.insert_chunk(chunk).unwrap().unwrap()]);
}

fn query_and_compare(
    caches: &Caches,
    store: &DataStore2,
    query: &RangeQuery,
    entity_path: &EntityPath,
    expected_all_points_indexed: &[((TimeInt, RowId), &[MyPoint])],
    expected_all_colors_indexed: &[((TimeInt, RowId), &[MyColor])],
) {
    re_log::setup_logging();

    let resolver = PromiseResolver::default();

    for _ in 0..3 {
        let cached = caches.range(
            store,
            query,
            entity_path,
            MyPoints::all_components().iter().copied(),
        );

        let cached_all_points = cached
            .get_required(MyPoint::name())
            .unwrap()
            .to_dense::<MyPoint>(&resolver);
        assert!(matches!(
            cached_all_points.status(),
            (PromiseResult::Ready(()), PromiseResult::Ready(())),
        ));
        let cached_all_points_indexed = cached_all_points.range_indexed();

        let cached_all_colors = cached
            .get_or_empty(MyColor::name())
            .to_dense::<MyColor>(&resolver);
        assert!(matches!(
            cached_all_colors.status(),
            (PromiseResult::Ready(()), PromiseResult::Ready(())),
        ));
        let cached_all_colors_indexed = cached_all_colors.range_indexed();

        eprintln!("{query:?}");
        eprintln!("{store}");

        similar_asserts::assert_eq!(
            expected_all_points_indexed,
            cached_all_points_indexed
                .map(|(index, data)| (*index, data))
                .collect_vec(),
        );

        similar_asserts::assert_eq!(
            expected_all_colors_indexed,
            cached_all_colors_indexed
                .map(|(index, data)| (*index, data))
                .collect_vec(),
        );
    }
}
