// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The wire and client of the resident's blob custody route (cycle breaks P2a).
//!
//! An admitted application holds scoped leases on the resident's one blob
//! store through the app door: `bind-scope`, a staged put, `lease`, `release`,
//! the serving authorizer's `retain`, `unretain` and readers, and the legacy
//! store migration. The design, grant and revocation shape are in the domain
//! contract's "The blob door route" section.
//!
//! Calls are one intent, `blob.call`, carrying a JSON [`BlobRequest`]. The
//! intent answers `Accepted` or `Rejected`; what the call returned is read from
//! the route's snapshot, a [`BlobLedger`], by call number. Bytes out are the
//! carrier's resource chunks addressed by BLAKE3 hash, which is the blob's own
//! hash. No identifier here is a Mere type: scopes, hashes and peers are
//! lowercase hex of 32 bytes, so the wire is the same for any resident.

use chirograph::{
    CarrierRequestBody, CarrierResponseBody, ContentHash, IntentInvocation, IntentResult,
    PortableCardV1, ProjectionRequest, ProjectionSession, ResourceChunkRequest, Revision,
    SceneEpoch, MAX_RESOURCE_CHUNK_BYTES,
};
use ostiary::app_admission::{AppId, AppRouteId};
use ostiary::app_client::AppClientError;
use sceno::InstanceId;
use serde::{Deserialize, Serialize};

use crate::projection::AppBrokerClient;

/// The resident route this module speaks to.
pub const BLOB_CUSTODY_ROUTE: &str = "blob-custody-v1";
/// The one intent the route accepts.
pub const BLOB_CALL_INTENT: &str = "blob.call";
/// The most bytes one `put-part` carries (a JSON byte array inside a 1 MiB frame).
pub const MAX_PART_BYTES: usize = 64 * 1024;
/// The most one staged upload may declare.
pub const MAX_UPLOAD_BYTES: u64 = 256 * 1024 * 1024;
/// How many call outcomes the ledger keeps.
pub const LEDGER_OUTCOMES: usize = 64;
/// How many verified-read cursors one session may hold open.
pub const MAX_BAO_CURSORS: usize = 4;

/// A range of chunks (1 KiB units) of a blob, `end` absent for open to the
/// end: the wire form of iroh's `ChunkRanges`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChunkInterval {
    pub start: u64,
    pub end: Option<u64>,
}

/// One call: `call` is the caller's own number, unique within the session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlobRequest {
    pub call: u64,
    pub op: BlobOp,
}

/// What a call asks. `scope`, `peer` and `hash` are lowercase hex of 32 bytes;
/// `subject` is lowercase hex of the lease's subject bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case", deny_unknown_fields)]
pub enum BlobOp {
    /// Claim `scope` for this application if nobody has, and rebuild serving
    /// custody from its durable leases. Answers [`BlobOutcome::Count`].
    BindScope { scope: String },
    /// Declare one upload: its final lease, its length and its BLAKE3 hash.
    PutBegin {
        scope: String,
        namespace: String,
        subject: String,
        len: u64,
        hash: String,
    },
    /// The next part of the declared upload, at `offset`.
    PutPart { offset: u64, bytes: Vec<u8> },
    /// Verify the staged bytes against the declared hash and install them
    /// under the lease. Answers [`BlobOutcome::Hash`].
    PutCommit,
    /// Drop the staged upload.
    PutAbort,
    /// Add or replace a lease for bytes the store already holds.
    Lease {
        scope: String,
        namespace: String,
        subject: String,
        hash: String,
    },
    /// Release one lease. Answers [`BlobOutcome::Bool`], true if it existed.
    Release {
        scope: String,
        namespace: String,
        subject: String,
    },
    /// The hash a lease names. Answers [`BlobOutcome::MaybeHash`].
    LeaseHash {
        scope: String,
        namespace: String,
        subject: String,
    },
    /// Let the scope's readers fetch `hash`. Answers [`BlobOutcome::Bool`].
    Retain { scope: String, hash: String },
    /// Stop serving `hash` from the scope. Answers [`BlobOutcome::Bool`].
    Unretain { scope: String, hash: String },
    AllowReader { scope: String, peer: String },
    DenyReader { scope: String, peer: String },
    ReplaceReaders { scope: String, peers: Vec<String> },
    /// Whether `peer` may fetch `hash` from `scope`. Answers [`BlobOutcome::Bool`].
    Allows {
        scope: String,
        peer: String,
        hash: String,
    },
    /// Open a verified read: the iroh-blobs response bytes for `ranges` of
    /// `hash`, for `peer`, from `scope`. Refused unless the scope holds a lease
    /// on the hash and the authorizer allows `(scope, peer, hash)`. Answers
    /// [`BlobOutcome::Bao`]; the bytes are the resource it names, read in
    /// order.
    BaoOpen {
        scope: String,
        peer: String,
        hash: String,
        ranges: Vec<ChunkInterval>,
    },
    /// Drop a verified-read cursor. Answers [`BlobOutcome::Bool`].
    BaoClose { resource: String },
    /// Copy an old per-lane blob store, read-only, under verified leases
    /// named `namespace` (which must begin `legacy.`). Answers
    /// [`BlobOutcome::Migration`].
    MigrateLegacy {
        scope: String,
        namespace: String,
        source: String,
    },
}

/// What a call returned.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum BlobOutcome {
    Done,
    Count { count: u64 },
    Hash { hash: String },
    MaybeHash { hash: Option<String> },
    Bool { value: bool },
    Migration { state: MigrationState, blobs: u64 },
    /// A verified read is open: read `total_len` bytes at `resource` in order.
    Bao { resource: String, total_len: u64 },
    Refused { reason: String },
}

/// The result of considering one legacy store (the in-process
/// `LegacyBlobMigration`, by name).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MigrationState {
    SourceAbsent,
    AlreadyShared,
    AlreadyComplete,
    Copied,
}

/// One answered call.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CallOutcome {
    pub call: u64,
    pub outcome: BlobOutcome,
}

/// What a bound scope holds now.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeLedger {
    pub scope: String,
    /// Hashes retained by any lease in the scope.
    pub hashes: Vec<String>,
    /// Peers allowed to read the scope.
    pub readers: Vec<String>,
}

/// The route's one document: the application the door admitted, the outcomes
/// of its last calls and each scope it owns.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlobLedger {
    pub version: u8,
    pub app: String,
    pub outcomes: Vec<CallOutcome>,
    pub scopes: Vec<ScopeLedger>,
}

/// Lowercase hex of 32 bytes.
pub fn hex32(bytes: &[u8; 32]) -> String {
    hex(bytes)
}

/// Lowercase hex of any bytes (a lease subject).
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Parse hex of exactly 32 bytes.
pub fn parse_hex32(value: &str) -> Result<[u8; 32], String> {
    let raw = parse_hex(value)?;
    <[u8; 32]>::try_from(raw).map_err(|_| "expected 32 bytes of hex".to_string())
}

/// Parse hex of any length (a lease subject).
pub fn parse_hex(value: &str) -> Result<Vec<u8>, String> {
    if value.len() % 2 != 0 || !value.is_ascii() {
        return Err("expected an even number of hex digits".into());
    }
    (0..value.len())
        .step_by(2)
        .map(|at| {
            u8::from_str_radix(&value[at..at + 2], 16).map_err(|_| "expected hex digits".into())
        })
        .collect()
}

/// Why a client call failed.
#[derive(Debug)]
pub enum BlobDoorError {
    Door(AppClientError),
    /// The resident refused the call, with its reason.
    Refused(String),
    /// The reply was not what the call expects.
    Unexpected(String),
}

impl std::fmt::Display for BlobDoorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Door(error) => write!(f, "blob custody door: {error}"),
            Self::Refused(reason) => write!(f, "blob custody refused: {reason}"),
            Self::Unexpected(reason) => {
                write!(f, "blob custody gave an unexpected reply: {reason}")
            },
        }
    }
}

impl std::error::Error for BlobDoorError {}

impl From<AppClientError> for BlobDoorError {
    fn from(error: AppClientError) -> Self {
        Self::Door(error)
    }
}

/// An open verified read: where it is and how long it is.
#[derive(Clone, Debug)]
pub struct BaoRead {
    resource: ContentHash,
    total_len: u64,
    position: u64,
}

impl BaoRead {
    pub fn total_len(&self) -> u64 {
        self.total_len
    }
}

/// The application side: one session on the blob custody route.
pub struct BlobDoorClient {
    client: AppBrokerClient,
    request: ProjectionRequest,
    next_call: u64,
}

impl BlobDoorClient {
    /// Open the route at `endpoint` as `app`. The owner must have granted the
    /// application `blob-custody-v1`.
    pub async fn open_at(endpoint: &str, app: AppId) -> Result<Self, BlobDoorError> {
        let route = AppRouteId::new(BLOB_CUSTODY_ROUTE).expect("the blob route id is valid");
        let client = AppBrokerClient::open_route_at(endpoint, app, route).await?;
        Self::begin(client).await
    }

    /// [`Self::open_at`], carrying the application's declaration, which must
    /// name `blob-custody-v1` among its routes.
    pub async fn open_declared_at(
        endpoint: &str,
        app: AppId,
        declaration: ostiary::declaration::AppDeclaration,
    ) -> Result<Self, BlobDoorError> {
        let route = AppRouteId::new(BLOB_CUSTODY_ROUTE).expect("the blob route id is valid");
        let client =
            AppBrokerClient::open_declared_route_at(endpoint, app, route, declaration).await?;
        Self::begin(client).await
    }

    async fn begin(mut client: AppBrokerClient) -> Result<Self, BlobDoorError> {
        let opened = client.open_session().await?;
        let request = opened
            .descriptor
            .projections
            .first()
            .ok_or_else(|| BlobDoorError::Unexpected("the route offered no projection".into()))?
            .request
            .clone();
        Ok(Self {
            client,
            request,
            next_call: 1,
        })
    }

    fn session(&self) -> ProjectionSession {
        self.request.session.clone()
    }

    /// Send one call and read what it returned.
    pub async fn call(&mut self, op: BlobOp) -> Result<BlobOutcome, BlobDoorError> {
        let call = self.next_call;
        self.next_call += 1;
        let payload = serde_json::to_vec(&BlobRequest { call, op })
            .map_err(|error| BlobDoorError::Unexpected(error.to_string()))?;
        let body = self
            .client
            .request_body(CarrierRequestBody::Intent(IntentInvocation {
                session: self.session(),
                target: InstanceId(0),
                observed_epoch: SceneEpoch(1),
                observed_revision: Revision(1),
                intent: BLOB_CALL_INTENT.into(),
                payload,
            }))
            .await?;
        match body {
            CarrierResponseBody::Intent(IntentResult::Accepted) => {},
            CarrierResponseBody::Intent(IntentResult::Rejected { reason }) => {
                return Err(BlobDoorError::Refused(reason));
            },
            other => {
                return Err(BlobDoorError::Unexpected(format!(
                    "an intent answered with {other:?}"
                )));
            },
        }
        let ledger = self.ledger().await?;
        let found = ledger
            .outcomes
            .into_iter()
            .find(|answered| answered.call == call)
            .ok_or_else(|| BlobDoorError::Unexpected(format!("call {call} has no outcome")))?;
        match found.outcome {
            BlobOutcome::Refused { reason } => Err(BlobDoorError::Refused(reason)),
            outcome => Ok(outcome),
        }
    }

    /// The route's ledger now: recent outcomes and each owned scope.
    pub async fn ledger(&mut self) -> Result<BlobLedger, BlobDoorError> {
        let snapshot = self.client.snapshot(self.request.clone()).await?;
        let offer = snapshot
            .presentation
            .offers
            .values()
            .flatten()
            .next()
            .ok_or_else(|| BlobDoorError::Unexpected("the snapshot has no card".into()))?;
        let card: PortableCardV1 = serde_json::from_slice(
            &self
                .client
                .resource(snapshot.session.clone(), offer.resource)
                .await?,
        )
        .map_err(|error| BlobDoorError::Unexpected(error.to_string()))?;
        let resource = *card
            .media
            .first()
            .ok_or_else(|| BlobDoorError::Unexpected("the card names no ledger".into()))?;
        serde_json::from_slice(&self.client.resource(snapshot.session, resource).await?)
            .map_err(|error| BlobDoorError::Unexpected(error.to_string()))
    }

    pub async fn bind_scope(&mut self, scope: [u8; 32]) -> Result<u64, BlobDoorError> {
        match self
            .call(BlobOp::BindScope {
                scope: hex32(&scope),
            })
            .await?
        {
            BlobOutcome::Count { count } => Ok(count),
            other => Err(unexpected("a count", &other)),
        }
    }

    /// Put `bytes` under the lease `(scope, namespace, subject)`, in parts,
    /// and return their BLAKE3 hash. The resident verifies it against the
    /// hash declared first.
    pub async fn put(
        &mut self,
        scope: [u8; 32],
        namespace: &str,
        subject: &[u8],
        bytes: &[u8],
    ) -> Result<[u8; 32], BlobDoorError> {
        let hash = *ContentHash::of(bytes).as_bytes();
        self.expect_done(BlobOp::PutBegin {
            scope: hex32(&scope),
            namespace: namespace.into(),
            subject: hex(subject),
            len: bytes.len() as u64,
            hash: hex32(&hash),
        })
        .await?;
        let mut offset = 0u64;
        for part in bytes.chunks(MAX_PART_BYTES) {
            if let Err(error) = self
                .expect_done(BlobOp::PutPart {
                    offset,
                    bytes: part.to_vec(),
                })
                .await
            {
                let _ = self.call(BlobOp::PutAbort).await;
                return Err(error);
            }
            offset += part.len() as u64;
        }
        match self.call(BlobOp::PutCommit).await? {
            BlobOutcome::Hash { hash } => parse_hex32(&hash).map_err(BlobDoorError::Unexpected),
            other => Err(unexpected("a hash", &other)),
        }
    }

    pub async fn lease(
        &mut self,
        scope: [u8; 32],
        namespace: &str,
        subject: &[u8],
        hash: [u8; 32],
    ) -> Result<(), BlobDoorError> {
        self.expect_done(BlobOp::Lease {
            scope: hex32(&scope),
            namespace: namespace.into(),
            subject: hex(subject),
            hash: hex32(&hash),
        })
        .await
    }

    pub async fn release(
        &mut self,
        scope: [u8; 32],
        namespace: &str,
        subject: &[u8],
    ) -> Result<bool, BlobDoorError> {
        self.boolean(BlobOp::Release {
            scope: hex32(&scope),
            namespace: namespace.into(),
            subject: hex(subject),
        })
        .await
    }

    pub async fn lease_hash(
        &mut self,
        scope: [u8; 32],
        namespace: &str,
        subject: &[u8],
    ) -> Result<Option<[u8; 32]>, BlobDoorError> {
        match self
            .call(BlobOp::LeaseHash {
                scope: hex32(&scope),
                namespace: namespace.into(),
                subject: hex(subject),
            })
            .await?
        {
            BlobOutcome::MaybeHash { hash } => hash
                .map(|hash| parse_hex32(&hash).map_err(BlobDoorError::Unexpected))
                .transpose(),
            other => Err(unexpected("a lease hash", &other)),
        }
    }

    pub async fn retain(&mut self, scope: [u8; 32], hash: [u8; 32]) -> Result<bool, BlobDoorError> {
        self.boolean(BlobOp::Retain {
            scope: hex32(&scope),
            hash: hex32(&hash),
        })
        .await
    }

    pub async fn unretain(
        &mut self,
        scope: [u8; 32],
        hash: [u8; 32],
    ) -> Result<bool, BlobDoorError> {
        self.boolean(BlobOp::Unretain {
            scope: hex32(&scope),
            hash: hex32(&hash),
        })
        .await
    }

    pub async fn allow_reader(
        &mut self,
        scope: [u8; 32],
        peer: [u8; 32],
    ) -> Result<bool, BlobDoorError> {
        self.boolean(BlobOp::AllowReader {
            scope: hex32(&scope),
            peer: hex32(&peer),
        })
        .await
    }

    pub async fn deny_reader(
        &mut self,
        scope: [u8; 32],
        peer: [u8; 32],
    ) -> Result<bool, BlobDoorError> {
        self.boolean(BlobOp::DenyReader {
            scope: hex32(&scope),
            peer: hex32(&peer),
        })
        .await
    }

    pub async fn replace_readers(
        &mut self,
        scope: [u8; 32],
        peers: &[[u8; 32]],
    ) -> Result<bool, BlobDoorError> {
        self.boolean(BlobOp::ReplaceReaders {
            scope: hex32(&scope),
            peers: peers.iter().map(hex32).collect(),
        })
        .await
    }

    pub async fn allows(
        &mut self,
        scope: [u8; 32],
        peer: [u8; 32],
        hash: [u8; 32],
    ) -> Result<bool, BlobDoorError> {
        self.boolean(BlobOp::Allows {
            scope: hex32(&scope),
            peer: hex32(&peer),
            hash: hex32(&hash),
        })
        .await
    }

    pub async fn migrate_legacy(
        &mut self,
        scope: [u8; 32],
        namespace: &str,
        source: &str,
    ) -> Result<(MigrationState, u64), BlobDoorError> {
        match self
            .call(BlobOp::MigrateLegacy {
                scope: hex32(&scope),
                namespace: namespace.into(),
                source: source.into(),
            })
            .await?
        {
            BlobOutcome::Migration { state, blobs } => Ok((state, blobs)),
            other => Err(unexpected("a migration", &other)),
        }
    }

    /// Open a verified read of `ranges` of `hash` for `peer` in `scope`.
    pub async fn bao_open(
        &mut self,
        scope: [u8; 32],
        peer: [u8; 32],
        hash: [u8; 32],
        ranges: Vec<ChunkInterval>,
    ) -> Result<BaoRead, BlobDoorError> {
        match self
            .call(BlobOp::BaoOpen {
                scope: hex32(&scope),
                peer: hex32(&peer),
                hash: hex32(&hash),
                ranges,
            })
            .await?
        {
            BlobOutcome::Bao {
                resource,
                total_len,
            } => Ok(BaoRead {
                resource: ContentHash(parse_hex32(&resource).map_err(BlobDoorError::Unexpected)?),
                total_len,
                position: 0,
            }),
            other => Err(unexpected("a verified read", &other)),
        }
    }

    /// The next piece of an open verified read, or `None` when it is complete.
    /// Pieces arrive in order; the chunk's hash is checked by the carrier.
    pub async fn bao_next(&mut self, read: &mut BaoRead) -> Result<Option<Vec<u8>>, BlobDoorError> {
        if read.position >= read.total_len {
            return Ok(None);
        }
        let body = self
            .client
            .request_body(CarrierRequestBody::ResourceChunk(ResourceChunkRequest {
                session: self.session(),
                resource: read.resource,
                offset: read.position,
                length: MAX_RESOURCE_CHUNK_BYTES,
            }))
            .await?;
        let CarrierResponseBody::ResourceChunk(chunk) = body else {
            return Err(BlobDoorError::Unexpected(format!(
                "a resource chunk was answered with {body:?}"
            )));
        };
        if chunk.resource != read.resource
            || chunk.offset != read.position
            || chunk.total_len != read.total_len
        {
            return Err(BlobDoorError::Unexpected(
                "the verified read answered out of order".into(),
            ));
        }
        let bytes = chunk
            .decode()
            .ok_or_else(|| BlobDoorError::Unexpected("a chunk failed its hash".into()))?;
        if bytes.is_empty() {
            return Err(BlobDoorError::Unexpected(
                "the verified read made no progress".into(),
            ));
        }
        read.position += bytes.len() as u64;
        Ok(Some(bytes))
    }

    /// Drop an open verified read.
    pub async fn bao_close(&mut self, read: &BaoRead) -> Result<bool, BlobDoorError> {
        self.boolean(BlobOp::BaoClose {
            resource: hex32(read.resource.as_bytes()),
        })
        .await
    }

    /// Read a blob leased in a scope this application owns, whole, verified
    /// against its hash by the carrier.
    pub async fn read(&mut self, hash: [u8; 32]) -> Result<Vec<u8>, BlobDoorError> {
        Ok(self
            .client
            .resource(self.session(), ContentHash(hash))
            .await?)
    }

    pub async fn close(self) -> Result<(), BlobDoorError> {
        Ok(self.client.close().await?)
    }

    async fn expect_done(&mut self, op: BlobOp) -> Result<(), BlobDoorError> {
        match self.call(op).await? {
            BlobOutcome::Done => Ok(()),
            other => Err(unexpected("done", &other)),
        }
    }

    async fn boolean(&mut self, op: BlobOp) -> Result<bool, BlobDoorError> {
        match self.call(op).await? {
            BlobOutcome::Bool { value } => Ok(value),
            other => Err(unexpected("a boolean", &other)),
        }
    }
}

fn unexpected(wanted: &str, got: &BlobOutcome) -> BlobDoorError {
    BlobDoorError::Unexpected(format!("wanted {wanted}, got {got:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trips_and_rejects_the_wrong_width() {
        let bytes = [0xab; 32];
        assert_eq!(parse_hex32(&hex32(&bytes)).unwrap(), bytes);
        assert!(parse_hex32("abcd").is_err());
        assert!(parse_hex("abc").is_err());
        assert!(parse_hex("zz").is_err());
        assert_eq!(parse_hex(&hex(b"subject")).unwrap(), b"subject");
    }

    #[test]
    fn a_request_is_the_wire_the_domain_contract_states() {
        let request = BlobRequest {
            call: 7,
            op: BlobOp::Release {
                scope: hex32(&[1; 32]),
                namespace: "knot.evidence".into(),
                subject: hex(b"x"),
            },
        };
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["call"], 7);
        assert_eq!(json["op"]["op"], "release");
        assert_eq!(serde_json::from_value::<BlobRequest>(json).unwrap(), request);
        let unknown = serde_json::json!({"call": 1, "op": {"op": "delete-everything"}});
        assert!(serde_json::from_value::<BlobRequest>(unknown).is_err());
    }
}
