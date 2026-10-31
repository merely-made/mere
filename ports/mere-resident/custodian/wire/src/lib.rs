// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The custody contract (custody isolation plan, §5.6), implemented once.
//!
//! Djinn and every other admitted client reach the independently launched
//! custodian through this crate and never link castellan. The custodian
//! derives each peer's principal from the accepted OS connection
//! ([`peer`]); a hello carries a version and a correlation nonce, never an
//! identity. Sessions open with the server's fresh instance id, its lock
//! generation and the principal's granted scopes. Requests name their
//! expected instance and generation; outcomes are typed, and a disconnect
//! is [`contract::Outcome::Unknown`], never success. Lock and revocation
//! arrive as monotonic [`contract::Fact`]s.
//!
//! Frames reuse the Graphshell door's wire (a native-endian 32-bit length
//! before one JSON message, bounded at 1 MiB) without linking that door,
//! and clear every buffer they allocate ([`frame`]).

pub mod client;
pub mod contract;
pub mod frame;
pub mod peer;
pub mod secret;
pub mod transport;

pub use client::{Client, ClientError, Observed};
pub use contract::{
    Answer, AttestationParts, ClientMessage, CredentialOperation, EpochKeyRequest, Fact,
    FactCause, Limits, LockReport, Operation, Outcome, PayloadOperation, Petition, Purpose,
    ReleasedEpochKey, ReleasedKey, ServerMessage, Session, Source, Status,
};
pub use peer::{Peer, ServerExpectation, SignedIdentity};
pub use secret::{Bytes, Secret};

/// The contract version a hello names. A server refuses any other.
pub const VERSION: u32 = 1;

/// The proposed fixture limits of §5.6. Measure before choosing production
/// settings; an operation is refused rather than truncated to fit them.
pub mod limits {
    use std::time::Duration;

    /// One frame, either direction.
    pub const MAX_FRAME_BYTES: u32 = 1024 * 1024;
    /// Connections served at once.
    pub const MAX_CONNECTIONS: usize = 8;
    /// Connections one principal may hold.
    pub const MAX_CONNECTIONS_PER_PRINCIPAL: usize = 2;
    /// Operations queued or running across all sessions.
    pub const MAX_QUEUED_OPERATIONS: usize = 16;
    /// Bytes of queued operation frames across all sessions.
    pub const MAX_QUEUED_BYTES: usize = 8 * 1024 * 1024;
    /// Consent prompts shown at once.
    pub const MAX_ACTIVE_PETITIONS: usize = 1;
    /// From accept to a valid hello.
    pub const HANDSHAKE: Duration = Duration::from_secs(5);
    /// A noninteractive operation's deadline.
    pub const NONINTERACTIVE: Duration = Duration::from_secs(30);
    /// How long a consent petition stays answerable.
    pub const CONSENT: Duration = Duration::from_secs(120);
    /// How long a lock waits for in-flight operations before reporting an
    /// incomplete drain.
    pub const DRAIN: Duration = Duration::from_secs(5);
}

#[cfg(test)]
mod tests;
