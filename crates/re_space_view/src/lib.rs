//! Rerun Space View utilities
//!
//! Types & utilities for defining Space View classes and communicating with the Viewport.

pub mod controls;

mod heuristics;
mod query;
mod results_ext;
mod screenshot;
mod view_property_ui;

pub use heuristics::suggest_space_view_for_each_entity;
pub use query::{latest_at_with_overrides, range_with_overrides, HybridResults};
pub use results_ext::RangeResultsExt;
pub use screenshot::ScreenshotMode;
pub use view_property_ui::view_property_ui;

pub mod external {
    pub use re_entity_db::external::*;
}

// -----------

/// Utility for implementing [`re_viewer_context::VisualizerAdditionalApplicabilityFilter`] using on the properties of a concrete component.
#[inline]
pub fn diff_component_filter<T: re_types_core::Component>(
    event: &re_data_store2::StoreEvent2,
    filter: impl Fn(&T) -> bool,
) -> bool {
    let filter = &filter;
    event
        .diff
        .chunk
        .components()
        .get(&T::name())
        .map_or(false, |list_array| {
            list_array
                .iter()
                .filter_map(|array| array.and_then(|array| T::from_arrow(&*array).ok()))
                .any(|instances| instances.iter().any(filter))
        })
}
