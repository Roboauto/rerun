use std::sync::Arc;

use itertools::Itertools;
use nohash_hasher::IntMap;
use parking_lot::Mutex;

use re_chunk::{Chunk, ChunkResult};
use re_data_store2::{DataStore2, DataStoreConfig2, StoreEvent2, StoreSubscriber2};
use re_log_types::{
    external::re_tuid::Tuid, ApplicationId, ComponentPath, EntityPath, EntityPathHash, LogMsg,
    ResolvedTimeRange, ResolvedTimeRangeF, RowId, SetStoreInfo, StoreId, StoreInfo, StoreKind,
    Timeline,
};
use re_types_core::{Archetype, Loggable};

use crate::{Error, TimesPerTimeline};

// ----------------------------------------------------------------------------

/// See [`insert_row_with_retries`].
const MAX_INSERT_ROW_ATTEMPTS: usize = 1_000;

/// See [`insert_row_with_retries`].
const DEFAULT_INSERT_ROW_STEP_SIZE: u64 = 100;

/// See [`GarbageCollectionOptions::time_budget`].
const DEFAULT_GC_TIME_BUDGET: std::time::Duration = std::time::Duration::from_micros(3500); // empirical

// ----------------------------------------------------------------------------

/// An in-memory database built from a stream of [`LogMsg`]es.
///
/// NOTE: all mutation is to be done via public functions!
pub struct EntityDb {
    /// Set by whomever created this [`EntityDb`].
    ///
    /// Clones of an [`EntityDb`] gets a `None` source.
    pub data_source: Option<re_smart_channel::SmartChannelSource>,

    /// Comes in a special message, [`LogMsg::SetStoreInfo`].
    set_store_info: Option<SetStoreInfo>,

    /// Keeps track of the last time data was inserted into this store (viewer wall-clock).
    last_modified_at: web_time::Instant,

    /// The highest `RowId` in the store,
    /// which corresponds to the last edit time.
    /// Ignores deletions.
    latest_row_id: Option<RowId>,

    /// In many places we just store the hashes, so we need a way to translate back.
    entity_path_from_hash: IntMap<EntityPathHash, EntityPath>,

    /// The global-scope time tracker.
    ///
    /// For each timeline, keeps track of what times exist, recursively across all
    /// entities/components.
    ///
    /// Used for time control.
    times_per_timeline: TimesPerTimeline,

    /// A tree-view (split on path components) of the entities.
    tree: crate::EntityTree,

    // TODO: gate this?
    /// Stores all components for all entities for all timelines.
    data_store2: DataStore2,

    /// The active promise resolver for this DB.
    resolver: re_query::PromiseResolver,

    /// Query caches for the data in [`Self::data_store`].
    query_caches: re_query::Caches,

    stats: IngestionStatistics,
}

impl EntityDb {
    pub fn new(store_id: StoreId) -> Self {
        let data_store2 = DataStore2::new(store_id.clone(), DataStoreConfig2::default());
        let query_caches = re_query::Caches::new(&data_store2);

        Self {
            data_source: None,
            set_store_info: None,
            last_modified_at: web_time::Instant::now(),
            latest_row_id: None,
            entity_path_from_hash: Default::default(),
            times_per_timeline: Default::default(),
            tree: crate::EntityTree::root(),
            data_store2,
            resolver: re_query::PromiseResolver::default(),
            query_caches,
            stats: IngestionStatistics::new(store_id),
        }
    }

    #[inline]
    pub fn tree(&self) -> &crate::EntityTree {
        &self.tree
    }

    #[inline]
    pub fn data_store2(&self) -> &DataStore2 {
        &self.data_store2
    }

    pub fn store_info_msg(&self) -> Option<&SetStoreInfo> {
        self.set_store_info.as_ref()
    }

    pub fn store_info(&self) -> Option<&StoreInfo> {
        self.store_info_msg().map(|msg| &msg.info)
    }

    pub fn app_id(&self) -> Option<&ApplicationId> {
        self.store_info().map(|ri| &ri.application_id)
    }

    #[inline]
    pub fn query_caches(&self) -> &re_query::Caches {
        &self.query_caches
    }

    #[inline]
    pub fn resolver(&self) -> &re_query::PromiseResolver {
        &self.resolver
    }

    /// Queries for the given `component_names` using latest-at semantics.
    ///
    /// See [`re_query::LatestAtResults`] for more information about how to handle the results.
    ///
    /// This is a cached API -- data will be lazily cached upon access.
    #[inline]
    pub fn latest_at(
        &self,
        query: &re_data_store2::LatestAtQuery,
        entity_path: &EntityPath,
        component_names: impl IntoIterator<Item = re_types_core::ComponentName>,
    ) -> re_query::LatestAtResults {
        self.query_caches()
            .latest_at(self.store(), query, entity_path, component_names)
    }

    /// Get the latest index and value for a given dense [`re_types_core::Component`].
    ///
    /// This assumes that the row we get from the store contains at most one instance for this
    /// component; it will log a warning otherwise.
    ///
    /// This should only be used for "mono-components" such as `Transform` and `Tensor`.
    ///
    /// This is a best-effort helper, it will merely log errors on failure.
    #[inline]
    pub fn latest_at_component<C: re_types_core::Component>(
        &self,
        entity_path: &EntityPath,
        query: &re_data_store2::LatestAtQuery,
    ) -> Option<re_query::LatestAtMonoResult<C>> {
        self.query_caches().latest_at_component::<C>(
            self.store(),
            self.resolver(),
            entity_path,
            query,
        )
    }

    /// Get the latest index and value for a given dense [`re_types_core::Component`].
    ///
    /// This assumes that the row we get from the store contains at most one instance for this
    /// component; it will log a warning otherwise.
    ///
    /// This should only be used for "mono-components" such as `Transform` and `Tensor`.
    ///
    /// This is a best-effort helper, and will quietly swallow any errors.
    #[inline]
    pub fn latest_at_component_quiet<C: re_types_core::Component>(
        &self,
        entity_path: &EntityPath,
        query: &re_data_store2::LatestAtQuery,
    ) -> Option<re_query::LatestAtMonoResult<C>> {
        self.query_caches().latest_at_component_quiet::<C>(
            self.store(),
            self.resolver(),
            entity_path,
            query,
        )
    }

    #[inline]
    pub fn latest_at_component_at_closest_ancestor<C: re_types_core::Component>(
        &self,
        entity_path: &EntityPath,
        query: &re_data_store2::LatestAtQuery,
    ) -> Option<(EntityPath, re_query::LatestAtMonoResult<C>)> {
        self.query_caches()
            .latest_at_component_at_closest_ancestor::<C>(
                self.store(),
                self.resolver(),
                entity_path,
                query,
            )
    }

    // TODO
    #[inline]
    pub fn store(&self) -> &DataStore2 {
        &self.data_store2
    }

    #[inline]
    pub fn store_kind(&self) -> StoreKind {
        self.store_id().kind
    }

    // TODO
    #[inline]
    pub fn store_id(&self) -> &StoreId {
        self.data_store2.id()
    }

    /// If this entity db is the result of a clone, which store was it cloned from?
    ///
    /// A cloned store always gets a new unique ID.
    ///
    /// We currently only use entity db cloning for blueprints:
    /// when we activate a _default_ blueprint that was received on the wire (e.g. from a recording),
    /// we clone it and make the clone the _active_ blueprint.
    /// This means all active blueprints are clones.
    #[inline]
    pub fn cloned_from(&self) -> Option<&StoreId> {
        self.store_info().and_then(|info| info.cloned_from.as_ref())
    }

    pub fn timelines(&self) -> impl ExactSizeIterator<Item = &Timeline> {
        self.times_per_timeline().keys()
    }

    pub fn times_per_timeline(&self) -> &TimesPerTimeline {
        &self.times_per_timeline
    }

    pub fn has_any_data_on_timeline(&self, timeline: &Timeline) -> bool {
        if let Some(times) = self.times_per_timeline.get(timeline) {
            !times.is_empty()
        } else {
            false
        }
    }

    /// Histogram of all events on the timeeline, of all entities.
    pub fn time_histogram(&self, timeline: &Timeline) -> Option<&crate::TimeHistogram> {
        self.tree().subtree.time_histogram.get(timeline)
    }

    /// Total number of static messages for any entity.
    pub fn num_static_messages(&self) -> u64 {
        self.tree.num_static_messages_recursive()
    }

    /// Returns whether a component is static.
    pub fn is_component_static(&self, component_path: &ComponentPath) -> Option<bool> {
        if let Some(entity_tree) = self.tree().subtree(component_path.entity_path()) {
            entity_tree
                .entity
                .components
                .get(&component_path.component_name)
                .map(|component_histogram| component_histogram.is_static())
        } else {
            None
        }
    }

    #[inline]
    pub fn num_rows(&self) -> usize {
        // TODO
        0
        // self.data_store2.num_static_rows() as usize + self.data_store2.num_temporal_rows() as usize
    }

    /// Return the current `StoreGeneration`. This can be used to determine whether the
    /// database has been modified since the last time it was queried.
    #[inline]
    pub fn generation(&self) -> re_data_store2::StoreGeneration2 {
        self.data_store2.generation()
    }

    #[inline]
    pub fn last_modified_at(&self) -> web_time::Instant {
        self.last_modified_at
    }

    /// The highest `RowId` in the store,
    /// which corresponds to the last edit time.
    /// Ignores deletions.
    #[inline]
    pub fn latest_row_id(&self) -> Option<RowId> {
        self.latest_row_id
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.set_store_info.is_none() && self.num_rows() == 0
    }

    /// A sorted list of all the entity paths in this database.
    pub fn entity_paths(&self) -> Vec<&EntityPath> {
        use itertools::Itertools as _;
        self.entity_path_from_hash.values().sorted().collect()
    }

    #[inline]
    pub fn ingestion_stats(&self) -> &IngestionStatistics {
        &self.stats
    }

    #[inline]
    pub fn entity_path_from_hash(&self, entity_path_hash: &EntityPathHash) -> Option<&EntityPath> {
        self.entity_path_from_hash.get(entity_path_hash)
    }

    /// Returns `true` also for entities higher up in the hierarchy.
    #[inline]
    pub fn is_known_entity(&self, entity_path: &EntityPath) -> bool {
        self.tree.subtree(entity_path).is_some()
    }

    /// If you log `world/points`, then that is a logged entity, but `world` is not,
    /// unless you log something to `world` too.
    #[inline]
    pub fn is_logged_entity(&self, entity_path: &EntityPath) -> bool {
        self.entity_path_from_hash.contains_key(&entity_path.hash())
    }

    pub fn add(&mut self, msg: &LogMsg) -> Result<(), Error> {
        re_tracing::profile_function!();

        debug_assert_eq!(msg.store_id(), self.store_id());

        match &msg {
            LogMsg::SetStoreInfo(msg) => self.set_store_info(msg.clone()),

            LogMsg::ArrowMsg(_, arrow_msg) => {
                self.last_modified_at = web_time::Instant::now();

                // let table = DataTable::from_arrow_msg(arrow_msg)?;
                // self.add_data_table(table)?;

                // TODO
                let mut chunk = re_chunk::Chunk::from_transport(&re_chunk::TransportChunk {
                    schema: arrow_msg.schema.clone(),
                    data: arrow_msg.chunk.clone(),
                })
                .unwrap();
                chunk.sort_if_unsorted();
                self.add_chunk(Arc::new(chunk))?;
            }

            LogMsg::BlueprintActivationCommand(_) => {
                // Not for us to handle
            }
        }

        Ok(())
    }

    // TODO
    pub fn add_chunk(&mut self, mut chunk: Arc<Chunk>) -> Result<(), Error> {
        assert!(chunk.is_sorted()); // TODO

        self.register_entity_path(chunk.entity_path());

        let store_event = self.data_store2.insert_chunk(&chunk).unwrap();

        if self
            .latest_row_id
            .map_or(true, |latest| latest < chunk.row_id_range().1)
        {
            self.latest_row_id = Some(chunk.row_id_range().1);
        }

        if let Some(store_event) = store_event {
            // First-pass: update our internal views by notifying them of resulting [`StoreEvent`]s.
            //
            // This might result in a [`ClearCascade`] if the events trigger one or more immediate
            // and/or pending clears.
            let original_store_events = &[store_event];
            self.times_per_timeline.on_events(original_store_events);
            self.query_caches.on_events(original_store_events);
            let clear_cascade = self.tree.on_store_additions(original_store_events);

            // We inform the stats last, since it measures e2e latency.
            self.stats.on_events(original_store_events);
        }

        Ok(())
    }

    fn register_entity_path(&mut self, entity_path: &EntityPath) {
        self.entity_path_from_hash
            .entry(entity_path.hash())
            .or_insert_with(|| entity_path.clone());
    }

    pub fn set_store_info(&mut self, store_info: SetStoreInfo) {
        self.set_store_info = Some(store_info);
    }

    pub fn gc_everything_but_the_latest_row(&mut self) {
        re_tracing::profile_function!();

        // TODO
        // self.gc(&GarbageCollectionOptions {
        //     target: re_data_store::GarbageCollectionTarget::Everything,
        //     protect_latest: 1, // TODO(jleibs): Bump this after we have an undo buffer
        //     purge_empty_tables: true,
        //     dont_protect: [
        //         re_types_core::components::ClearIsRecursive::name(),
        //         re_types_core::archetypes::Clear::indicator().name(),
        //     ]
        //     .into_iter()
        //     .collect(),
        //     enable_batching: false,
        //     time_budget: DEFAULT_GC_TIME_BUDGET,
        // });
    }

    /// Free up some RAM by forgetting the older parts of all timelines.
    pub fn purge_fraction_of_ram(&mut self, fraction_to_purge: f32) {
        re_tracing::profile_function!();

        assert!((0.0..=1.0).contains(&fraction_to_purge));
        // TODO
        // self.gc(&GarbageCollectionOptions {
        //     target: re_data_store::GarbageCollectionTarget::DropAtLeastFraction(
        //         fraction_to_purge as _,
        //     ),
        //     protect_latest: 1,
        //     purge_empty_tables: false,
        //     dont_protect: Default::default(),
        //     enable_batching: false,
        //     time_budget: DEFAULT_GC_TIME_BUDGET,
        // });
    }

    // TODO
    // pub fn gc(&mut self, gc_options: &GarbageCollectionOptions) {
    //     re_tracing::profile_function!();
    //
    //     let (store_events, stats_diff) = self.data_store.gc(gc_options);
    //
    //     re_log::trace!(
    //         num_row_ids_dropped = store_events.len(),
    //         size_bytes_dropped = re_format::format_bytes(stats_diff.total.num_bytes as _),
    //         "purged datastore"
    //     );
    //
    //     // self.on_store_deletions(&store_events);
    // }

    fn on_store_deletions(&mut self, store_events: &[StoreEvent2]) {
        re_tracing::profile_function!();

        let Self {
            data_source: _,
            set_store_info: _,
            last_modified_at: _,
            latest_row_id: _,
            entity_path_from_hash: _,
            times_per_timeline,
            tree,
            data_store2: _,
            resolver: _,
            query_caches,
            stats: _,
        } = self;

        times_per_timeline.on_events(store_events);
        query_caches.on_events(store_events);

        let store_events = store_events.iter().collect_vec();
        tree.on_store_deletions(&store_events);
    }

    /// Key used for sorting recordings in the UI.
    pub fn sort_key(&self) -> impl Ord + '_ {
        self.store_info()
            .map(|info| (info.application_id.0.as_str(), info.started))
    }

    /// Export the contents of the current database to a sequence of messages.
    ///
    /// If `time_selection` is specified, then only data for that specific timeline over that
    /// specific time range will be accounted for.
    // TODO
    pub fn to_messages(
        &self,
        time_selection: Option<(Timeline, ResolvedTimeRangeF)>,
    ) -> ChunkResult<Vec<LogMsg>> {
        re_tracing::profile_function!();

        let set_store_info_msg = self
            .store_info_msg()
            .map(|msg| Ok(LogMsg::SetStoreInfo(msg.clone())));

        let time_filter = time_selection.map(|(timeline, range)| {
            (
                timeline,
                ResolvedTimeRange::new(range.min.floor(), range.max.ceil()),
            )
        });

        // TODO: time filter
        let data_messages = self.store().iter_chunks().map(|(_row_id, chunk)| {
            chunk.to_transport().map(|transport| {
                LogMsg::ArrowMsg(
                    self.store_id().clone(),
                    re_log_types::ArrowMsg {
                        table_id: Tuid::from_u128(chunk.id().as_u128()),
                        timepoint_max: chunk.timepoint_max(),
                        schema: transport.schema,
                        chunk: transport.data,
                        on_release: None,
                    },
                )
            })
        });

        // If this is a blueprint, make sure to include the `BlueprintActivationCommand` message.
        // We generally use `to_messages` to export a blueprint via "save". In that
        // case, we want to make the blueprint active and default when it's reloaded.
        // TODO(jleibs): Coupling this with the stored file instead of injecting seems
        // architecturally weird. Would be great if we didn't need this in `.rbl` files
        // at all.
        let blueprint_ready = if self.store_kind() == StoreKind::Blueprint {
            let activate_cmd =
                re_log_types::BlueprintActivationCommand::make_active(self.store_id().clone());

            itertools::Either::Left(std::iter::once(Ok(activate_cmd.into())))
        } else {
            itertools::Either::Right(std::iter::empty())
        };

        let messages: Result<Vec<_>, _> = set_store_info_msg
            .into_iter()
            .chain(data_messages)
            .chain(blueprint_ready)
            .collect();

        messages
    }

    /// Make a clone of this [`EntityDb`], assigning it a new [`StoreId`].
    pub fn clone_with_new_id(&self, new_id: StoreId) -> Result<Self, Error> {
        re_tracing::profile_function!();

        let mut new_db = Self::new(new_id.clone());

        new_db.last_modified_at = self.last_modified_at;
        new_db.latest_row_id = self.latest_row_id;

        // We do NOT clone the `data_source`, because the reason we clone an entity db
        // is so that we can modify it, and then it would be wrong to say its from the same source.
        // Specifically: if we load a blueprint from an `.rdd`, then modify it heavily and save it,
        // it would be wrong to claim that this was the blueprint from that `.rrd`,
        // and it would confuse the user.
        // TODO(emilk): maybe we should use a special `Cloned` data source,
        // wrapping either the original source, the original StoreId, or both.

        if let Some(store_info) = self.store_info() {
            let mut new_info = store_info.clone();
            new_info.store_id = new_id;
            new_info.cloned_from = Some(self.store_id().clone());

            new_db.set_store_info(SetStoreInfo {
                row_id: RowId::new(),
                info: new_info,
            });
        }

        // TODO: clone chunks newb
        for (_, chunk) in self.store().iter_chunks() {
            new_db.add_chunk(Arc::clone(chunk))?;
        }

        Ok(new_db)
    }
}

impl re_types_core::SizeBytes for EntityDb {
    #[inline]
    fn heap_size_bytes(&self) -> u64 {
        // TODO
        0
        // // TODO(emilk): size of entire EntityDb, including secondary indices etc
        // self.data_store2.heap_size_bytes()
    }
}

// ----------------------------------------------------------------------------

pub struct IngestionStatistics {
    store_id: StoreId,
    e2e_latency_sec_history: Mutex<emath::History<f32>>,
}

impl StoreSubscriber2 for IngestionStatistics {
    fn name(&self) -> String {
        "rerun.testing.store_subscribers.IngestionStatistics".into()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn on_events(&mut self, events: &[StoreEvent2]) {
        for event in events {
            if event.store_id == self.store_id {
                for &row_id in event.diff.chunk.row_ids() {
                    self.on_new_row_id(row_id);
                }
            }
        }
    }
}

impl IngestionStatistics {
    pub fn new(store_id: StoreId) -> Self {
        let min_samples = 0; // 0: we stop displaying e2e latency if input stops
        let max_samples = 1024; // don't waste too much memory on this - we just need enough to get a good average
        let max_age = 1.0; // don't keep too long of a rolling average, or the stats get outdated.
        Self {
            store_id,
            e2e_latency_sec_history: Mutex::new(emath::History::new(
                min_samples..max_samples,
                max_age,
            )),
        }
    }

    fn on_new_row_id(&mut self, row_id: RowId) {
        if let Ok(duration_since_epoch) = web_time::SystemTime::UNIX_EPOCH.elapsed() {
            let nanos_since_epoch = duration_since_epoch.as_nanos() as u64;

            // This only makes sense if the clocks are very good, i.e. if the recording was on the same machine!
            if let Some(nanos_since_log) =
                nanos_since_epoch.checked_sub(row_id.nanoseconds_since_epoch())
            {
                let now = nanos_since_epoch as f64 / 1e9;
                let sec_since_log = nanos_since_log as f32 / 1e9;

                self.e2e_latency_sec_history.lock().add(now, sec_since_log);
            }
        }
    }

    /// What is the mean latency between the time data was logged in the SDK and the time it was ingested?
    ///
    /// This is based on the clocks of the viewer and the SDK being in sync,
    /// so if the recording was done on another machine, this is likely very inaccurate.
    pub fn current_e2e_latency_sec(&self) -> Option<f32> {
        let mut e2e_latency_sec_history = self.e2e_latency_sec_history.lock();

        if let Ok(duration_since_epoch) = web_time::SystemTime::UNIX_EPOCH.elapsed() {
            let nanos_since_epoch = duration_since_epoch.as_nanos() as u64;
            let now = nanos_since_epoch as f64 / 1e9;
            e2e_latency_sec_history.flush(now); // make sure the average is up-to-date.
        }

        e2e_latency_sec_history.average()
    }
}
