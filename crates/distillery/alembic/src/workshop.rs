// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Versioned evidence-brief records and the shared workshop read model.
//! These records carry identities and observations, never permission.

use serde::{Deserialize, Serialize};

pub type Digest = [u8; 32];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Owner {
    pub host: String,
    pub workspace: String,
    pub session: String,
}

/// An owner-routed immutable version, including the exact UTF-8 byte selector.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub owner: Owner,
    pub resource: String,
    pub version: String,
    pub content_digest: Digest,
    pub start: u64,
    pub end: u64,
    pub title: String,
}

/// Public retained bytes only in v1. A private-input adapter must supply its
/// qualified custody contract before another classification can be admitted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Passage {
    pub source: Source,
    pub text: String,
    pub text_digest: Digest,
    pub public: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Correlation {
    pub run: u64,
    pub step: u64,
    pub attempt: u64,
    pub generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Proposal {
    pub schema: u32,
    pub definition: Digest,
    pub installation: [u8; 16],
    pub run: u64,
    pub sources: Vec<Passage>,
    pub djot: String,
    pub digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Destination {
    pub owner: Owner,
    pub graph: String,
    pub revision: String,
}

/// Fixed once per proposal, before any destination effect. A retry must use
/// these same arguments, operation ID and accepting human.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Acceptance {
    pub operation: Digest,
    pub proposal: Digest,
    pub destination: Destination,
    pub human: [u8; 32],
    pub installation: [u8; 16],
    pub run: u64,
    pub proposing_subject: [u8; 32],
    pub requested_at_ms: u64,
    pub deadline_at_ms: u64,
}

/// Supplied only by the destination's durable operation ledger. A worker
/// result, an in-memory edit or an untracked handoff cannot supply this.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub operation: Digest,
    pub proposal: Digest,
    pub destination: Destination,
    pub resource: String,
    pub storage_witness: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Review {
    Generating,
    AwaitingReview,
    Rejected,
    Applying(Acceptance),
    Reconciling(Acceptance),
    Added {
        acceptance: Acceptance,
        receipt: Receipt,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Usage {
    pub decisions: u64,
    pub tool_calls: u64,
    pub tokens: u64,
    pub elapsed_ms: u64,
    pub consecutive_failures: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub decisions: u64,
    pub tool_calls: u64,
    pub tokens: u64,
    pub elapsed_ms: u64,
    pub consecutive_failures: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallationStatus {
    Active,
    Paused,
    Revoked,
    Dissolved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunStatus {
    Installed,
    Ready(Correlation),
    Awaiting(Correlation),
    Paused(Correlation),
    Reconciling(Correlation),
    Completed,
    Refused,
    Cancelled,
    Failed,
    BudgetExhausted,
}

/// Product adapters render these same actions and invoke Athanor's matching
/// operations. `accept` still goes through the human's destination gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Control {
    Run,
    Pause,
    Resume,
    Revoke,
    Dissolve,
    Accept,
    Reject,
    Reconcile,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkshopView {
    pub installation: [u8; 16],
    pub run: Option<u64>,
    pub generation: u64,
    pub lifecycle: InstallationStatus,
    pub phase: RunStatus,
    pub review: Option<Review>,
    pub proposal: Option<Proposal>,
    pub usage: Usage,
    pub limits: Option<Limits>,
    pub controls: Vec<Control>,
}
