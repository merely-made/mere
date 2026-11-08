// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The blob door route: an admitted application's scoped leases on the
//! resident's one blob store (cycle breaks P2a).
//!
//! The server side of `graphshell_door::blob_door`. The design, the grant and
//! the revocation are in the domain contract's "The blob door route" section.
//! In short: the application is named by the door's hello and bound by its
//! owner grant for the route; a scope belongs to the first application that
//! binds it (`ResidentBlobCustody::claim_scope`); every call behaves as the
//! in-process call of the same name does, with one narrowing, that the door
//! never retains a hash for serving unless the scope holds a lease on it.
//!
//! Revocation ([`BlobDoor::revoke`]) cuts live sessions and withdraws serving
//! authority; leases and ownership stay, since custody data is never deleted.

use std::collections::{BTreeSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use base64::Engine as _;
use chirograph::{
    BoundsRelationship, CachePolicy, CardValueV1, ContentHash, EndpointDescriptor,
    IntentInvocation, IntentResult, PortableCardV1, PresentationBinding, PresentationCapability,
    PresentationCodec, PresentationKey, PresentationManifest, PresentationOffer,
    PresentationSemantics, ProjectionOffer, ProjectionRequest, ProjectionSession,
    ProjectionSnapshot, ProtocolVersion, ResourceChunkRequest, ResourceChunkResponse,
    ResourceRequest, ResourceResponse, SemanticRole, MAX_RESOURCE_CHUNK_BYTES,
};
use graphshell_door::blob_door::{
    BLOB_CALL_INTENT, BLOB_CUSTODY_ROUTE, BlobLedger, BlobOp, BlobOutcome, BlobRequest,
    CallOutcome, LEDGER_OUTCOMES, MAX_PART_BYTES, MAX_UPLOAD_BYTES, MigrationState, ScopeLedger,
    hex32, parse_hex, parse_hex32,
};
use graphshell_host::endpoint_catalog::{
    ResidentEndpoint, ResidentEndpointCatalog, ResidentEndpointCatalogError,
};
use ostiary::route::ResidentEndpointRoute;
use sceno::{
    Arrangement, Footprint, InstanceId, ProjectedItem, Representation, Scene, Score, Size2,
    SourceRef, Transform2,
};
use scenotime::{Revision, SceneEpoch, SceneSnapshot};
use transport::{BlobHash, BlobLease, BlobScope};

use crate::resident_blobs::{LegacyBlobMigration, ResidentBlobCustody};

const SESSION: &str = "mere.blob-custody/v1";
const RESOURCE_LABEL: &str = "mere.blob-custody/v1";

/// The route's door: the custody it lends and the applications it has cut off.
#[derive(Clone)]
pub struct BlobDoor {
    custody: ResidentBlobCustody,
    revoked: Arc<Mutex<BTreeSet<String>>>,
}

impl BlobDoor {
    pub fn new(custody: ResidentBlobCustody) -> Self {
        Self {
            custody,
            revoked: Arc::default(),
        }
    }

    /// Register the route in the app catalog. It is served to the declared
    /// applications whose owner grant names `blob-custody-v1`.
    pub fn register(
        &self,
        catalog: &mut ResidentEndpointCatalog,
    ) -> Result<ResidentEndpointRoute, ResidentEndpointCatalogError> {
        let door = self.clone();
        catalog.register_erased(BLOB_CUSTODY_ROUTE, "Blob custody", move |context| {
            let app = context
                .application()
                .ok_or_else(|| "the blob custody route needs an admitted application".to_string())?
                .to_string();
            if door.is_revoked(&app) {
                return Err(format!("{app}'s blob custody access is revoked"));
            }
            Ok(Box::new(BlobEndpoint::new(door.clone(), app)) as Box<dyn ResidentEndpoint>)
        })?;
        Ok(ResidentEndpointRoute::new(BLOB_CUSTODY_ROUTE, Duration::from_millis(50))
            .expect("the blob custody route is valid"))
    }

    /// Cut `app` off: its live sessions refuse every call, and its serving
    /// authority is withdrawn. Leases and scope ownership stay. Returns the
    /// hashes unretained.
    pub async fn revoke(&self, app: &str) -> Result<usize, String> {
        self.revoked
            .lock()
            .map_err(|_| "revoked applications poisoned".to_string())?
            .insert(app.to_string());
        self.custody.withdraw_serving(app).await
    }

    /// Let `app` open the route again. It re-binds its scopes, which rebuilds
    /// serving custody from the durable leases.
    pub fn restore(&self, app: &str) {
        if let Ok(mut revoked) = self.revoked.lock() {
            revoked.remove(app);
        }
    }

    pub fn is_revoked(&self, app: &str) -> bool {
        self.revoked
            .lock()
            .map(|revoked| revoked.contains(app))
            .unwrap_or(true)
    }
}

struct Upload {
    scope: BlobScope,
    lease: BlobLease,
    len: u64,
    hash: BlobHash,
    bytes: Vec<u8>,
}

struct BlobEndpoint {
    door: BlobDoor,
    app: String,
    outcomes: VecDeque<CallOutcome>,
    upload: Option<Upload>,
    /// The card and ledger bytes of the last snapshot, so the resources it
    /// names stay servable while calls move the ledger.
    current: Option<(Vec<u8>, Vec<u8>)>,
}

fn run<T>(future: impl std::future::Future<Output = T>) -> T {
    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(future))
}

fn scope_of(value: &str) -> Result<BlobScope, String> {
    parse_hex32(value)
        .map(BlobScope::new)
        .map_err(|error| format!("scope: {error}"))
}

fn hash_of(value: &str) -> Result<BlobHash, String> {
    parse_hex32(value)
        .map(BlobHash::from_bytes)
        .map_err(|error| format!("hash: {error}"))
}

fn peer_of(value: &str) -> Result<[u8; 32], String> {
    parse_hex32(value).map_err(|error| format!("peer: {error}"))
}

fn lease_of(scope: BlobScope, namespace: &str, subject: &str) -> Result<BlobLease, String> {
    let subject = parse_hex(subject).map_err(|error| format!("subject: {error}"))?;
    BlobLease::new(scope, namespace, subject).map_err(|error| error.to_string())
}

impl BlobEndpoint {
    fn new(door: BlobDoor, app: String) -> Self {
        Self {
            door,
            app,
            outcomes: VecDeque::new(),
            upload: None,
            current: None,
        }
    }

    fn session() -> ProjectionSession {
        ProjectionSession(SESSION.into())
    }

    fn request() -> ProjectionRequest {
        ProjectionRequest {
            version: ProtocolVersion::V1,
            session: Self::session(),
            score: Score::new(Arrangement::Spiral(Default::default())),
        }
    }

    fn live(&self) -> Result<(), String> {
        if self.door.is_revoked(&self.app) {
            return Err(format!("{}'s blob custody access is revoked", self.app));
        }
        Ok(())
    }

    /// The scope, if this application holds it.
    fn owned(&self, value: &str) -> Result<BlobScope, String> {
        let scope = scope_of(value)?;
        match self.door.custody.scope_owner(scope) {
            Some(owner) if owner == self.app => Ok(scope),
            Some(owner) => Err(format!("scope {value} is held by {owner}, not {}", self.app)),
            None => Err(format!("scope {value} is not bound: bind-scope it first")),
        }
    }

    fn exec(&mut self, op: BlobOp) -> Result<BlobOutcome, String> {
        let custody = self.door.custody.clone();
        let blobs = custody.blobs();
        let readers = custody.authorizer();
        match op {
            BlobOp::BindScope { scope } => {
                let scope = scope_of(&scope)?;
                custody.claim_scope(scope, &self.app)?;
                let count = run(custody.bind_scope(scope))?;
                Ok(BlobOutcome::Count {
                    count: count as u64,
                })
            },
            BlobOp::PutBegin {
                scope,
                namespace,
                subject,
                len,
                hash,
            } => {
                let scope = self.owned(&scope)?;
                if self.upload.is_some() {
                    return Err("an upload is already staged: put-abort it first".into());
                }
                if len > MAX_UPLOAD_BYTES {
                    return Err(format!("an upload may declare at most {MAX_UPLOAD_BYTES} bytes"));
                }
                self.upload = Some(Upload {
                    scope,
                    lease: lease_of(scope, &namespace, &subject)?,
                    len,
                    hash: hash_of(&hash)?,
                    bytes: Vec::new(),
                });
                Ok(BlobOutcome::Done)
            },
            BlobOp::PutPart { offset, bytes } => {
                let upload = self
                    .upload
                    .as_mut()
                    .ok_or_else(|| "no upload is staged: put-begin first".to_string())?;
                if bytes.len() > MAX_PART_BYTES {
                    return Err(format!("a part may carry at most {MAX_PART_BYTES} bytes"));
                }
                if offset != upload.bytes.len() as u64 {
                    return Err(format!(
                        "the staged upload is at {}, not {offset}",
                        upload.bytes.len()
                    ));
                }
                if upload.bytes.len() as u64 + bytes.len() as u64 > upload.len {
                    return Err("the part runs past the declared length".into());
                }
                upload.bytes.extend_from_slice(&bytes);
                Ok(BlobOutcome::Done)
            },
            BlobOp::PutCommit => {
                let upload = self
                    .upload
                    .take()
                    .ok_or_else(|| "no upload is staged: put-begin first".to_string())?;
                if upload.bytes.len() as u64 != upload.len {
                    return Err(format!(
                        "the staged upload holds {} of {} declared bytes",
                        upload.bytes.len(),
                        upload.len
                    ));
                }
                if blake3::hash(&upload.bytes).as_bytes() != upload.hash.as_bytes() {
                    return Err("the staged bytes do not match the declared hash".into());
                }
                // Ownership is re-checked: the scope may have been reserved
                // by the resident while the upload was staged.
                if custody.scope_owner(upload.scope).as_deref() != Some(self.app.as_str()) {
                    return Err("the scope is no longer held by this application".into());
                }
                let hash = run(blobs.put_bytes_leased(upload.bytes, &upload.lease))
                    .map_err(|error| format!("could not store the blob: {error}"))?;
                Ok(BlobOutcome::Hash {
                    hash: hex32(hash.as_bytes()),
                })
            },
            BlobOp::PutAbort => {
                self.upload = None;
                Ok(BlobOutcome::Done)
            },
            BlobOp::Lease {
                scope,
                namespace,
                subject,
                hash,
            } => {
                let scope = self.owned(&scope)?;
                let lease = lease_of(scope, &namespace, &subject)?;
                let hash = hash_of(&hash)?;
                if !run(blobs.has(hash)).map_err(|error| error.to_string())? {
                    return Err("the store does not hold that blob".into());
                }
                run(blobs.lease(&lease, hash)).map_err(|error| error.to_string())?;
                Ok(BlobOutcome::Done)
            },
            BlobOp::Release {
                scope,
                namespace,
                subject,
            } => {
                let scope = self.owned(&scope)?;
                let lease = lease_of(scope, &namespace, &subject)?;
                let value = run(blobs.release_lease(&lease)).map_err(|error| error.to_string())?;
                Ok(BlobOutcome::Bool { value })
            },
            BlobOp::LeaseHash {
                scope,
                namespace,
                subject,
            } => {
                let scope = self.owned(&scope)?;
                let lease = lease_of(scope, &namespace, &subject)?;
                let hash = run(blobs.lease_hash(&lease)).map_err(|error| error.to_string())?;
                Ok(BlobOutcome::MaybeHash {
                    hash: hash.map(|hash| hex32(hash.as_bytes())),
                })
            },
            BlobOp::Retain { scope, hash } => {
                let scope = self.owned(&scope)?;
                let hash = hash_of(&hash)?;
                // The door never authorizes serving of a hash the scope holds
                // no lease on: otherwise one application could ask to serve
                // another's bytes by naming their hash.
                let leased = run(blobs.leased_hashes(scope)).map_err(|error| error.to_string())?;
                if !leased.contains(&hash) {
                    return Err("the scope holds no lease on that hash".into());
                }
                Ok(BlobOutcome::Bool {
                    value: readers.retain(scope, hash),
                })
            },
            BlobOp::Unretain { scope, hash } => {
                let scope = self.owned(&scope)?;
                Ok(BlobOutcome::Bool {
                    value: readers.release(scope, &hash_of(&hash)?),
                })
            },
            BlobOp::AllowReader { scope, peer } => {
                let scope = self.owned(&scope)?;
                Ok(BlobOutcome::Bool {
                    value: readers.allow_reader(scope, peer_of(&peer)?),
                })
            },
            BlobOp::DenyReader { scope, peer } => {
                let scope = self.owned(&scope)?;
                Ok(BlobOutcome::Bool {
                    value: readers.deny_reader(scope, &peer_of(&peer)?),
                })
            },
            BlobOp::ReplaceReaders { scope, peers } => {
                let scope = self.owned(&scope)?;
                let peers = peers
                    .iter()
                    .map(|peer| peer_of(peer))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(BlobOutcome::Bool {
                    value: readers.replace_readers(scope, peers),
                })
            },
            BlobOp::Allows { scope, peer, hash } => {
                let scope = self.owned(&scope)?;
                Ok(BlobOutcome::Bool {
                    value: readers.allows(scope, &peer_of(&peer)?, hash_of(&hash)?),
                })
            },
            BlobOp::MigrateLegacy {
                scope,
                namespace,
                source,
            } => {
                let scope = self.owned(&scope)?;
                if !namespace.starts_with("legacy.") {
                    return Err("a legacy migration's namespace must begin \"legacy.\"".into());
                }
                let source = std::path::PathBuf::from(source);
                if !source.is_absolute() {
                    return Err("a legacy store is named by an absolute path".into());
                }
                let migrated = run(custody.migrate_legacy_store(&source, scope, &namespace))?;
                Ok(match migrated {
                    LegacyBlobMigration::SourceAbsent => BlobOutcome::Migration {
                        state: MigrationState::SourceAbsent,
                        blobs: 0,
                    },
                    LegacyBlobMigration::AlreadyShared => BlobOutcome::Migration {
                        state: MigrationState::AlreadyShared,
                        blobs: 0,
                    },
                    LegacyBlobMigration::AlreadyComplete { blobs } => BlobOutcome::Migration {
                        state: MigrationState::AlreadyComplete,
                        blobs: blobs as u64,
                    },
                    LegacyBlobMigration::Copied { blobs } => BlobOutcome::Migration {
                        state: MigrationState::Copied,
                        blobs: blobs as u64,
                    },
                })
            },
        }
    }

    fn record(&mut self, call: u64, outcome: BlobOutcome) {
        self.outcomes.push_back(CallOutcome { call, outcome });
        while self.outcomes.len() > LEDGER_OUTCOMES {
            self.outcomes.pop_front();
        }
    }

    fn ledger(&self) -> Result<BlobLedger, String> {
        let custody = &self.door.custody;
        let blobs = custody.blobs();
        let readers = custody.authorizer();
        let mut scopes = Vec::new();
        for scope in custody.scopes_of(&self.app) {
            let mut hashes: Vec<String> = run(blobs.leased_hashes(scope))
                .map_err(|error| error.to_string())?
                .iter()
                .map(|hash| hex32(hash.as_bytes()))
                .collect();
            hashes.sort();
            scopes.push(ScopeLedger {
                scope: hex32(&scope.to_bytes()),
                hashes,
                readers: readers.readers(scope).iter().map(hex32).collect(),
            });
        }
        Ok(BlobLedger {
            version: 1,
            app: self.app.clone(),
            outcomes: self.outcomes.iter().cloned().collect(),
            scopes,
        })
    }

    fn leased_anywhere(&self, hash: BlobHash) -> Result<bool, String> {
        let blobs = self.door.custody.blobs();
        for scope in self.door.custody.scopes_of(&self.app) {
            let leased = run(blobs.leased_hashes(scope)).map_err(|error| error.to_string())?;
            if leased.contains(&hash) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl ResidentEndpoint for BlobEndpoint {
    fn is_live(&self) -> bool {
        !self.door.is_revoked(&self.app)
    }

    fn describe(&self) -> EndpointDescriptor {
        EndpointDescriptor {
            label: "Mere blob custody".into(),
            projections: vec![ProjectionOffer {
                label: "Blob custody ledger".into(),
                request: Self::request(),
            }],
        }
    }

    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, String> {
        self.live()?;
        if request.session != Self::session() {
            return Err("blob custody snapshot names another session".into());
        }
        let ledger = self.ledger()?;
        let ledger_bytes = serde_json::to_vec(&ledger).map_err(|error| error.to_string())?;
        let card = PortableCardV1 {
            title: "Blob custody".into(),
            values: ledger
                .scopes
                .iter()
                .map(|scope| CardValueV1 {
                    label: scope.scope.clone(),
                    value: format!("{} leased, {} readers", scope.hashes.len(), scope.readers.len()),
                })
                .collect(),
            badges: vec![format!("{} scopes", ledger.scopes.len())],
            media: vec![ContentHash::of(&ledger_bytes)],
        };
        let card_bytes = serde_json::to_vec(&card).map_err(|error| error.to_string())?;
        self.current = Some((ledger_bytes, card_bytes.clone()));

        let mut scene = Scene::new();
        let source = scene.intern_source(SourceRef::new("mere.blob-custody", "ledger"));
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
        let key = PresentationKey(RESOURCE_LABEL.into());
        let mut presentation = PresentationManifest::default();
        presentation.bindings.push(PresentationBinding {
            instance: InstanceId(0),
            key: key.clone(),
        });
        presentation.offers.insert(
            key,
            vec![PresentationOffer {
                codec: PresentationCodec::PortableCardV1,
                resource: ContentHash::of(&card_bytes),
                byte_size: card_bytes.len() as u64,
                requires: PresentationCapability::PortableCard,
                semantics: PresentationSemantics {
                    label: "Blob custody".into(),
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

    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, String> {
        self.live()?;
        if request.session != Self::session() {
            return Err("blob custody resource names another session".into());
        }
        if let Some((ledger, card)) = self.current.as_ref() {
            for bytes in [card, ledger] {
                if request.resource == ContentHash::of(bytes) {
                    return Ok(ResourceResponse {
                        session: Self::session(),
                        resource: request.resource,
                        bytes: bytes.clone(),
                    });
                }
            }
        }
        let hash = BlobHash::from_bytes(*request.resource.as_bytes());
        if !self.leased_anywhere(hash)? {
            return Err("that resource is neither in the ledger nor leased in a scope you hold".into());
        }
        let bytes = run(self.door.custody.blobs().get_bytes(hash))
            .map_err(|error| error.to_string())?
            .to_vec();
        Ok(ResourceResponse {
            session: Self::session(),
            resource: request.resource,
            bytes,
        })
    }

    fn resource_chunk(
        &mut self,
        request: ResourceChunkRequest,
    ) -> Result<ResourceChunkResponse, String> {
        self.live()?;
        if request.session != Self::session() {
            return Err("blob custody resource names another session".into());
        }
        if let Some((ledger, card)) = self.current.as_ref() {
            for bytes in [card, ledger] {
                if request.resource == ContentHash::of(bytes) {
                    let whole = ResourceResponse {
                        session: Self::session(),
                        resource: request.resource,
                        bytes: bytes.clone(),
                    };
                    return Ok(ResourceChunkResponse::slice(
                        &whole,
                        request.offset,
                        request.length,
                    ));
                }
            }
        }
        let hash = BlobHash::from_bytes(*request.resource.as_bytes());
        if !self.leased_anywhere(hash)? {
            return Err("that resource is neither in the ledger nor leased in a scope you hold".into());
        }
        let want = if request.length == 0 {
            MAX_RESOURCE_CHUNK_BYTES
        } else {
            request.length.min(MAX_RESOURCE_CHUNK_BYTES)
        } as usize;
        let (bytes, total) = run(self
            .door
            .custody
            .blobs()
            .read_range(hash, request.offset, want))
        .map_err(|error| error.to_string())?;
        Ok(ResourceChunkResponse {
            session: Self::session(),
            resource: request.resource,
            offset: request.offset.min(total),
            total_len: total,
            chunk: ContentHash::of(&bytes),
            bytes: base64::engine::general_purpose::STANDARD.encode(&bytes),
        })
    }

    fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, String> {
        self.live()?;
        if intent.session != Self::session() {
            return Err("blob custody intent names another session".into());
        }
        if intent.intent != BLOB_CALL_INTENT {
            return Ok(IntentResult::Rejected {
                reason: format!("the blob custody route has one intent, {BLOB_CALL_INTENT}"),
            });
        }
        let request: BlobRequest = match serde_json::from_slice(&intent.payload) {
            Ok(request) => request,
            Err(error) => {
                return Ok(IntentResult::Rejected {
                    reason: format!("the call is malformed: {error}"),
                });
            },
        };
        match self.exec(request.op) {
            Ok(outcome) => {
                self.record(request.call, outcome);
                Ok(IntentResult::Accepted)
            },
            Err(reason) => {
                self.record(
                    request.call,
                    BlobOutcome::Refused {
                        reason: reason.clone(),
                    },
                );
                Ok(IntentResult::Rejected { reason })
            },
        }
    }
}

#[cfg(test)]
mod tests;
