// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Concrete browser composition. A fresh mandatory IndexedDB admission precedes
//! every load/epoch/fixture write. Neither writer nor backend is exported.
use crate::{
    app::{AppError, GraphshellApp},
    mere_host::{HOST_SLOT, MereHost, MereHostError, SelectedPersonaRef, fixture_handlers},
    product::*,
    resident_host::*,
};
use mere::kernel::graph::Graph;
use muniment::{Backend, IndexedDbBackend};
pub use pandect::{
    ViewKey,
    addressable_graph::{GraphAddress, RefreshOutcome, ResidencyPolicy},
};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Typed read result; its owner identity and publication inputs remain private.
pub type WebRefreshCompletion = HostRefreshCompletion<IndexedDbBackend>;

pub struct WebResidentApp {
    pub host: WebResidentHost,
    pub client: graphshell_client::ClientState,
}
pub struct WebResidentHost {
    inner: Option<MereHost<IndexedDbBackend>>,
    startup: bool,
    publication: Option<String>,
    complete_work: Option<std::rc::Rc<CompleteWork>>,
    progress: Option<WebPersistenceWatch>,
}
/// Observation and a coalesced publication Save notice only. This handle
/// owns no backend, writer, or transaction and cannot start or retry I/O.
#[derive(Clone)]
pub struct WebPersistenceWatch(std::rc::Rc<std::cell::Cell<BrowserPersistenceSnapshot>>);
impl WebPersistenceWatch {
    pub fn snapshot(&self) -> BrowserPersistenceSnapshot {
        self.0.get()
    }
    pub fn request_save(&self) -> bool {
        let mut state = self.0.get();
        let accepted = state.request_save();
        self.0.set(state);
        accepted
    }
}

impl WebResidentHost {
    fn persistence_phase(&self) -> BrowserPersistencePhase {
        if self.pending_publication() {
            BrowserPersistencePhase::PublicationPending
        } else if self.has_unstored() {
            BrowserPersistencePhase::AppliedButUnstored
        } else {
            BrowserPersistencePhase::Stored
        }
    }
    /// A new observation belongs only to the next exclusive retained operation.
    pub fn observe_persistence(&mut self) -> WebPersistenceWatch {
        let watch = WebPersistenceWatch(std::rc::Rc::new(std::cell::Cell::new(
            BrowserPersistenceSnapshot::new(self.persistence_phase()),
        )));
        self.progress = Some(watch.clone());
        watch
    }
    fn publish_persistence_observation(&self) {
        if let Some(watch) = &self.progress {
            let mut state = watch.0.get();
            state.observe(self.persistence_phase());
            watch.0.set(state);
        }
    }
    fn get(&self) -> &MereHost<IndexedDbBackend> {
        self.inner.as_ref().expect("owned browser host")
    }
    fn get_mut(&mut self) -> &mut MereHost<IndexedDbBackend> {
        self.inner.as_mut().expect("owned browser host")
    }
    fn active(&self) -> Result<(), MereHostError> {
        if !self.get().store().is_open() {
            return Err(muniment::StoreError::OwnershipLost(
                "reopen this page from stored truth".into(),
            )
            .into());
        }
        Ok(())
    }
    fn mutation(&self) -> Result<(), MereHostError> {
        self.active()?;
        if self.publication.is_some() || self.complete_work.is_some() {
            return Err(ResidentHostError::Busy.into());
        }
        Ok(())
    }
    pub fn graph(&self) -> &Graph {
        self.get().presentation_graph()
    }
    pub fn presentation_graph(&self) -> &Graph {
        self.get().presentation_graph()
    }
    pub fn session_id(&self) -> pandect::SessionId {
        self.get().graph_session().id()
    }
    pub fn selected_persona(&self) -> &SelectedPersonaRef {
        self.get().selected_persona()
    }
    pub fn was_reopened(&self) -> bool {
        self.get().was_reopened()
    }
    pub fn has_unstored(&self) -> bool {
        self.get().has_unstored()
    }
    pub fn resident_busy(&self) -> bool {
        self.get().resident_busy()
            || self.publication.is_some()
            || self.complete_work.is_some()
            || !self.get().store().is_open()
    }
    pub fn resident_settings(&self) -> &ResidentSettings {
        self.get().resident_settings()
    }
    pub fn resident_reader(
        &self,
    ) -> Option<&pandect::addressable_graph::ResidentGraph<IndexedDbBackend>> {
        self.get().resident_reader()
    }
    pub fn projection_choices(&self) -> Vec<(String, String)> {
        self.get()
            .graph()
            .nodes()
            .filter_map(|(_, node)| {
                let value = self.get().graph().facets().get(
                    &node.id,
                    &chartulary::FacetId::new(PROJECTION_DEFINITION_FACET),
                )?;
                let snapshot: crate::projection_compile::ProjectionSnapshot =
                    serde_json::from_value(value.clone()).ok()?;
                Some((snapshot.definition.id, snapshot.definition.label))
            })
            .collect()
    }
    pub fn coverage_text(&self) -> String {
        crate::view::resident_coverage_text(self.presentation_graph())
    }
    pub fn observation_text(&self, address: GraphAddress) -> String {
        use pandect::addressable_graph::ResidencyStatus;
        let state = self.resident_reader().map(|reader| reader.status(address));
        let label = match state {
            Some(ResidencyStatus::Loaded) => "Loaded",
            Some(ResidencyStatus::NotLoaded) => "Not loaded",
            Some(ResidencyStatus::Absent) => "Missing from published catalog",
            None => "Unreported",
        };
        format!("{label}: {address:?}")
    }
    pub fn set_resident_settings(
        &mut self,
        value: ResidentSettings,
    ) -> Result<(), ResidentHostError> {
        self.mutation()?;
        self.get_mut().set_resident_settings(value)
    }
    pub fn set_resident_pane(&mut self, key: pandect::ViewKey, roots: PaneRoots) {
        self.get_mut().set_resident_pane(key, roots);
    }
    /// Foreground selection is transient context within the configured pane.
    /// Its authored roster and every other pane remain independent.
    pub fn set_foreground_focus(&mut self, focus: Option<GraphAddress>) {
        let pane = ViewKey::new("graphshell", "tree-controls").expect("static foreground pane key");
        self.get_mut().set_resident_pane_focus(pane, focus);
    }
    pub fn resident_settings_need_validation(&self) -> bool {
        self.get().resident_settings_need_validation()
    }
    pub fn resident_settings_pending(&self) -> bool {
        self.get().resident_settings_pending()
    }
    pub fn begin_session_store_attempt(&mut self) -> SessionStoreAttempt {
        self.get_mut().begin_session_store_attempt()
    }
    pub fn session_store_attempt_is_current(&self, attempt: &SessionStoreAttempt) -> bool {
        self.get().session_store_attempt_is_current(attempt)
    }
    pub fn prepare_resident_settings_validation(
        &mut self,
    ) -> Result<Option<HostRefreshRequest<IndexedDbBackend>>, ResidentHostError> {
        self.active()?;
        self.get_mut().prepare_resident_settings_validation()
    }
    pub fn continue_settings_store(
        &mut self,
        attempt: &SessionStoreAttempt,
        completion: HostRefreshCompletion<IndexedDbBackend>,
        store: &mut SessionStore,
        error: &mut String,
    ) -> bool {
        self.get_mut()
            .continue_settings_store(attempt, completion, store, error)
    }
    pub fn prepare_resident_refresh(
        &mut self,
    ) -> Result<Option<HostRefreshRequest<IndexedDbBackend>>, ResidentHostError> {
        self.active()?;
        self.get_mut().prepare_resident_refresh()
    }
    pub fn prepare_resident_demand(
        &mut self,
        address: pandect::addressable_graph::GraphAddress,
    ) -> Result<Option<HostRefreshRequest<IndexedDbBackend>>, ResidentHostError> {
        self.active()?;
        self.get_mut().prepare_resident_demand(address)
    }
    pub fn install_resident_refresh(
        &mut self,
        completion: HostRefreshCompletion<IndexedDbBackend>,
    ) -> Result<pandect::addressable_graph::RefreshOutcome, ResidentHostError> {
        self.active()?;
        let result = self.get_mut().install_resident_refresh(completion)?;
        if self.publication.as_deref().is_some_and(|generation| {
            self.get()
                .resident_reader()
                .is_some_and(|reader| reader.generation() == generation)
        }) {
            self.publication = None;
        }
        self.publish_persistence_observation();
        Ok(result)
    }
    pub fn notice_resident_refresh(&mut self) {
        self.get_mut().notice_resident_refresh();
    }
    pub fn attach_resident_refresh(&mut self) {
        self.get_mut().attach_resident_refresh();
    }
    pub fn detach_resident_refresh(&mut self) {
        self.get_mut().detach_resident_refresh();
    }
    pub fn suspend_resident_refresh(&mut self, value: bool) {
        self.get_mut().suspend_resident_refresh(value);
    }
    pub fn tick_resident_refresh(&mut self, time: f64) {
        self.get_mut().tick_resident_refresh(time);
    }
    pub fn resident_publication_due(&mut self, time: f64, flush: bool) -> bool {
        self.get_mut().resident_publication_due(time, flush)
    }
    pub fn prepare_owned_store(&mut self, secs: u64) -> Result<WebStoreWork, MereHostError> {
        self.active()?;
        if self.publication.is_some() {
            return Err(ResidentHostError::Busy.into());
        }
        let host = self.get_mut();
        if host.resident_admitted {
            Ok(WebStoreWork {
                completed: std::cell::Cell::new(false),
                inner: WebStoreKind::Resident(
                    host.prepare_resident_io(SystemTime::UNIX_EPOCH + Duration::from_secs(secs))?,
                ),
            })
        } else {
            if self.complete_work.is_none() {
                let work = CompleteWork {
                    staged: std::cell::RefCell::new(Some(self.get_mut().prepare_store(secs)?)),
                    backend: self.get().store(),
                };
                self.complete_work = Some(std::rc::Rc::new(work));
            }
            Ok(WebStoreWork {
                completed: std::cell::Cell::new(false),
                inner: WebStoreKind::Complete(self.complete_work.as_ref().unwrap().clone()),
            })
        }
    }
    pub fn pending_publication(&self) -> bool {
        self.publication.is_some()
    }
    pub fn persistence_label(&self) -> &'static str {
        if self.publication.is_some() {
            "Stored; presentation pending"
        } else if self.has_unstored() {
            "Applied but unstored"
        } else {
            "Stored"
        }
    }
    pub fn install_owned_store(&mut self, work: WebStoreWork) -> Result<bool, MereHostError> {
        self.active()?;
        match work.inner {
            WebStoreKind::Resident(io) => {
                let generation = self.get_mut().install_resident_io(&io)?;
                let stored = generation.is_some();
                if stored {
                    self.publication = generation;
                }
                self.publish_persistence_observation();
                Ok(stored)
            },
            WebStoreKind::Complete(retained) if work.completed.get() => {
                if !self
                    .complete_work
                    .as_ref()
                    .is_some_and(|current| std::rc::Rc::ptr_eq(current, &retained))
                {
                    return Err(ResidentHostError::StaleCompletion.into());
                }
                if let Some(staged) = retained.staged.borrow_mut().take() {
                    self.get_mut().staged(staged);
                }
                self.complete_work = None;
                self.publish_persistence_observation();
                Ok(true)
            },
            _ => Ok(false),
        }
    }
    pub async fn refresh(&mut self) -> Result<(), MereHostError> {
        self.publish_persistence_observation();
        let result = self.refresh_owned().await;
        self.publish_persistence_observation();
        result
    }
    async fn refresh_owned(&mut self) -> Result<(), MereHostError> {
        self.active()?;
        self.get_mut().notice_resident_refresh();
        let request = self
            .prepare_resident_refresh()?
            .ok_or(ResidentHostError::Busy)?;
        self.install_resident_refresh(request.load().await)?;
        Ok(())
    }
    pub async fn persist(&mut self, secs: u64) -> Result<(), MereHostError> {
        self.publish_persistence_observation();
        let result = self.persist_owned(secs).await;
        self.publish_persistence_observation();
        result
    }
    async fn persist_owned(&mut self, secs: u64) -> Result<(), MereHostError> {
        self.active()?;
        if self.pending_publication() {
            return self.refresh().await;
        }
        if self.resident_settings_need_validation() {
            let request = self
                .prepare_resident_settings_validation()?
                .ok_or(ResidentHostError::Busy)?;
            self.install_resident_refresh(request.load().await)?;
        }
        let work = self.prepare_owned_store(secs)?;
        let result = work.write().await;
        if !self.install_owned_store(work)? {
            return Err(muniment::StoreError::Backend(
                result
                    .err()
                    .unwrap_or_else(|| "stored result is not acknowledged".into()),
            )
            .into());
        }
        if self.pending_publication() {
            self.refresh().await?;
        }
        self.get_mut().persist_projection_epoch().await
    }
    pub fn view(&self, via: &str, key: &str) -> Option<&pandect::ViewIntent> {
        self.get().view(via, key)
    }
    pub fn set_view_now(
        &mut self,
        via: &str,
        key: &str,
        view: pandect::ViewIntent,
    ) -> Result<(), MereHostError> {
        self.mutation()?;
        self.get_mut().set_view_now(via, key, view)
    }
    pub fn undo_now(&mut self, via: &str) -> Result<Option<pandect::Reverted>, MereHostError> {
        self.mutation()?;
        let result = self.get_mut().undo_now(via)?;
        if result.is_some() && self.get().resident_admitted {
            self.get_mut().hold_resident();
        }
        Ok(result)
    }
    pub fn redo_now(&mut self, via: &str) -> Result<Option<pandect::Reverted>, MereHostError> {
        self.mutation()?;
        let result = self.get_mut().redo_now(via)?;
        if result.is_some() && self.get().resident_admitted {
            self.get_mut().hold_resident();
        }
        Ok(result)
    }
    pub fn facet_value(&self, address: &str, facet: &str) -> Option<&serde_json::Value> {
        let (_, node) = self.presentation_graph().get_node_by_url(address)?;
        self.presentation_graph()
            .facets()
            .get(&node.id, &chartulary::FacetId::new(facet))
    }
    pub fn saved_projection(
        &self,
        id: &str,
    ) -> Result<Option<crate::projection_compile::ProjectionSnapshot>, ProductError> {
        self.get().saved_projection(id)
    }
    pub fn save_projection(
        &mut self,
        value: &crate::projection_compile::ProjectionSnapshot,
    ) -> Result<Uuid, ProductError> {
        self.mutation()?;
        self.get_mut().save_projection(value)
    }
    pub fn product_scene(&self, address: &str) -> Result<SavedSceneV3, ProductError> {
        self.get().ensure_complete_operation()?;
        self.get().product_scene(address)
    }
    pub fn save_product_scene(
        &mut self,
        address: &str,
        scene: &SavedSceneV3,
    ) -> Result<Uuid, ProductError> {
        self.mutation()?;
        self.get_mut().save_product_scene(address, scene)
    }
    pub fn export_product_codicil(&self, request: ExportRequest) -> Result<Vec<u8>, ProductError> {
        self.get().ensure_complete_operation()?;
        self.get().export_product_codicil(request)
    }
    pub fn replace_with_product_codicil(
        &mut self,
        bytes: &[u8],
    ) -> Result<(ImportReceipt, Option<SavedSceneV3>), ProductError> {
        self.mutation()?;
        self.get_mut().replace_with_product_codicil(bytes)
    }
    pub fn create_address(&mut self, address: &str, title: &str) -> Result<Uuid, ProductError> {
        self.mutation()?;
        self.get_mut().create_address(address, title)
    }
    pub fn create_file_metadata(&mut self, value: LocalFileMetadata) -> Result<Uuid, ProductError> {
        self.mutation()?;
        self.get_mut().create_file_metadata(value)
    }
    pub fn edit_node(
        &mut self,
        id: Uuid,
        title: &str,
        tags: impl IntoIterator<Item = String>,
    ) -> Result<(), ProductError> {
        self.mutation()?;
        self.get_mut().edit_node(id, title, tags)
    }
    pub fn edit_metadata(
        &mut self,
        id: Uuid,
        title: &str,
        tags: Vec<String>,
        facet: Option<(String, serde_json::Value)>,
    ) -> Result<(), ProductError> {
        self.mutation()?;
        self.get_mut().edit_product_metadata(id, title, tags, facet)
    }
    pub fn set_product_facet(
        &mut self,
        id: Uuid,
        facet: &str,
        value: &str,
    ) -> Result<(), ProductError> {
        self.mutation()?;
        self.get_mut().set_product_facet(id, facet, value)
    }
    pub fn set_product_pinned(&mut self, id: Uuid, pinned: bool) -> Result<(), ProductError> {
        self.mutation()?;
        self.get_mut().set_product_pinned(id, pinned)
    }
    pub fn assert_product_relation(
        &mut self,
        from: Uuid,
        to: Uuid,
        kind: EditableRelation,
    ) -> Result<(), ProductError> {
        self.mutation()?;
        self.get_mut().assert_product_relation(from, to, kind)
    }
    pub fn matching_members(&self, query: &str, family: RelationFamilyFilter) -> Vec<Uuid> {
        crate::product::matching_members_in_graph(self.presentation_graph(), query, family)
    }
}

/// Bound I/O permits only the prepared operation. It cannot open another writer
/// or expose a backend. Retained ResidentIo preserves uncertain bytes on Drop.
pub struct WebStoreWork {
    completed: std::cell::Cell<bool>,
    inner: WebStoreKind,
}
enum WebStoreKind {
    Resident(ResidentIo<IndexedDbBackend>),
    Complete(std::rc::Rc<CompleteWork>),
}
struct CompleteWork {
    staged: std::cell::RefCell<Option<crate::mere_host::Staged>>,
    backend: IndexedDbBackend,
}
impl WebStoreWork {
    pub async fn write(&self) -> Result<(), String> {
        let result = match &self.inner {
            WebStoreKind::Resident(io) => io.write().await.map_err(|e| e.to_string()),
            WebStoreKind::Complete(work) => {
                let staged = work.staged.borrow();
                staged
                    .as_ref()
                    .ok_or("store already acknowledged")?
                    .write(&work.backend)
                    .await
                    .map_err(|e| e.to_string())
            },
        };
        self.completed.set(result.is_ok());
        result
    }
}
impl WebResidentApp {
    pub async fn open(persona: SelectedPersonaRef) -> Result<Self, String> {
        let backend = IndexedDbBackend::open("graphshell-reference-host-h5", "muniment")
            .await
            .map_err(|e| e.to_string())?;
        let empty = pandect::MereSessions::new(backend.clone())
            .list()
            .await
            .map_err(|e| e.to_string())?
            .is_empty()
            && backend
                .get(HOST_SLOT)
                .await
                .map_err(|e| e.to_string())?
                .is_none();
        let app = if empty {
            GraphshellApp::new(
                MereHost::fixture_recorded(backend, persona, fixture_handlers())
                    .map_err(|e| e.to_string())?,
            )
        } else {
            GraphshellApp::open_or_fixture(backend, persona)
                .await
                .map_err(|e| e.to_string())?
        };
        Ok(Self {
            host: WebResidentHost {
                inner: Some(app.host),
                startup: true,
                publication: None,
                complete_work: None,
                progress: None,
            },
            client: app.client,
        })
    }
    /// Startup capture runs under the acquired backend lease before the first
    /// resident publication. Legacy unqualified sessions remain explicit Full.
    pub async fn finish_startup(&mut self, secs: u64) -> Result<(), String> {
        self.host.active().map_err(|e| e.to_string())?;
        if !self.host.startup {
            return Ok(());
        }
        let host = self.host.get_mut();
        host.persist(secs).await.map_err(|e| e.to_string())?;
        if host.snapshot_placement().is_some() {
            host.admit_resident();
            host.persist_owned(secs).await.map_err(|e| e.to_string())?;
        }
        self.host.startup = false;
        Ok(())
    }
    fn with_app<R>(&mut self, work: impl FnOnce(&mut GraphshellApp<IndexedDbBackend>) -> R) -> R {
        let mut app = GraphshellApp {
            host: self.host.inner.take().expect("browser writer"),
            client: std::mem::take(&mut self.client),
        };
        let result = work(&mut app);
        self.host.inner = Some(app.host);
        self.client = app.client;
        result
    }
    pub fn mount_local(&mut self) -> Result<chirograph::ProjectionSession, AppError> {
        self.with_app(|app| app.mount_local())
    }
    pub fn mount_remote(
        &mut self,
        snapshot: chirograph::ProjectionSnapshot,
    ) -> Result<chirograph::ProjectionSession, AppError> {
        self.with_app(|app| app.mount_remote(snapshot))
    }
    pub fn open_address(
        &mut self,
        address: &str,
        handler: &str,
    ) -> Result<chirograph::IntentResult, AppError> {
        self.host.active()?;
        self.with_app(|app| app.open_address(address, handler))
    }
    pub fn pin_portable_card(
        &mut self,
        session: &chirograph::ProjectionSession,
        instance: sceno::InstanceId,
    ) -> Result<Uuid, AppError> {
        self.host.mutation()?;
        self.with_app(|app| app.pin_portable_card(session, instance))
    }
    pub fn use_fixture_phone(&mut self, at: u64) {
        self.with_app(|app| app.use_fixture_phone(at));
    }
    pub fn metadata(&self, id: Uuid) -> Result<crate::local_edit::NodeMetadata, String> {
        let graph = self.host.presentation_graph();
        let (key, node) = graph
            .get_node_by_id(id)
            .ok_or("Not loaded; demand this item first")?;
        let mut tags: Vec<_> = graph
            .node_content_tags(key)
            .unwrap_or_default()
            .into_iter()
            .collect();
        tags.sort();
        Ok(crate::local_edit::NodeMetadata {
            member: id,
            address: node.url().into(),
            title: node.title.clone(),
            tags,
        })
    }
    pub async fn save_metadata(
        &mut self,
        id: Uuid,
        title: &str,
        tags: &str,
        secs: u64,
    ) -> Result<crate::local_edit::NodeMetadata, String> {
        if self.host.has_unstored() || self.host.pending_publication() {
            return Err("Use Graph residency Save to recover the retained operation before submitting another edit".into());
        }
        self.host
            .edit_node(id, title, tags.split(',').map(str::to_owned))
            .map_err(|e| e.to_string())?;
        self.host.persist(secs).await.map_err(|e| e.to_string())?;
        self.mount_local().map_err(|e| e.to_string())?;
        self.metadata(id)
    }
    pub async fn persist_intake(
        &mut self,
        id: Uuid,
        secs: u64,
    ) -> Result<crate::local_edit::NodeMetadata, String> {
        self.host.persist(secs).await.map_err(|e| e.to_string())?;
        // Intake becomes the selected foreground item, without altering the
        // authored roster or retaining a second pane after selection changes.
        self.host
            .set_foreground_focus(Some(GraphAddress::Surface(id)));
        if let Some(request) = self
            .host
            .prepare_resident_refresh()
            .map_err(|e| e.to_string())?
        {
            self.host
                .install_resident_refresh(request.load().await)
                .map_err(|e| e.to_string())?;
        }
        self.mount_local().map_err(|e| e.to_string())?;
        self.metadata(id)
    }
    pub async fn demand(
        &mut self,
        address: pandect::addressable_graph::GraphAddress,
    ) -> Result<(), String> {
        let request = self
            .host
            .prepare_resident_demand(address)
            .map_err(|e| e.to_string())?
            .ok_or("presentation refresh is busy")?;
        self.host
            .install_resident_refresh(request.load().await)
            .map_err(|e| e.to_string())?;
        self.mount_local().map_err(|e| e.to_string())?;
        Ok(())
    }
    pub async fn capture_startup(
        &mut self,
        policy: crate::capture::HistoryCapturePolicy,
        visits: Vec<crate::capture::BrowserVisit>,
        persona: &str,
        device: &str,
        secs: u64,
    ) -> Result<Vec<crate::capture::CaptureOutcome>, String> {
        if !self.host.startup {
            return Err("history capture requires the startup Complete operation".into());
        }
        self.host.active().map_err(|e| e.to_string())?;
        let mut backend = self.host.get().store();
        let mut capture = crate::capture::BrowserHistoryCapture::load(&mut backend, policy)
            .await
            .map_err(|e| e.to_string())?;
        capture
            .ingest_batch(
                self.host.get_mut(),
                &mut backend,
                visits,
                persona,
                device,
                secs,
            )
            .await
            .map_err(|e| e.to_string())
    }
    pub async fn forget_startup(
        &mut self,
        policy: crate::capture::HistoryCapturePolicy,
        url: &str,
        mode: crate::capture::ForgetMode,
        secs: u64,
    ) -> Result<usize, String> {
        if !self.host.startup {
            return Err("history withdrawal requires the startup Complete operation".into());
        }
        self.host.active().map_err(|e| e.to_string())?;
        let mut backend = self.host.get().store();
        let mut capture = crate::capture::BrowserHistoryCapture::load(&mut backend, policy)
            .await
            .map_err(|e| e.to_string())?;
        capture
            .forget_url(self.host.get_mut(), &mut backend, url, mode, secs)
            .await
            .map_err(|e| e.to_string())
    }
    pub async fn access_records(
        &self,
        filter: &crate::access::AccessRecordFilter,
    ) -> Result<Vec<crate::access::AccessRecord>, String> {
        let mut backend = self.host.get().store();
        crate::access::query_access_records(&mut backend, filter)
            .await
            .map_err(|e| e.to_string())
    }
    pub fn workbench(&self) -> WebWorkbench {
        WebWorkbench {
            backend: self.host.get().store(),
            session: self.host.session_id().0,
            // Reconciliation needs complete membership, not the partial reader.
            // Freeze IDs before async work; keep graph/backend authority private.
            members: std::rc::Rc::new(
                self.host
                    .get()
                    .graph()
                    .nodes()
                    .map(|(_, node)| node.id)
                    .collect(),
            ),
        }
    }
}
/// This bound capability can only load/save this session's validated workbench.
/// It cannot read/write arbitrary keys or construct a session writer.
#[derive(Clone)]
pub struct WebWorkbench {
    backend: IndexedDbBackend,
    session: Uuid,
    members: std::rc::Rc<std::collections::HashSet<Uuid>>,
}
impl WebWorkbench {
    pub async fn load(&self) -> Result<crate::forme_session::FormeSession, String> {
        crate::forme_session::FormeSession::load_for_members(
            self.backend.clone(),
            self.session,
            &self.members,
        )
        .await
    }
    pub async fn save(&self, value: &crate::forme_session::PreparedSave) -> Result<(), String> {
        value
            .save_for_session(self.backend.clone(), self.session)
            .await
    }
}
