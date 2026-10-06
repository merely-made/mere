// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The resident's account of itself and its owner-only stop, on the
//! application door (djinn test harness plan, rulings 7 and 9).
//!
//! `resident-status-v1` is read-only: how this start unlocked, what protects
//! the vault, the lock state (Unlocked until the vault lock exists), the
//! endpoints it serves, the ticket it listens on, and whether it is ready.
//! `resident-control-v1` takes two intents: stop, and the resident leaves the
//! way Ctrl-C makes it leave; and unlock (vault lock ruling 41), carrying the
//! passphrase `djinn --unlock` read at the terminal. Only this route, never
//! the status route, carries it. They are separate routes so that granting the
//! status to an application never grants it the stop. Both are granted to
//! the `djinn` label alone, as the device directory is.

use std::future::Future;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use chirograph::{
    BoundsRelationship, CachePolicy, CardValueV1, CarrierRequestBody, CarrierResponseBody,
    ContentHash, EndpointDescriptor, IntentInvocation, IntentResult, PortableCardV1,
    PresentationBinding, PresentationCapability, PresentationCodec, PresentationKey,
    PresentationManifest, PresentationOffer, PresentationSemantics, ProjectionOffer,
    ProjectionRequest, ProjectionSession, ProjectionSnapshot, ProtocolVersion, ResourceRequest,
    ResourceResponse, SemanticRole,
};
use graphshell::identity::{VaultLockView, VaultProtectionView};
use graphshell::native::app_admission::{AppId, AppRouteGrants};
use graphshell::native::app_client::{AppBrokerClient, AppClientError};
use graphshell::native::endpoint_catalog::{
    ResidentEndpointCatalog, ResidentEndpointCatalogError, ResidentEndpointRoute,
};
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionCatalog, ProjectionSource};
use sceno::{
    Arrangement, Footprint, InstanceId, ProjectedItem, Representation, Scene, Score, Size2,
    SourceRef, Transform2,
};
use scenotime::{Revision, SceneEpoch, SceneSnapshot};
use serde::{Deserialize, Serialize};
use tokio::sync::Notify;

use crate::resident_devices::DEVICE_DIRECTORY_APP;

pub const RESIDENT_STATUS_ROUTE: &str = "resident-status-v1";
pub const RESIDENT_CONTROL_ROUTE: &str = "resident-control-v1";
/// The one caller both routes are granted to: djinn's own CLI.
pub const RESIDENT_APP: &str = DEVICE_DIRECTORY_APP;
pub const STATUS_SCHEMA: &str = "djinn.resident-status/v1";
pub const STOP_INTENT: &str = "djinn.resident/stop-v1";
/// Unlock by passphrase; the payload is the passphrase's raw bytes.
pub const UNLOCK_INTENT: &str = "djinn.resident/unlock-v1";
/// Show the resident's own unlock prompt; the payload is empty, and no
/// credential crosses the pipe (vault lock ruling 47).
pub const UNLOCK_NATIVE_INTENT: &str = "djinn.resident/unlock-native-v1";
const STATUS_SESSION: &str = "djinn.resident-status/v1";
const CONTROL_SESSION: &str = "djinn.resident-control/v1";

/// How this start opened the vault.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StartupUnlockV1 {
    AutoOs,
    Passphrase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectionV1 {
    OsProtected,
    Passphrase,
    Ephemeral,
}

impl From<VaultProtectionView> for ProtectionV1 {
    fn from(view: VaultProtectionView) -> Self {
        match view {
            VaultProtectionView::OsProtected => Self::OsProtected,
            VaultProtectionView::Passphrase => Self::Passphrase,
            VaultProtectionView::Ephemeral => Self::Ephemeral,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LockStateV1 {
    Unlocked,
    Locked,
}

impl From<VaultLockView> for LockStateV1 {
    fn from(view: VaultLockView) -> Self {
        match view {
            VaultLockView::Unlocked => Self::Unlocked,
            VaultLockView::Locked => Self::Locked,
        }
    }
}

/// Which listener holds the agent endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentListenerV1 {
    Standard,
    Receipt,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResidentEndpointsV1 {
    pub agent: String,
    pub agent_listener: AgentListenerV1,
    pub browser: String,
    pub app: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncStatusV1 {
    pub node_id: String,
    pub ticket: String,
}

/// The resident, as it reports itself.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResidentStatusV1 {
    pub schema: String,
    pub pid: u32,
    pub started_ms: u64,
    /// Launched by the installer with `--installed`.
    pub installed: bool,
    /// Every lane open and every door served.
    pub ready: bool,
    pub startup_unlock: StartupUnlockV1,
    pub protection: ProtectionV1,
    pub lock: LockStateV1,
    pub endpoints: ResidentEndpointsV1,
    /// Absent when personal sync is off.
    pub sync: Option<SyncStatusV1>,
}

/// The status the resident keeps current and the route reads.
#[derive(Clone)]
pub struct ResidentStatusSource(Arc<RwLock<ResidentStatusV1>>);

impl ResidentStatusSource {
    pub fn new(status: ResidentStatusV1) -> Self {
        Self(Arc::new(RwLock::new(status)))
    }

    pub fn read(&self) -> ResidentStatusV1 {
        self.0.read().expect("status is never poisoned").clone()
    }

    pub fn update(&self, change: impl FnOnce(&mut ResidentStatusV1)) {
        change(&mut self.0.write().expect("status is never poisoned"));
    }
}

/// Unlocks the resident's vault by passphrase; the reason on refusal.
pub type Unlocker = Arc<dyn Fn(&[u8]) -> Result<(), String> + Send + Sync>;

/// Shows the resident's own unlock prompt and unlocks from it.
pub type NativeUnlocker = Arc<dyn Fn() -> Result<(), String> + Send + Sync>;

/// The control route's two ways in to an unlock.
#[derive(Clone, Default)]
pub struct ControlUnlock {
    /// `djinn --unlock`: a passphrase read at the caller's terminal.
    pub passphrase: Option<Unlocker>,
    /// `djinn --unlock --native`: the resident's own prompt.
    pub native: Option<NativeUnlocker>,
}

/// The stop the control route raises and the run loop waits on.
#[derive(Clone, Default)]
pub struct StopSignal(Arc<Notify>);

impl StopSignal {
    /// `notify_one` keeps a permit, so a stop before the loop waits is kept.
    pub fn raise(&self) {
        self.0.notify_one();
    }

    pub fn raised(&self) -> impl Future<Output = ()> + '_ {
        self.0.notified()
    }
}

/// The default endpoints a start would take, which only the installed
/// resident may hold (harness plan ruling 8). Empty when `installed`.
pub fn default_endpoints_taken(
    installed: bool,
    standard_agent: bool,
    (browser, default_browser): (&str, &str),
    (app, default_app): (&str, &str),
) -> Vec<&'static str> {
    if installed {
        return Vec::new();
    }
    let same = |a: &str, b: &str| {
        if cfg!(windows) {
            a.eq_ignore_ascii_case(b)
        } else {
            a == b
        }
    };
    let mut taken = Vec::new();
    if standard_agent {
        taken.push("the standard SSH agent endpoint");
    }
    if same(browser, default_browser) {
        taken.push("the default browser endpoint");
    }
    if same(app, default_app) {
        taken.push("the default app endpoint");
    }
    taken
}

/// Grant both routes to their one caller.
pub fn grant(grants: &AppRouteGrants, route: ResidentEndpointRoute) {
    grants.grant(AppId::new(RESIDENT_APP), route);
}

fn route(id: &str) -> ResidentEndpointRoute {
    ResidentEndpointRoute::new(id, Duration::from_millis(50)).expect("resident route is valid")
}

fn request(session: &str) -> ProjectionRequest {
    ProjectionRequest {
        version: ProtocolVersion::V1,
        session: ProjectionSession(session.into()),
        score: Score::new(Arrangement::Spiral(Default::default())),
    }
}

/// One card whose media is a typed JSON resource, as the device directory
/// serves its own.
fn card_snapshot(session: &str, label: &str, card_bytes: &[u8]) -> ProjectionSnapshot {
    let mut scene = Scene::new();
    let source = scene.intern_source(SourceRef::new(session, "status"));
    scene.items.push(ProjectedItem {
        source,
        space: Scene::WORLD,
        transform: Transform2::IDENTITY,
        footprint: Footprint::Rect {
            size: Size2::new(1.0, 1.0),
        },
        representation: Representation::Card,
        layer: 0,
        visible: false,
        hit: None,
        channels: Vec::new(),
    });
    let key = PresentationKey(session.into());
    let mut presentation = PresentationManifest::default();
    presentation.bindings.push(PresentationBinding {
        instance: InstanceId(0),
        key: key.clone(),
    });
    presentation.offers.insert(
        key,
        vec![PresentationOffer {
            codec: PresentationCodec::PortableCardV1,
            resource: ContentHash::of(card_bytes),
            byte_size: card_bytes.len() as u64,
            requires: PresentationCapability::PortableCard,
            semantics: PresentationSemantics {
                label: label.into(),
                role: SemanticRole::Graphic,
                bounds: BoundsRelationship::FitWithinFootprint,
                actions: Vec::new(),
            },
        }],
    );
    ProjectionSnapshot {
        version: ProtocolVersion::V1,
        session: ProjectionSession(session.into()),
        scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene)
            .expect("a one-item scene is dense"),
        presentation,
        cache_policy: CachePolicy::default(),
    }
}

/// The status route's endpoint. A snapshot keeps its bytes, so the
/// resources it names stay servable while the status moves.
pub struct ResidentStatusEndpoint {
    source: ResidentStatusSource,
    current: Option<(Vec<u8>, Vec<u8>)>,
}

impl ResidentStatusEndpoint {
    pub fn new(source: ResidentStatusSource) -> Self {
        Self {
            source,
            current: None,
        }
    }

    pub fn register(
        source: ResidentStatusSource,
        catalog: &mut ResidentEndpointCatalog,
    ) -> Result<ResidentEndpointRoute, ResidentEndpointCatalogError> {
        catalog.register(RESIDENT_STATUS_ROUTE, "Resident status", move |_| {
            Ok(Self::new(source.clone()))
        })?;
        Ok(route(RESIDENT_STATUS_ROUTE))
    }
}

impl ProjectionCatalog for ResidentStatusEndpoint {
    fn describe(&self) -> EndpointDescriptor {
        EndpointDescriptor {
            label: "Djinn resident status".into(),
            projections: vec![ProjectionOffer {
                label: "Resident status".into(),
                request: request(STATUS_SESSION),
            }],
        }
    }
}

impl ProjectionSource for ResidentStatusEndpoint {
    type Error = String;

    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, Self::Error> {
        if request.session.0 != STATUS_SESSION {
            return Err("resident-status snapshot names another session".into());
        }
        let status = self.source.read();
        let status_bytes = serde_json::to_vec(&status).map_err(|error| error.to_string())?;
        let value = |label: &str, value: String| CardValueV1 {
            label: label.into(),
            value,
        };
        let card = PortableCardV1 {
            title: "Resident".into(),
            values: vec![
                value("ready", status.ready.to_string()),
                value("lock", format!("{:?}", status.lock)),
                value("unlocked by", format!("{:?}", status.startup_unlock)),
                value("agent", status.endpoints.agent.clone()),
            ],
            badges: Vec::new(),
            media: vec![ContentHash::of(&status_bytes)],
        };
        let card_bytes = serde_json::to_vec(&card).map_err(|error| error.to_string())?;
        let snapshot = card_snapshot(STATUS_SESSION, "Resident status", &card_bytes);
        self.current = Some((status_bytes, card_bytes));
        Ok(snapshot)
    }
}

impl PresentationSource for ResidentStatusEndpoint {
    type Error = String;

    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, Self::Error> {
        if request.session.0 != STATUS_SESSION {
            return Err("resident-status resource names another session".into());
        }
        let Some((status, card)) = self.current.as_ref() else {
            return Err("request a resident-status snapshot first".into());
        };
        let bytes = [card, status]
            .into_iter()
            .find(|bytes| ContentHash::of(bytes) == request.resource)
            .ok_or("that resource is not in the current resident-status snapshot")?
            .clone();
        Ok(ResourceResponse {
            session: request.session,
            resource: request.resource,
            bytes,
        })
    }
}

impl IntentSink for ResidentStatusEndpoint {
    type Error = String;

    fn invoke(&mut self, _: IntentInvocation) -> Result<IntentResult, Self::Error> {
        Ok(IntentResult::Rejected {
            reason: "the resident status is read-only; stop is on resident-control-v1".into(),
        })
    }
}

/// The control route's endpoint: stop, and unlock when given an unlocker.
pub struct ResidentControlEndpoint {
    stop: StopSignal,
    unlock: ControlUnlock,
}

impl ResidentControlEndpoint {
    pub fn new(stop: StopSignal) -> Self {
        Self {
            stop,
            unlock: ControlUnlock::default(),
        }
    }

    pub fn register(
        stop: StopSignal,
        catalog: &mut ResidentEndpointCatalog,
    ) -> Result<ResidentEndpointRoute, ResidentEndpointCatalogError> {
        Self::register_with_unlock(stop, ControlUnlock::default(), catalog)
    }

    /// Register the route with its unlock intents served by `unlock`.
    pub fn register_with_unlock(
        stop: StopSignal,
        unlock: ControlUnlock,
        catalog: &mut ResidentEndpointCatalog,
    ) -> Result<ResidentEndpointRoute, ResidentEndpointCatalogError> {
        catalog.register(RESIDENT_CONTROL_ROUTE, "Resident control", move |_| {
            Ok(Self {
                stop: stop.clone(),
                unlock: unlock.clone(),
            })
        })?;
        Ok(route(RESIDENT_CONTROL_ROUTE))
    }
}

impl ProjectionCatalog for ResidentControlEndpoint {
    fn describe(&self) -> EndpointDescriptor {
        EndpointDescriptor {
            label: "Djinn resident control".into(),
            projections: vec![ProjectionOffer {
                label: "Resident control".into(),
                request: request(CONTROL_SESSION),
            }],
        }
    }
}

impl ProjectionSource for ResidentControlEndpoint {
    type Error = String;

    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, Self::Error> {
        if request.session.0 != CONTROL_SESSION {
            return Err("resident-control snapshot names another session".into());
        }
        Ok(card_snapshot(CONTROL_SESSION, "Resident control", b"{}"))
    }
}

impl PresentationSource for ResidentControlEndpoint {
    type Error = String;

    fn resource(&mut self, _: ResourceRequest) -> Result<ResourceResponse, Self::Error> {
        Err("resident control serves no resources".into())
    }
}

impl IntentSink for ResidentControlEndpoint {
    type Error = String;

    fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, Self::Error> {
        if intent.session.0 != CONTROL_SESSION {
            return Err("resident-control intent names another session".into());
        }
        // Owned and cleared on every path out, refused or not.
        let payload = zeroize::Zeroizing::new(intent.payload);
        match intent.intent.as_str() {
            STOP_INTENT => {
                self.stop.raise();
                Ok(IntentResult::Accepted)
            },
            UNLOCK_INTENT => Ok(match &self.unlock.passphrase {
                Some(unlock) => answer(unlock(&payload)),
                None => refused("a passphrase unlock"),
            }),
            UNLOCK_NATIVE_INTENT => Ok(match &self.unlock.native {
                Some(unlock) if payload.is_empty() => answer(unlock()),
                Some(_) => IntentResult::Rejected {
                    reason: "the native unlock carries nothing".into(),
                },
                None => refused("a native unlock"),
            }),
            _ => Ok(IntentResult::Rejected {
                reason: format!(
                    "resident control takes {STOP_INTENT}, {UNLOCK_INTENT} or {UNLOCK_NATIVE_INTENT}"
                ),
            }),
        }
    }
}

fn answer(result: Result<(), String>) -> IntentResult {
    match result {
        Ok(()) => IntentResult::Accepted,
        Err(reason) => IntentResult::Rejected { reason },
    }
}

fn refused(what: &str) -> IntentResult {
    IntentResult::Rejected {
        reason: format!("this resident takes no {what} on its control route"),
    }
}

/// Read the status through an open `resident-status-v1` route.
pub async fn read_status(client: &mut AppBrokerClient) -> Result<ResidentStatusV1, AppClientError> {
    client.open_session().await?;
    let snapshot = client.snapshot(request(STATUS_SESSION)).await?;
    let offer = snapshot
        .presentation
        .offers
        .values()
        .flatten()
        .next()
        .ok_or_else(|| AppClientError::Refused("the status snapshot has no card".into()))?;
    let card: PortableCardV1 = serde_json::from_slice(
        &client
            .resource(snapshot.session.clone(), offer.resource)
            .await?,
    )?;
    let resource = *card
        .media
        .first()
        .ok_or_else(|| AppClientError::Refused("the status card has no typed resource".into()))?;
    Ok(serde_json::from_slice(
        &client.resource(snapshot.session, resource).await?,
    )?)
}

/// Ask the resident to stop through an open `resident-control-v1` route.
pub async fn request_stop(client: &mut AppBrokerClient) -> Result<(), AppClientError> {
    control_intent(client, STOP_INTENT, Vec::new(), "stop").await
}

/// Unlock the resident by passphrase through an open `resident-control-v1`
/// route: what `djinn --unlock` sends once the terminal has the passphrase.
pub async fn request_unlock(
    client: &mut AppBrokerClient,
    passphrase: &[u8],
) -> Result<(), AppClientError> {
    control_intent(client, UNLOCK_INTENT, passphrase.to_vec(), "unlock").await
}

/// Ask the resident to show its own unlock prompt: nothing crosses the pipe.
pub async fn request_native_unlock(client: &mut AppBrokerClient) -> Result<(), AppClientError> {
    control_intent(client, UNLOCK_NATIVE_INTENT, Vec::new(), "the native unlock").await
}

async fn control_intent(
    client: &mut AppBrokerClient,
    intent: &str,
    payload: Vec<u8>,
    what: &str,
) -> Result<(), AppClientError> {
    client.open_session().await?;
    let invocation = IntentInvocation {
        session: ProjectionSession(CONTROL_SESSION.into()),
        target: InstanceId(0),
        observed_epoch: SceneEpoch(1),
        observed_revision: Revision(1),
        intent: intent.into(),
        payload,
    };
    let body = client
        .request_body(CarrierRequestBody::Intent(invocation))
        .await?;
    match body {
        CarrierResponseBody::Intent(IntentResult::Accepted) => Ok(()),
        CarrierResponseBody::Intent(other) => Err(AppClientError::Refused(format!(
            "{what} was not accepted: {other:?}"
        ))),
        _ => Err(AppClientError::Refused(format!(
            "the resident answered {what} with something else"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    fn invocation(intent: &str, payload: &[u8]) -> IntentInvocation {
        IntentInvocation {
            session: ProjectionSession(CONTROL_SESSION.into()),
            target: InstanceId(0),
            observed_epoch: SceneEpoch(1),
            observed_revision: Revision(1),
            intent: intent.into(),
            payload: payload.to_vec(),
        }
    }

    /// Ruling 47: `unlock-native-v1` carries nothing. A payload, a
    /// passphrase sent where none belongs, is refused before the resident's
    /// prompt runs; the empty one runs it.
    #[test]
    fn the_native_unlock_refuses_any_payload() {
        let prompts = Arc::new(AtomicUsize::new(0));
        let counted = Arc::clone(&prompts);
        let mut control = ResidentControlEndpoint {
            stop: StopSignal::default(),
            unlock: ControlUnlock {
                passphrase: None,
                native: Some(Arc::new(move || {
                    counted.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })),
            },
        };
        let refused = control
            .invoke(invocation(UNLOCK_NATIVE_INTENT, b"a passphrase"))
            .unwrap();
        assert!(
            matches!(&refused, IntentResult::Rejected { reason } if reason.contains("carries nothing")),
            "{refused:?}"
        );
        assert_eq!(prompts.load(Ordering::SeqCst), 0, "no prompt ran");
        let accepted = control.invoke(invocation(UNLOCK_NATIVE_INTENT, b"")).unwrap();
        assert_eq!(accepted, IntentResult::Accepted);
        assert_eq!(prompts.load(Ordering::SeqCst), 1);
    }
}
