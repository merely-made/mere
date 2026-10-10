// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! [`IdentityProvider`] trait and reference implementations.

use crate::{DerivedKeypair, Ed25519Keypair, Ed25519PublicKey, IdentityError};

use insigne::DerivedKeyAttestation;

/// Read an attestation's keys as personae's key type.
///
/// insigne holds them as plain bytes since the 2026-09-24 move; bring this
/// into scope for the typed accessors callers used before it.
pub trait AttestationKeys {
    /// The durable master public key that authorized the derived key.
    fn master_public_key(&self) -> Result<Ed25519PublicKey, IdentityError>;

    /// The derived public key authorized by the master.
    fn derived_public_key(&self) -> Result<Ed25519PublicKey, IdentityError>;
}

impl AttestationKeys for DerivedKeyAttestation {
    fn master_public_key(&self) -> Result<Ed25519PublicKey, IdentityError> {
        Ed25519PublicKey::from_bytes(self.master())
    }

    fn derived_public_key(&self) -> Result<Ed25519PublicKey, IdentityError> {
        Ed25519PublicKey::from_bytes(self.derived())
    }
}

/// A source of identity for the Mere browser.
///
/// Implementors hold the user's master Ed25519 keypair and derive
/// per-protocol keypairs deterministically from a master secret + a
/// protocol-specific salt. **The master secret never leaves the provider.**
///
/// [`SealedIdentityProvider`] is the production implementation for hosts that
/// already have an unlocked [`SealedRecordStorage`]. [`InMemoryProvider`] is
/// intended for tests, ephemeral runtimes, and standalone use.
///
/// ## Why a trait?
///
/// Identity has two stable concerns (master public key + per-protocol
/// derivation) and one platform-specific concern (where the master secret
/// is stored: keychain, in-memory, hardware token, etc.). The trait
/// separates them.
pub trait IdentityProvider: Send + Sync {
    /// The master public key.
    ///
    /// In Mere, the iroh `NodeId` is derived from this (per
    /// [`transport`](https://crates.io/crates/transport)).
    fn master_public_key(&self) -> Ed25519PublicKey;

    /// Derive a per-protocol keypair from a salt.
    ///
    /// Derivation is `BLAKE3-keyed(master_seed, salt)`, the result
    /// becoming the seed of a new Ed25519 keypair (see
    /// [`Ed25519Keypair::derive_child`]).
    ///
    /// Salts are protocol- or use-case-specific. Typical salts:
    /// - Cable: the cabal key (32 bytes)
    /// - MLS: the MLS group identifier
    /// - Co-op: the session identifier
    fn derive_keypair(&self, salt: &[u8]) -> Result<Ed25519Keypair, IdentityError>;

    /// [`derive_keypair`](Self::derive_keypair), typed as derived, for an API
    /// that must never receive the master, such as a transport (vault lock
    /// plan, ruling 96).
    fn derived_keypair(&self, salt: &[u8]) -> Result<DerivedKeypair, IdentityError> {
        self.derive_keypair(salt).map(DerivedKeypair::new)
    }

    /// Certify the key derived from `salt` under this identity's master key.
    fn attest_derived_key(&self, salt: &[u8]) -> Result<DerivedKeyAttestation, IdentityError>;
}

/// Certify the key `master` derives from `salt`: the attestation every
/// provider gives, as a free function so custody's providers (castellan) give
/// the same one.
pub fn attest_derived_key(master: &Ed25519Keypair, salt: &[u8]) -> DerivedKeyAttestation {
    let master_key = master.public_key().to_bytes();
    let derived = master.derive_child(salt).public_key().to_bytes();
    let signature = master
        .sign(&DerivedKeyAttestation::message(&master_key, &derived, salt))
        .to_bytes()
        .to_vec();
    DerivedKeyAttestation::from_parts(master_key, derived, signature)
}

/// In-memory identity provider for tests, standalone runtimes, and ephemeral
/// uses.
///
/// Holds the master keypair in memory; never persisted. Production code
/// should use a keychain-backed provider implemented by the host (e.g.
/// `graphshell`'s desktop keychain backend).
///
/// **Not intended for production**. The master keypair is lost when the
/// provider drops; if you need persistent identity across runs you need a
/// persistent backend.
pub struct InMemoryProvider {
    master: Ed25519Keypair,
}

impl InMemoryProvider {
    /// Create a new provider with a freshly-generated random master keypair.
    pub fn random() -> Self {
        Self {
            master: Ed25519Keypair::generate(),
        }
    }

    /// Create from a 32-byte master seed.
    ///
    /// Useful for test reproducibility and for scenarios where the master
    /// secret is provisioned out-of-band.
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            master: Ed25519Keypair::from_seed(seed),
        }
    }

    /// Borrow the master keypair.
    ///
    /// Exposed for transport-layer use — the iroh QUIC handshake needs the
    /// master signing key to authenticate the local node. Most consumers
    /// should use [`derive_keypair`](IdentityProvider::derive_keypair)
    /// instead; this escape hatch is for transport identity specifically.
    pub fn master_keypair(&self) -> &Ed25519Keypair {
        &self.master
    }
}

impl IdentityProvider for InMemoryProvider {
    fn master_public_key(&self) -> Ed25519PublicKey {
        self.master.public_key()
    }

    fn derive_keypair(&self, salt: &[u8]) -> Result<Ed25519Keypair, IdentityError> {
        Ok(self.master.derive_child(salt))
    }

    fn attest_derived_key(&self, salt: &[u8]) -> Result<DerivedKeyAttestation, IdentityError> {
        Ok(attest_derived_key(&self.master, salt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_key_attestation_binds_master_key_and_salt() {
        let provider = InMemoryProvider::from_seed([0x31; 32]);
        let attestation = provider.attest_derived_key(b"strophe/session/one").unwrap();

        attestation
            .check(b"strophe/session/one")
            .expect("checks under its own salt");
        assert_eq!(
            attestation.check(b"strophe/session/two"),
            Err(insigne::CheckFault::BadSignature)
        );
        assert_eq!(
            attestation.master_public_key().unwrap(),
            provider.master_public_key()
        );
        assert_eq!(
            attestation.derived_public_key().unwrap(),
            provider
                .derive_keypair(b"strophe/session/one")
                .unwrap()
                .public_key()
        );
    }

    #[test]
    fn derived_key_attestation_rejects_tampering() {
        let provider = InMemoryProvider::from_seed([0x31; 32]);
        let attestation = provider.attest_derived_key(b"strophe/session/one").unwrap();
        let mut derived = *attestation.derived();
        derived[0] ^= 1;
        let tampered = DerivedKeyAttestation::from_parts(
            *attestation.master(),
            derived,
            attestation.signature().to_vec(),
        );

        assert_eq!(
            tampered.check(b"strophe/session/one"),
            Err(insigne::CheckFault::BadSignature)
        );
    }
}
