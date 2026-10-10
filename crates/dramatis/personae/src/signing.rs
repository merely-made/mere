// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Approval and receipt boundary for native signing adapters.
//!
//! The broker carries only public request facts and a payload digest. Secret
//! key bytes and cleartext payloads remain inside the adapter that performs the
//! cryptographic operation.
//!
//! The records here are plain data and build without `agent`, so the identity
//! surface's views can name them (dramatis repo plan, ruling D25). The broker
//! that holds requests open is custody, in castellan since DR-B.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::UnlockTier;

/// Facts a signing carrier can prove without disclosing the payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SigningRequest {
    /// Stable id used by the approval intent.
    pub request_id: Uuid,
    /// Selected Personae profile.
    pub profile: String,
    /// Public key fingerprint, never the private slot payload.
    pub public_key_fingerprint: String,
    /// Adapter-scoped operation name, such as `ssh.sign`.
    pub operation: String,
    /// BLAKE3 digest of the payload presented to the signer.
    pub payload_digest: String,
    /// Native adapter that produced the request.
    pub adapter: String,
    /// Wall-clock request time.
    pub requested_at_ms: u64,
    /// Authenticated requester identity, when the carrier proves one.
    pub authenticated_requester: Option<String>,
    /// Authenticated local process, when the carrier proves one.
    pub authenticated_process: Option<String>,
    /// Authenticated target, such as a verified SSH host key.
    pub authenticated_target: Option<String>,
    /// Authenticated session binding digest, when available.
    pub session_binding: Option<String>,
    /// Related public graph object or Graphshell session, when one exists.
    pub related_object: Option<String>,
}

impl SigningRequest {
    /// Build a secret-free request from an opaque payload.
    pub fn new(
        profile: impl Into<String>,
        public_key_fingerprint: impl Into<String>,
        operation: impl Into<String>,
        payload: &[u8],
        adapter: impl Into<String>,
    ) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            profile: profile.into(),
            public_key_fingerprint: public_key_fingerprint.into(),
            operation: operation.into(),
            payload_digest: format!("blake3:{}", blake3::hash(payload).to_hex()),
            adapter: adapter.into(),
            requested_at_ms: now_ms(),
            authenticated_requester: None,
            authenticated_process: None,
            authenticated_target: None,
            session_binding: None,
            related_object: None,
        }
    }

    /// Attach a target proved by the carrier.
    pub fn with_authenticated_target(mut self, target: impl Into<String>) -> Self {
        self.authenticated_target = Some(target.into());
        self
    }

    /// Attach a session binding proved by the carrier.
    pub fn with_session_binding(mut self, binding: impl Into<String>) -> Self {
        self.session_binding = Some(binding.into());
        self
    }

    /// Attach a related public graph object or session.
    pub fn with_related_object(mut self, related: impl Into<String>) -> Self {
        self.related_object = Some(related.into());
        self
    }
}

/// Approval behavior selected for one key and adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SigningPolicy {
    /// The unlocked native session may sign without another prompt.
    Session,
    /// One approval remains valid until the configured idle window expires.
    ShortTtl {
        /// Idle window in seconds.
        idle_seconds: u32,
    },
    /// Every request waits for a visible decision.
    PerUse,
}

impl From<UnlockTier> for SigningPolicy {
    fn from(tier: UnlockTier) -> Self {
        match tier {
            UnlockTier::Session => Self::Session,
            UnlockTier::ShortTtl { idle_seconds } => Self::ShortTtl { idle_seconds },
            UnlockTier::PerUse => Self::PerUse,
        }
    }
}

/// Bounded scope requested by an approval decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RememberApproval {
    /// Approve only the pending request.
    Once,
    /// Use the key's configured short idle window.
    UntilIdle,
}

/// Visible user decision for a pending signing request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SigningDecision {
    /// Approve under the stated bounded remember scope.
    Approve {
        /// How long the approval may remain useful.
        remember: RememberApproval,
    },
    /// Refuse the request.
    Deny,
}

/// Why a signing request was authorized.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalSource {
    /// The key's session policy allowed the request.
    SessionPolicy,
    /// A still-live short-TTL approval allowed the request.
    CachedShortTtl,
    /// A visible user approved only this request.
    UserOnce,
    /// A visible user approved the configured short-TTL window.
    UserUntilIdle,
}

/// One successful authorization, completed by the signing adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SigningAuthorization {
    /// Secret-free request facts.
    pub request: SigningRequest,
    /// Policy enforced for this request.
    pub policy: SigningPolicy,
    /// Source of the approval.
    pub source: ApprovalSource,
}

/// Final result retained in the signing history.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SigningRecordResult {
    /// The native adapter produced a signature.
    Signed {
        /// Public reference to the result, normally a digest.
        signature_ref: String,
    },
    /// The user denied the request.
    Denied,
    /// The visible request expired without a decision.
    TimedOut,
    /// The native adapter failed after authorization.
    Failed {
        /// Bounded failure class. Free-form adapter errors are not retained.
        code: SigningFailureCode,
    },
}

/// Secret-safe failure classes retained by signing history.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SigningFailureCode {
    /// The requester disappeared while its decision was pending.
    ApprovalChannelClosed,
    /// The native adapter could not complete the authorized operation.
    AdapterFailure,
}

/// Append-only, secret-free decision and result record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SigningRecord {
    /// Request facts and payload digest.
    pub request: SigningRequest,
    /// Policy enforced for the request.
    pub policy: SigningPolicy,
    /// Approval source when authorization succeeded.
    pub approval_source: Option<ApprovalSource>,
    /// Final result.
    pub result: SigningRecordResult,
    /// Completion time.
    pub completed_at_ms: u64,
}

/// One request available to Graphshell's approval surface.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingSigningRequest {
    /// Secret-free request facts.
    pub request: SigningRequest,
    /// Policy enforced for the request.
    pub policy: SigningPolicy,
    /// Deadline after which the request becomes a timeout record.
    pub expires_at_ms: u64,
}

/// Authorization failure returned to the native adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthorizationError {
    /// A visible user denied the request.
    Denied,
    /// No decision arrived before the configured deadline.
    TimedOut,
    /// The decision channel closed unexpectedly.
    BrokerClosed,
}

impl std::fmt::Display for AuthorizationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Denied => write!(formatter, "signing request denied"),
            Self::TimedOut => write!(formatter, "signing request timed out"),
            Self::BrokerClosed => write!(formatter, "signing approval broker closed"),
        }
    }
}

impl std::error::Error for AuthorizationError {}

/// Invalid or stale visible decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionError {
    /// The request is no longer pending.
    NotPending,
    /// The key policy does not permit the requested remember scope.
    RememberNotAllowed,
    /// The waiting adapter disappeared before receiving the decision.
    RequestClosed,
}

impl std::fmt::Display for DecisionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotPending => write!(formatter, "signing request is not pending"),
            Self::RememberNotAllowed => {
                write!(
                    formatter,
                    "the key policy does not allow that remember scope"
                )
            },
            Self::RequestClosed => write!(formatter, "signing requester is no longer waiting"),
        }
    }
}

impl std::error::Error for DecisionError {}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
