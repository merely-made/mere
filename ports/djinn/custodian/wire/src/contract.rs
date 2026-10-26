// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The messages of §5.6. Every decoder refuses unknown fields, so an extra
//! `approved` or `principal` on a request is an invalid request rather than
//! an ignored hint. There is no client approval operation: decisions are
//! accepted only from a session the custodian admitted as its trusted
//! consent channel, and only for the exact petition digest, request and
//! generation they name.

use serde::{Deserialize, Serialize};

use crate::secret::{Bytes, Secret, fixed, fixed_opt};

/// Client to custodian.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ClientMessage {
    /// The first frame. `nonce` correlates this session's replies; it is not
    /// an identity and grants nothing.
    Hello { version: u32, nonce: String },
    /// One typed operation, bound to the session's instance and the
    /// generation the client last observed.
    Request {
        id: u64,
        instance: String,
        generation: u64,
        operation: Operation,
    },
    /// A consent decision. Honoured only from the trusted consent channel.
    Decide {
        request: u64,
        digest: String,
        generation: u64,
        accept: bool,
    },
}

/// The operations a principal may be granted. Profile scope is requested
/// explicitly and never implied by the connection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Operation {
    /// Public status, answerable while locked. Empty braces, not a unit
    /// variant: serde ignores extra fields on internally tagged unit variants.
    Status {},
    /// Sign inside custody. `salt` names the derived key; an empty salt with
    /// [`Purpose::Root`] is the profile master itself.
    Sign {
        profile: String,
        purpose: Purpose,
        salt: Bytes,
        message: Bytes,
    },
    /// Release the reviewed derived keys for `salts`, each with the master's
    /// attestation. Never the master.
    Release { profile: String, salts: Vec<Bytes> },
    /// Lock now. Admission closes before the reply.
    Lock {},
    /// The documented unlock exception (§5.3): a passphrase over the
    /// mutually authenticated channel, accepted only from a principal granted
    /// unlock. The passphrase is itself the credential.
    Unlock { passphrase: Secret },
    /// Withdraw a named principal's grants until the custodian restarts.
    Revoke { principal: String },
}

/// What a signature is for, checked against the bytes signed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "purpose", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Purpose {
    /// A delegation certificate or revocation in `audience`'s domain, signed
    /// by that domain's scope key (insigne's delegation statements).
    Delegation { audience: String },
    /// The profile master signs `message` directly.
    Root {},
}

/// Custodian to client.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ServerMessage {
    /// The session is admitted.
    Opened(Session),
    /// The hello was refused; the connection closes.
    Refused { code: String },
    /// The outcome of one request.
    Outcome {
        request: u64,
        generation: u64,
        outcome: Outcome,
    },
    /// A monotonic lock, unlock, revocation or restart fact.
    Fact(Fact),
    /// A consent petition, sent only to the trusted consent channel.
    Petition(Petition),
    /// Whether a consent decision was recorded.
    Decided {
        request: u64,
        recorded: bool,
        code: Option<String>,
    },
}

/// An admitted session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    /// Fresh per custodian start.
    pub instance: String,
    pub generation: u64,
    pub locked: bool,
    /// The principal's policy name and the OS fact it was matched on.
    pub principal: String,
    pub peer: crate::peer::Peer,
    pub scopes: Vec<Scope>,
    pub consent_channel: bool,
    pub limits: Limits,
    /// The hello's nonce, echoed.
    pub nonce: String,
}

/// One granted scope, as the session reports it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub profile: String,
    pub operation: String,
    pub namespace: String,
    pub consent: String,
}

/// Finite session limits.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub max_frame_bytes: u32,
    pub max_queued_operations: u32,
    pub noninteractive_ms: u64,
    pub consent_ms: u64,
}

/// A typed outcome. A client that loses its connection reports
/// [`Outcome::Unknown`]: an ambiguous mutation is never retried as success.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Outcome {
    Success { answer: Answer },
    Refused { code: String },
    /// Partly done: the answer says what is true.
    Degraded { code: String, answer: Option<Answer> },
    Unknown { code: String },
}

impl Outcome {
    /// A refusal with a stable code.
    pub fn refused(code: &str) -> Self {
        Self::Refused { code: code.into() }
    }

    /// An unknown outcome with a stable code.
    pub fn unknown(code: &str) -> Self {
        Self::Unknown { code: code.into() }
    }

    /// The refusal or unknown code, if any.
    pub fn code(&self) -> Option<&str> {
        match self {
            Self::Success { .. } => None,
            Self::Refused { code } | Self::Degraded { code, .. } | Self::Unknown { code } => {
                Some(code)
            },
        }
    }
}

/// What a successful operation answers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "answer", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Answer {
    Status {
        status: Status,
    },
    Signature {
        #[serde(with = "fixed")]
        public_key: [u8; 32],
        #[serde(with = "fixed")]
        signature: [u8; 64],
        /// The master's attestation of a derived signing key; absent for a
        /// root signature.
        attestation: Option<AttestationParts>,
    },
    Released {
        #[serde(with = "fixed")]
        master_public_key: [u8; 32],
        keys: Vec<ReleasedKey>,
    },
    Lock {
        report: LockReport,
    },
    Unlocked {
        generation: u64,
    },
    Revoked {
        generation: u64,
    },
}

/// Public status.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Status {
    pub instance: String,
    pub generation: u64,
    pub locked: bool,
    pub profile: Option<String>,
    #[serde(with = "fixed_opt")]
    pub master_public_key: Option<[u8; 32]>,
    /// What protects the vault, for the owner to read.
    pub protection: String,
}

/// One released derived key: the salt, its seed and the master's
/// attestation for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleasedKey {
    pub salt: Bytes,
    pub seed: Secret,
    pub attestation: AttestationParts,
}

/// A derived-key attestation as plain parts, so the wire needs no identity
/// crate. Clients rebuild insigne's attestation and check it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttestationParts {
    #[serde(with = "fixed")]
    pub master: [u8; 32],
    #[serde(with = "fixed")]
    pub derived: [u8; 32],
    pub signature: Bytes,
}

/// The three separate lock facts of §5.4.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockReport {
    pub generation: u64,
    /// The root and every grant are gone from memory.
    pub memory_locked: bool,
    /// In-flight operations finished or were refused within the drain budget.
    pub drained: bool,
    /// The persisted restart barrier was written and synced.
    pub durable: bool,
}

/// Why the generation moved.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FactCause {
    Lock,
    Unlock,
    Revoke,
}

/// A monotonic invalidation fact. Honest clients forget released keys of
/// every earlier generation on receipt; a later unlock cannot clear the fact
/// that the generation advanced.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fact {
    pub instance: String,
    pub generation: u64,
    pub cause: FactCause,
    pub locked: bool,
    pub report: Option<LockReport>,
    pub revoked: Option<String>,
}

/// A consent petition: custodian-generated digest and public summary, no
/// secret payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Petition {
    pub request: u64,
    pub digest: String,
    pub generation: u64,
    /// The authenticated principal asking, by policy name.
    pub principal: String,
    pub operation: String,
    pub profile: String,
    pub namespace: String,
    /// BLAKE3 of the bytes to be signed or the salts to be released.
    pub payload_digest: String,
    pub expires_in_ms: u64,
}
