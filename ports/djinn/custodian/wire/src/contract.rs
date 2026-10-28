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
    /// Derive and release keys from the persona (`profile`, or the open one)
    /// or the wallet's identity seed. H2's general form of [`Self::Release`].
    Derive {
        source: Source,
        profile: Option<String>,
        salts: Vec<Bytes>,
    },
    /// The root's attestation of the key `salt` derives; no key crosses.
    Attest {
        source: Source,
        profile: Option<String>,
        salt: Bytes,
    },
    /// Sign with the key `salt` derives from `source`, inside custody.
    SignDerived {
        source: Source,
        profile: Option<String>,
        salt: Bytes,
        message: Bytes,
    },
    /// The public key of `source`'s root (a named persona or the wallet).
    PublicKey {
        source: Source,
        profile: Option<String>,
    },
    /// The public identity read model, answerable while locked.
    Snapshot {},
    /// The wallet's public records, answerable while locked: the persona
    /// index and the device roster. No seed, epoch or device secret is in
    /// either, and none of those files is ever opened by the caller.
    WalletPublic {},
    /// One typed identity intent (dramatis names). Signing decisions are
    /// refused: only the custodian's consent channel decides them.
    Intent { intent: String, payload: Bytes },
    /// Whether the vault holds `profile`.
    HasProfile { profile: String },
    /// Replace the passphrase; both are credentials, over the checked channel.
    ChangePassphrase { old: Secret, new: Secret },
    /// Store an OpenSSH private key the owner picked natively.
    ImportSshKey {
        key: Secret,
        passphrase: Option<Secret>,
        options: Bytes,
    },
    /// A terminal vault command run inside custody; public output only.
    VaultCommand {
        profile: String,
        command: String,
        args: Vec<String>,
    },
    /// Keys derived from a persona wallet's current private epoch.
    EpochKeys {
        persona: String,
        device_label: Option<String>,
        keys: Vec<EpochKeyRequest>,
    },
    /// Issue a sited station's RemoteAuth grant with the wallet seed.
    StationGrant {
        request: serde_json::Value,
        issuer_public_key: Bytes,
    },
    /// Revoke one delegated device in the wallet.
    RevokeDevice { device_id: String },
    /// Seal or unseal one retained payload under a persona's current epoch.
    Payload {
        profile: String,
        persona: String,
        master: Bytes,
        scope: Bytes,
        /// The payload's canonical content hash, in eidetic's text form.
        content_hash: String,
        operation: PayloadOperation,
    },
    /// The credential catalog the custodian holds.
    Credentials { operation: CredentialOperation },
}

/// Which root a derivation, attestation or signature comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Source {
    Persona,
    Wallet,
}

/// One epoch derivation: blake3's `derive_key` under `context` over the epoch
/// secret, followed by this device's public root when `device_bound`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpochKeyRequest {
    pub context: String,
    pub device_bound: bool,
    pub public_only: bool,
}

/// One retained-payload operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum PayloadOperation {
    Seal { bytes: Bytes },
    Unseal { bytes: Bytes, marker: serde_json::Value },
}

/// One credential-catalog operation. Requests and results are the
/// catalog's own public records, carried as JSON.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum CredentialOperation {
    Quarantine { profile: String },
    UnassignedImports {},
    AssignImport { request: serde_json::Value },
    Review { request: serde_json::Value },
    CxfPreview { source: Bytes },
    CxfImport {
        source: Bytes,
        destinations: serde_json::Value,
        preview_sha256: String,
    },
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
    /// The mutation is done; nothing to report.
    Done {},
    PublicKey {
        #[serde(with = "fixed")]
        key: [u8; 32],
    },
    Bool {
        value: bool,
    },
    Attestation {
        attestation: AttestationParts,
    },
    Snapshot {
        snapshot: Box<dramatis::view::IdentitySurfaceSnapshot>,
    },
    /// The wallet's public records. `personas` are the persona UUIDs that
    /// hold a wallet manifest, sorted; `roster` is the device roster as
    /// stored, absent when the wallet has none.
    WalletPublic {
        personas: Vec<String>,
        roster: Option<serde_json::Value>,
    },
    /// A public result record (intent outcome, grant, revocation, cards,
    /// receipts), carried as JSON.
    Value {
        value: serde_json::Value,
    },
    VaultOutput {
        text: String,
    },
    VaultEnrollment {
        target: String,
        script: String,
        confirmation: String,
    },
    EpochKeys {
        keys: Vec<ReleasedEpochKey>,
    },
    Payload {
        bytes: Bytes,
        marker: Option<serde_json::Value>,
    },
}

/// One released epoch key: the request it answers and the derived secret,
/// or the Ed25519 public key it seeds when the request was public-only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleasedEpochKey {
    pub request: EpochKeyRequest,
    pub key: Secret,
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
    /// The wallet root's public key, absent with no wallet or while locked.
    #[serde(with = "fixed_opt", default)]
    pub wallet_public_key: Option<[u8; 32]>,
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
    /// The active profile or master changed. Older strict version-1 decoders
    /// disconnect on this fact and must reconnect using this supplier cohort.
    ProfileChanged,
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
