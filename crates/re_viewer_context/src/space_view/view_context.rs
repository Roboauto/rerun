use re_log_types::{DataCell, EntityPath, TimePoint};
use re_types::{AsComponents, ComponentBatch, ComponentName};

use crate::{DataQueryResult, SpaceViewId};

pub struct ViewContext<'a> {
    pub viewer_ctx: &'a crate::ViewerContext<'a>,
    pub view_id: SpaceViewId,
    pub view_state: &'a dyn crate::SpaceViewState,
    pub visualizer_collection: &'a crate::VisualizerCollection,
}

impl<'a> ViewContext<'a> {
    /// The active recording.
    #[inline]
    pub fn recording(&self) -> &re_entity_db::EntityDb {
        self.viewer_ctx.recording()
    }

    /// The data store of the active recording.
    #[inline]
    pub fn recording_store(&self) -> &re_data_store::DataStore {
        self.viewer_ctx.recording_store()
    }

    /// The active blueprint.
    #[inline]
    pub fn blueprint_db(&self) -> &re_entity_db::EntityDb {
        self.viewer_ctx.blueprint_db()
    }

    /// The `StoreId` of the active recording.
    #[inline]
    pub fn recording_id(&self) -> &re_log_types::StoreId {
        self.viewer_ctx.recording_id()
    }

    /// Returns the current selection.
    #[inline]
    pub fn selection(&self) -> &crate::ItemCollection {
        self.viewer_ctx.selection()
    }

    /// Returns the currently hovered objects.
    #[inline]
    pub fn hovered(&self) -> &crate::ItemCollection {
        self.viewer_ctx.hovered()
    }

    #[inline]
    pub fn selection_state(&self) -> &crate::ApplicationSelectionState {
        self.viewer_ctx.selection_state()
    }

    /// The current time query, based on the current time control.
    #[inline]
    pub fn current_query(&self) -> re_data_store::LatestAtQuery {
        self.viewer_ctx.current_query()
    }

    /// Set hover/select/focus for a given selection based on an egui response.
    #[inline]
    pub fn select_hovered_on_click(
        &self,
        response: &egui::Response,
        selection: impl Into<crate::ItemCollection>,
    ) {
        self.viewer_ctx.select_hovered_on_click(response, selection);
    }

    #[inline]
    pub fn lookup_query_result(&self, id: SpaceViewId) -> &DataQueryResult {
        self.viewer_ctx.lookup_query_result(id)
    }

    #[inline]
    pub fn save_blueprint_archetype(&self, entity_path: EntityPath, components: &dyn AsComponents) {
        self.viewer_ctx
            .save_blueprint_archetype(entity_path, components);
    }

    #[inline]
    pub fn save_blueprint_component(
        &self,
        entity_path: &EntityPath,
        components: &dyn ComponentBatch,
    ) {
        self.viewer_ctx
            .save_blueprint_component(entity_path, components);
    }

    #[inline]
    pub fn save_blueprint_data_cell(&self, entity_path: &EntityPath, data_cell: DataCell) {
        self.viewer_ctx
            .save_blueprint_data_cell(entity_path, data_cell);
    }

    #[inline]
    pub fn save_empty_blueprint_component<C>(&self, entity_path: &EntityPath)
    where
        C: re_types::Component + 'a,
    {
        self.viewer_ctx
            .save_empty_blueprint_component::<C>(entity_path);
    }

    #[inline]
    pub fn reset_blueprint_component_by_name(
        &self,
        entity_path: &EntityPath,
        component_name: ComponentName,
    ) {
        self.viewer_ctx
            .reset_blueprint_component_by_name(entity_path, component_name);
    }

    #[inline]
    pub fn save_empty_blueprint_component_by_name(
        &self,
        entity_path: &EntityPath,
        component_name: ComponentName,
    ) {
        self.viewer_ctx
            .save_empty_blueprint_component_by_name(entity_path, component_name);
    }

    #[inline]
    pub fn blueprint_timepoint_for_writes(&self) -> TimePoint {
        self.viewer_ctx
            .store_context
            .blueprint_timepoint_for_writes()
    }
}
