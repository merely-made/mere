// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Which persona is behind a mesh author key.
//!
//! A mesh operation is signed by a key *derived* from a persona master key
//! (salt [`MESH_AUTHOR_SALT`]). Since the vault lock plan's ruling 92 that same
//! author key is the device's transport identity, so a peer is addressed by
//! its author directly and the master never reaches a transport. What the
//! board alone cannot say is which persona an author acts for, and which
//! authors are attested devices at all: that is this directory.
//!
//! *Before ruling 92* transports addressed the master, and this directory was
//! how an author was resolved to an address.
//!
//! The binding already existed. [`personae`](identity) mints a
//! [`DerivedKeyAttestation`]: a statement, signed *by the master*, that this
//! derived key belongs to it. Publishing one on the mesh turns the board into
//! a directory, and it is self-authenticating — no new record format, no
//! trusted third party, nothing to get wrong.
//!
//! Two rules make it safe, and both are checked before the fact is stored:
//!
//! - the master's signature must verify over the mesh-author salt; and
//! - the attested derived key must be the **operation's own author**, so a
//!   device can only ever attest itself. Without that second rule anyone could
//!   publish a true attestation about somebody else and point the ring's blob
//!   fetches at a device of their choosing.

use identity::AttestationKeys;
use std::collections::BTreeMap;

use insigne::DerivedKeyAttestation;
use serde::{Deserialize, Serialize};

/// The derivation salt a mesh authoring key is minted under. A host that signs
/// mesh operations with a key derived under any other salt cannot be found by
/// its peers.
pub const MESH_AUTHOR_SALT: &[u8] = b"mesh-author";

/// Mesh author key → the persona master key that authorized it. The author
/// key is also the device's transport address (ruling 92); the master is
/// recorded only as which persona the device belongs to.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceDirectory {
    devices: BTreeMap<[u8; 32], [u8; 32]>,
}

impl DeviceDirectory {
    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }

    pub fn len(&self) -> usize {
        self.devices.len()
    }

    /// The master key behind a mesh author, if that device has attested itself.
    pub fn master_of(&self, author: &[u8; 32]) -> Option<[u8; 32]> {
        self.devices.get(author).copied()
    }

    /// Whether `author` is an attested device.
    pub fn is_attested(&self, author: &[u8; 32]) -> bool {
        self.devices.contains_key(author)
    }

    /// Every attested device, in author order.
    pub fn entries(&self) -> impl Iterator<Item = (&[u8; 32], &[u8; 32])> {
        self.devices.iter()
    }

    /// Record a self-attestation. Rejected unless it verifies and the attested
    /// derived key is `author`.
    pub(crate) fn admit(&mut self, author: [u8; 32], attestation: &DerivedKeyAttestation) -> bool {
        if !attests(author, attestation) {
            return false;
        }
        let Ok(master) = attestation.master_public_key() else {
            return false;
        };
        self.devices.insert(author, master.to_bytes());
        true
    }
}

/// Whether `attestation` is `author`'s own, and genuinely signed by its master.
///
/// Public because the store checks it before a mutation, and the fold checks it
/// again on the way in — one rule, two doors.
pub fn attests(author: [u8; 32], attestation: &DerivedKeyAttestation) -> bool {
    attestation.check(MESH_AUTHOR_SALT).is_ok_and(|checked| {
        checked
            .attestation()
            .derived_public_key()
            .is_ok_and(|derived| derived.to_bytes() == author)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use identity::{IdentityProvider, InMemoryProvider};

    fn device(seed: u8) -> (InMemoryProvider, [u8; 32], DerivedKeyAttestation) {
        let provider = InMemoryProvider::from_seed([seed; 32]);
        let author = provider
            .derive_keypair(MESH_AUTHOR_SALT)
            .unwrap()
            .public_key()
            .to_bytes();
        let attestation = provider.attest_derived_key(MESH_AUTHOR_SALT).unwrap();
        (provider, author, attestation)
    }

    #[test]
    fn a_self_attestation_resolves_the_author_to_its_master() {
        let (provider, author, attestation) = device(1);
        let mut directory = DeviceDirectory::default();
        assert!(directory.admit(author, &attestation));
        assert_eq!(
            directory.master_of(&author),
            Some(provider.master_public_key().to_bytes()),
            "the mesh author resolves to the key a transport addresses"
        );
        assert_ne!(
            directory.master_of(&author),
            Some(author),
            "the derived key and the master key are genuinely different"
        );
    }

    #[test]
    fn a_device_may_only_attest_itself() {
        let (_, alice, alice_attestation) = device(1);
        let (_, bob, _) = device(2);
        let mut directory = DeviceDirectory::default();

        // Alice's attestation is perfectly valid — and still refused when Bob
        // publishes it, because it does not attest Bob. Otherwise anyone could
        // point the ring's fetches at a device of their choosing.
        assert!(!directory.admit(bob, &alice_attestation));
        assert!(directory.is_empty());
        assert!(directory.admit(alice, &alice_attestation));
    }

    #[test]
    fn an_attestation_for_another_salt_does_not_count() {
        let provider = InMemoryProvider::from_seed([3; 32]);
        let author = provider
            .derive_keypair(b"mesh-author")
            .unwrap()
            .public_key()
            .to_bytes();
        let other = provider.attest_derived_key(b"some-other-protocol").unwrap();
        assert!(
            !attests(author, &other),
            "a key minted for another protocol is not this device's mesh identity"
        );
    }

    #[test]
    fn a_tampered_attestation_is_refused() {
        let (_, author, attestation) = device(4);
        let mut forged: DerivedKeyAttestation = p2panda_core::cbor::decode_cbor(
            p2panda_core::cbor::encode_cbor(&attestation)
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        assert!(attests(author, &forged), "the round trip is faithful");

        // Swap in another master and the signature no longer covers it.
        let (_, _, other) = device(5);
        forged = p2panda_core::cbor::decode_cbor(
            p2panda_core::cbor::encode_cbor(&other).unwrap().as_slice(),
        )
        .unwrap();
        assert!(!attests(author, &forged));
    }
}
