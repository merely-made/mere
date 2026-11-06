// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// The session status a client may render without inferring authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Live,
    Stale,
    Disconnected,
    Expired,
    Revoked,
}

/// Open a session **after** the carrier has admitted it.
///
/// This message carries no principal, and that absence is the design. The
/// carrier's own handshake (`notochord`) is the sole admission step: it
/// binds a Personae subject, its attested signer, its delegation chain, the
/// requested action, and a nonce to *this* connection, and withholds the
/// application stream until that verifies. A principal repeated here would be
/// a claim bound to nothing — a second, weaker admission path beside the real
/// one, which is how an identity ends up trusted because it was asserted twice
/// rather than proved once.
///
/// So the session plane spans two layers: the carrier establishes **who**, and
/// this message negotiates **what we can both speak**. On a carrier that
/// cannot authenticate (loopback, stdio), there is no principal to have, and
/// `open` is refused rather than answered.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionOpen {
    pub version: ProtocolVersion,
    /// What the client can render, so an endpoint may refuse early rather than
    /// after a snapshot it cannot present.
    pub capabilities: CapabilityProfile,
}

/// An accepted session.
///
/// Carries no endpoint key either: the peer's identity is whatever the
/// carrier authenticated, and a key asserted in this frame would be
/// unverified by construction. A client pins the peer its transport proved,
/// never a field its peer chose.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionOpened {
    pub version: ProtocolVersion,
    pub descriptor: EndpointDescriptor,
    pub status: SessionStatus,
    /// When the admitting grant expires, if it is bounded. A hint for
    /// pre-emptive renewal — never a substitute for the endpoint's own check,
    /// since a grant can be revoked long before it expires.
    pub expires_at_ms: Option<u64>,
}

/// Carrier-neutral requests used by the local process proof and future
/// transports. The carrier supplies framing; these variants supply meaning.
///
/// Section 4.1's five session verbs map here as: `open` →
/// [`CarrierRequestBody::Open`], `close` → [`CarrierRequestBody::Close`],
/// `suspend` → [`CarrierRequestBody::Suspend`], and **both** `resume` and
/// `resynchronize` → [`CarrierRequestBody::Resume`]. That last pairing is not
/// a shortcut: a client that finds its epoch or base revision disagreeing
/// emits a `ResumeRequest` and the endpoint answers with diffs, an ack, or a
/// full snapshot — `graphshell-client` already names that path
/// `Resynchronize(ResumeRequest)`. [`CarrierRequestBody::Snapshot`] is the
/// projection plane's request and carries a score, so it selects a projection
/// rather than recovering a session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CarrierRequestBody {
    Discover,
    Snapshot(ProjectionRequest),
    Resource(ResourceRequest),
    /// Read part of a resource. Requesting the next chunk is what
    /// acknowledges the last one, so flow control is the client's outstanding
    /// request count and needs no separate ack frame.
    ResourceChunk(ResourceChunkRequest),
    Resume(ResumeRequest),
    Intent(IntentInvocation),
    /// Negotiate version and capabilities on an already-admitted carrier.
    Open(Box<SessionOpen>),
    /// Tear the session down; the endpoint may release its state.
    Close,
    /// Going away, but keep the session resumable.
    Suspend,
    /// A bounded, read-only read of the page text the mere has retained, for
    /// the graph session the client holds. Appended last: older peers never
    /// send it, and an endpoint that keeps no page text refuses it.
    RetainedPages(RetainedPagesRequest),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CarrierRequest {
    pub id: u64,
    pub body: CarrierRequestBody,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CarrierResponseBody {
    Descriptor(EndpointDescriptor),
    Snapshot(Box<ProjectionSnapshot>),
    Resource(ResourceResponse),
    ResourceChunk(Box<ResourceChunkResponse>),
    Resume(ResumeReply),
    Intent(IntentResult),
    Opened(Box<SessionOpened>),
    Closed,
    Suspended,
    RetainedPages(Box<RetainedPagesResponse>),
}

/// A failure reported by an endpoint without exposing its native error type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CarrierFailure {
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CarrierResponse {
    pub id: u64,
    pub body: Result<CarrierResponseBody, CarrierFailure>,
}

/// Payload-free signal that a mounted projection has advanced.
///
/// The host still resumes from its own last acknowledgement before receiving
/// any scene or presentation data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CarrierNotice {
    pub session: ProjectionSession,
    pub epoch: SceneEpoch,
    pub revision: Revision,
}

/// One line emitted by a carrier endpoint.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CarrierOutput {
    Response(CarrierResponse),
    Notice(CarrierNotice),
}

/// Why a carrier did not return an answer.
///
/// The distinction is the whole reason this is not a `String`. A host holding
/// a mounted scene has to act differently depending on which happened, and for
/// a long time it could not tell: both arrived as an opaque message, so
/// nothing could mark a session disconnected without also marking it
/// disconnected every time an endpoint declined a request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CarrierError {
    /// The endpoint answered, and its answer was a refusal.
    ///
    /// The session is intact. The peer is still there and still admitted; it
    /// declined this request, and the next one may well succeed.
    Refused(String),
    /// The exchange could not complete.
    ///
    /// The session is gone. Nothing further will be answered on it, and a host
    /// still presenting its scene as live is showing something it can no
    /// longer save to.
    Disconnected(String),
    /// The session's authority ran out: `Expired` on schedule, `Revoked` by
    /// its issuer (capability door D20). Nothing more is disclosed on it; the
    /// host marks the scene as the status says and purges per its cache
    /// policy. Any other status here is a carrier's bug.
    Lapsed(SessionStatus),
}

impl CarrierError {
    /// Whether the session behind this error is finished.
    pub fn is_disconnected(&self) -> bool {
        matches!(self, CarrierError::Disconnected(_))
    }

    /// The human-readable half, without asking which kind it was.
    pub fn message(&self) -> &str {
        match self {
            CarrierError::Refused(message) | CarrierError::Disconnected(message) => message,
            CarrierError::Lapsed(SessionStatus::Expired) => "the session's authority expired",
            CarrierError::Lapsed(_) => "the session's authority was revoked",
        }
    }

    /// The status a lapse leaves the session in, if this is one.
    pub fn lapse(&self) -> Option<SessionStatus> {
        match self {
            CarrierError::Lapsed(status) => Some(*status),
            _ => None,
        }
    }
}

impl core::fmt::Display for CarrierError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CarrierError::Refused(message) => write!(f, "the endpoint refused: {message}"),
            CarrierError::Disconnected(message) => {
                write!(f, "the endpoint is no longer reachable: {message}")
            },
            CarrierError::Lapsed(_) => write!(f, "{}", self.message()),
        }
    }
}

impl std::error::Error for CarrierError {}
