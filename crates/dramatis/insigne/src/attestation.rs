// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The link from a derived key to the master key that authorized it.
//!
//! A persona signs with a different derived key per protocol and scope; an
//! attestation is the master's signed word that one derived key is its own.
//! Moved here from `personae` on 2026-09-24 with its domain string and format
//! unchanged. Producing one needs the master key and stays in `personae`.

use serde::{Deserialize, Serialize};

use crate::key::TypedKey;

const ATTESTATION_VERSION: u16 = 1;
const ATTESTATION_DOMAIN: &[u8] = b"personae/derived-key-attestation/v1";

/// A master-signed statement binding one deterministically derived key to its
/// identity root and derivation salt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedKeyAttestation {
    format_version: u16,
    master: [u8; 32],
    derived: [u8; 32],
    signature: Vec<u8>,
}

impl DerivedKeyAttestation {
    /// The canonical message a master signs to attest `derived` under `salt`.
    pub fn message(master: &[u8; 32], derived: &[u8; 32], salt: &[u8]) -> Vec<u8> {
        message(ATTESTATION_VERSION, master, derived, salt)
    }

    /// Assemble an attestation from its parts, after the master has signed
    /// [`DerivedKeyAttestation::message`]. Nothing is checked here.
    pub fn from_parts(master: [u8; 32], derived: [u8; 32], signature: Vec<u8>) -> Self {
        Self {
            format_version: ATTESTATION_VERSION,
            master,
            derived,
            signature,
        }
    }

    /// The master's Ed25519 public key bytes.
    pub fn master(&self) -> &[u8; 32] {
        &self.master
    }

    /// The derived Ed25519 public key bytes.
    pub fn derived(&self) -> &[u8; 32] {
        &self.derived
    }

    /// The master key, typed.
    pub fn master_key(&self) -> TypedKey {
        TypedKey::ed25519(self.master)
    }

    /// The derived key, typed.
    pub fn derived_key(&self) -> TypedKey {
        TypedKey::ed25519(self.derived)
    }

    /// The master's signature bytes.
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }

    /// Verify the master signature and the supplied derivation salt.
    #[cfg(feature = "verify")]
    pub fn verify(&self, salt: &[u8]) -> bool {
        self.format_version == ATTESTATION_VERSION
            && crate::check::ed25519(
                &self.master,
                &self.signature,
                &message(self.format_version, &self.master, &self.derived, salt),
            )
    }
}

fn message(version: u16, master: &[u8; 32], derived: &[u8; 32], salt: &[u8]) -> Vec<u8> {
    let mut message = Vec::with_capacity(ATTESTATION_DOMAIN.len() + 2 + 8 + salt.len() + 32 + 32);
    message.extend_from_slice(ATTESTATION_DOMAIN);
    message.extend_from_slice(&version.to_le_bytes());
    message.extend_from_slice(&(salt.len() as u64).to_le_bytes());
    message.extend_from_slice(salt);
    message.extend_from_slice(master);
    message.extend_from_slice(derived);
    message
}
