// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Concrete native ownership. Backend handles and mutable writers never escape.

use super::*;
use chirograph::{
    IntentInvocation, IntentResult, ProjectionRequest, ProjectionSession, ProjectionSnapshot,
    ResourceRequest, ResourceResponse,
};
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionSource};
use mere::kernel::graph::{CapturedDelta, Graph, Seq};
use pandect::{Author, GraphSession, GraphSessionManifest, SessionError, SessionId, ViewIntent};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex as AsyncMutex;

/// One fresh Directory lease and one complete writer per cached SessionId.
/// No backend, MereSessions, raw host or caller-host callback is exposed.
pub struct DirectoryHostCache<B = muniment::DirectoryBackend> {
    backend: B,
    root: PathBuf,
    hosts: AsyncMutex<HashMap<SessionId, Arc<AsyncMutex<CachedResidentSession<B>>>>>,
}

impl DirectoryHostCache<muniment::DirectoryBackend> {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ResidentHostError> {
        let root = path.as_ref().to_owned();
        let backend = muniment::DirectoryBackend::open(&root)?;
        Ok(Self::from_fresh_backend(backend, root))
    }
}

impl<B: Backend + Clone> DirectoryHostCache<B> {
    // Only the concrete open above and the sealed fresh-Directory fault fixture
    // may call this. It is never a public generic admission hook.
    pub(super) fn from_fresh_backend(backend: B, root: PathBuf) -> Self {
        Self {
            backend,
            root,
            hosts: AsyncMutex::new(HashMap::new()),
        }
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub async fn cached_sessions(&self) -> usize {
        self.hosts.lock().await.len()
    }
    pub async fn host_checked(
        &self,
        id: SessionId,
        persona: SelectedPersonaRef,
        handlers: HandlerRegistry,
        access: AccessContext,
        validator: Option<pandect::GraphValidator>,
        projection: ProjectionSession,
    ) -> Result<Arc<AsyncMutex<CachedResidentSession<B>>>, MereHostError> {
        let mut hosts = self.hosts.lock().await;
        if let Some(host) = hosts.get(&id) {
            return Ok(Arc::clone(host));
        }
        // Product validation occurs in this existing constructor before epoch IO.
        let mut host = MereHost::open_session_checked(
            self.backend.clone(),
            id,
            persona,
            handlers,
            access,
            validator,
        )
        .await?
        .with_projection_session(projection)
        .with_session_item();
        if host.graph_session().placement().is_some() {
            host.admit_resident();
            // A qualified ordinary checkpoint can precede its first resident
            // publication. Only actual absence permits that first write;
            // existing malformed records and read errors remain failures.
            if self
                .backend
                .get(&host.resident_key("head"))
                .await?
                .is_none()
            {
                host.persist_resident(SystemTime::now()).await?;
            } else {
                host.refresh_resident_reader().await?;
            }
        }
        let mut session = CachedResidentSession {
            host,
            applied: None,
            publication: None,
        };
        session.status(SessionStore::Stored, false, None);
        let host = Arc::new(AsyncMutex::new(session));
        hosts.insert(id, Arc::clone(&host));
        Ok(host)
    }
    fn sessions(&self) -> pandect::MereSessions<B> {
        pandect::MereSessions::new(self.backend.clone())
    }
    pub async fn list(&self) -> Result<Vec<GraphSessionManifest>, SessionError> {
        self.sessions().list().await
    }
    pub async fn latest_live(&self) -> Result<Option<GraphSessionManifest>, SessionError> {
        self.sessions().latest_live().await
    }
    pub async fn mint(
        &self,
        author: Author,
        name: Option<String>,
    ) -> Result<GraphSessionManifest, SessionError> {
        self.sessions().mint(author, name).await
    }
    pub async fn fork_at(
        &self,
        parent: &CachedResidentSession<B>,
        author: Author,
    ) -> Result<GraphSessionManifest, MereHostError> {
        parent.idle()?;
        let source = parent.graph_session();
        Ok(self
            .sessions()
            .fork_at(source, source.journal().live_cursor(), author)
            .await?)
    }
    /// Validate and write the first edit with its new fork, then drop the
    /// temporary writer before the caller can obtain the cached session.
    pub async fn fork_with_intent(
        &self,
        parent: &CachedResidentSession<B>,
        author: Author,
        app: &str,
        intent: &IntentInvocation,
    ) -> Result<SessionId, String> {
        parent.idle().map_err(|e| e.to_string())?;
        let source = parent.graph_session();
        let mut fork = self
            .sessions()
            .begin_fork_at(source, source.journal().live_cursor(), author.clone())
            .map_err(|e| e.to_string())?;
        match intent.intent.as_str() {
            crate::session_item::APPLY_EDITS_INTENT => {
                let payload: crate::session_item::ApplyEditsV1 =
                    serde_json::from_slice(&intent.payload).map_err(|e| e.to_string())?;
                if payload.schema != crate::session_item::APPLY_EDITS_SCHEMA
                    || payload.edits.is_empty()
                {
                    return Err("invalid or empty archive edit".into());
                }
                fork.apply_now(author, payload.edits)
                    .map_err(|e| e.to_string())?;
                if fork.journal().live_cursor().0 == 0 {
                    return Err("archive edit changes nothing".into());
                }
            },
            crate::session_item::SET_VIEW_INTENT => {
                let payload: crate::session_item::SetViewV1 =
                    serde_json::from_slice(&intent.payload).map_err(|e| e.to_string())?;
                if payload.schema != crate::session_item::SET_VIEW_SCHEMA {
                    return Err("unknown view schema".into());
                }
                fork.set_view_now(
                    author,
                    pandect::ViewKey::new(app, &payload.view).map_err(|e| e.to_string())?,
                    payload.state,
                )
                .map_err(|e| e.to_string())?;
            },
            _ => return Err("read-only archive; fork before this action".into()),
        }
        fork.flush(SystemTime::now())
            .await
            .map_err(|e| e.to_string())?;
        Ok(fork.id())
    }
    pub async fn open_codicil_from_checked(
        &self,
        store: &mut dyn eidetic::Store,
        id: eidetic::ManifestId,
        author: Author,
        validate: impl FnOnce(&Graph) -> Result<(), String>,
    ) -> Result<GraphSessionManifest, SessionError> {
        self.sessions()
            .open_codicil_from_checked(store, id, author, validate)
            .await
    }
    #[cfg(any(test, feature = "resident-test-support"))]
    pub async fn stored_bytes(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        self.backend.get(key).await
    }
    #[cfg(any(test, feature = "resident-test-support"))]
    pub async fn all_stored_bytes(&self) -> Result<BTreeMap<String, Vec<u8>>, StoreError> {
        let mut values = BTreeMap::new();
        for key in self.backend.list("").await? {
            if let Some(value) = self.backend.get(&key).await? {
                values.insert(key, value);
            }
        }
        Ok(values)
    }
}

/// Named operations over a private cached writer. Read access cannot yield a
/// backend clone or a second mutable session.
pub struct CachedResidentSession<B> {
    host: MereHost<B>,
    applied: Option<Applied>,
    publication: Option<String>,
}

impl<B: Backend + Clone> CachedResidentSession<B> {
    fn idle(&self) -> Result<(), MereHostError> {
        if self.recovery_pending() {
            Err(MereHostError::ResidentBusy)
        } else {
            Ok(())
        }
    }
    pub fn recovery_pending(&self) -> bool {
        self.host.resident_admitted
            && (self.applied.is_some()
                || self.publication.is_some()
                || self.host.resident_busy()
                || (self.host.has_unstored()
                    && self
                        .host
                        .owned_persistence_status
                        .as_ref()
                        .is_some_and(|s| s.error.is_some())))
    }
    /// Check before an external archive write, as well as its session receipt.
    pub fn ensure_metadata_idle(&self) -> Result<(), MereHostError> {
        self.idle()?;
        if self.host.resident_admitted && self.host.has_unstored() {
            return Err(MereHostError::ResidentBusy);
        }
        Ok(())
    }
    fn status(&mut self, store: SessionStore, publication_pending: bool, error: Option<String>) {
        let status = OwnedPersistenceStatus {
            store,
            publication_pending,
            error,
        };
        if self.host.owned_persistence_status.as_ref() != Some(&status) {
            self.host.owned_persistence_status = Some(status);
            self.host.projection_revision = self.host.projection_revision.wrapping_add(1);
        }
    }
    fn applied(&mut self, first: Seq) {
        self.applied = Some(Applied {
            first,
            end: self.graph_session().journal().live_cursor(),
            revision: self.graph_session().revision(),
        });
        if self.host.resident_admitted {
            self.host.hold_resident();
        }
        self.status(SessionStore::Pending, true, None);
    }
    pub fn apply_edits_as(
        &mut self,
        via: &str,
        edits: Vec<CapturedDelta>,
    ) -> Result<(), MereHostError> {
        self.idle()?;
        let first = self.graph_session().journal().live_cursor();
        self.host
            .through(via.to_string(), |host| host.apply_edits(edits))?;
        if self.host.has_unstored() {
            self.applied(first);
        }
        Ok(())
    }
    pub fn invoke_as(
        &mut self,
        via: &str,
        intent: IntentInvocation,
    ) -> Result<IntentResult, MereHostError> {
        self.idle()?;
        let first = self.graph_session().journal().live_cursor();
        let result = self
            .host
            .through(via.to_string(), |host| host.invoke(intent))?;
        if result == IntentResult::Accepted && self.host.has_unstored() {
            self.applied(first);
        }
        Ok(result)
    }
    pub fn undo_now(&mut self, via: &str) -> Result<Option<pandect::Reverted>, MereHostError> {
        self.idle()?;
        let first = self.graph_session().journal().live_cursor();
        let result = self.host.undo_now(via)?;
        if result.is_some() {
            self.applied(first);
        }
        Ok(result)
    }
    pub fn redo_now(&mut self, via: &str) -> Result<Option<pandect::Reverted>, MereHostError> {
        self.idle()?;
        let first = self.graph_session().journal().live_cursor();
        let result = self.host.redo_now(via)?;
        if result.is_some() {
            self.applied(first);
        }
        Ok(result)
    }
    pub fn set_view_now(
        &mut self,
        via: &str,
        view: &str,
        state: ViewIntent,
    ) -> Result<(), MereHostError> {
        self.idle()?;
        let first = self.graph_session().journal().live_cursor();
        self.host.set_view_now(via, view, state)?;
        self.applied(first);
        Ok(())
    }
    /// Storage outcomes remain distinct from command validation. A failed
    /// write retains the exact batch; a failed reader load retains only publication.
    pub async fn persist_owned(
        &mut self,
        at_secs: u64,
    ) -> Result<ResidentPersistence, ResidentHostError> {
        let result = self.persist_inner(at_secs).await;
        if let Err(error) = &result {
            self.status(
                if self.host.has_unstored() {
                    SessionStore::Failed
                } else {
                    SessionStore::Stored
                },
                true,
                Some(error.to_string()),
            );
        }
        result
    }
    async fn persist_inner(
        &mut self,
        at_secs: u64,
    ) -> Result<ResidentPersistence, ResidentHostError> {
        if !self.host.resident_admitted {
            self.host.persist(at_secs).await?;
            self.applied = None;
            self.status(SessionStore::Stored, false, None);
            return Ok(ResidentPersistence::Complete);
        }
        if self.publication.is_none() {
            if !self.host.has_unstored()
                && self.host.resident_reader().is_some()
                && self.applied.is_none()
            {
                self.status(SessionStore::Stored, false, None);
                return Ok(ResidentPersistence::Durable {
                    generation: self
                        .host
                        .resident_reader()
                        .unwrap()
                        .generation()
                        .to_string(),
                });
            }
            if self.host.resident_settings_need_validation() {
                let request = self
                    .host
                    .prepare_resident_settings_validation()?
                    .ok_or(ResidentHostError::Busy)?;
                self.host.install_resident_refresh(request.load().await)?;
            }
            let io = self.host.prepare_resident_io(
                SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(at_secs),
            )?;
            let write = io.write().await;
            let generation = self.host.install_resident_io(&io)?;
            let Some(generation) = generation else {
                let reason = write.err().unwrap_or(ResidentHostError::Busy);
                self.status(SessionStore::Failed, true, Some(reason.to_string()));
                if matches!(reason, ResidentHostError::MixedCommit) {
                    return Err(reason);
                }
                let first = self
                    .applied
                    .as_ref()
                    .map(|a| a.first.0)
                    .unwrap_or(self.graph_session().journal().live_cursor().0);
                return Ok(ResidentPersistence::AppliedButUnstored {
                    first,
                    end: self.graph_session().journal().live_cursor().0,
                    revision: self.graph_session().revision(),
                });
            };
            self.applied = None;
            self.publication = Some(generation);
        }
        let generation = self.publication.clone().unwrap();
        if let Err(error) = self.host.refresh_resident_reader().await {
            let reason = error.to_string();
            self.status(SessionStore::Stored, true, Some(reason.clone()));
            return Ok(ResidentPersistence::PendingPublication { generation, reason });
        }
        if let Err(error) = self.host.persist_projection_epoch().await {
            let reason = error.to_string();
            self.status(SessionStore::Stored, true, Some(reason.clone()));
            return Ok(ResidentPersistence::PendingPublication { generation, reason });
        }
        self.publication = None;
        self.status(SessionStore::Stored, false, None);
        Ok(ResidentPersistence::Durable { generation })
    }
    pub fn graph(&self) -> &Graph {
        self.host.graph()
    }
    pub fn graph_session(&self) -> &GraphSession<B> {
        self.host.graph_session()
    }
    pub fn presentation_graph(&self) -> &Graph {
        self.host.presentation_graph()
    }
    pub fn resident_reader(&self) -> Option<&ResidentGraph<B>> {
        self.host.resident_reader()
    }
    pub fn has_unstored(&self) -> bool {
        self.host.has_unstored()
    }
    pub fn projection_epoch(&self) -> scenotime::SceneEpoch {
        self.host.projection_epoch()
    }
    pub fn projection_revision(&self) -> scenotime::Revision {
        self.host.projection_revision()
    }
    pub fn session(&self) -> ProjectionSession {
        self.host.session()
    }
    pub fn local_request(&self) -> ProjectionRequest {
        self.host.local_request()
    }
    pub fn session_instance(&self) -> Option<sceno::InstanceId> {
        self.host.session_instance()
    }
    pub fn snapshot(
        &mut self,
        request: ProjectionRequest,
    ) -> Result<ProjectionSnapshot, MereHostError> {
        self.host.snapshot(request)
    }
    pub fn resource(
        &mut self,
        request: ResourceRequest,
    ) -> Result<ResourceResponse, MereHostError> {
        self.host.resource(request)
    }
    pub fn sync_projection_context(&mut self) {
        self.host.sync_projection_context();
    }
    pub fn resident_roots(&self) -> &HostRoots {
        self.host.resident_roots()
    }
    pub fn set_resident_pane(&mut self, pane: pandect::ViewKey, roots: PaneRoots) {
        self.host.set_resident_pane(pane, roots);
    }
    pub fn close_resident_pane(&mut self, pane: &pandect::ViewKey) {
        self.host.close_resident_pane(pane);
    }
    pub fn set_resident_pins(&mut self, pins: BTreeSet<GraphAddress>) {
        self.host.set_resident_pins(pins);
    }
    pub fn notice_resident_refresh(&mut self) {
        self.host.notice_resident_refresh();
    }
    pub async fn refresh_resident_reader(&mut self) -> Result<(), ResidentHostError> {
        self.idle()?;
        self.host.refresh_resident_reader().await
    }
    pub fn prepare_resident_refresh(
        &mut self,
    ) -> Result<Option<HostRefreshRequest<B>>, ResidentHostError> {
        self.idle()?;
        self.host.prepare_resident_refresh()
    }
    pub fn prepare_resident_demand(
        &mut self,
        address: GraphAddress,
    ) -> Result<Option<HostRefreshRequest<B>>, ResidentHostError> {
        self.idle()?;
        self.host.prepare_resident_demand(address)
    }
    #[cfg(any(test, feature = "resident-test-support"))]
    pub fn refresh_in_flight_for_test(&self) -> bool {
        self.host.resident_refresh.gate.in_flight.is_some()
    }

    #[cfg(any(test, feature = "resident-test-support"))]
    pub fn refresh_notices_for_test(&self) -> (u64, u64) {
        (
            self.host.resident_refresh.gate.newest_notice,
            self.host.resident_refresh.gate.consumed_notice,
        )
    }

    pub fn install_resident_refresh(
        &mut self,
        completion: HostRefreshCompletion<B>,
    ) -> Result<pandect::addressable_graph::RefreshOutcome, ResidentHostError> {
        if self.recovery_pending() {
            return self.host.decline_resident_refresh(completion);
        }
        self.host.install_resident_refresh(completion)
    }
    pub fn set_resident_settings(
        &mut self,
        settings: ResidentSettings,
    ) -> Result<(), ResidentHostError> {
        self.idle()?;
        self.host.set_resident_settings(settings)
    }
    pub fn set_resident_host_coverage(
        &mut self,
        note: mere::kernel::graph::CoverageNote,
    ) -> Result<(), ResidentHostError> {
        self.host.set_resident_host_coverage(note)
    }
    pub fn save_projection(
        &mut self,
        projection: &crate::projection_compile::ProjectionSnapshot,
    ) -> Result<uuid::Uuid, crate::product::ProductError> {
        self.idle().map_err(crate::product::ProductError::Host)?;
        let first = self.graph_session().journal().live_cursor();
        let result = self.host.save_projection(projection)?;
        self.applied(first);
        Ok(result)
    }
    pub fn view(&self, via: &str, view: &str) -> Option<&ViewIntent> {
        self.host.view(via, view)
    }
    pub async fn archive_receipt(&mut self, via: &str, id: String) -> Result<(), MereHostError> {
        self.idle()?;
        self.host.archive_receipt(via, id).await
    }
    pub async fn trash(&mut self, via: &str) -> Result<(), MereHostError> {
        self.idle()?;
        self.host.trash(via).await
    }
    pub async fn restore(&mut self, via: &str) -> Result<(), MereHostError> {
        self.idle()?;
        self.host.restore(via).await
    }
    #[cfg(any(test, feature = "resident-test-support"))]
    pub async fn publish_pending_retention_for_test(
        &self,
        retention: mere::kernel::graph::PendingLinkRetention,
    ) -> Result<(), StoreError> {
        let mut source = self.host.graph().clone();
        source.set_pending_link_retention(retention);
        pandect::addressable_graph::AddressableGraphStore::new(
            self.host.store(),
            *self.graph_session().manifest().root_graph_id.as_uuid(),
        )
        .write_recorded_graph(&source)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod completion_tests {
    use super::*;

    async fn reject_unrelated_completion(kind: u8) {
        let root = tempfile::tempdir().unwrap();
        let cache = DirectoryHostCache::open(root.path()).unwrap();
        let id = cache.mint(Author::user(), None).await.unwrap().session_id;
        let owner = cache
            .host_checked(
                id,
                SelectedPersonaRef {
                    persona: "completion-owner".into(),
                    profile: "test".into(),
                },
                HandlerRegistry::default(),
                AccessContext {
                    persona: "completion-owner".into(),
                    device: "test".into(),
                    at_ms: 0,
                },
                None,
                ProjectionSession("v1:completion".into()),
            )
            .await
            .unwrap();
        let request = {
            let mut host = owner.lock().await;
            host.notice_resident_refresh();
            host.prepare_resident_refresh().unwrap().unwrap()
        };
        let held = request.load().await;
        assert!(held.result.is_ok());
        let mut wrong = HostRefreshCompletion {
            identity: held.identity.clone(),
            token: held.token.clone(),
            preflight: held.preflight,
            result: Err(StoreError::Backend(
                "unrelated completion must not retire active flight".into(),
            )),
        };
        match kind {
            0 => wrong.identity = Arc::new(()),
            1 => wrong.token.session = SessionId::new(),
            2 => wrong.token.request = wrong.token.request.wrapping_add(1),
            _ => unreachable!(),
        }
        let mut host = owner.lock().await;
        host.apply_edits_as(
            "peer",
            vec![CapturedDelta::ReplayAddNodeWithIdIfMissing {
                id: uuid::Uuid::from_u128(7501).to_string(),
                url: "https://7501.test/".into(),
                position: [0.0, 0.0],
            }],
        )
        .unwrap();
        assert!(matches!(
            host.persist_owned(0).await.unwrap(),
            ResidentPersistence::PendingPublication { .. }
        ));
        let active = host.host.resident_refresh.gate.in_flight.clone();
        let notice = host.host.resident_refresh.gate.newest_notice;
        let consumed = host.host.resident_refresh.gate.consumed_notice;
        let status = host.host.owned_persistence_status.clone();
        let publication = host.publication.clone();
        let cursor = host.graph_session().journal().live_cursor();
        assert!(active.is_some());
        assert!(host.install_resident_refresh(wrong).is_err());
        assert_eq!(
            host.host.resident_refresh.gate.in_flight, active,
            "wrong owner/session/attempt cannot clear the active request"
        );
        assert_eq!(host.host.resident_refresh.gate.newest_notice, notice);
        assert_eq!(host.host.resident_refresh.gate.consumed_notice, consumed);
        assert_eq!(host.host.owned_persistence_status, status);
        assert_eq!(host.publication, publication);
        assert_eq!(host.graph_session().journal().live_cursor(), cursor);
        assert!(host.host.resident_refresh.gate.needs_check());
        assert!(host.recovery_pending());
        // The matching opaque completion remains held, not substituted by the
        // unrelated value. Matching retirement is covered through peer endpoints.
        assert_eq!(held.token, active.unwrap().0);
    }

    #[test]
    fn cached_refresh_wrong_owner_completion_cannot_retire_active_slot() {
        pollster::block_on(reject_unrelated_completion(0));
    }

    #[test]
    fn cached_refresh_wrong_session_completion_cannot_retire_active_slot() {
        pollster::block_on(reject_unrelated_completion(1));
    }

    #[test]
    fn cached_refresh_wrong_attempt_completion_cannot_retire_active_slot() {
        pollster::block_on(reject_unrelated_completion(2));
    }
}
