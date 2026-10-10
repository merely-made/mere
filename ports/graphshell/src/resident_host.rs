// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Retained checkpoint writes under concrete composition-owned storage.
//! A failed completion never means that the user's command should be repeated.
use crate::access::AccessContext;
use crate::handlers::HandlerRegistry;
use crate::mere_host::{MereHost, MereHostError, SelectedPersonaRef};
use muniment::{Backend, StoreError, WriteOp};
use pandect::addressable_graph::{GraphAddress, ResidencyPolicy, ResidencyRoots, ResidentGraph};
use pandect::live_graph::{LiveEditError, ObservedCommand};
use pandect::{Applied, Pending, PreparedResidentCheckpoint};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Explicitly opted-in native fault fixtures, never production ownership proof.
#[cfg(all(feature = "resident-test-support", not(target_arch = "wasm32")))]
#[path = "resident_host_test_support.rs"]
pub mod test_support;

#[cfg(all(feature = "resident-storage", not(target_arch = "wasm32")))]
#[path = "resident_directory_owner.rs"]
mod directory_owner;
#[cfg(all(feature = "resident-storage", not(target_arch = "wasm32")))]
pub use directory_owner::{CachedResidentSession, DirectoryHostCache};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OwnedPersistenceStatus {
    pub store: SessionStore,
    pub publication_pending: bool,
    pub error: Option<String>,
}

/// Read-only presentation observation actions on an admitted endpoint's session item.
pub const DEMAND_INTENT: &str = "mere.resident.demand";
pub const REFRESH_INTENT: &str = "mere.resident.refresh";
pub const OBSERVATION_SCHEMA: &str = "mere.resident.observation/v1";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationRequestV1 {
    pub schema: String,
    /// Typed strata are preserved; UUID decoding rejects malformed external IDs.
    pub address: Option<GraphAddress>,
}

pub fn observation_actions() -> Vec<chirograph::AdvertisedAction> {
    [
        (
            DEMAND_INTENT,
            "Load item",
            "Load the named stored Surface or Resource into this presentation.",
        ),
        (
            REFRESH_INTENT,
            "Refresh",
            "Check the current stored presentation without editing truth.",
        ),
    ]
    .into_iter()
    .map(
        |(intent, label, explanation)| chirograph::AdvertisedAction {
            intent: chirograph::IntentReference(intent.into()),
            label: label.into(),
            explanation: explanation.into(),
            payload_schema: OBSERVATION_SCHEMA.into(),
            input_form: None,
            effect: chirograph::IntentEffect::Curation,
        },
    )
    .collect()
}

/// Where the session's changes stand, as the page's `data-session-store` token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionStore {
    Stored,
    Pending,
    Writing,
    Failed,
}

impl SessionStore {
    /// The page's token. `unstored` is a settled state with changes nobody
    /// asked to store, such as a scene save (Scenograph editor plan, E2b
    /// findings), so the token never claims what the store does not hold.
    pub fn token(self, unstored: bool) -> &'static str {
        match self {
            Self::Stored if unstored => "unstored",
            Self::Stored => "stored",
            Self::Pending => "pending",
            Self::Writing => "writing",
            Self::Failed => "failed",
        }
    }

    /// Whether the frame pump should keep running until this settles.
    pub fn busy(self) -> bool {
        matches!(self, Self::Pending | Self::Writing)
    }
}

/// Ownership of one browser store continuation, independent of refresh changes.
pub struct SessionStoreAttempt {
    identity: Arc<()>,
    session: pandect::SessionId,
    sequence: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefreshToken {
    pub lifecycle: u64,
    pub session: pandect::SessionId,
    pub graph: uuid::Uuid,
    pub roots_revision: u64,
    pub policy_revision: u64,
    pub request: u64,
}
#[derive(Default)]
pub struct RefreshGate {
    in_flight: Option<(RefreshToken, u64)>,
    newest_notice: u64,
    consumed_notice: u64,
    invalidated: bool,
}
impl RefreshGate {
    pub fn notice(&mut self, sequence: u64) {
        self.newest_notice = self.newest_notice.max(sequence);
    }
    pub fn begin(&mut self, token: RefreshToken) -> bool {
        if self.in_flight.is_some() {
            return false;
        }
        self.in_flight = Some((token, self.newest_notice));
        self.invalidated = false;
        true
    }
    pub fn finish(&mut self, token: &RefreshToken, current: &RefreshToken) -> bool {
        if !self
            .in_flight
            .as_ref()
            .is_some_and(|(active, _)| active == token)
        {
            return false;
        }
        let (_, notice) = self.in_flight.take().expect("matched request");
        if token != current || self.invalidated {
            self.invalidated = true;
            return false;
        }
        self.consumed_notice = self.consumed_notice.max(notice);
        true
    }
    fn fail(&mut self, token: &RefreshToken) {
        if self
            .in_flight
            .as_ref()
            .is_some_and(|(active, _)| active == token)
        {
            self.in_flight = None;
            self.invalidated = true;
        }
    }
    pub fn needs_check(&self) -> bool {
        self.invalidated || self.newest_notice > self.consumed_notice
    }
}
#[derive(Default)]
pub(crate) struct HostRefresh {
    identity: Arc<()>,
    gate: RefreshGate,
    lifecycle: u64,
    policy_revision: u64,
    request: u64,
    notice: u64,
    suspended: bool,
    last_clock: Option<f64>,
    last_check: Option<f64>,
    pub(crate) publication_since: Option<f64>,
    validated_settings: Option<RefreshToken>,
    store_attempt: u64,
}
impl HostRefresh {
    pub(crate) fn invalidate(&mut self) {
        self.lifecycle = self.lifecycle.wrapping_add(1);
        self.gate.invalidated = true;
        self.validated_settings = None;
    }
    pub(crate) fn notice(&mut self) {
        self.notice = self
            .notice
            .checked_add(1)
            .expect("refresh notice exhausted");
        self.gate.notice(self.notice);
    }
}
/// One owned load; neither its backend nor its token can be substituted by callers.
pub struct HostRefreshRequest<B> {
    identity: Arc<()>,
    token: RefreshToken,
    preflight: bool,
    load: HostRefreshLoad<B>,
}
enum HostRefreshLoad<B> {
    Open(
        pandect::addressable_graph::AddressableGraphStore<B>,
        ResidencyRoots,
        ResidencyPolicy,
    ),
    Refresh(pandect::addressable_graph::ResidentRefreshRequest<B>),
}
enum HostRefreshView<B> {
    Open(ResidentGraph<B>),
    Refresh(pandect::addressable_graph::ResidentRefreshCandidate),
}
pub struct HostRefreshCompletion<B> {
    identity: Arc<()>,
    token: RefreshToken,
    preflight: bool,
    result: Result<HostRefreshView<B>, StoreError>,
}
impl<B: Backend> HostRefreshRequest<B> {
    pub async fn load(self) -> HostRefreshCompletion<B> {
        let result = match self.load {
            HostRefreshLoad::Open(store, roots, policy) => store
                .open_with_policy(roots, policy)
                .await
                .map(HostRefreshView::Open),
            HostRefreshLoad::Refresh(request) => request.load().await.map(HostRefreshView::Refresh),
        };
        HostRefreshCompletion {
            identity: self.identity,
            token: self.token,
            preflight: self.preflight,
            result,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ResidentRefreshMode {
    #[default]
    Manual,
    HostDriven {
        minimum_check_ms: u64,
    },
}
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResidentSettings {
    pub policy: ResidencyPolicy,
    pub projection: Option<String>,
    pub refresh: ResidentRefreshMode,
    pub publication_batch_ms: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PaneRoots {
    pub roster: BTreeSet<uuid::Uuid>,
    pub focus: Option<GraphAddress>,
}
#[derive(Default)]
pub struct HostRoots {
    panes: BTreeMap<pandect::ViewKey, PaneRoots>,
    pins: BTreeSet<GraphAddress>,
    revision: u64,
}
impl HostRoots {
    pub fn set_pane(&mut self, pane: pandect::ViewKey, roots: PaneRoots) {
        if self.panes.get(&pane) != Some(&roots) {
            self.panes.insert(pane, roots);
            self.revision = self.revision.wrapping_add(1);
        }
    }
    pub fn close_pane(&mut self, pane: &pandect::ViewKey) {
        if self.panes.remove(pane).is_some() {
            self.revision = self.revision.wrapping_add(1);
        }
    }
    pub fn set_pins(&mut self, pins: BTreeSet<GraphAddress>) {
        if self.pins != pins {
            self.pins = pins;
            self.revision = self.revision.wrapping_add(1);
        }
    }
    pub fn union(&self) -> ResidencyRoots {
        ResidencyRoots {
            projection_surfaces: self
                .panes
                .values()
                .flat_map(|p| p.roster.iter().copied())
                .collect(),
            transient: self.panes.values().filter_map(|p| p.focus).collect(),
            pinned: self.pins.clone(),
            focused: None,
        }
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
}

impl<B: Backend + Clone> MereHost<B> {
    /// Persist through the cached owner's retained checkpoint, or the existing
    /// Complete path when this host has no residency admission.
    pub async fn persist_owned(&mut self, saved_at_secs: u64) -> Result<(), MereHostError> {
        if !self.resident_admitted {
            return self.persist(saved_at_secs).await;
        }
        self.persist_resident(
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(saved_at_secs),
        )
        .await?;
        self.persist_projection_epoch().await
    }

    /// Native cached owners retain the exact staged batch on uncertain storage.
    /// Ordinary Complete hosts continue to use their original flush path.
    pub(crate) async fn persist_resident(
        &mut self,
        at: SystemTime,
    ) -> Result<(), ResidentHostError> {
        if self.resident_settings_need_validation() {
            let request = self
                .prepare_resident_settings_validation()?
                .ok_or(ResidentHostError::Busy)?;
            self.install_resident_refresh(request.load().await)?;
        }
        let io = self.prepare_resident_io(at)?;
        let write = io.write().await;
        let generation = self.install_resident_io(&io)?;
        if generation.is_none() {
            return Err(write.err().unwrap_or(ResidentHostError::Busy));
        }
        self.refresh_resident_reader().await
    }

    fn resident_key(&self, suffix: &str) -> String {
        format!(
            "graphs/{}/resident/v1/{suffix}",
            self.graph_session().manifest().root_graph_id.as_uuid()
        )
    }
    pub(crate) async fn load_resident_settings(mut self) -> Result<Self, MereHostError> {
        let slots = muniment::JsonSlots::new(self.store());
        let settings = slots
            .load::<ResidentSettings>(&self.resident_key("host-settings"))
            .await?
            .unwrap_or_default();
        self.validate_resident_settings(&settings)?;
        self.resident_settings = settings;
        Ok(self)
    }
    pub(crate) fn validate_resident_settings(
        &self,
        settings: &ResidentSettings,
    ) -> Result<(), MereHostError> {
        let invalid = |reason: &str| MereHostError::InvalidSnapshot(reason.to_owned());
        if matches!(
            settings.refresh,
            ResidentRefreshMode::HostDriven {
                minimum_check_ms: 0
            }
        ) {
            return Err(invalid(
                "host-driven resident refresh requires a positive minimum interval",
            ));
        }
        if matches!(settings.policy, ResidencyPolicy::Neighborhood { .. }) {
            let id = settings
                .projection
                .as_deref()
                .filter(|id| !id.is_empty())
                .ok_or_else(|| invalid("Neighborhood requires a selected authored projection"))?;
            let snapshot = self
                .saved_projection(id)
                .map_err(|e| invalid(&e.to_string()))?
                .ok_or_else(|| invalid("selected authored projection is missing"))?;
            if snapshot.definition.id != id {
                return Err(invalid("selected authored projection identity mismatch"));
            }
            let draft: crate::projection_editor::ProjectionDraft = serde_json::from_value(
                serde_json::to_value(&snapshot.definition).map_err(|e| invalid(&e.to_string()))?,
            )
            .map_err(|e| invalid(&e.to_string()))?;
            draft
                .validate()
                .map_err(|issues| invalid(&format!("invalid authored projection: {issues:?}")))?;
        }
        Ok(())
    }
    pub fn resident_settings(&self) -> &ResidentSettings {
        &self.resident_settings
    }
    /// Stage graph settings under the admitted writer's retained checkpoint batch.
    pub fn set_resident_settings(
        &mut self,
        settings: ResidentSettings,
    ) -> Result<(), ResidentHostError> {
        if !self.resident_admitted {
            return Err(ResidentHostError::OwnershipUnproven);
        }
        self.ensure_resident_idle()?;
        self.validate_resident_settings(&settings)?;
        if self.resident_settings != settings {
            self.resident_refresh.policy_revision =
                self.resident_refresh.policy_revision.wrapping_add(1);
            self.resident_refresh.invalidate();
        }
        self.resident_settings_dirty |= self.resident_settings != settings;
        self.resident_settings = settings;
        Ok(())
    }
    pub(crate) fn resident_settings_ops(&self) -> Result<Vec<WriteOp>, StoreError> {
        use muniment::Codec;
        Ok(vec![
            WriteOp::Put {
                key: self.resident_key("host-settings"),
                value: muniment::JsonCodec::encode(&self.resident_settings)?,
            },
            WriteOp::Put {
                key: self.resident_key("policy"),
                value: muniment::JsonCodec::encode(&self.resident_settings.policy)?,
            },
        ])
    }
    pub fn resident_roots(&self) -> &HostRoots {
        &self.resident_roots
    }
    pub fn set_resident_pane(&mut self, pane: pandect::ViewKey, roots: PaneRoots) {
        let revision = self.resident_roots.revision();
        self.resident_roots.set_pane(pane, roots);
        if revision != self.resident_roots.revision() {
            self.resident_refresh.notice();
        }
    }
    pub fn close_resident_pane(&mut self, pane: &pandect::ViewKey) {
        let revision = self.resident_roots.revision();
        self.resident_roots.close_pane(pane);
        if revision != self.resident_roots.revision() {
            self.resident_refresh.notice();
        }
    }
    /// These roots reflect explicit graph pinning, independently of per-view Keep.
    pub fn set_resident_pins(&mut self, pins: BTreeSet<GraphAddress>) {
        let revision = self.resident_roots.revision();
        self.resident_roots.set_pins(pins);
        if revision != self.resident_roots.revision() {
            self.resident_refresh.notice();
        }
    }
    /// Supply host context to the installed immutable reader, never to truth.
    pub fn set_resident_host_coverage(
        &mut self,
        note: mere::kernel::graph::CoverageNote,
    ) -> Result<(), ResidentHostError> {
        self.resident_reader
            .as_mut()
            .ok_or(ResidentHostError::Busy)?
            .set_host_coverage(note);
        self.sync_projection_context();
        Ok(())
    }
    pub fn resident_reader(&self) -> Option<&ResidentGraph<B>> {
        self.resident_reader.as_ref()
    }
    fn refresh_token(&self) -> RefreshToken {
        RefreshToken {
            lifecycle: self.resident_refresh.lifecycle,
            session: self.graph_session().id(),
            graph: *self.graph_session().manifest().root_graph_id.as_uuid(),
            roots_revision: self.resident_roots.revision(),
            policy_revision: self.resident_refresh.policy_revision,
            request: self.resident_refresh.request,
        }
    }
    pub fn notice_resident_refresh(&mut self) {
        self.resident_refresh.notice();
    }
    pub fn resident_needs_check(&self) -> bool {
        self.resident_refresh.gate.needs_check()
    }
    /// Detach and suspension invalidate outstanding loads without releasing their
    /// single-flight slot until completion. Resume checks immediately.
    pub fn suspend_resident_refresh(&mut self, suspended: bool) {
        if self.resident_refresh.suspended != suspended {
            self.resident_refresh.suspended = suspended;
            self.resident_refresh.invalidate();
            if !suspended {
                self.resident_refresh.notice();
            }
        }
    }
    pub fn detach_resident_refresh(&mut self) {
        self.resident_refresh.suspended = true;
        self.resident_refresh.invalidate();
    }
    pub fn attach_resident_refresh(&mut self) {
        self.resident_refresh.suspended = false;
        self.resident_refresh.invalidate();
        self.resident_refresh.notice();
    }
    /// Host monotonic elapsed milliseconds, never wall time. Invalid samples
    /// cannot move either the interval baseline or the pending notice backwards.
    pub fn tick_resident_refresh(&mut self, elapsed_ms: f64) {
        if self.resident_refresh.suspended
            || !elapsed_ms.is_finite()
            || elapsed_ms < 0.0
            || self
                .resident_refresh
                .last_clock
                .is_some_and(|old| elapsed_ms < old)
        {
            return;
        }
        self.resident_refresh.last_clock = Some(elapsed_ms);
        if let ResidentRefreshMode::HostDriven { minimum_check_ms } = self.resident_settings.refresh
        {
            if self
                .resident_refresh
                .last_check
                .is_none_or(|last| elapsed_ms - last >= minimum_check_ms as f64)
            {
                self.resident_refresh.notice();
                self.resident_refresh.last_check = Some(elapsed_ms);
            }
        }
    }
    /// Idle publication cadence is independent from reader polling. Explicit
    /// save/detach callers pass flush; persist() is already an explicit flush.
    pub fn resident_publication_due(&mut self, elapsed_ms: f64, flush: bool) -> bool {
        if !self.resident_admitted {
            return true;
        }
        if self.resident_refresh.suspended {
            return false;
        }
        if flush {
            return true;
        }
        if !elapsed_ms.is_finite()
            || elapsed_ms < 0.0
            || self
                .resident_refresh
                .last_clock
                .is_some_and(|old| elapsed_ms < old)
        {
            return false;
        }
        self.resident_refresh.last_clock = Some(elapsed_ms);
        let start = *self
            .resident_refresh
            .publication_since
            .get_or_insert(elapsed_ms);
        elapsed_ms - start >= self.resident_settings.publication_batch_ms as f64
    }
    pub fn resident_settings_pending(&self) -> bool {
        self.resident_settings_dirty
    }
    pub fn resident_settings_need_validation(&self) -> bool {
        self.resident_settings_dirty
            && self.retained_resident_work().is_none()
            && self.resident_refresh.validated_settings.as_ref() != Some(&self.refresh_token())
    }
    pub fn prepare_resident_refresh(
        &mut self,
    ) -> Result<Option<HostRefreshRequest<B>>, ResidentHostError> {
        self.prepare_refresh_request(false, None)
    }
    pub fn prepare_resident_settings_validation(
        &mut self,
    ) -> Result<Option<HostRefreshRequest<B>>, ResidentHostError> {
        self.resident_refresh.notice();
        self.prepare_refresh_request(true, None)
    }
    pub fn prepare_resident_demand(
        &mut self,
        address: GraphAddress,
    ) -> Result<Option<HostRefreshRequest<B>>, ResidentHostError> {
        self.resident_refresh.notice();
        self.prepare_refresh_request(false, Some(address))
    }
    fn prepare_refresh_request(
        &mut self,
        preflight: bool,
        demand: Option<GraphAddress>,
    ) -> Result<Option<HostRefreshRequest<B>>, ResidentHostError> {
        if !self.resident_admitted {
            return Err(ResidentHostError::OwnershipUnproven);
        }
        if self.resident_refresh.suspended
            || !self.resident_refresh.gate.needs_check()
            || self.resident_refresh.gate.in_flight.is_some()
        {
            return Ok(None);
        }
        // Dirty settings need validation before writing, and must not become
        // presentation policy until their retained settings batch is stored.
        if !preflight && self.resident_settings_dirty {
            return Ok(None);
        }
        self.resident_refresh.request = self.resident_refresh.request.wrapping_add(1);
        let token = self.refresh_token();
        if !self.resident_refresh.gate.begin(token.clone()) {
            return Ok(None);
        }
        self.resident_refresh.last_check = self.resident_refresh.last_clock;
        let roots = self.resident_roots.union();
        let policy = self.resident_settings.policy;
        let load = if let Some(reader) = &self.resident_reader {
            HostRefreshLoad::Refresh(match demand {
                Some(address) => reader.refresh_request(roots, policy).with_demand(address),
                None => reader.refresh_request(roots, policy),
            })
        } else {
            if demand.is_some() {
                self.resident_refresh.gate.fail(&token);
                return Err(ResidentHostError::Busy);
            }
            HostRefreshLoad::Open(
                pandect::addressable_graph::AddressableGraphStore::new(self.store(), token.graph),
                roots,
                policy,
            )
        };
        Ok(Some(HostRefreshRequest {
            identity: self.resident_refresh.identity.clone(),
            token,
            preflight,
            load,
        }))
    }
    /// Decline installation during owned publication recovery, retiring only
    /// the outstanding load that produced this completion. Newer notices stay
    /// pending; the cached owner's publication work is not acknowledged here.
    pub(crate) fn decline_resident_refresh(
        &mut self,
        completion: HostRefreshCompletion<B>,
    ) -> Result<pandect::addressable_graph::RefreshOutcome, ResidentHostError> {
        if !Arc::ptr_eq(&completion.identity, &self.resident_refresh.identity)
            || !self
                .resident_refresh
                .gate
                .in_flight
                .as_ref()
                .is_some_and(|(active, _)| active == &completion.token)
        {
            return Err(ResidentHostError::StaleCompletion);
        }
        self.resident_refresh.gate.fail(&completion.token);
        match completion.result {
            Err(error) => Err(error.into()),
            Ok(_) => Err(ResidentHostError::Busy),
        }
    }

    pub fn install_resident_refresh(
        &mut self,
        completion: HostRefreshCompletion<B>,
    ) -> Result<pandect::addressable_graph::RefreshOutcome, ResidentHostError> {
        use pandect::addressable_graph::RefreshOutcome;
        let current = self.refresh_token();
        if !Arc::ptr_eq(&completion.identity, &self.resident_refresh.identity) {
            return Err(ResidentHostError::StaleCompletion);
        }
        if completion.token != current || self.resident_refresh.suspended {
            self.resident_refresh.gate.fail(&completion.token);
            return Err(ResidentHostError::StaleCompletion);
        }
        let view = match completion.result {
            Ok(view) => view,
            Err(error) => {
                self.resident_refresh.gate.fail(&completion.token);
                return Err(error.into());
            },
        };
        if !self
            .resident_refresh
            .gate
            .finish(&completion.token, &current)
        {
            return Err(ResidentHostError::StaleCompletion);
        }
        if completion.preflight {
            self.resident_refresh.validated_settings = Some(current);
            return Ok(RefreshOutcome::Unchanged);
        }
        let result = match view {
            HostRefreshView::Open(reader) => {
                self.resident_reader = Some(reader);
                Ok(RefreshOutcome::TruthChanged)
            },
            HostRefreshView::Refresh(candidate) => self
                .resident_reader
                .as_mut()
                .ok_or(ResidentHostError::StaleCompletion)?
                .install_refresh(candidate)
                .map_err(Into::into),
        };
        if matches!(result, Ok(RefreshOutcome::TruthChanged)) {
            self.projection_revision = self.projection_revision.wrapping_add(1);
        }
        self.sync_projection_context();
        if result.is_err() {
            self.resident_refresh.gate.invalidated = true;
        }
        result
    }
    /// Capture ownership before dispatching a browser store task.
    pub fn begin_session_store_attempt(&mut self) -> SessionStoreAttempt {
        self.resident_refresh.store_attempt = self
            .resident_refresh
            .store_attempt
            .checked_add(1)
            .expect("session store attempt exhausted");
        SessionStoreAttempt {
            identity: self.resident_refresh.identity.clone(),
            session: self.graph_session().id(),
            sequence: self.resident_refresh.store_attempt,
        }
    }

    pub fn session_store_attempt_is_current(&self, attempt: &SessionStoreAttempt) -> bool {
        Arc::ptr_eq(&attempt.identity, &self.resident_refresh.identity)
            && attempt.session == self.graph_session().id()
            && attempt.sequence == self.resident_refresh.store_attempt
    }

    /// Continue the browser's settings preflight after its owned load ends.
    /// The returned flag permits synchronous write preparation only on success.
    pub fn continue_settings_store(
        &mut self,
        attempt: &SessionStoreAttempt,
        completion: HostRefreshCompletion<B>,
        status: &mut SessionStore,
        error: &mut String,
    ) -> bool {
        if !self.session_store_attempt_is_current(attempt) {
            // Release only this completion's flight. A newer flight, notice,
            // store status and diagnostic belong to the current operation.
            if Arc::ptr_eq(&completion.identity, &self.resident_refresh.identity) {
                self.resident_refresh.gate.fail(&completion.token);
            }
            return false;
        }
        match self.install_resident_refresh(completion) {
            Ok(_) => true,
            Err(ResidentHostError::StaleCompletion) => {
                // Lifecycle/roots/settings changes cancel validation, but the
                // settings are still dirty. Pending lets the pump retry after
                // resume or once the current inputs can be validated.
                *status = SessionStore::Pending;
                false
            },
            Err(failure) => {
                *status = SessionStore::Failed;
                *error = failure.to_string();
                false
            },
        }
    }

    /// Convenience for exclusively borrowed native owners. Browser callers use
    /// prepare/load/install so their RefCell borrow ends before I/O starts.
    pub async fn refresh_resident_reader(&mut self) -> Result<(), ResidentHostError> {
        self.notice_resident_refresh();
        let request = self
            .prepare_resident_refresh()?
            .ok_or(ResidentHostError::Busy)?;
        self.install_resident_refresh(request.load().await)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResidentPersistence {
    Rejected,
    /// The ordinary unrecorded Complete path has no resident generation.
    Complete,
    AppliedButUnstored {
        first: u64,
        end: u64,
        revision: u64,
    },
    Durable {
        generation: String,
    },
    PendingPublication {
        generation: String,
        reason: String,
    },
}
#[derive(Debug)]
pub enum ResidentHostError {
    Live(LiveEditError),
    Host(MereHostError),
    Busy,
    OwnershipUnproven,
    UnsupportedStorage,
    StaleCompletion,
    MixedCommit,
}
impl std::fmt::Display for ResidentHostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Live(e) => write!(f, "{e}"),
            Self::Host(e) => write!(f, "{e}"),
            Self::Busy => write!(f, "resident write is pending"),
            Self::OwnershipUnproven => write!(f, "resident storage ownership is unproven"),
            Self::UnsupportedStorage => write!(f, "unsupported resident storage"),
            Self::StaleCompletion => write!(f, "stale resident completion"),
            Self::MixedCommit => write!(f, "mixed resident commit; reopen storage before writing"),
        }
    }
}
impl std::error::Error for ResidentHostError {}
impl From<MereHostError> for ResidentHostError {
    fn from(e: MereHostError) -> Self {
        Self::Host(e)
    }
}
impl From<StoreError> for ResidentHostError {
    fn from(e: StoreError) -> Self {
        Self::Host(e.into())
    }
}

#[derive(Default)]
struct Progress {
    preimages: Option<BTreeMap<String, Option<Vec<u8>>>>,
    uncertain: bool,
    complete: bool,
    mixed: bool,
}
struct Part {
    ops: Vec<WriteOp>,
    final_values: BTreeMap<String, Option<Vec<u8>>>,
    progress: Mutex<Progress>,
}
impl Part {
    fn new(ops: Vec<WriteOp>) -> Self {
        let final_values = ops
            .iter()
            .map(|op| match op {
                WriteOp::Put { key, value } => (key.clone(), Some(value.clone())),
                WriteOp::Delete { key } => (key.clone(), None),
            })
            .collect();
        Self {
            ops,
            final_values,
            progress: Mutex::new(Progress::default()),
        }
    }
    async fn write<B: Backend>(&self, backend: &B) -> Result<(), ResidentHostError> {
        let (complete, mixed, uncertain, preimages) = {
            let p = self.progress.lock().unwrap();
            (p.complete, p.mixed, p.uncertain, p.preimages.clone())
        };
        if mixed {
            return Err(ResidentHostError::MixedCommit);
        }
        if complete {
            return Ok(());
        }
        if uncertain {
            let before = preimages.as_ref().expect("preimages precede apply");
            let mut all_final = true;
            let mut all_before = true;
            for (key, final_value) in &self.final_values {
                let now = backend.get(key).await?;
                all_final &= now == *final_value;
                all_before &= now == before[key];
            }
            if all_final {
                self.progress.lock().unwrap().complete = true;
                return Ok(());
            }
            // Keys unchanged by the batch satisfy both comparisons. A mixture
            // of genuinely changed old/new keys is never safe to overwrite.
            if !all_before {
                self.progress.lock().unwrap().mixed = true;
                return Err(ResidentHostError::MixedCommit);
            }
        }
        if preimages.is_none() {
            let mut before = BTreeMap::new();
            for key in self.final_values.keys() {
                before.insert(key.clone(), backend.get(key).await?);
            }
            self.progress.lock().unwrap().preimages = Some(before);
        }
        // Retained before yielding, including when this future is cancelled.
        self.progress.lock().unwrap().uncertain = true;
        backend.apply(&self.ops).await?;
        self.progress.lock().unwrap().complete = true;
        Ok(())
    }
}
struct WriteState {
    parts: Vec<Part>,
    running: Mutex<bool>,
}
/// Detached I/O work. It shares the exact retained bytes and preimages with
/// its host and never owns a mutable graph or command.
#[derive(Clone)]
pub(crate) struct ResidentWrite {
    state: Arc<WriteState>,
}
struct Running<'a>(&'a Mutex<bool>);
impl Drop for Running<'_> {
    fn drop(&mut self) {
        *self.0.lock().unwrap() = false;
    }
}
impl ResidentWrite {
    pub(crate) async fn write<B: Backend>(&self, backend: &B) -> Result<(), ResidentHostError> {
        {
            let mut running = self.state.running.lock().unwrap();
            if *running {
                return Err(ResidentHostError::Busy);
            }
            *running = true;
        }
        let _running = Running(&self.state.running);
        for part in &self.state.parts {
            part.write(backend).await?;
        }
        Ok(())
    }
}
/// Detached I/O bound to the admitted host's own backend. Callers cannot
/// substitute another store and then acknowledge the original host's batch.
pub struct ResidentIo<B> {
    work: ResidentWrite,
    backend: B,
}
impl<B: Backend + Clone> ResidentIo<B> {
    pub(crate) fn prepare(
        host: &mut MereHost<B>,
        at: SystemTime,
    ) -> Result<Self, ResidentHostError> {
        let work = host.prepare_resident_store(at)?;
        Ok(Self {
            work,
            backend: host.store(),
        })
    }
    pub async fn write(&self) -> Result<(), ResidentHostError> {
        self.work.write(&self.backend).await
    }
    pub(crate) fn work(&self) -> &ResidentWrite {
        &self.work
    }
}
pub(crate) struct ResidentStaged {
    pub(crate) current: Option<PreparedResidentCheckpoint>,
    pub(crate) retired: Vec<Option<Pending>>,
    work: ResidentWrite,
}
impl ResidentStaged {
    pub(crate) fn new(
        current: PreparedResidentCheckpoint,
        retired: Vec<Pending>,
        settings: Vec<WriteOp>,
    ) -> Self {
        let mut current_ops = settings;
        current_ops.extend_from_slice(current.ops());
        let parts = retired
            .iter()
            .map(|p| Part::new(p.ops().to_vec()))
            .chain([Part::new(current_ops)])
            .collect();
        Self {
            current: Some(current),
            retired: retired.into_iter().map(Some).collect(),
            work: ResidentWrite {
                state: Arc::new(WriteState {
                    parts,
                    running: Mutex::new(false),
                }),
            },
        }
    }
    pub(crate) fn work(&self) -> ResidentWrite {
        self.work.clone()
    }
    pub(crate) fn matches(&self, work: &ResidentWrite) -> bool {
        Arc::ptr_eq(&self.work.state, &work.state)
    }
    pub(crate) fn complete(&self, index: usize) -> bool {
        self.work.state.parts[index]
            .progress
            .lock()
            .unwrap()
            .complete
    }
    pub(crate) fn running(&self) -> bool {
        *self.work.state.running.lock().unwrap()
    }
}

/// An owner keeps the complete writer private, with a last coherent resident
/// reader. Only concrete factories below can grant write admission.
pub struct ResidentHost<B> {
    host: Option<MereHost<B>>,
    applied: Option<Applied>,
    publication: Option<String>,
    #[cfg(all(feature = "resident-storage", not(target_arch = "wasm32")))]
    directory: Option<std::path::PathBuf>,
}
impl<B: Backend + Clone> ResidentHost<B> {
    fn owned(mut host: MereHost<B>) -> Self {
        host.admit_resident();
        Self {
            host: Some(host),
            applied: None,
            publication: None,
            #[cfg(all(feature = "resident-storage", not(target_arch = "wasm32")))]
            directory: None,
        }
    }
    /// Generic callers cannot certify backend semantics or writer ownership.
    pub fn try_from_host(_host: MereHost<B>) -> Result<Self, ResidentHostError> {
        Err(ResidentHostError::OwnershipUnproven)
    }
    pub fn reader(&self) -> Option<&ResidentGraph<B>> {
        self.host.as_ref().and_then(|host| host.resident_reader())
    }
    pub fn presentation_graph(&self) -> &mere::kernel::graph::Graph {
        self.host
            .as_ref()
            .expect("open resident")
            .presentation_graph()
    }
    pub fn settings(&self) -> &ResidentSettings {
        self.host
            .as_ref()
            .expect("open resident")
            .resident_settings()
    }
    pub fn set_settings(&mut self, settings: ResidentSettings) -> Result<(), ResidentHostError> {
        if self.is_busy() {
            return Err(ResidentHostError::Busy);
        }
        self.host
            .as_mut()
            .expect("open resident")
            .set_resident_settings(settings)
    }
    pub async fn set_pane(
        &mut self,
        pane: pandect::ViewKey,
        roots: PaneRoots,
    ) -> Result<(), ResidentHostError> {
        let host = self.host.as_mut().ok_or(ResidentHostError::Busy)?;
        host.set_resident_pane(pane, roots);
        host.refresh_resident_reader().await?;
        Ok(())
    }
    pub async fn close_pane(&mut self, pane: &pandect::ViewKey) -> Result<(), ResidentHostError> {
        let host = self.host.as_mut().ok_or(ResidentHostError::Busy)?;
        host.close_resident_pane(pane);
        host.refresh_resident_reader().await?;
        Ok(())
    }
    pub fn session(&self) -> &pandect::GraphSession<B> {
        self.host.as_ref().expect("open resident").graph_session()
    }
    pub fn is_busy(&self) -> bool {
        self.host.as_ref().is_none_or(|h| h.resident_busy()) || self.publication.is_some()
    }
    pub fn edit(
        &mut self,
        via: &str,
        commands: Vec<ObservedCommand>,
        at: SystemTime,
    ) -> Result<ResidentPersistence, ResidentHostError> {
        if self.is_busy() {
            return Err(ResidentHostError::Busy);
        }
        let host = self.host.as_mut().ok_or(ResidentHostError::Busy)?;
        let applied = host.edit_resident(via, commands)?;
        self.applied = Some(applied);
        // Preparation failure retains the already-applied identity and Busy.
        let _ = host.prepare_resident_store(at);
        Ok(self.unstored())
    }
    pub fn undo(
        &mut self,
        via: &str,
        at: SystemTime,
    ) -> Result<ResidentPersistence, ResidentHostError> {
        if self.is_busy() {
            return Err(ResidentHostError::Busy);
        }
        let host = self.host.as_mut().ok_or(ResidentHostError::Busy)?;
        let Some(reverted) = host.undo_now(via)? else {
            return Ok(ResidentPersistence::Rejected);
        };
        self.applied = Some(reverted.applied);
        host.hold_resident();
        let _ = host.prepare_resident_store(at);
        Ok(self.unstored())
    }
    pub fn redo(
        &mut self,
        via: &str,
        at: SystemTime,
    ) -> Result<ResidentPersistence, ResidentHostError> {
        if self.is_busy() {
            return Err(ResidentHostError::Busy);
        }
        let host = self.host.as_mut().ok_or(ResidentHostError::Busy)?;
        let Some(reverted) = host.redo_now(via)? else {
            return Ok(ResidentPersistence::Rejected);
        };
        self.applied = Some(reverted.applied);
        host.hold_resident();
        let _ = host.prepare_resident_store(at);
        Ok(self.unstored())
    }
    fn unstored(&self) -> ResidentPersistence {
        match &self.applied {
            Some(a) => ResidentPersistence::AppliedButUnstored {
                first: a.first.0,
                end: a.end.0,
                revision: a.revision,
            },
            None => ResidentPersistence::Rejected,
        }
    }
    pub async fn persist(
        &mut self,
        at: SystemTime,
    ) -> Result<ResidentPersistence, ResidentHostError> {
        if self.publication.is_some() {
            return self.refresh().await;
        }
        let host = self.host.as_mut().ok_or(ResidentHostError::Busy)?;
        if host.resident_settings_need_validation() && host.retained_resident_work().is_none() {
            let request = host
                .prepare_resident_settings_validation()?
                .ok_or(ResidentHostError::Busy)?;
            host.install_resident_refresh(request.load().await)?;
        }
        let work = match host.prepare_resident_store(at) {
            Ok(w) => w,
            Err(_) => return Ok(self.unstored()),
        };
        let result = work.write(&host.store()).await;
        let generation = host.install_resident_store(&work)?;
        if let Some(generation) = generation {
            self.applied = None;
            self.publication = Some(generation);
            return self.refresh().await;
        }
        if matches!(result, Err(ResidentHostError::MixedCommit)) {
            return Err(ResidentHostError::MixedCommit);
        }
        Ok(self.unstored())
    }
    pub async fn refresh(&mut self) -> Result<ResidentPersistence, ResidentHostError> {
        let generation = self.publication.clone().ok_or(ResidentHostError::Busy)?;
        let host = self.host.as_mut().ok_or(ResidentHostError::Busy)?;
        let result = host.refresh_resident_reader().await;
        match result {
            Ok(()) => {
                self.publication = None;
                Ok(ResidentPersistence::Durable { generation })
            },
            Err(error) => Ok(ResidentPersistence::PendingPublication {
                generation,
                reason: error.to_string(),
            }),
        }
    }
}

/// An existing cloneable handle cannot prove unique writer ownership.
/// Use the concrete DirectoryHostCache, which opens and contains its backend.
#[cfg(all(feature = "resident-storage", not(target_arch = "wasm32")))]
pub async fn open_directory_cached_checked(
    _backend: muniment::DirectoryBackend,
    _id: pandect::SessionId,
    _persona: SelectedPersonaRef,
    _handlers: HandlerRegistry,
    _access: AccessContext,
    _validator: Option<pandect::GraphValidator>,
) -> Result<MereHost<muniment::DirectoryBackend>, MereHostError> {
    Err(ResidentHostError::OwnershipUnproven.into())
}

#[cfg(all(feature = "resident-storage", not(target_arch = "wasm32")))]
impl ResidentHost<muniment::DirectoryBackend> {
    pub async fn open_directory(
        path: impl AsRef<std::path::Path>,
        id: pandect::SessionId,
        persona: SelectedPersonaRef,
        handlers: HandlerRegistry,
        access: AccessContext,
    ) -> Result<Self, ResidentHostError> {
        let path = path.as_ref().to_owned();
        let backend = muniment::DirectoryBackend::open(&path)?;
        let host = MereHost::open_session(backend, id, persona, handlers, access).await?;
        let mut owner = Self::owned(host);
        owner.directory = Some(path);
        Ok(owner)
    }
    /// Discard every owner clone before Directory redo recovery. Reopening
    /// reads authoritative history and never resubmits the previous command.
    pub async fn reopen(
        mut self,
        persona: SelectedPersonaRef,
        handlers: HandlerRegistry,
        access: AccessContext,
    ) -> Result<Self, ResidentHostError> {
        let id = self.session().id();
        let path = self
            .directory
            .take()
            .ok_or(ResidentHostError::OwnershipUnproven)?;
        let work = self.host.as_ref().and_then(|h| h.retained_resident_work());
        drop(self.host.take());
        // Opening finishes a durable redo batch before any comparison. The
        // retained worker owns bytes/preimages only, so it cannot hold the lock.
        if let Some(work) = work {
            let recovered = muniment::DirectoryBackend::open(&path)?;
            work.write(&recovered).await?;
            drop(recovered);
        }
        Self::open_directory(path, id, persona, handlers, access).await
    }
}
#[cfg(all(feature = "resident-storage", not(target_arch = "wasm32")))]
impl ResidentHost<muniment::RedbBackend> {
    pub async fn open_redb(
        path: impl AsRef<std::path::Path>,
        id: pandect::SessionId,
        persona: SelectedPersonaRef,
        handlers: HandlerRegistry,
        access: AccessContext,
    ) -> Result<Self, ResidentHostError> {
        let backend = muniment::RedbBackend::open(path)?;
        Ok(Self::owned(
            MereHost::open_session(backend, id, persona, handlers, access).await?,
        ))
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use muniment::MemoryBackend;
    use pandect::ViewKey;
    use pandect::addressable_graph::GraphAddress;
    use pandect::live_graph::{LiveCheck, LiveCommand};
    use std::collections::BTreeSet;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use uuid::Uuid;

    #[test]
    fn a_new_notice_survives_completion_and_switch_rejects_install() {
        let session = pandect::MereSessions::new(muniment::MemoryBackend::new())
            .begin_recorded(mere::kernel::graph::Author::user(), None)
            .id();
        let old = RefreshToken {
            lifecycle: 1,
            session,
            graph: uuid::Uuid::from_u128(5000),
            roots_revision: 1,
            policy_revision: 1,
            request: 1,
        };
        let mut gate = RefreshGate::default();
        gate.notice(1);
        assert!(gate.begin(old.clone()));
        assert!(!gate.begin(old.clone()));
        gate.notice(2);
        assert!(gate.finish(&old, &old));
        assert!(gate.needs_check());
        let mut switched = old.clone();
        switched.lifecycle += 1;
        switched.request += 1;
        assert!(gate.begin(old.clone()));
        assert!(!gate.finish(&old, &switched));
        assert!(gate.needs_check());
    }

    #[test]
    fn closing_one_pane_releases_only_its_transient_roots() {
        let first = ViewKey::new("graphshell", "first").unwrap();
        let second = ViewKey::new("graphshell", "second").unwrap();
        let a = Uuid::from_u128(4000);
        let b = Uuid::from_u128(4001);
        let focus = GraphAddress::Resource(Uuid::from_u128(4002));
        let pin = GraphAddress::Surface(Uuid::from_u128(4003));
        let mut roots = HostRoots::default();
        roots.set_pins(BTreeSet::from([pin]));
        roots.set_pane(
            first.clone(),
            PaneRoots {
                roster: BTreeSet::from([a]),
                focus: Some(focus),
            },
        );
        roots.set_pane(
            second,
            PaneRoots {
                roster: BTreeSet::from([a, b]),
                focus: Some(focus),
            },
        );
        assert_eq!(roots.union().projection_surfaces, BTreeSet::from([a, b]));
        assert_eq!(roots.union().transient, BTreeSet::from([focus]));
        assert_eq!(roots.union().pinned, BTreeSet::from([pin]));
        roots.close_pane(&first);
        assert_eq!(roots.union().projection_surfaces, BTreeSet::from([a, b]));
        assert_eq!(roots.union().transient, BTreeSet::from([focus]));
        assert_eq!(roots.union().pinned, BTreeSet::from([pin]));
    }

    #[derive(Clone, Default)]
    struct Fault {
        inner: MemoryBackend,
        mode: Arc<AtomicUsize>,
        calls: Arc<Mutex<Vec<Vec<WriteOp>>>>,
    }
    #[async_trait::async_trait]
    impl Backend for Fault {
        async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
            std::future::poll_fn(|_| {
                if self.mode.load(Ordering::SeqCst) == 7 {
                    std::task::Poll::Pending
                } else {
                    std::task::Poll::Ready(())
                }
            })
            .await;
            if self.mode.load(Ordering::SeqCst) == 4 {
                return Err(StoreError::Backend("read fault".into()));
            }
            self.inner.get(key).await
        }
        async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
            self.inner.put(key, bytes).await
        }
        async fn delete(&self, key: &str) -> Result<(), StoreError> {
            self.inner.delete(key).await
        }
        async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
            self.inner.list(prefix).await
        }
        async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> {
            self.inner.scan(start, end).await
        }
        async fn apply(&self, ops: &[WriteOp]) -> Result<(), StoreError> {
            self.calls.lock().unwrap().push(ops.to_vec());
            let mode = self.mode.load(Ordering::SeqCst);
            if mode == 1 {
                return Err(StoreError::Backend("before apply".into()));
            }
            if mode == 3 {
                for op in ops {
                    let (key, value) = match op {
                        WriteOp::Put { key, value } => (key, Some(value.clone())),
                        WriteOp::Delete { key } => (key, None),
                    };
                    if self.inner.get(key).await? != value {
                        self.inner.apply(std::slice::from_ref(op)).await?;
                        break;
                    }
                }
                return Err(StoreError::Backend("mixed commit".into()));
            }
            self.inner.apply(ops).await?;
            if mode == 6 {
                std::future::pending::<()>().await;
            }
            if mode == 2 {
                return Err(StoreError::Backend("lost completion".into()));
            }
            if mode == 5 {
                self.mode.store(4, Ordering::SeqCst);
            }
            Ok(())
        }
    }
    fn persona() -> SelectedPersonaRef {
        SelectedPersonaRef {
            persona: "test-author".into(),
            profile: "test".into(),
        }
    }
    fn access() -> AccessContext {
        AccessContext {
            persona: "test-author".into(),
            device: "test".into(),
            at_ms: 0,
        }
    }
    fn command(n: u128) -> Vec<ObservedCommand> {
        let id = Uuid::from_u128(n);
        vec![ObservedCommand {
            command: LiveCommand::CreateAddress {
                id,
                address: format!("https://resident.test/{n}"),
                title: format!("node {n}"),
            },
            checks: vec![LiveCheck::Absent(GraphAddress::Surface(id))],
        }]
    }
    async fn owner() -> (ResidentHost<Fault>, Fault) {
        let backend = Fault::default();
        let mut host = MereHost::empty(
            backend.clone(),
            persona(),
            HandlerRegistry::default(),
            access(),
        );
        host.begin_profiled_session(
            mere::kernel::graph::Graph::new(),
            Some(pandect::graph_placement::PlacementProfile::RecordedStrataV1),
        )
        .unwrap();
        let mut owner = ResidentHost::owned(host);
        assert!(matches!(
            owner.persist(SystemTime::UNIX_EPOCH).await.unwrap(),
            ResidentPersistence::Durable { .. }
        ));
        backend.calls.lock().unwrap().clear();
        (owner, backend)
    }
    fn authored_projection() -> crate::projection_compile::ProjectionSnapshot {
        use crate::projection_editor::*;
        let mut draft = ProjectionDraft::new();
        draft.id = "resident-test".into();
        draft.label = "Resident test".into();
        draft.source = SourceBinding {
            authority: "local".into(),
            domain: "notes".into(),
            resource: "graph:notes".into(),
        };
        draft.reading = Reading {
            kind: "nodes".into(),
            key: "id".into(),
            value: None,
        };
        draft.encoding.x = Channel::Field("x".into());
        draft.encoding.y = Channel::Field("y".into());
        draft.appearance.title = "Resident test".into();
        draft.appearance.realization = "canvas".into();
        draft.provenance.author = "test-author".into();
        draft.provenance.source_revision = Some("1".into());
        crate::projection_compile::ProjectionSnapshot {
            definition: draft.to_definition().unwrap(),
            selected_occurrence: None,
        }
    }

    #[cfg(feature = "resident-storage")]
    #[test]
    fn unproven_directory_handle_and_duplicate_admission_refuse_before_write() {
        pollster::block_on(async {
            let root = tempfile::tempdir().unwrap();
            let backend = muniment::DirectoryBackend::open(root.path()).unwrap();
            let sessions = pandect::MereSessions::new(backend.clone());
            let mut recorded = sessions.begin_recorded(mere::kernel::graph::Author::user(), None);
            recorded.flush(SystemTime::UNIX_EPOCH).await.unwrap();
            let id = recorded.id();
            drop(recorded);
            let before = directory_bytes(&backend).await;
            let first = open_directory_cached_checked(
                backend.clone(),
                id,
                persona(),
                HandlerRegistry::default(),
                access(),
                None,
            )
            .await;
            let duplicate = open_directory_cached_checked(
                backend.clone(),
                id,
                persona(),
                HandlerRegistry::default(),
                access(),
                None,
            )
            .await;
            for result in [first, duplicate] {
                assert!(
                    matches!(result, Err(MereHostError::Resident(error))
                    if matches!(*error, ResidentHostError::OwnershipUnproven)),
                    "a supplied existing handle cannot prove sole writer ownership"
                );
            }
            assert_eq!(
                directory_bytes(&backend).await,
                before,
                "unproven and duplicate admission must not advance epoch or write storage"
            );
        });
    }

    #[cfg(feature = "resident-storage")]
    #[test]
    fn ordinary_live_directory_writer_blocks_admission_before_write() {
        pollster::block_on(async {
            let root = tempfile::tempdir().unwrap();
            let backend = muniment::DirectoryBackend::open(root.path()).unwrap();
            let sessions = pandect::MereSessions::new(backend.clone());
            let mut recorded = sessions.begin_recorded(mere::kernel::graph::Author::user(), None);
            recorded.flush(SystemTime::UNIX_EPOCH).await.unwrap();
            let id = recorded.id();
            drop(recorded);
            let ordinary = MereHost::open_session_checked(
                backend.clone(),
                id,
                persona(),
                HandlerRegistry::default(),
                access(),
                None,
            )
            .await
            .unwrap();
            let before = directory_bytes(&backend).await;
            let result = open_directory_cached_checked(
                backend.clone(),
                id,
                persona(),
                HandlerRegistry::default(),
                access(),
                None,
            )
            .await;
            assert!(
                matches!(result, Err(MereHostError::Resident(error))
                if matches!(*error, ResidentHostError::OwnershipUnproven)),
                "an ordinary live writer's backend clone is unproven"
            );
            assert_eq!(directory_bytes(&backend).await, before);
            assert_eq!(ordinary.graph_session().id(), id);
            assert!(
                muniment::DirectoryBackend::open(root.path()).is_err(),
                "an independently opened owner must honor the existing lifetime lease"
            );
            assert!(
                DirectoryHostCache::open(root.path()).is_err(),
                "the concrete cache cannot bypass an ordinary live writer lease"
            );
        });
    }

    #[cfg(feature = "resident-storage")]
    async fn directory_bytes(backend: &muniment::DirectoryBackend) -> BTreeMap<String, Vec<u8>> {
        let mut bytes = BTreeMap::new();
        for key in backend.list("").await.unwrap() {
            bytes.insert(key.clone(), backend.get(&key).await.unwrap().unwrap());
        }
        bytes
    }

    #[cfg(feature = "resident-storage")]
    #[test]
    fn cached_directory_factory_checks_before_epoch_and_preserves_placement() {
        pollster::block_on(async {
            let root = tempfile::tempdir().unwrap();
            let (id, legacy_id) = {
                let backend = muniment::DirectoryBackend::open(root.path()).unwrap();
                let sessions = pandect::MereSessions::new(backend);
                let mut recorded =
                    sessions.begin_recorded(mere::kernel::graph::Author::user(), None);
                recorded.flush(SystemTime::UNIX_EPOCH).await.unwrap();
                let mut legacy = sessions.begin(
                    mere::kernel::graph::Author::user(),
                    Some(mere::kernel::graph::Graph::new()),
                );
                legacy.flush(SystemTime::UNIX_EPOCH).await.unwrap();
                (recorded.id(), legacy.id())
            };
            let cache = DirectoryHostCache::open(root.path()).unwrap();
            let before = cache.all_stored_bytes().await.unwrap();
            assert!(!before.contains_key(crate::mere_host::EPOCH_SLOT));
            let validator: pandect::GraphValidator =
                Arc::new(|_| Err("fixture validation refusal".into()));
            let rejected = cache
                .host_checked(
                    id,
                    persona(),
                    HandlerRegistry::default(),
                    access(),
                    Some(validator),
                    chirograph::ProjectionSession("v1:test".into()),
                )
                .await;
            assert!(matches!(
                rejected,
                Err(MereHostError::Session(pandect::SessionError::Validation(_)))
            ));
            assert_eq!(cache.all_stored_bytes().await.unwrap(), before);
            assert_eq!(cache.cached_sessions().await, 0);
            let host = cache
                .host_checked(
                    id,
                    persona(),
                    HandlerRegistry::default(),
                    access(),
                    None,
                    chirograph::ProjectionSession("v1:test".into()),
                )
                .await
                .unwrap();
            let mut host = host.lock().await;
            assert_eq!(host.graph_session().id(), id);
            assert!(host.resident_reader().is_some());
            assert!(matches!(
                host.persist_owned(0).await.unwrap(),
                ResidentPersistence::Durable { .. }
            ));
            drop(host);
            // Supplied legacy baselines remain on the ordinary Complete path.
            let host = cache
                .host_checked(
                    legacy_id,
                    persona(),
                    HandlerRegistry::default(),
                    access(),
                    None,
                    chirograph::ProjectionSession("v1:test".into()),
                )
                .await
                .unwrap();
            let mut host = host.lock().await;
            assert!(host.graph_session().placement().is_none());
            assert!(matches!(
                host.set_resident_settings(Default::default()),
                Err(ResidentHostError::OwnershipUnproven)
            ));
            assert!(matches!(
                host.prepare_resident_demand(GraphAddress::Surface(Uuid::from_u128(6199))),
                Err(ResidentHostError::OwnershipUnproven)
            ));
            assert!(matches!(
                host.persist_owned(0).await.unwrap(),
                ResidentPersistence::Complete
            ));
            assert!(host.resident_reader().is_none());
        });
    }

    #[cfg(feature = "resident-storage")]
    #[test]
    fn concrete_directory_cache_reuses_one_writer_and_refuses_second_owner() {
        pollster::block_on(async {
            let root = tempfile::tempdir().unwrap();
            let cache = DirectoryHostCache::open(root.path()).unwrap();
            let id = cache
                .mint(mere::kernel::graph::Author::user(), None)
                .await
                .unwrap()
                .session_id;
            let first = cache
                .host_checked(
                    id,
                    persona(),
                    HandlerRegistry::default(),
                    access(),
                    None,
                    chirograph::ProjectionSession("v1:test".into()),
                )
                .await
                .unwrap();
            let before = cache.all_stored_bytes().await.unwrap();
            let duplicate = cache
                .host_checked(
                    id,
                    persona(),
                    HandlerRegistry::default(),
                    access(),
                    None,
                    chirograph::ProjectionSession("v1:test".into()),
                )
                .await
                .unwrap();
            assert!(Arc::ptr_eq(&first, &duplicate));
            assert_eq!(cache.cached_sessions().await, 1);
            assert_eq!(
                cache.all_stored_bytes().await.unwrap(),
                before,
                "duplicate selection does not reopen or advance the epoch"
            );
            assert!(
                DirectoryHostCache::open(root.path()).is_err(),
                "fresh second composition must respect the actual Directory lease"
            );
            assert!(
                muniment::DirectoryBackend::open(root.path()).is_err(),
                "an ordinary writer cannot reopen the admitted backend"
            );
            assert_eq!(cache.all_stored_bytes().await.unwrap(), before);
        });
    }

    #[cfg(feature = "resident-storage")]
    #[test]
    fn concrete_directory_cache_does_not_replace_malformed_existing_head() {
        pollster::block_on(async {
            let root = tempfile::tempdir().unwrap();
            let (id, head) = {
                let backend = muniment::DirectoryBackend::open(root.path()).unwrap();
                let sessions = pandect::MereSessions::new(backend.clone());
                let mut recorded =
                    sessions.begin_recorded(mere::kernel::graph::Author::user(), None);
                recorded.flush(SystemTime::UNIX_EPOCH).await.unwrap();
                let head = format!(
                    "graphs/{}/resident/v1/head",
                    recorded.manifest().root_graph_id.as_uuid()
                );
                backend.put(&head, b"malformed-head").await.unwrap();
                (recorded.id(), head)
            };
            let cache = DirectoryHostCache::open(root.path()).unwrap();
            let result = cache
                .host_checked(
                    id,
                    persona(),
                    HandlerRegistry::default(),
                    access(),
                    None,
                    chirograph::ProjectionSession("v1:test".into()),
                )
                .await;
            assert!(
                result.is_err(),
                "an existing malformed head is not a missing first publication"
            );
            assert_eq!(
                cache.stored_bytes(&head).await.unwrap(),
                Some(b"malformed-head".to_vec())
            );
            assert_eq!(cache.cached_sessions().await, 0);
        });
    }

    #[test]
    fn cached_host_persistence_retains_uncertain_batch_and_never_replays_command() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            let host = owner.host.as_mut().unwrap();
            host.apply_edits(vec![
                mere::kernel::graph::CapturedDelta::ReplayAddNodeWithIdIfMissing {
                    id: Uuid::from_u128(6198).to_string(),
                    url: "https://6198.test/".into(),
                    position: [0.0, 0.0],
                },
            ])
            .unwrap();
            let cursor = host.graph_session().journal().live_cursor();
            let revision = host.graph_session().revision();
            let changes = host.graph_session().changes().len();
            assert!(matches!(
                host.undo("app").await,
                Err(MereHostError::ResidentBusy)
            ));
            assert!(matches!(
                host.redo("app").await,
                Err(MereHostError::ResidentBusy)
            ));
            assert!(matches!(
                host.set_view("app", "view", Default::default()).await,
                Err(MereHostError::ResidentBusy)
            ));
            assert!(matches!(
                host.archive_receipt("app", "invalid".into()).await,
                Err(MereHostError::ResidentBusy)
            ));
            assert!(matches!(
                host.trash("app").await,
                Err(MereHostError::ResidentBusy)
            ));
            assert!(matches!(
                host.restore("app").await,
                Err(MereHostError::ResidentBusy)
            ));
            assert!(matches!(
                host.prepare_store(0),
                Err(MereHostError::ResidentBusy)
            ));
            assert_eq!(host.graph_session().journal().live_cursor(), cursor);
            assert_eq!(host.graph_session().revision(), revision);
            assert_eq!(host.graph_session().changes().len(), changes);
            backend.mode.store(2, Ordering::SeqCst);
            assert!(host.persist_owned(0).await.is_err());
            let work = host
                .retained_resident_work()
                .expect("uncertain batch retained");
            let prepared = work.state.parts.last().unwrap().ops.clone();
            assert!(matches!(
                host.persist(0).await,
                Err(MereHostError::ResidentBusy)
            ));
            backend.mode.store(0, Ordering::SeqCst);
            host.persist_owned(999).await.unwrap();
            assert_eq!(host.graph_session().journal().live_cursor(), cursor);
            assert!(host.retained_resident_work().is_none());
            assert_eq!(backend.calls.lock().unwrap().as_slice(), &[prepared]);
            assert_eq!(host.presentation_graph().node_count(), 1);
        });
    }

    #[test]
    fn observation_address_decoding_keeps_equal_uuid_strata_distinct() {
        let id = Uuid::from_u128(6100);
        let surface = ObservationRequestV1 {
            schema: OBSERVATION_SCHEMA.into(),
            address: Some(GraphAddress::Surface(id)),
        };
        let resource = ObservationRequestV1 {
            schema: OBSERVATION_SCHEMA.into(),
            address: Some(GraphAddress::Resource(id)),
        };
        let decoded_surface: ObservationRequestV1 =
            serde_json::from_slice(&serde_json::to_vec(&surface).unwrap()).unwrap();
        let decoded_resource: ObservationRequestV1 =
            serde_json::from_slice(&serde_json::to_vec(&resource).unwrap()).unwrap();
        assert_ne!(decoded_surface.address, decoded_resource.address);
        assert!(
            serde_json::from_str::<ObservationRequestV1>(
                r#"{"schema":"mere.resident.observation/v1","address":{"Resource":"invalid"}}"#
            )
            .is_err()
        );
    }

    #[test]
    fn settings_are_validated_and_both_slots_share_the_retained_checkpoint() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            assert_eq!(owner.settings(), &ResidentSettings::default());
            assert_eq!(owner.settings().policy, ResidencyPolicy::Full);
            assert_eq!(owner.settings().refresh, ResidentRefreshMode::Manual);
            assert_eq!(owner.settings().publication_batch_ms, 0);
            let invalid_interval = ResidentSettings {
                refresh: ResidentRefreshMode::HostDriven {
                    minimum_check_ms: 0,
                },
                ..Default::default()
            };
            assert!(owner.set_settings(invalid_interval).is_err());
            let mut settings = ResidentSettings {
                policy: ResidencyPolicy::Neighborhood { hops: 1 },
                projection: Some("resident-test".into()),
                refresh: ResidentRefreshMode::HostDriven {
                    minimum_check_ms: 75,
                },
                publication_batch_ms: 20,
            };
            assert!(owner.set_settings(settings.clone()).is_err());
            assert_eq!(owner.settings(), &ResidentSettings::default());
            assert!(backend.calls.lock().unwrap().is_empty());
            owner
                .host
                .as_mut()
                .unwrap()
                .save_projection(&authored_projection())
                .unwrap();
            owner.persist(SystemTime::UNIX_EPOCH).await.unwrap();
            let cursor = owner.session().journal().live_cursor();
            owner.set_settings(settings.clone()).unwrap();
            assert!(owner.host.as_ref().unwrap().has_unstored());
            backend.mode.store(1, Ordering::SeqCst);
            owner.persist(SystemTime::UNIX_EPOCH).await.unwrap();
            assert!(owner.is_busy());
            assert!(matches!(
                owner.set_settings(ResidentSettings::default()),
                Err(ResidentHostError::Busy)
            ));
            let graph = owner.session().manifest().root_graph_id;
            let prefix = format!("graphs/{}/resident/v1", graph.as_uuid());
            let slots = muniment::JsonSlots::new(backend.clone());
            assert_eq!(
                slots
                    .load::<ResidentSettings>(&format!("{prefix}/host-settings"))
                    .await
                    .unwrap(),
                Some(ResidentSettings::default())
            );
            let batch = backend.calls.lock().unwrap().last().unwrap().clone();
            assert!(batch.iter().any(|op| matches!(op, WriteOp::Put { key, .. } if key == &format!("{prefix}/host-settings"))));
            assert!(batch.iter().any(
                |op| matches!(op, WriteOp::Put { key, .. } if key == &format!("{prefix}/policy"))
            ));
            assert!(batch.iter().any(
                |op| matches!(op, WriteOp::Put { key, .. } if key == &format!("{prefix}/head"))
            ));
            backend.mode.store(0, Ordering::SeqCst);
            assert!(matches!(
                owner.persist(SystemTime::UNIX_EPOCH).await.unwrap(),
                ResidentPersistence::Durable { .. }
            ));
            assert_eq!(backend.calls.lock().unwrap().last(), Some(&batch));
            assert_eq!(
                slots
                    .load::<ResidentSettings>(&format!("{prefix}/host-settings"))
                    .await
                    .unwrap(),
                Some(settings.clone())
            );
            assert_eq!(
                slots
                    .load::<ResidencyPolicy>(&format!("{prefix}/policy"))
                    .await
                    .unwrap(),
                Some(settings.policy)
            );
            assert_eq!(owner.session().journal().live_cursor(), cursor);
            assert!(!owner.host.as_ref().unwrap().has_unstored());
            let reopened = MereHost::open_session(
                backend.clone(),
                owner.session().id(),
                persona(),
                HandlerRegistry::default(),
                access(),
            )
            .await
            .unwrap();
            assert_eq!(reopened.resident_settings(), &settings);
            settings.policy = ResidencyPolicy::Full;
            owner.set_settings(settings).unwrap();
            owner.persist(SystemTime::UNIX_EPOCH).await.unwrap();
            assert!(
                !owner
                    .presentation_graph()
                    .known_coverage()
                    .limits
                    .iter()
                    .any(|l| l.layer == mere::kernel::graph::CoverageLayer::Residency)
            );
        });
    }

    #[test]
    fn pane_close_reconciles_the_actual_presentation_and_preserves_writer_and_views() {
        pollster::block_on(async {
            use pandect::addressable_graph::ResidencyStatus;
            let (mut owner, _) = owner().await;
            for id in 4100..4104 {
                owner
                    .edit("test", command(id), SystemTime::UNIX_EPOCH)
                    .unwrap();
                owner.persist(SystemTime::UNIX_EPOCH).await.unwrap();
            }
            let host = owner.host.as_mut().unwrap();
            host.save_projection(&authored_projection()).unwrap();
            let view = pandect::view_intent_store::ViewIntent {
                focus: Some("https://resident.test/4103".into()),
                ..Default::default()
            };
            host.set_view_now("graphshell", "kept-view", view.clone())
                .unwrap();
            host.set_resident_pins(BTreeSet::from([GraphAddress::Surface(Uuid::from_u128(
                4103,
            ))]));
            owner
                .set_settings(ResidentSettings {
                    policy: ResidencyPolicy::Neighborhood { hops: 0 },
                    projection: Some("resident-test".into()),
                    ..Default::default()
                })
                .unwrap();
            let first = ViewKey::new("graphshell", "first").unwrap();
            let second = ViewKey::new("graphshell", "second").unwrap();
            let shared = Uuid::from_u128(4100);
            let missing = Uuid::from_u128(4199);
            owner.host.as_mut().unwrap().set_resident_pane(
                first.clone(),
                PaneRoots {
                    roster: BTreeSet::from([shared, missing]),
                    focus: Some(GraphAddress::Surface(Uuid::from_u128(4101))),
                },
            );
            owner.host.as_mut().unwrap().set_resident_pane(
                second.clone(),
                PaneRoots {
                    roster: BTreeSet::from([shared, missing]),
                    focus: Some(GraphAddress::Surface(Uuid::from_u128(4102))),
                },
            );
            owner.persist(SystemTime::UNIX_EPOCH).await.unwrap();
            let before = owner.reader().unwrap().footprint().retained_record_bytes();
            let writer = owner.session().graph().to_snapshot_at(0);
            let cursor = owner.session().journal().live_cursor();
            owner.close_pane(&first).await.unwrap();
            let reader = owner.reader().unwrap();
            assert_eq!(
                reader.status(GraphAddress::Surface(Uuid::from_u128(4101))),
                ResidencyStatus::NotLoaded
            );
            assert_eq!(
                reader.status(GraphAddress::Surface(Uuid::from_u128(4102))),
                ResidencyStatus::Loaded
            );
            assert_eq!(
                reader.status(GraphAddress::Surface(Uuid::from_u128(4103))),
                ResidencyStatus::Loaded
            );
            assert!(reader.footprint().retained_record_bytes() < before);
            let host = owner.host.as_ref().unwrap();
            assert!(std::ptr::eq(host.presentation_graph(), reader.graph()));
            assert!(host.presentation_graph().node_count() < host.graph().node_count());
            assert_eq!(
                host.resident_roots().union().projection_surfaces,
                BTreeSet::from([shared, missing])
            );
            assert_eq!(
                host.resident_roots().union().pinned,
                BTreeSet::from([GraphAddress::Surface(Uuid::from_u128(4103))])
            );
            assert_eq!(host.view("graphshell", "kept-view"), Some(&view));
            assert_eq!(
                serde_json::to_value(host.graph().to_snapshot_at(0)).unwrap(),
                serde_json::to_value(writer).unwrap()
            );
            assert_eq!(owner.session().journal().live_cursor(), cursor);
            owner.close_pane(&second).await.unwrap();
            assert_eq!(
                owner
                    .reader()
                    .unwrap()
                    .status(GraphAddress::Surface(shared)),
                ResidencyStatus::NotLoaded
            );
            assert_eq!(
                owner
                    .reader()
                    .unwrap()
                    .status(GraphAddress::Surface(Uuid::from_u128(4103))),
                ResidencyStatus::Loaded
            );
        });
    }

    #[test]
    fn retry_twice_keeps_exact_batch_author_and_change_identity() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            let applied = owner
                .edit("turnstone", command(10), SystemTime::UNIX_EPOCH)
                .unwrap();
            let cursor = owner.session().journal().live_cursor();
            let count = owner.session().changes().len();
            backend.mode.store(1, Ordering::SeqCst);
            for _ in 0..2 {
                assert_eq!(owner.persist(SystemTime::now()).await.unwrap(), applied);
                assert_eq!(owner.session().journal().live_cursor(), cursor);
                assert_eq!(owner.session().changes().len(), count);
                assert_eq!(owner.reader().unwrap().graph().node_count(), 0);
                assert!(matches!(
                    owner.edit("turnstone", command(11), SystemTime::now()),
                    Err(ResidentHostError::Busy)
                ));
                assert!(matches!(
                    owner.undo("turnstone", SystemTime::now()),
                    Err(ResidentHostError::Busy)
                ));
                assert!(matches!(
                    owner.redo("turnstone", SystemTime::now()),
                    Err(ResidentHostError::Busy)
                ));
                let host = owner.host.as_mut().unwrap();
                assert!(matches!(
                    host.apply_edits(Vec::new()),
                    Err(MereHostError::ResidentBusy)
                ));
                assert!(matches!(
                    host.set_view_now("app", "main", Default::default()),
                    Err(MereHostError::ResidentBusy)
                ));
                assert!(matches!(
                    host.begin_profiled_session(Default::default(), None),
                    Err(MereHostError::ResidentBusy)
                ));
                assert!(matches!(
                    host.prepare_store(0),
                    Err(MereHostError::ResidentBusy)
                ));
                assert!(matches!(
                    host.persist(0).await,
                    Err(MereHostError::ResidentBusy)
                ));
            }
            backend.mode.store(0, Ordering::SeqCst);
            assert!(matches!(
                owner.persist(SystemTime::now()).await.unwrap(),
                ResidentPersistence::Durable { .. }
            ));
            let calls = backend.calls.lock().unwrap();
            assert_eq!(calls.len(), 3);
            assert_eq!(calls[0], calls[1]);
            assert_eq!(calls[1], calls[2]);
            assert_eq!(owner.session().journal().live_cursor(), cursor);
            assert_eq!(
                owner.session().changes().last().unwrap().author,
                pandect::Author::person("test-author").via("turnstone")
            );
            assert_eq!(owner.reader().unwrap().graph().node_count(), 1);
        });
    }
    #[test]
    fn applied_error_acknowledges_once_without_a_second_apply() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            owner
                .edit("turnstone", command(20), SystemTime::UNIX_EPOCH)
                .unwrap();
            let cursor = owner.session().journal().live_cursor();
            backend.mode.store(2, Ordering::SeqCst);
            assert!(matches!(
                owner.persist(SystemTime::now()).await.unwrap(),
                ResidentPersistence::AppliedButUnstored { .. }
            ));
            backend.mode.store(0, Ordering::SeqCst);
            assert!(matches!(
                owner.persist(SystemTime::now()).await.unwrap(),
                ResidentPersistence::Durable { .. }
            ));
            assert_eq!(backend.calls.lock().unwrap().len(), 1);
            assert_eq!(owner.session().journal().live_cursor(), cursor);
            assert!(!owner.session().has_unstored());
        });
    }
    #[test]
    fn cancelled_io_retains_uncertainty_and_reconciles_without_reapplying() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            owner
                .edit("turnstone", command(25), SystemTime::UNIX_EPOCH)
                .unwrap();
            let cursor = owner.session().journal().live_cursor();
            let io = owner
                .host
                .as_mut()
                .unwrap()
                .prepare_resident_io(SystemTime::UNIX_EPOCH)
                .unwrap();
            backend.mode.store(6, Ordering::SeqCst);
            {
                let mut future = std::pin::pin!(io.write());
                let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
                assert!(std::future::Future::poll(future.as_mut(), &mut cx).is_pending());
                assert!(matches!(io.write().await, Err(ResidentHostError::Busy)));
                assert!(matches!(
                    owner.host.as_mut().unwrap().install_resident_io(&io),
                    Err(ResidentHostError::Busy)
                ));
                assert_eq!(owner.reader().unwrap().graph().node_count(), 0);
            }
            backend.mode.store(0, Ordering::SeqCst);
            assert!(matches!(
                owner.persist(SystemTime::now()).await.unwrap(),
                ResidentPersistence::Durable { .. }
            ));
            assert_eq!(backend.calls.lock().unwrap().len(), 1);
            assert_eq!(owner.session().journal().live_cursor(), cursor);
            assert_eq!(owner.reader().unwrap().graph().node_count(), 1);
        });
    }
    #[test]
    fn mixed_keys_stop_writing_and_keep_old_reader() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            owner
                .edit("turnstone", command(30), SystemTime::UNIX_EPOCH)
                .unwrap();
            backend.mode.store(3, Ordering::SeqCst);
            owner.persist(SystemTime::now()).await.unwrap();
            backend.mode.store(0, Ordering::SeqCst);
            assert!(matches!(
                owner.persist(SystemTime::now()).await,
                Err(ResidentHostError::MixedCommit)
            ));
            assert!(matches!(
                owner.persist(SystemTime::now()).await,
                Err(ResidentHostError::MixedCommit)
            ));
            assert_eq!(backend.calls.lock().unwrap().len(), 1);
            assert_eq!(owner.reader().unwrap().graph().node_count(), 0);
            assert!(owner.is_busy());
        });
    }
    #[test]
    fn preparation_codec_failure_retains_applied_identity() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            owner.host.as_mut().unwrap().resident_prepare_error =
                Some(StoreError::Codec("injected preparation codec fault".into()));
            let applied = owner
                .edit("turnstone", command(40), SystemTime::UNIX_EPOCH)
                .unwrap();
            assert!(matches!(
                applied,
                ResidentPersistence::AppliedButUnstored { .. }
            ));
            assert!(backend.calls.lock().unwrap().is_empty());
            assert!(owner.is_busy());
            let cursor = owner.session().journal().live_cursor();
            assert!(matches!(
                owner.persist(SystemTime::now()).await.unwrap(),
                ResidentPersistence::Durable { .. }
            ));
            assert_eq!(owner.session().journal().live_cursor(), cursor);
        });
    }
    #[test]
    fn failed_reader_refresh_retries_only_publication() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            owner
                .edit("turnstone", command(50), SystemTime::UNIX_EPOCH)
                .unwrap();
            backend.mode.store(5, Ordering::SeqCst);
            assert!(matches!(
                owner.persist(SystemTime::now()).await.unwrap(),
                ResidentPersistence::PendingPublication { .. }
            ));
            assert_eq!(owner.reader().unwrap().graph().node_count(), 0);
            assert!(!owner.session().has_unstored());
            backend.mode.store(0, Ordering::SeqCst);
            assert!(matches!(
                owner.persist(SystemTime::now()).await.unwrap(),
                ResidentPersistence::Durable { .. }
            ));
            assert_eq!(backend.calls.lock().unwrap().len(), 1);
            assert_eq!(owner.reader().unwrap().graph().node_count(), 1);
        });
    }
    #[test]
    fn foreign_and_duplicate_completion_are_refused() {
        pollster::block_on(async {
            let (mut first, backend) = owner().await;
            let (mut second, _) = owner().await;
            first
                .edit("turnstone", command(60), SystemTime::UNIX_EPOCH)
                .unwrap();
            second
                .edit("turnstone", command(61), SystemTime::UNIX_EPOCH)
                .unwrap();
            let work = first
                .host
                .as_mut()
                .unwrap()
                .prepare_resident_store(SystemTime::now())
                .unwrap();
            assert!(matches!(
                second.host.as_mut().unwrap().install_resident_store(&work),
                Err(ResidentHostError::StaleCompletion)
            ));
            work.write(&backend).await.unwrap();
            assert!(
                first
                    .host
                    .as_mut()
                    .unwrap()
                    .install_resident_store(&work)
                    .unwrap()
                    .is_some()
            );
            assert!(matches!(
                first.host.as_mut().unwrap().install_resident_store(&work),
                Err(ResidentHostError::StaleCompletion)
            ));
        });
    }
    #[test]
    fn detached_io_uses_only_its_owner_store() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            let foreign = Fault::default();
            owner
                .edit("turnstone", command(65), SystemTime::UNIX_EPOCH)
                .unwrap();
            let host = owner.host.as_mut().unwrap();
            let io = host.prepare_resident_io(SystemTime::UNIX_EPOCH).unwrap();
            io.write().await.unwrap();
            assert_eq!(backend.calls.lock().unwrap().len(), 1);
            assert!(foreign.calls.lock().unwrap().is_empty());
            assert!(host.install_resident_io(&io).unwrap().is_some());
            // Generic preparation never grants owner admission, even for the
            // same concrete backend type used by the admitted test owner.
            let mut unowned =
                MereHost::empty(foreign, persona(), HandlerRegistry::default(), access());
            assert!(matches!(
                unowned.prepare_resident_io(SystemTime::UNIX_EPOCH),
                Err(ResidentHostError::OwnershipUnproven)
            ));
        });
    }
    #[test]
    fn publication_without_an_edit_can_remain_pending() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            let cursor = owner.session().journal().live_cursor();
            backend.mode.store(5, Ordering::SeqCst);
            assert!(matches!(
                owner.persist(SystemTime::UNIX_EPOCH).await.unwrap(),
                ResidentPersistence::PendingPublication { .. }
            ));
            assert!(owner.is_busy());
            backend.mode.store(0, Ordering::SeqCst);
            assert!(matches!(
                owner.persist(SystemTime::now()).await.unwrap(),
                ResidentPersistence::Durable { .. }
            ));
            assert_eq!(backend.calls.lock().unwrap().len(), 1);
            assert_eq!(owner.session().journal().live_cursor(), cursor);
        });
    }
    #[test]
    fn retired_success_is_acknowledged_when_current_batch_fails() {
        pollster::block_on(async {
            let backend = Fault::default();
            let mut host = MereHost::empty(
                backend.clone(),
                persona(),
                HandlerRegistry::default(),
                access(),
            );
            let retired_id = host.graph_session().id();
            host.begin_profiled_session(
                Default::default(),
                Some(pandect::graph_placement::PlacementProfile::RecordedStrataV1),
            )
            .unwrap();
            host.admit_resident();
            let work = host.prepare_resident_store(SystemTime::UNIX_EPOCH).unwrap();
            work.state.parts[0].write(&backend).await.unwrap();
            backend.mode.store(1, Ordering::SeqCst);
            assert!(work.write(&backend).await.is_err());
            assert!(host.install_resident_store(&work).unwrap().is_none());
            assert!(
                pandect::MereSessions::new(backend.clone())
                    .open(retired_id)
                    .await
                    .is_ok()
            );
            backend.mode.store(0, Ordering::SeqCst);
            work.write(&backend).await.unwrap();
            assert!(host.install_resident_store(&work).unwrap().is_some());
            let calls = backend.calls.lock().unwrap();
            assert_eq!(calls.len(), 3);
            assert_eq!(calls[1], calls[2]);
            assert!(!host.has_unstored());
        });
    }
    fn pause_load<B: Backend>(
        request: HostRefreshRequest<B>,
    ) -> std::pin::Pin<Box<impl std::future::Future<Output = HostRefreshCompletion<B>>>> {
        use std::future::Future;
        let mut future = Box::pin(request.load());
        let mut context = std::task::Context::from_waker(std::task::Waker::noop());
        assert!(future.as_mut().poll(&mut context).is_pending());
        future
    }

    #[test]
    fn owned_load_releases_refcell_and_retains_newer_notices_and_context() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            let state = std::cell::RefCell::new(owner.host.take().unwrap());
            let request = {
                let mut host = state.borrow_mut();
                host.notice_resident_refresh();
                host.prepare_resident_refresh().unwrap().unwrap()
            };
            backend.mode.store(7, Ordering::SeqCst);
            let pending = pause_load(request);
            {
                let mut host = state.borrow_mut();
                for _ in 0..100 {
                    host.notice_resident_refresh();
                }
                assert!(host.prepare_resident_refresh().unwrap().is_none());
                let mut note = mere::kernel::graph::CoverageNote::default();
                note.push(mere::kernel::graph::CoverageLimit::new(
                    mere::kernel::graph::CoverageLayer::Disclosure,
                    "arrived during load",
                ));
                host.resident_reader
                    .as_mut()
                    .unwrap()
                    .set_host_coverage(note);
            }
            backend.mode.store(0, Ordering::SeqCst);
            let completion = pending.await;
            let mut host = state.borrow_mut();
            assert_eq!(
                host.install_resident_refresh(completion).unwrap(),
                pandect::addressable_graph::RefreshOutcome::Unchanged
            );
            assert!(host.resident_needs_check());
            assert!(
                host.resident_reader()
                    .unwrap()
                    .graph()
                    .known_coverage()
                    .limits
                    .iter()
                    .any(|l| l.reason == "arrived during load")
            );
            let request = host.prepare_resident_refresh().unwrap().unwrap();
            drop(host);
            let completion = request.load().await;
            let mut host = state.borrow_mut();
            host.install_resident_refresh(completion).unwrap();
            assert!(!host.resident_needs_check());
        });
    }

    #[test]
    fn controlled_load_rejects_roots_policy_session_and_lifecycle_switches() {
        pollster::block_on(async {
            for change in 0..5 {
                let (mut owner, backend) = owner().await;
                let host = owner.host.as_mut().unwrap();
                host.notice_resident_refresh();
                let request = host.prepare_resident_refresh().unwrap().unwrap();
                backend.mode.store(7, Ordering::SeqCst);
                let pending = pause_load(request);
                match change {
                    0 => host.set_resident_pane(
                        ViewKey::new("test", "changed").unwrap(),
                        PaneRoots::default(),
                    ),
                    1 => host
                        .set_resident_settings(ResidentSettings {
                            publication_batch_ms: 9,
                            ..Default::default()
                        })
                        .unwrap(),
                    2 => host
                        .begin_profiled_session(
                            mere::kernel::graph::Graph::new(),
                            Some(pandect::graph_placement::PlacementProfile::RecordedStrataV1),
                        )
                        .unwrap(),
                    3 => host.suspend_resident_refresh(true),
                    _ => host.detach_resident_refresh(),
                }
                backend.mode.store(0, Ordering::SeqCst);
                assert!(matches!(
                    host.install_resident_refresh(pending.await),
                    Err(ResidentHostError::StaleCompletion)
                ));
                assert!(host.resident_needs_check());
                if change == 2 {
                    assert!(host.resident_reader().is_none());
                } else {
                    assert_eq!(host.resident_reader().unwrap().graph().node_count(), 0);
                }
            }
        });
    }

    #[test]
    fn failed_load_and_failed_demand_preserve_reader_and_retry_notice() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            let host = owner.host.as_mut().unwrap();
            let generation = host.resident_reader().unwrap().generation().to_owned();
            host.notice_resident_refresh();
            let request = host.prepare_resident_refresh().unwrap().unwrap();
            backend.mode.store(7, Ordering::SeqCst);
            let pending = pause_load(request);
            backend.mode.store(4, Ordering::SeqCst);
            assert!(host.install_resident_refresh(pending.await).is_err());
            assert!(host.resident_needs_check());
            backend.mode.store(0, Ordering::SeqCst);
            let request = host
                .prepare_resident_demand(GraphAddress::Surface(Uuid::from_u128(98765)))
                .unwrap()
                .unwrap();
            assert!(host.install_resident_refresh(request.load().await).is_err());
            assert!(host.resident_needs_check());
            assert_eq!(host.resident_reader().unwrap().generation(), generation);
            assert!(host.resident_roots().union().pinned.is_empty());
            host.refresh_resident_reader().await.unwrap();
            assert!(!host.resident_needs_check());
        });
    }

    #[test]
    fn staged_settings_validate_before_writes_and_keep_the_old_reader() {
        pollster::block_on(async {
            for change in 0..3 {
                let (mut owner, backend) = owner().await;
                let host = owner.host.as_mut().unwrap();
                host.set_resident_settings(ResidentSettings {
                    publication_batch_ms: 50,
                    ..Default::default()
                })
                .unwrap();
                assert!(host.prepare_resident_io(SystemTime::UNIX_EPOCH).is_err());
                let request = host
                    .prepare_resident_settings_validation()
                    .unwrap()
                    .unwrap();
                backend.mode.store(7, Ordering::SeqCst);
                let pending = pause_load(request);
                if change == 1 {
                    host.set_resident_settings(ResidentSettings {
                        publication_batch_ms: 60,
                        ..Default::default()
                    })
                    .unwrap();
                }
                if change == 2 {
                    host.detach_resident_refresh();
                    host.attach_resident_refresh();
                }
                backend
                    .mode
                    .store(if change == 0 { 4 } else { 0 }, Ordering::SeqCst);
                assert!(host.install_resident_refresh(pending.await).is_err());
                assert!(host.resident_settings_pending());
                assert!(host.resident_needs_check());
                assert!(backend.calls.lock().unwrap().is_empty());
                assert_eq!(
                    host.resident_reader().unwrap().policy(),
                    ResidencyPolicy::Full
                );
                backend.mode.store(0, Ordering::SeqCst);
                let request = host
                    .prepare_resident_settings_validation()
                    .unwrap()
                    .unwrap();
                host.install_resident_refresh(request.load().await).unwrap();
                let work = host.prepare_resident_io(SystemTime::UNIX_EPOCH).unwrap();
                assert!(backend.calls.lock().unwrap().is_empty());
                work.write().await.unwrap();
                host.install_resident_io(&work).unwrap();
                assert!(host.resident_needs_check());
                host.refresh_resident_reader().await.unwrap();
                assert!(!host.resident_settings_pending());
            }
        });
    }

    async fn stale_settings_continuation_retries(change: u8) {
        let (mut owner, backend) = owner().await;
        let host = owner.host.as_mut().unwrap();
        host.set_resident_settings(ResidentSettings {
            publication_batch_ms: 50,
            ..Default::default()
        })
        .unwrap();
        let generation = host.resident_reader().unwrap().generation().to_owned();
        let mut status = SessionStore::Writing;
        let mut error = "existing diagnostic".to_owned();
        let attempt = host.begin_session_store_attempt();
        let request = host
            .prepare_resident_settings_validation()
            .unwrap()
            .unwrap();
        backend.mode.store(7, Ordering::SeqCst);
        let pending = pause_load(request);
        match change {
            0 => host.suspend_resident_refresh(true),
            1 => host
                .set_resident_settings(ResidentSettings {
                    publication_batch_ms: 60,
                    ..Default::default()
                })
                .unwrap(),
            _ => host.set_resident_pane(
                ViewKey::new("test", "intervening-root").unwrap(),
                PaneRoots::default(),
            ),
        }
        backend.mode.store(0, Ordering::SeqCst);
        assert!(!host.continue_settings_store(&attempt, pending.await, &mut status, &mut error));
        assert_eq!(
            status,
            SessionStore::Pending,
            "stale validation must remain schedulable"
        );
        assert_eq!(error, "existing diagnostic");
        assert!(host.resident_settings_pending());
        assert!(host.resident_needs_check());
        assert!(backend.calls.lock().unwrap().is_empty());
        assert_eq!(host.resident_reader().unwrap().generation(), generation);
        if change == 0 {
            assert!(
                host.prepare_resident_settings_validation()
                    .unwrap()
                    .is_none()
            );
            host.suspend_resident_refresh(false);
        }
        // Exercise the same Pending guard and real continuation used by the pump.
        assert_eq!(status, SessionStore::Pending);
        status = SessionStore::Writing;
        let attempt = host.begin_session_store_attempt();
        let request = host
            .prepare_resident_settings_validation()
            .unwrap()
            .unwrap();
        assert!(host.continue_settings_store(
            &attempt,
            request.load().await,
            &mut status,
            &mut error
        ));
        let work = host.prepare_resident_io(SystemTime::UNIX_EPOCH).unwrap();
        assert!(backend.calls.lock().unwrap().is_empty());
        work.write().await.unwrap();
        host.install_resident_io(&work).unwrap();
        assert!(!host.resident_settings_pending());
        host.refresh_resident_reader().await.unwrap();
        assert!(!host.resident_needs_check());
    }

    #[test]
    fn settings_continuation_retries_after_hide_resume() {
        pollster::block_on(stale_settings_continuation_retries(0));
    }

    #[test]
    fn settings_continuation_retries_after_settings_change() {
        pollster::block_on(stale_settings_continuation_retries(1));
    }

    #[test]
    fn settings_continuation_retries_after_roots_change() {
        pollster::block_on(stale_settings_continuation_retries(2));
    }

    #[test]
    fn settings_continuation_preserves_new_session_status_and_error() {
        pollster::block_on(async {
            for current in [
                SessionStore::Stored,
                SessionStore::Pending,
                SessionStore::Writing,
                SessionStore::Failed,
            ] {
                let (mut owner, backend) = owner().await;
                let host = owner.host.as_mut().unwrap();
                host.set_resident_settings(ResidentSettings {
                    publication_batch_ms: 50,
                    ..Default::default()
                })
                .unwrap();
                let attempt = host.begin_session_store_attempt();
                let request = host
                    .prepare_resident_settings_validation()
                    .unwrap()
                    .unwrap();
                backend.mode.store(7, Ordering::SeqCst);
                let pending = pause_load(request);
                host.begin_profiled_session(
                    mere::kernel::graph::Graph::new(),
                    Some(pandect::graph_placement::PlacementProfile::RecordedStrataV1),
                )
                .unwrap();
                let mut status = current;
                let mut error = "new session diagnostic".to_owned();
                backend.mode.store(0, Ordering::SeqCst);
                assert!(!host.continue_settings_store(
                    &attempt,
                    pending.await,
                    &mut status,
                    &mut error
                ));
                assert_eq!(status, current);
                assert_eq!(error, "new session diagnostic");
                assert!(host.resident_needs_check());
                assert!(backend.calls.lock().unwrap().is_empty());
                assert!(host.resident_reader().is_none());
            }
        });
    }

    #[test]
    fn settings_continuation_current_read_failure_remains_actionable() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            let host = owner.host.as_mut().unwrap();
            host.set_resident_settings(ResidentSettings {
                publication_batch_ms: 50,
                ..Default::default()
            })
            .unwrap();
            let attempt = host.begin_session_store_attempt();
            let request = host
                .prepare_resident_settings_validation()
                .unwrap()
                .unwrap();
            backend.mode.store(7, Ordering::SeqCst);
            let pending = pause_load(request);
            let mut status = SessionStore::Writing;
            let mut error = String::new();
            backend.mode.store(4, Ordering::SeqCst);
            assert!(!host.continue_settings_store(
                &attempt,
                pending.await,
                &mut status,
                &mut error
            ));
            assert_eq!(status, SessionStore::Failed);
            assert!(!error.is_empty());
            assert!(host.resident_settings_pending());
            assert!(host.resident_needs_check());
            assert!(backend.calls.lock().unwrap().is_empty());
        });
    }

    #[test]
    fn settings_continuation_preserves_newer_store_attempt_and_queued_work() {
        pollster::block_on(async {
            for read_mode in [0, 4] {
                for current in [
                    SessionStore::Stored,
                    SessionStore::Pending,
                    SessionStore::Writing,
                    SessionStore::Failed,
                ] {
                    let (mut owner, backend) = owner().await;
                    let host = owner.host.as_mut().unwrap();
                    host.set_resident_settings(ResidentSettings {
                        publication_batch_ms: 50,
                        ..Default::default()
                    })
                    .unwrap();
                    let old_attempt = host.begin_session_store_attempt();
                    let request = host
                        .prepare_resident_settings_validation()
                        .unwrap()
                        .unwrap();
                    backend.mode.store(7, Ordering::SeqCst);
                    let pending = pause_load(request);
                    // An actual browser Pending writer can dispatch a newer
                    // attempt while the first preflight still owns the slot.
                    let newer_attempt = host.begin_session_store_attempt();
                    assert!(!host.session_store_attempt_is_current(&old_attempt));
                    assert!(host.session_store_attempt_is_current(&newer_attempt));
                    assert!(
                        host.prepare_resident_settings_validation()
                            .unwrap()
                            .is_none()
                    );
                    host.notice_resident_refresh();
                    let consumed = host.resident_refresh.gate.consumed_notice;
                    let notice = host.resident_refresh.gate.newest_notice;
                    let generation = host.resident_reader().unwrap().generation().to_owned();
                    let mut status = current;
                    let mut error = "new attempt diagnostic".to_owned();
                    backend.mode.store(read_mode, Ordering::SeqCst);
                    assert!(!host.continue_settings_store(
                        &old_attempt,
                        pending.await,
                        &mut status,
                        &mut error
                    ));
                    assert_eq!(status, current);
                    assert_eq!(error, "new attempt diagnostic");
                    assert_eq!(host.resident_refresh.gate.consumed_notice, consumed);
                    assert_eq!(host.resident_refresh.gate.newest_notice, notice);
                    assert!(host.resident_refresh.gate.in_flight.is_none());
                    assert!(host.resident_settings_need_validation());
                    assert!(host.resident_needs_check());
                    assert!(backend.calls.lock().unwrap().is_empty());
                    assert_eq!(host.resident_reader().unwrap().generation(), generation);
                    // The current attempt can retry after the cancelled load
                    // frees the slot. Its validation alone permits staging.
                    backend.mode.store(0, Ordering::SeqCst);
                    let request = host
                        .prepare_resident_settings_validation()
                        .unwrap()
                        .unwrap();
                    assert!(host.continue_settings_store(
                        &newer_attempt,
                        request.load().await,
                        &mut status,
                        &mut error
                    ));
                    let work = host.prepare_resident_io(SystemTime::UNIX_EPOCH).unwrap();
                    assert!(backend.calls.lock().unwrap().is_empty());
                    work.write().await.unwrap();
                    host.install_resident_io(&work).unwrap();
                    assert!(!host.resident_settings_pending());
                }
            }
        });
    }

    #[test]
    fn settings_continuation_retry_keeps_exact_uncertain_bytes() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            let host = owner.host.as_mut().unwrap();
            host.set_resident_settings(ResidentSettings {
                publication_batch_ms: 50,
                ..Default::default()
            })
            .unwrap();
            let attempt = host.begin_session_store_attempt();
            let request = host
                .prepare_resident_settings_validation()
                .unwrap()
                .unwrap();
            let mut status = SessionStore::Writing;
            let mut error = String::new();
            assert!(host.continue_settings_store(
                &attempt,
                request.load().await,
                &mut status,
                &mut error
            ));
            let work = host.prepare_resident_io(SystemTime::UNIX_EPOCH).unwrap();
            assert!(backend.calls.lock().unwrap().is_empty());
            backend.mode.store(1, Ordering::SeqCst);
            assert!(
                work.write()
                    .await
                    .unwrap_err()
                    .to_string()
                    .contains("before apply")
            );
            assert!(host.install_resident_io(&work).unwrap().is_none());
            let bytes = backend.calls.lock().unwrap().last().unwrap().clone();
            host.suspend_resident_refresh(true);
            host.suspend_resident_refresh(false);
            let newer = host.begin_session_store_attempt();
            assert!(!host.session_store_attempt_is_current(&attempt));
            assert!(host.session_store_attempt_is_current(&newer));
            assert!(
                !host.resident_settings_need_validation(),
                "retained bytes bypass fresh preflight"
            );
            let retry = host
                .prepare_resident_io(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(100))
                .unwrap();
            backend.mode.store(0, Ordering::SeqCst);
            retry.write().await.unwrap();
            assert_eq!(backend.calls.lock().unwrap().last(), Some(&bytes));
            host.install_resident_io(&retry).unwrap();
            assert!(!host.resident_settings_pending());
            assert!(!host.resident_busy());
        });
    }

    #[test]
    fn foreground_catches_up_and_clocks_and_publication_intervals_are_separate() {
        pollster::block_on(async {
            let (mut owner, _) = owner().await;
            let host = owner.host.as_mut().unwrap();
            host.resident_settings.refresh = ResidentRefreshMode::HostDriven {
                minimum_check_ms: 100,
            };
            host.resident_settings.publication_batch_ms = 500;
            host.tick_resident_refresh(100.0);
            let request = host.prepare_resident_refresh().unwrap().unwrap();
            host.install_resident_refresh(request.load().await).unwrap();
            for clock in [99.0, f64::NAN, f64::INFINITY, 199.0] {
                host.tick_resident_refresh(clock);
            }
            assert!(!host.resident_needs_check());
            assert!(!host.resident_publication_due(199.0, false));
            assert!(host.resident_publication_due(199.0, true));
            host.tick_resident_refresh(200.0);
            assert!(host.resident_needs_check());
            host.suspend_resident_refresh(true);
            assert!(host.prepare_resident_refresh().unwrap().is_none());
            host.tick_resident_refresh(1000.0);
            host.suspend_resident_refresh(false);
            let request = host.prepare_resident_refresh().unwrap().unwrap();
            host.install_resident_refresh(request.load().await).unwrap();
            assert!(!host.resident_needs_check());
            assert!(host.resident_publication_due(1000.0, false));
        });
    }
}
