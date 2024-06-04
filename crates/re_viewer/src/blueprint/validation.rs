use re_data_store2::LatestAtQuery;
use re_entity_db::EntityDb;
use re_log_types::Timeline;
use re_types_core::Component;

pub(crate) fn validate_component<C: Component>(blueprint: &EntityDb) -> bool {
    // TODO: this is really problematic in a multi-datatype world.
    // can we get around this?
    return true;

    let query = LatestAtQuery::latest(Timeline::default());

    // if let Some(data_type) = blueprint.data_store2().lookup_datatype(&C::name()) {
    //     if data_type != &C::arrow_datatype() {
    //         // If the schemas don't match, we definitely have a problem
    //         re_log::debug!(
    //             "Unexpected datatype for component {:?}.\nFound: {:#?}\nExpected: {:#?}",
    //             C::name(),
    //             data_type,
    //             C::arrow_datatype()
    //         );
    //         return false;
    //     } else {
    //         // Otherwise, our usage of serde-fields means we still might have a problem
    //         // this can go away once we stop using serde-fields.
    //         // Walk the blueprint and see if any cells fail to deserialize for this component type.
    //         for path in blueprint.entity_paths() {
    //             if let Some([Some(cell)]) = blueprint
    //                 .data_store()
    //                 .latest_at(&query, path, C::name(), &[C::name()])
    //                 .map(|(_, _, cells)| cells)
    //             {
    //                 if let Err(err) = cell.try_to_native_mono::<C>() {
    //                     re_log::debug!(
    //                         "Failed to deserialize component {:?}: {:?}",
    //                         C::name(),
    //                         err
    //                     );
    //                     return false;
    //                 }
    //             }
    //         }
    //     }
    // }

    true
}
