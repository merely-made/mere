// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The independently launched Castellan custodian (custody isolation plan,
//! §5): the only process that opens Djinn's vault, unlocks it, locks it,
//! signs with its root or a root-derived key and releases reviewed derived
//! keys. Djinn and every other client reach it through `custody-wire` over
//! authenticated local IPC and never link castellan.
//!
//! - [`policy`]: the installed, deny-by-default principal and grant policy.
//! - [`host`]: Castellan's resident authority, wallet and credential catalog
//!   over Djinn's real vault, locked at every start.
//! - [`core`]: the transport-free state machine for §5.3 to §5.6.
//! - [`driver`]: serves the core over any `custody-wire` listener.
//! - [`consent`]: consent mechanisms the custodian runs itself.
//! - [`delegation`]: the H1 signing consumer's purpose and audience check.
//! - [`platform`]: one launcher, one peer-identity check and one confinement
//!   recipe per OS.

pub mod agent;
pub mod consent;
pub mod core;
mod credentials;
pub mod delegation;
pub mod driver;
#[cfg(feature = "fixture")]
pub mod fixture;
pub mod platform;
pub mod policy;
pub mod host;
pub mod synthetic;
pub mod wallet;

/// A refusal with a stable, secret-free code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal(pub &'static str);

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for Refusal {}

pub type Result<T> = std::result::Result<T, Refusal>;

/// A fresh random identifier.
pub(crate) fn nonce() -> Result<String> {
    let mut bytes = [0; 32];
    getrandom::fill(&mut bytes).map_err(|_| Refusal("randomness-unavailable"))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

#[cfg(test)]
mod tests;
