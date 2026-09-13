// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Djinn-owned lifetime for an explicitly handed-off Gemini publication.
//!
//! Knot selects a completed immutable snapshot. Djinn copies the snapshot into
//! its scoped blob custody, records only its digest and listener policy, and
//! owns the listener afterwards. This module never opens a Knot site directory
//! or a vault, and it deliberately has no governed-hosting interpretation.

use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

use chirograph::{
    BoundsRelationship, CachePolicy, CardValueV1, ContentHash, EndpointDescriptor,
    IntentInvocation, IntentResult, PortableCardV1, PresentationBinding, PresentationCapability,
    PresentationCodec, PresentationKey, PresentationManifest, PresentationOffer,
    PresentationSemantics, ProjectionOffer, ProjectionRequest, ProjectionSession,
    ProjectionSnapshot, ProtocolVersion, ResourceRequest, ResourceResponse, SemanticRole,
};
use graphshell::native::endpoint_catalog::{
    ResidentEndpointCatalog, ResidentEndpointCatalogError, ResidentEndpointRoute,
};
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionCatalog, ProjectionSource};
use knot_site::{MAX_ENCODED_SNAPSHOT_BYTES, Publication, PublishedSnapshotV1, SiteFormat};
use sceno::{
    Arrangement, Footprint, InstanceId, ProjectedItem, Representation, Scene, Score, Size2,
    SourceRef, Transform2,
};
use scenotime::{Revision, SceneEpoch, SceneSnapshot};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use tokio::sync::watch;
use transport::{BlobHash, BlobLease, BlobScope};

use crate::resident_blobs::ResidentBlobCustody;

const DESCRIPTOR: &str = "published-site.json";
const IDENTITY: &str = "published-site-identity.json";
const RETIRED: &str = "published-site-retired.json";
const LEASE_NAMESPACE: &str = "djinn.published-site";
const MAX_CONNECTIONS: u16 = 32;
const MAX_TIMEOUT_MS: u64 = 30_000;
const MAX_DESCRIPTOR_BYTES: usize = 16 * 1024;
const MAX_IDENTITY_BYTES: usize = 16 * 1024;
pub const PUBLISHED_SITE_ROUTE: &str = "published-site-v1";
const SESSION: &str = "djinn.published-site/v1";
pub const PUBLISHED_SITE_PREPARE_INTENT: &str = "djinn.published-site.prepare";
pub const PUBLISHED_SITE_CHUNK_INTENT: &str = "djinn.published-site.chunk";
pub const PUBLISHED_SITE_COMMIT_INTENT: &str = "djinn.published-site.commit";
pub const PUBLISHED_SITE_STOP_INTENT: &str = "djinn.published-site.stop";
pub const PUBLISHED_SITE_REMOVE_INTENT: &str = "djinn.published-site.remove";
pub const MAX_PUBLISHED_SITE_UPLOAD_CHUNK_BYTES: usize = 64 * 1024;
const STATUS_RESOURCE_LABEL: &str = "djinn.published-site.status/v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeminiListenerPolicyV1 {
    /// A loopback port. Zero asks the OS for one on each start.
    pub port: u16,
    pub max_connections: u16,
    pub request_timeout_ms: u64,
    /// Serving resumes after a Djinn restart only when this is explicitly true.
    pub resume_on_restart: bool,
}

impl Default for GeminiListenerPolicyV1 {
    fn default() -> Self {
        Self {
            port: 0,
            max_connections: 8,
            request_timeout_ms: 2_000,
            resume_on_restart: false,
        }
    }
}

impl GeminiListenerPolicyV1 {
    fn validate(&self) -> Result<(), String> {
        if self.max_connections == 0 || self.max_connections > MAX_CONNECTIONS {
            return Err(format!(
                "Gemini max_connections must be 1–{MAX_CONNECTIONS}"
            ));
        }
        if self.request_timeout_ms == 0 || self.request_timeout_ms > MAX_TIMEOUT_MS {
            return Err(format!(
                "Gemini request_timeout_ms must be 1–{MAX_TIMEOUT_MS}"
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PublishedSiteDescriptorV1 {
    version: u8,
    snapshot_digest: [u8; 32],
    blob_hash: [u8; 32],
    policy: GeminiListenerPolicyV1,
    certificate_sha256: [u8; 32],
    running: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GeminiIdentityV1 {
    version: u8,
    certificate_der: Vec<u8>,
    private_key_der: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RetiredSiteCustodyV1 {
    version: u8,
    snapshot_digest: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublishedSiteStateV1 {
    Unavailable,
    Stopped {
        snapshot_digest: [u8; 32],
    },
    Serving {
        snapshot_digest: [u8; 32],
        address: SocketAddr,
        certificate_sha256: [u8; 32],
    },
}

/// Read-only status returned through the admitted endpoint's typed snapshot
/// resource. `resume_on_restart` is policy, separate from the current state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishedSiteStatusV1 {
    pub version: u8,
    pub state: PublishedSiteStateV1,
    pub resume_on_restart: bool,
    /// A descriptor was removed but its custody lease release must be retried.
    pub retained_custody: Option<[u8; 32]>,
}

struct RunningGemini {
    address: SocketAddr,
    certificate_sha256: [u8; 32],
    ready: Arc<AtomicBool>,
    shutdown: Option<watch::Sender<bool>>,
    worker: Option<thread::JoinHandle<()>>,
}

impl Drop for RunningGemini {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(true);
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Durable service state held by the one Djinn process for a profile.
pub struct PublishedSiteService {
    root: PathBuf,
    scope: BlobScope,
    custody: ResidentBlobCustody,
    descriptor: Option<PublishedSiteDescriptorV1>,
    retired: Option<RetiredSiteCustodyV1>,
    running: Option<RunningGemini>,
}

/// A framed upload is intentionally bounded below the app-broker message cap.
/// The endpoint holds incomplete bytes only in session memory and never writes
/// a descriptor until `commit` verifies the complete snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishedSitePrepareV1 {
    pub snapshot_digest: [u8; 32],
    pub total_bytes: u64,
    pub chunks: u32,
    pub policy: GeminiListenerPolicyV1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishedSiteChunkV1 {
    pub index: u32,
    pub bytes: Vec<u8>,
}

struct PendingUpload {
    prepare: PublishedSitePrepareV1,
    bytes: Vec<u8>,
    next_chunk: u32,
}

/// Endpoint opened only after the local application broker has admitted the
/// caller. `AppId` is a routing label; the broker's owner-only admission is
/// the authorization boundary for these destructive local operations.
pub struct PublishedSiteEndpoint {
    service: Arc<tokio::sync::Mutex<PublishedSiteService>>,
    pending: Option<PendingUpload>,
}

impl PublishedSiteEndpoint {
    pub fn new(service: Arc<tokio::sync::Mutex<PublishedSiteService>>) -> Self {
        Self {
            service,
            pending: None,
        }
    }

    pub fn register(
        service: Arc<tokio::sync::Mutex<PublishedSiteService>>,
        catalog: &mut ResidentEndpointCatalog,
    ) -> Result<ResidentEndpointRoute, ResidentEndpointCatalogError> {
        catalog.register(PUBLISHED_SITE_ROUTE, "Published Gemini site", move |_| {
            Ok(Self::new(Arc::clone(&service)))
        })?;
        Ok(
            ResidentEndpointRoute::new(PUBLISHED_SITE_ROUTE, Duration::from_millis(50))
                .expect("published-site route is valid"),
        )
    }

    fn session() -> ProjectionSession {
        ProjectionSession(SESSION.into())
    }

    fn check_intent(intent: &IntentInvocation) -> Result<(), IntentResult> {
        if intent.session != Self::session() || intent.target != InstanceId(0) {
            return Err(IntentResult::Rejected {
                reason: "intent names another published-site endpoint".into(),
            });
        }
        if intent.observed_epoch != SceneEpoch(1) || intent.observed_revision != Revision(1) {
            return Err(IntentResult::Stale {
                current_epoch: SceneEpoch(1),
                current_revision: Revision(1),
            });
        }
        Ok(())
    }

    fn run_async<T>(
        &self,
        operation: impl std::future::Future<Output = Result<T, String>>,
    ) -> Result<T, String> {
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(operation))
    }
}

impl PublishedSiteEndpoint {
    fn status_resources(&self) -> Result<(Vec<u8>, Vec<u8>), String> {
        let service = Arc::clone(&self.service);
        self.run_async(async move {
            let status = service.lock().await.status();
            let status_bytes = serde_json::to_vec(&status).map_err(|error| error.to_string())?;
            let status_resource = ContentHash::of(&status_bytes);
            let card = PortableCardV1 {
                title: "Published Gemini site".into(),
                values: status_card_values(&status),
                badges: status_badges(&status),
                media: vec![status_resource],
            };
            let card_bytes = serde_json::to_vec(&card).map_err(|error| error.to_string())?;
            Ok((status_bytes, card_bytes))
        })
    }

    fn request() -> ProjectionRequest {
        ProjectionRequest {
            version: ProtocolVersion::V1,
            session: Self::session(),
            score: Score::new(Arrangement::Spiral(Default::default())),
        }
    }
}

impl ProjectionCatalog for PublishedSiteEndpoint {
    fn describe(&self) -> EndpointDescriptor {
        EndpointDescriptor {
            label: "Djinn published Gemini site".into(),
            projections: vec![ProjectionOffer {
                label: "Published Gemini site status".into(),
                request: Self::request(),
            }],
        }
    }
}

impl ProjectionSource for PublishedSiteEndpoint {
    type Error = String;
    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, Self::Error> {
        if request.session != Self::session() {
            return Err("published-site snapshot names another session".into());
        }
        let (_, card_bytes) = self.status_resources()?;
        let resource = ContentHash::of(&card_bytes);
        let mut scene = Scene::new();
        let source = scene.intern_source(SourceRef::new("djinn.published-site", "status"));
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
        let key = PresentationKey(STATUS_RESOURCE_LABEL.into());
        let mut presentation = PresentationManifest::default();
        presentation.bindings.push(PresentationBinding {
            instance: InstanceId(0),
            key: key.clone(),
        });
        presentation.offers.insert(
            key,
            vec![PresentationOffer {
                codec: PresentationCodec::PortableCardV1,
                resource,
                byte_size: card_bytes.len() as u64,
                requires: PresentationCapability::PortableCard,
                semantics: PresentationSemantics {
                    label: "Published Gemini site status".into(),
                    role: SemanticRole::Graphic,
                    bounds: BoundsRelationship::FitWithinFootprint,
                    actions: Vec::new(),
                },
            }],
        );
        Ok(ProjectionSnapshot {
            version: ProtocolVersion::V1,
            session: Self::session(),
            scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene)
                .map_err(|error| format!("{error:?}"))?,
            presentation,
            cache_policy: CachePolicy::default(),
        })
    }
}

impl PresentationSource for PublishedSiteEndpoint {
    type Error = String;
    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, Self::Error> {
        if request.session != Self::session() {
            return Err("published-site resource names another session".into());
        }
        let (status_bytes, card_bytes) = self.status_resources()?;
        let bytes = if request.resource == ContentHash::of(&card_bytes) {
            card_bytes
        } else if request.resource == ContentHash::of(&status_bytes) {
            status_bytes
        } else {
            return Err(
                "published-site status resource is no longer current; request a new snapshot"
                    .into(),
            );
        };
        Ok(ResourceResponse {
            session: Self::session(),
            resource: request.resource,
            bytes,
        })
    }
}

fn status_card_values(status: &PublishedSiteStatusV1) -> Vec<CardValueV1> {
    let mut values = vec![CardValueV1 {
        label: "State".into(),
        value: match status.state {
            PublishedSiteStateV1::Unavailable => "unavailable".into(),
            PublishedSiteStateV1::Stopped { .. } => "stopped".into(),
            PublishedSiteStateV1::Serving { .. } => "serving".into(),
        },
    }];
    match &status.state {
        PublishedSiteStateV1::Unavailable => {},
        PublishedSiteStateV1::Stopped { snapshot_digest } => values.push(CardValueV1 {
            label: "Snapshot digest".into(),
            value: hex_bytes(snapshot_digest),
        }),
        PublishedSiteStateV1::Serving {
            snapshot_digest,
            address,
            certificate_sha256,
        } => {
            values.extend([
                CardValueV1 {
                    label: "Snapshot digest".into(),
                    value: hex_bytes(snapshot_digest),
                },
                CardValueV1 {
                    label: "Address".into(),
                    value: address.to_string(),
                },
                CardValueV1 {
                    label: "Certificate SHA-256".into(),
                    value: hex_bytes(certificate_sha256),
                },
            ]);
        },
    }
    values.push(CardValueV1 {
        label: "Resume on restart".into(),
        value: status.resume_on_restart.to_string(),
    });
    if let Some(digest) = status.retained_custody {
        values.push(CardValueV1 {
            label: "Retained custody cleanup".into(),
            value: hex_bytes(&digest),
        });
    }
    values
}

fn status_badges(status: &PublishedSiteStatusV1) -> Vec<String> {
    let mut badges = vec![
        match status.state {
            PublishedSiteStateV1::Unavailable => "unavailable",
            PublishedSiteStateV1::Stopped { .. } => "stopped",
            PublishedSiteStateV1::Serving { .. } => "serving",
        }
        .into(),
    ];
    if status.resume_on_restart {
        badges.push("resume-enabled".into());
    }
    if status.retained_custody.is_some() {
        badges.push("custody-cleanup-pending".into());
    }
    badges
}

fn hex_bytes(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

impl IntentSink for PublishedSiteEndpoint {
    type Error = String;

    fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, Self::Error> {
        if let Err(result) = Self::check_intent(&intent) {
            return Ok(result);
        }
        match intent.intent.as_str() {
            PUBLISHED_SITE_PREPARE_INTENT => {
                let prepare: PublishedSitePrepareV1 = serde_json::from_slice(&intent.payload)
                    .map_err(|error| format!("invalid published-site prepare: {error}"))?;
                prepare.policy.validate()?;
                if prepare.total_bytes == 0
                    || prepare.total_bytes > MAX_ENCODED_SNAPSHOT_BYTES as u64
                {
                    return Ok(IntentResult::Rejected {
                        reason: "published-site upload must be 1–24 MiB".into(),
                    });
                }
                let expected_chunks = prepare
                    .total_bytes
                    .div_ceil(MAX_PUBLISHED_SITE_UPLOAD_CHUNK_BYTES as u64)
                    as u32;
                if prepare.chunks != expected_chunks {
                    return Ok(IntentResult::Rejected {
                        reason: "published-site chunk count does not match bounded upload size"
                            .into(),
                    });
                }
                self.pending = Some(PendingUpload {
                    prepare,
                    bytes: Vec::new(),
                    next_chunk: 0,
                });
                Ok(IntentResult::Accepted)
            },
            PUBLISHED_SITE_CHUNK_INTENT => {
                let chunk: PublishedSiteChunkV1 = serde_json::from_slice(&intent.payload)
                    .map_err(|error| format!("invalid published-site chunk: {error}"))?;
                let Some(pending) = self.pending.as_mut() else {
                    return Ok(IntentResult::Rejected {
                        reason: "prepare a published-site upload first".into(),
                    });
                };
                if chunk.index != pending.next_chunk
                    || chunk.index >= pending.prepare.chunks
                    || chunk.bytes.is_empty()
                    || chunk.bytes.len() > MAX_PUBLISHED_SITE_UPLOAD_CHUNK_BYTES
                {
                    self.pending = None;
                    return Ok(IntentResult::Rejected {
                        reason: "published-site chunk is out of order or out of bounds".into(),
                    });
                }
                if pending.bytes.len().saturating_add(chunk.bytes.len())
                    > pending.prepare.total_bytes as usize
                {
                    self.pending = None;
                    return Ok(IntentResult::Rejected {
                        reason: "published-site upload exceeds its prepared length".into(),
                    });
                }
                pending.bytes.extend_from_slice(&chunk.bytes);
                pending.next_chunk += 1;
                Ok(IntentResult::Accepted)
            },
            PUBLISHED_SITE_COMMIT_INTENT => {
                let Some(pending) = self.pending.take() else {
                    return Ok(IntentResult::Rejected {
                        reason: "prepare and upload a published site before commit".into(),
                    });
                };
                if pending.next_chunk != pending.prepare.chunks
                    || pending.bytes.len() != pending.prepare.total_bytes as usize
                    || *blake3::hash(&pending.bytes).as_bytes() != pending.prepare.snapshot_digest
                {
                    return Ok(IntentResult::Rejected {
                        reason: "published-site upload is incomplete or has the wrong digest"
                            .into(),
                    });
                }
                let snapshot = match PublishedSnapshotV1::decode(&pending.bytes) {
                    Ok(snapshot) => snapshot,
                    Err(error) => {
                        return Ok(IntentResult::Rejected {
                            reason: format!("published-site snapshot is invalid: {error}"),
                        });
                    },
                };
                let service = Arc::clone(&self.service);
                match self.run_async(async move {
                    service
                        .lock()
                        .await
                        .publish(snapshot, pending.prepare.policy)
                        .await
                        .map(|_| ())
                }) {
                    Ok(()) => Ok(IntentResult::Accepted),
                    Err(error) => Ok(IntentResult::Rejected { reason: error }),
                }
            },
            PUBLISHED_SITE_STOP_INTENT => {
                let service = Arc::clone(&self.service);
                match self.run_async(async move { service.lock().await.stop().map(|_| ()) }) {
                    Ok(()) => Ok(IntentResult::Accepted),
                    Err(error) => Ok(IntentResult::Rejected { reason: error }),
                }
            },
            PUBLISHED_SITE_REMOVE_INTENT => {
                let service = Arc::clone(&self.service);
                match self.run_async(async move { service.lock().await.remove().await }) {
                    Ok(()) => Ok(IntentResult::Accepted),
                    Err(error) => Ok(IntentResult::Rejected { reason: error }),
                }
            },
            _ => Ok(IntentResult::Rejected {
                reason: "unknown published-site intent".into(),
            }),
        }
    }
}

impl PublishedSiteService {
    pub async fn open(
        data_root: &Path,
        scope: BlobScope,
        custody: ResidentBlobCustody,
    ) -> Result<Self, String> {
        let root = data_root.join("content").join("published-site");
        std::fs::create_dir_all(&root)
            .map_err(|error| format!("could not create published-site directory: {error}"))?;
        let descriptor_path = root.join(DESCRIPTOR);
        let retired_path = root.join(RETIRED);
        let retired = if retired_path.exists() {
            Some(load_json::<RetiredSiteCustodyV1>(
                &retired_path,
                MAX_DESCRIPTOR_BYTES,
            )?)
        } else {
            None
        };
        if retired.as_ref().is_some_and(|retired| retired.version != 1) {
            return Err("unsupported published-site retired custody version".into());
        }
        if !descriptor_path.exists() {
            return Ok(Self {
                root,
                scope,
                custody,
                descriptor: None,
                retired,
                running: None,
            });
        }
        let descriptor =
            load_json::<PublishedSiteDescriptorV1>(&descriptor_path, MAX_DESCRIPTOR_BYTES)?;
        validate_descriptor(&descriptor)?;
        let retired = match retired {
            Some(retired) if retired.snapshot_digest == descriptor.snapshot_digest => {
                std::fs::remove_file(&retired_path).map_err(|error| {
                    format!("could not cancel interrupted published-site removal: {error}")
                })?;
                None
            },
            retired => retired,
        };
        let mut service = Self {
            root,
            scope,
            custody,
            descriptor: Some(descriptor),
            retired,
            running: None,
        };
        service.verify_retained().await?;
        if service
            .descriptor
            .as_ref()
            .is_some_and(|record| record.running && record.policy.resume_on_restart)
        {
            service.restore().await?;
        }
        Ok(service)
    }

    pub fn state(&self) -> PublishedSiteStateV1 {
        match (&self.descriptor, &self.running) {
            (None, _) => PublishedSiteStateV1::Unavailable,
            (Some(descriptor), Some(running)) => PublishedSiteStateV1::Serving {
                snapshot_digest: descriptor.snapshot_digest,
                address: running.address,
                certificate_sha256: running.certificate_sha256,
            },
            (Some(descriptor), None) => PublishedSiteStateV1::Stopped {
                snapshot_digest: descriptor.snapshot_digest,
            },
        }
    }

    pub fn status(&self) -> PublishedSiteStatusV1 {
        PublishedSiteStatusV1 {
            version: 1,
            state: self.state(),
            resume_on_restart: self
                .descriptor
                .as_ref()
                .is_some_and(|descriptor| descriptor.policy.resume_on_restart),
            retained_custody: self.retired.as_ref().map(|retired| retired.snapshot_digest),
        }
    }

    /// Stage bytes first, then atomically install a complete descriptor. A
    /// listener starts gated and cannot disclose content until the descriptor
    /// is durable, so a failed descriptor write never leaks a new revision.
    pub async fn publish(
        &mut self,
        snapshot: PublishedSnapshotV1,
        policy: GeminiListenerPolicyV1,
    ) -> Result<PublishedSiteStateV1, String> {
        policy.validate()?;
        self.retry_retired_custody().await?;
        if snapshot.format != SiteFormat::Gemini {
            return Err("Djinn's first persistent service accepts Gemini snapshots only".into());
        }
        let bytes = snapshot.encode()?;
        if bytes.len() > MAX_ENCODED_SNAPSHOT_BYTES {
            return Err("published snapshot encoding exceeds Djinn's 24 MiB custody bound".into());
        }
        let digest = *blake3::hash(&bytes).as_bytes();
        let identity = self.identity_for_publish()?;
        let lease = self.site_lease(digest)?;
        let blob_hash = self
            .custody
            .blobs()
            .put_bytes_leased(bytes, &lease)
            .await
            .map_err(|error| format!("could not retain published snapshot: {error}"))?;
        if let Err(error) = self.custody.blobs().flush().await {
            let cleanup = self.release_staged_lease(digest).await.err();
            return Err(with_cleanup_error(
                format!("could not flush published snapshot: {error}"),
                cleanup,
            ));
        }
        let next = PublishedSiteDescriptorV1 {
            version: 1,
            snapshot_digest: digest,
            blob_hash: *blob_hash.as_bytes(),
            policy,
            certificate_sha256: identity_fingerprint(&identity)?,
            running: true,
        };
        let publication = Publication::from_snapshot_v1(snapshot)?;

        // A replacement commonly keeps its selected fixed port. Give the old
        // listener up only after bytes are durable, then restore the previous
        // durable descriptor if the new bind cannot complete.
        let had_running = self.running.is_some();
        drop(self.running.take());
        let runtime = match start_gemini(publication, &next.policy, &identity) {
            Ok(runtime) => runtime,
            Err(error) => {
                let cleanup = self.release_staged_lease(digest).await.err();
                if had_running {
                    self.restore().await?;
                }
                return Err(with_cleanup_error(error, cleanup));
            },
        };

        // Install the durable intent only after the listener was proven able
        // to bind. Then replace the previous in-memory listener.
        if let Err(error) = save_json_atomic(&self.root.join(DESCRIPTOR), &next) {
            drop(runtime);
            let cleanup = self.release_staged_lease(digest).await.err();
            if had_running {
                self.restore().await?;
            }
            return Err(with_cleanup_error(error, cleanup));
        }
        runtime.activate();
        let previous = self.descriptor.replace(next);
        let old_runtime = self.running.replace(runtime);
        drop(old_runtime);
        let retired_cleanup = match previous {
            Some(previous) if previous.snapshot_digest != digest => self
                .release_staged_lease(previous.snapshot_digest)
                .await
                .err(),
            _ => None,
        };
        let state = self.state();
        match retired_cleanup {
            Some(error) => Err(format!(
                "published-site is serving its new snapshot but prior custody cleanup is pending: {error}"
            )),
            None => Ok(state),
        }
    }

    pub fn stop(&mut self) -> Result<PublishedSiteStateV1, String> {
        let Some(descriptor) = self.descriptor.as_ref() else {
            return Err("no published Gemini snapshot is configured".into());
        };
        let mut next = descriptor.clone();
        next.running = false;
        save_json_atomic(&self.root.join(DESCRIPTOR), &next)?;
        self.descriptor = Some(next);
        drop(self.running.take());
        Ok(self.state())
    }

    pub async fn remove(&mut self) -> Result<(), String> {
        self.retry_retired_custody().await?;
        if self.descriptor.is_none() {
            return self.retry_retired_custody().await;
        }
        let descriptor = self.descriptor.as_ref().expect("checked above").clone();
        let retired = RetiredSiteCustodyV1 {
            version: 1,
            snapshot_digest: descriptor.snapshot_digest,
        };
        let retired_path = self.root.join(RETIRED);
        save_json_atomic(&retired_path, &retired)?;
        let descriptor_path = self.root.join(DESCRIPTOR);
        if let Err(error) = std::fs::remove_file(&descriptor_path) {
            let _ = std::fs::remove_file(&retired_path);
            return Err(format!(
                "could not remove published-site descriptor: {error}"
            ));
        }
        self.descriptor = None;
        self.retired = Some(retired);
        drop(self.running.take());
        self.retry_retired_custody().await
    }

    async fn retry_retired_custody(&mut self) -> Result<(), String> {
        let Some(retired) = self.retired.as_ref().cloned() else {
            return Ok(());
        };
        if self
            .descriptor
            .as_ref()
            .is_some_and(|descriptor| descriptor.snapshot_digest == retired.snapshot_digest)
        {
            return Err("published-site retained marker still names the active descriptor".into());
        }
        self.custody
            .blobs()
            .release_lease(&self.site_lease(retired.snapshot_digest)?)
            .await
            .map_err(|error| format!("published-site descriptor was removed; retained custody cleanup is pending: {error}"))?;
        let retired_path = self.root.join(RETIRED);
        std::fs::remove_file(&retired_path).map_err(|error| {
            format!("could not clear published-site retired custody record: {error}")
        })?;
        self.retired = None;
        Ok(())
    }

    fn site_lease(&self, digest: [u8; 32]) -> Result<BlobLease, String> {
        BlobLease::new(self.scope, LEASE_NAMESPACE, &digest)
            .map_err(|error| format!("could not name published-site custody: {error}"))
    }

    async fn release_staged_lease(&mut self, digest: [u8; 32]) -> Result<(), String> {
        if self
            .descriptor
            .as_ref()
            .is_some_and(|current| current.snapshot_digest == digest)
        {
            return Ok(());
        }
        if let Err(error) = self
            .custody
            .blobs()
            .release_lease(&self.site_lease(digest)?)
            .await
        {
            let retired = RetiredSiteCustodyV1 {
                version: 1,
                snapshot_digest: digest,
            };
            save_json_atomic(&self.root.join(RETIRED), &retired)?;
            self.retired = Some(retired);
            return Err(format!(
                "published-site staged custody cleanup is pending: {error}"
            ));
        }
        Ok(())
    }

    pub fn shutdown(mut self) {
        drop(self.running.take());
    }

    async fn verify_retained(&self) -> Result<(), String> {
        let descriptor = self
            .descriptor
            .as_ref()
            .expect("checked before verification");
        let bytes = self
            .custody
            .blobs()
            .get_bytes(BlobHash::from_bytes(descriptor.blob_hash))
            .await
            .map_err(|error| format!("published-site snapshot is unavailable: {error}"))?;
        if *blake3::hash(&bytes).as_bytes() != descriptor.snapshot_digest {
            return Err("published-site snapshot digest does not match its descriptor".into());
        }
        let snapshot = PublishedSnapshotV1::decode(&bytes)?;
        if snapshot.format != SiteFormat::Gemini {
            return Err("published-site descriptor does not name a Gemini snapshot".into());
        }
        let identity = self.load_identity()?;
        if identity_fingerprint(&identity)? != descriptor.certificate_sha256 {
            return Err("published-site Gemini identity does not match its descriptor".into());
        }
        Ok(())
    }

    async fn restore(&mut self) -> Result<(), String> {
        let descriptor = self.descriptor.as_ref().expect("checked before restore");
        let bytes = self
            .custody
            .blobs()
            .get_bytes(BlobHash::from_bytes(descriptor.blob_hash))
            .await
            .map_err(|error| format!("published-site snapshot is unavailable: {error}"))?;
        if *blake3::hash(&bytes).as_bytes() != descriptor.snapshot_digest {
            return Err("published-site snapshot digest does not match its descriptor".into());
        }
        let snapshot = PublishedSnapshotV1::decode(&bytes)?;
        if snapshot.format != SiteFormat::Gemini {
            return Err("published-site descriptor does not name a Gemini snapshot".into());
        }
        let identity = self.load_identity()?;
        if identity_fingerprint(&identity)? != descriptor.certificate_sha256 {
            return Err("published-site Gemini identity does not match its descriptor".into());
        }
        let runtime = start_gemini(
            Publication::from_snapshot_v1(snapshot)?,
            &descriptor.policy,
            &identity,
        )?;
        runtime.activate();
        self.running = Some(runtime);
        Ok(())
    }

    fn identity_for_publish(&self) -> Result<GeminiIdentityV1, String> {
        let path = self.root.join(IDENTITY);
        if path.exists() {
            let identity = self.load_identity()?;
            if let Some(descriptor) = self.descriptor.as_ref()
                && identity_fingerprint(&identity)? != descriptor.certificate_sha256
            {
                return Err("published-site Gemini identity does not match its descriptor".into());
            }
            return Ok(identity);
        }
        if self.descriptor.is_some() {
            return Err(
                "published-site identity is missing; restore requires its original key".into(),
            );
        }
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".into(), "127.0.0.1".into()])
            .map_err(|error| format!("could not create persistent Gemini identity: {error}"))?;
        let identity = GeminiIdentityV1 {
            version: 1,
            certificate_der: cert.cert.der().as_ref().to_vec(),
            private_key_der: cert.key_pair.serialize_der(),
        };
        save_json_atomic(&path, &identity)?;
        Ok(identity)
    }

    fn load_identity(&self) -> Result<GeminiIdentityV1, String> {
        let identity =
            load_json::<GeminiIdentityV1>(&self.root.join(IDENTITY), MAX_IDENTITY_BYTES)?;
        identity_fingerprint(&identity)?;
        Ok(identity)
    }
}

fn validate_descriptor(descriptor: &PublishedSiteDescriptorV1) -> Result<(), String> {
    if descriptor.version != 1 {
        return Err("unsupported published-site descriptor version".into());
    }
    descriptor.policy.validate()
}

fn with_cleanup_error(primary: String, cleanup: Option<String>) -> String {
    match cleanup {
        Some(cleanup) => format!("{primary}; {cleanup}"),
        None => primary,
    }
}

fn identity_fingerprint(identity: &GeminiIdentityV1) -> Result<[u8; 32], String> {
    if identity.version != 1
        || identity.certificate_der.is_empty()
        || identity.private_key_der.is_empty()
        || identity.certificate_der.len() > MAX_IDENTITY_BYTES
        || identity.private_key_der.len() > MAX_IDENTITY_BYTES
    {
        return Err("published-site Gemini identity is invalid".into());
    }
    Ok(sha2::Sha256::digest(&identity.certificate_der).into())
}

fn load_json<T: for<'de> Deserialize<'de>>(path: &Path, limit: usize) -> Result<T, String> {
    use std::io::Read;

    let mut bytes = Vec::with_capacity(limit.min(4096));
    std::fs::File::open(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    if bytes.len() > limit {
        return Err(format!("{} exceeds its {limit}-byte limit", path.display()));
    }
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("could not decode {}: {error}", path.display()))
}

fn save_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    let temporary = path.with_extension("tmp");
    use std::io::Write;

    let mut file = std::fs::File::create(&temporary)
        .map_err(|error| format!("could not write {}: {error}", temporary.display()))?;
    file.write_all(&bytes)
        .map_err(|error| format!("could not write {}: {error}", temporary.display()))?;
    file.sync_all()
        .map_err(|error| format!("could not sync {}: {error}", temporary.display()))?;
    drop(file);
    std::fs::rename(&temporary, path)
        .map_err(|error| format!("could not install {}: {error}", path.display()))
}

fn start_gemini(
    publication: Publication,
    policy: &GeminiListenerPolicyV1,
    identity: &GeminiIdentityV1,
) -> Result<RunningGemini, String> {
    policy.validate()?;
    let certificate_sha256 = identity_fingerprint(identity)?;
    let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, policy.port))
        .map_err(|error| format!("could not bind loopback Gemini listener: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    let address = listener.local_addr().map_err(|error| error.to_string())?;
    let key = rustls::pki_types::PrivatePkcs8KeyDer::from(identity.private_key_der.clone());
    let acceptor = gemini_protocol::server::acceptor(
        vec![rustls::pki_types::CertificateDer::from(
            identity.certificate_der.clone(),
        )],
        key.into(),
    )
    .map_err(|error| error.to_string())?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    let publication = Arc::new(RwLock::new(publication));
    let (shutdown, shutdown_rx) = watch::channel(false);
    let ready = Arc::new(AtomicBool::new(false));
    let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
    let timeout = Duration::from_millis(policy.request_timeout_ms);
    let capacity = policy.max_connections as usize;
    let worker = thread::spawn({
        let ready = Arc::clone(&ready);
        move || {
            runtime.block_on(async move {
                let listener = match tokio::net::TcpListener::from_std(listener) {
                    Ok(listener) => listener,
                    Err(error) => {
                        let _ = ready_tx.send(Err(error.to_string()));
                        return;
                    },
                };
                let _ = ready_tx.send(Ok(()));
                let permits = Arc::new(tokio::sync::Semaphore::new(capacity));
                let mut shutdown_rx = shutdown_rx;
                loop {
                    tokio::select! {
                        _ = shutdown_rx.changed() => break,
                        accepted = listener.accept() => match accepted {
                            Ok((stream, peer)) => {
                                let Ok(permit) = Arc::clone(&permits).try_acquire_owned() else {
                                    drop(stream);
                                    continue;
                                };
                                let acceptor = acceptor.clone();
                                let publication = Arc::clone(&publication);
                                let ready = Arc::clone(&ready);
                                tokio::spawn(async move {
                                    let _permit = permit;
                                    let _ = tokio::time::timeout(
                                        timeout,
                                        serve_gemini_connection(
                                            stream,
                                            peer,
                                            acceptor,
                                            publication,
                                            ready,
                                            address.port(),
                                            timeout,
                                        ),
                                    )
                                    .await;
                                });
                            },
                            Err(error) => tracing::warn!("published-site accept failed: {error}"),
                        },
                    }
                }
            });
        }
    });
    ready_rx.recv().map_err(|error| error.to_string())??;
    Ok(RunningGemini {
        address,
        certificate_sha256,
        ready,
        shutdown: Some(shutdown),
        worker: Some(worker),
    })
}

async fn serve_gemini_connection(
    stream: tokio::net::TcpStream,
    peer: SocketAddr,
    acceptor: tokio_rustls::TlsAcceptor,
    publication: Arc<RwLock<Publication>>,
    ready: Arc<AtomicBool>,
    public_port: u16,
    timeout: Duration,
) -> std::io::Result<()> {
    use tokio::io::AsyncReadExt;

    let mut tls = tokio::time::timeout(timeout, acceptor.accept(stream))
        .await
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "tls handshake"))??;
    let mut line = Vec::with_capacity(64);
    let mut byte = [0_u8; 1];
    loop {
        let count = tokio::time::timeout(timeout, tls.read(&mut byte))
            .await
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "request read"))??;
        if count == 0 || byte[0] == b'\n' {
            break;
        }
        line.push(byte[0]);
        if line.len() > 1024 {
            return write_gemini_reply(
                &mut tls,
                &gemini_protocol::server::Reply::header(59, "request exceeds 1024 bytes"),
            )
            .await;
        }
    }
    let reply = if !ready.load(Ordering::Acquire) {
        gemini_protocol::server::Reply::header(44, "Publication not committed")
    } else {
        match gemini_protocol::server::parse_request(&line) {
            Ok(url) => publication
                .read()
                .map_err(|_| ())
                .map(|snapshot| snapshot.gemini_reply(&url, public_port))
                .unwrap_or_else(|_| {
                    gemini_protocol::server::Reply::header(40, "Service state unavailable")
                }),
            Err(reason) => gemini_protocol::server::Reply::header(59, reason),
        }
    };
    let _ = peer;
    write_gemini_reply(&mut tls, &reply).await
}

async fn write_gemini_reply(
    tls: &mut tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
    reply: &gemini_protocol::server::Reply,
) -> std::io::Result<()> {
    use tokio::io::AsyncWriteExt;

    tls.write_all(format!("{} {}\r\n", reply.code, reply.meta).as_bytes())
        .await?;
    if (20..30).contains(&reply.code) {
        tls.write_all(&reply.body).await?;
    }
    tls.shutdown().await
}

impl RunningGemini {
    fn activate(&self) {
        self.ready.store(true, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::ResidentContentSettings;

    fn test_scope() -> BlobScope {
        BlobScope::new([0x51; 32])
    }

    async fn snapshot(root: &Path) -> PublishedSnapshotV1 {
        let site = knot_site::Site::create_for(&root.join("site"), SiteFormat::Gemini).unwrap();
        site.publication().unwrap().to_snapshot_v1().unwrap()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn durable_saved_snapshot_reopens_with_the_same_certificate_identity() {
        let temp = tempfile::tempdir().unwrap();
        let custody = ResidentBlobCustody::open(
            &temp.path().join("data"),
            &ResidentContentSettings::default(),
        )
        .await
        .unwrap();
        let mut service = PublishedSiteService::open(temp.path(), test_scope(), custody.clone())
            .await
            .unwrap();
        let first = service
            .publish(
                snapshot(temp.path()).await,
                GeminiListenerPolicyV1 {
                    resume_on_restart: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let PublishedSiteStateV1::Serving {
            snapshot_digest,
            certificate_sha256,
            ..
        } = first
        else {
            panic!("published snapshot was not serving")
        };
        service.shutdown();

        let mut reopened = PublishedSiteService::open(temp.path(), test_scope(), custody.clone())
            .await
            .unwrap();
        assert!(matches!(
            reopened.state(),
            PublishedSiteStateV1::Serving {
                snapshot_digest: observed,
                certificate_sha256: observed_certificate,
                ..
            } if observed == snapshot_digest && observed_certificate == certificate_sha256
        ));
        reopened.remove().await.unwrap();
        drop(reopened);
        custody.shutdown().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn missing_identity_refuses_restore_instead_of_rotating_the_certificate() {
        let temp = tempfile::tempdir().unwrap();
        let custody = ResidentBlobCustody::open(
            &temp.path().join("data"),
            &ResidentContentSettings::default(),
        )
        .await
        .unwrap();
        let mut service = PublishedSiteService::open(temp.path(), test_scope(), custody.clone())
            .await
            .unwrap();
        service
            .publish(
                snapshot(temp.path()).await,
                GeminiListenerPolicyV1 {
                    resume_on_restart: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        service.shutdown();
        std::fs::remove_file(
            temp.path()
                .join("content")
                .join("published-site")
                .join(IDENTITY),
        )
        .unwrap();
        assert!(
            PublishedSiteService::open(temp.path(), test_scope(), custody.clone())
                .await
                .is_err()
        );
        custody.shutdown().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn publication_does_not_resume_without_the_explicit_policy() {
        let temp = tempfile::tempdir().unwrap();
        let custody = ResidentBlobCustody::open(
            &temp.path().join("data"),
            &ResidentContentSettings::default(),
        )
        .await
        .unwrap();
        let mut service = PublishedSiteService::open(temp.path(), test_scope(), custody.clone())
            .await
            .unwrap();
        service
            .publish(
                snapshot(temp.path()).await,
                GeminiListenerPolicyV1::default(),
            )
            .await
            .unwrap();
        service.shutdown();
        let reopened = PublishedSiteService::open(temp.path(), test_scope(), custody.clone())
            .await
            .unwrap();
        assert!(matches!(
            reopened.state(),
            PublishedSiteStateV1::Stopped { .. }
        ));
        assert!(!reopened.status().resume_on_restart);
        reopened.shutdown();
        custody.shutdown().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn stalled_connections_consume_the_public_connection_limit() {
        let temp = tempfile::tempdir().unwrap();
        let custody = ResidentBlobCustody::open(
            &temp.path().join("data"),
            &ResidentContentSettings::default(),
        )
        .await
        .unwrap();
        let mut service = PublishedSiteService::open(temp.path(), test_scope(), custody.clone())
            .await
            .unwrap();
        let state = service
            .publish(
                snapshot(temp.path()).await,
                GeminiListenerPolicyV1 {
                    max_connections: 1,
                    request_timeout_ms: 500,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let PublishedSiteStateV1::Serving { address, .. } = state else {
            panic!("published snapshot was not serving")
        };
        let first = std::net::TcpStream::connect(address).unwrap();
        std::thread::sleep(Duration::from_millis(100));
        let mut second = std::net::TcpStream::connect(address).unwrap();
        second
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        let mut byte = [0_u8; 1];
        assert_eq!(std::io::Read::read(&mut second, &mut byte).unwrap(), 0);
        drop(first);
        service.shutdown();
        custody.shutdown().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn status_projection_advertises_a_decodable_card_and_typed_status_media() {
        let temp = tempfile::tempdir().unwrap();
        let custody = ResidentBlobCustody::open(
            &temp.path().join("data"),
            &ResidentContentSettings::default(),
        )
        .await
        .unwrap();
        let service = PublishedSiteService::open(temp.path(), test_scope(), custody.clone())
            .await
            .unwrap();
        let shared = Arc::new(tokio::sync::Mutex::new(service));
        let mut endpoint = PublishedSiteEndpoint::new(Arc::clone(&shared));
        let snapshot = endpoint.snapshot(PublishedSiteEndpoint::request()).unwrap();
        let offer = snapshot
            .presentation
            .offers
            .values()
            .flatten()
            .next()
            .unwrap();
        assert_eq!(offer.codec, PresentationCodec::PortableCardV1);
        let card_bytes = endpoint
            .resource(ResourceRequest {
                session: snapshot.session.clone(),
                resource: offer.resource,
            })
            .unwrap()
            .bytes;
        let card: PortableCardV1 = serde_json::from_slice(&card_bytes).unwrap();
        let status_resource = *card.media.first().unwrap();
        let status_bytes = endpoint
            .resource(ResourceRequest {
                session: snapshot.session,
                resource: status_resource,
            })
            .unwrap()
            .bytes;
        assert_eq!(
            serde_json::from_slice::<PublishedSiteStatusV1>(&status_bytes)
                .unwrap()
                .state,
            PublishedSiteStateV1::Unavailable
        );
        drop(endpoint);
        let service = Arc::try_unwrap(shared)
            .ok()
            .expect("endpoint dropped its only service clone")
            .into_inner();
        service.shutdown();
        custody.shutdown().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn failed_replacement_keeps_a_reopened_nonresuming_site_stopped() {
        let temp = tempfile::tempdir().unwrap();
        let custody = ResidentBlobCustody::open(
            &temp.path().join("data"),
            &ResidentContentSettings::default(),
        )
        .await
        .unwrap();
        let mut service = PublishedSiteService::open(temp.path(), test_scope(), custody.clone())
            .await
            .unwrap();
        service
            .publish(
                snapshot(temp.path()).await,
                GeminiListenerPolicyV1::default(),
            )
            .await
            .unwrap();
        service.shutdown();
        let mut reopened = PublishedSiteService::open(temp.path(), test_scope(), custody.clone())
            .await
            .unwrap();
        let blocker = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = blocker.local_addr().unwrap().port();
        let replacement_root = temp.path().join("replacement");
        std::fs::create_dir_all(&replacement_root).unwrap();
        assert!(
            reopened
                .publish(
                    snapshot(&replacement_root).await,
                    GeminiListenerPolicyV1 {
                        port,
                        ..Default::default()
                    },
                )
                .await
                .is_err()
        );
        assert!(matches!(
            reopened.state(),
            PublishedSiteStateV1::Stopped { .. }
        ));
        drop(blocker);
        reopened.remove().await.unwrap();
        drop(reopened);
        custody.shutdown().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn open_cancels_an_interrupted_removal_marker_for_the_active_descriptor() {
        let temp = tempfile::tempdir().unwrap();
        let custody = ResidentBlobCustody::open(
            &temp.path().join("data"),
            &ResidentContentSettings::default(),
        )
        .await
        .unwrap();
        let mut service = PublishedSiteService::open(temp.path(), test_scope(), custody.clone())
            .await
            .unwrap();
        let PublishedSiteStateV1::Serving {
            snapshot_digest, ..
        } = service
            .publish(
                snapshot(temp.path()).await,
                GeminiListenerPolicyV1 {
                    resume_on_restart: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap()
        else {
            panic!("published snapshot was not serving")
        };
        service.shutdown();
        let marker = temp
            .path()
            .join("content")
            .join("published-site")
            .join(RETIRED);
        save_json_atomic(
            &marker,
            &RetiredSiteCustodyV1 {
                version: 1,
                snapshot_digest,
            },
        )
        .unwrap();
        let reopened = PublishedSiteService::open(temp.path(), test_scope(), custody.clone())
            .await
            .unwrap();
        assert!(matches!(
            reopened.state(),
            PublishedSiteStateV1::Serving { .. }
        ));
        assert!(!marker.exists());
        reopened.shutdown();
        let mut final_open = PublishedSiteService::open(temp.path(), test_scope(), custody.clone())
            .await
            .unwrap();
        final_open.remove().await.unwrap();
        drop(final_open);
        custody.shutdown().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn incomplete_or_wrong_format_bytes_never_install_a_descriptor() {
        let temp = tempfile::tempdir().unwrap();
        let custody = ResidentBlobCustody::open(
            &temp.path().join("data"),
            &ResidentContentSettings::default(),
        )
        .await
        .unwrap();
        let mut service = PublishedSiteService::open(temp.path(), test_scope(), custody.clone())
            .await
            .unwrap();
        let site =
            knot_site::Site::create_for(&temp.path().join("scroll"), SiteFormat::Scroll).unwrap();
        assert!(
            service
                .publish(
                    site.publication().unwrap().to_snapshot_v1().unwrap(),
                    GeminiListenerPolicyV1::default()
                )
                .await
                .is_err()
        );
        assert_eq!(service.state(), PublishedSiteStateV1::Unavailable);
        drop(service);
        custody.shutdown().await.unwrap();
    }
}
