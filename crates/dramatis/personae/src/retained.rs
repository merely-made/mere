// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Keys a vault lock leaves in place (vault lock rulings 24, 40 and 46).
//!
//! [`RetainedKeys`] is a restricted provider: captured from an unlocked
//! provider for a fixed list of salts, it answers the master public key and
//! derives or attests those salts only. Every other salt is
//! [`IdentityError::Locked`]. It holds no master secret, so it cannot derive
//! anything it was not given; each key it holds is a derived child, which
//! reveals neither the master nor any sibling.

use insigne::DerivedKeyAttestation;

use crate::{Ed25519Keypair, Ed25519PublicKey, IdentityError, IdentityProvider};

struct RetainedKey {
    salt: Vec<u8>,
    keypair: Ed25519Keypair,
    attestation: DerivedKeyAttestation,
}

/// A restricted provider over a fixed set of derived keys.
pub struct RetainedKeys {
    master: Ed25519PublicKey,
    keys: Vec<RetainedKey>,
}

impl RetainedKeys {
    /// Capture `salts`' keys and attestations from `provider`, which must be
    /// unlocked.
    pub fn capture<P: IdentityProvider + ?Sized>(
        provider: &P,
        salts: &[Vec<u8>],
    ) -> Result<Self, IdentityError> {
        let mut keys = Vec::with_capacity(salts.len());
        for salt in salts {
            keys.push(RetainedKey {
                salt: salt.clone(),
                keypair: provider.derive_keypair(salt)?,
                attestation: provider.attest_derived_key(salt)?,
            });
        }
        Ok(Self {
            master: provider.master_public_key(),
            keys,
        })
    }

    /// Rebuild a release received from a custodian (dramatis repo plan,
    /// D11): each salt with the derived keypair and the custodian's
    /// attestation for it. Every attestation must check under its salt,
    /// name `master`, and name the keypair it came with, so a release that
    /// mixes keys or masters is refused rather than trusted.
    pub fn from_released(
        master: Ed25519PublicKey,
        released: impl IntoIterator<Item = (Vec<u8>, Ed25519Keypair, DerivedKeyAttestation)>,
    ) -> Result<Self, IdentityError> {
        let mut keys = Vec::new();
        for (salt, keypair, attestation) in released {
            let checked = attestation.check(&salt).map_err(|error| {
                IdentityError::DerivationFailed(format!("released key attestation: {error:?}"))
            })?;
            if checked.master() != &master.to_bytes()
                || checked.derived() != &keypair.public_key().to_bytes()
            {
                return Err(IdentityError::DerivationFailed(
                    "a released key does not belong to the released master".into(),
                ));
            }
            keys.push(RetainedKey {
                salt,
                keypair,
                attestation,
            });
        }
        Ok(Self { master, keys })
    }

    /// Whether this holds exactly `salts`, in order.
    pub fn holds(&self, salts: &[Vec<u8>]) -> bool {
        self.keys.len() == salts.len() && self.keys.iter().zip(salts).all(|(k, s)| k.salt == *s)
    }

    /// The salts this holds, in order.
    pub fn salts(&self) -> Vec<Vec<u8>> {
        self.keys.iter().map(|key| key.salt.clone()).collect()
    }

    fn find(&self, salt: &[u8]) -> Result<&RetainedKey, IdentityError> {
        self.keys
            .iter()
            .find(|key| key.salt == salt)
            .ok_or(IdentityError::Locked)
    }
}

impl std::fmt::Debug for RetainedKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetainedKeys")
            .field("salts", &self.keys.len())
            .finish_non_exhaustive()
    }
}

impl IdentityProvider for RetainedKeys {
    fn master_public_key(&self) -> Ed25519PublicKey {
        self.master
    }

    fn derive_keypair(&self, salt: &[u8]) -> Result<Ed25519Keypair, IdentityError> {
        Ok(self.find(salt)?.keypair.clone())
    }

    fn attest_derived_key(&self, salt: &[u8]) -> Result<DerivedKeyAttestation, IdentityError> {
        Ok(self.find(salt)?.attestation.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InMemoryProvider;

    fn salts() -> Vec<Vec<u8>> {
        vec![b"door/one".to_vec(), b"door/two".to_vec()]
    }

    #[test]
    fn it_answers_its_salts_exactly_as_the_vault_did() {
        let full = InMemoryProvider::from_seed([0x71; 32]);
        let retained = RetainedKeys::capture(&full, &salts()).unwrap();
        assert_eq!(retained.master_public_key(), full.master_public_key());
        for salt in salts() {
            assert_eq!(
                retained.derive_keypair(&salt).unwrap().to_seed(),
                full.derive_keypair(&salt).unwrap().to_seed()
            );
            assert_eq!(
                retained.attest_derived_key(&salt).unwrap(),
                full.attest_derived_key(&salt).unwrap()
            );
        }
        assert!(retained.holds(&salts()));
    }

    /// The guard: any salt it was not given is Locked, a near miss included.
    #[test]
    fn every_other_salt_is_locked() {
        let full = InMemoryProvider::from_seed([0x72; 32]);
        let retained = RetainedKeys::capture(&full, &salts()).unwrap();
        for salt in [
            &b"mere/network-policy/session-signer/v1"[..],
            b"door/one/",
            b"door/on",
            b"",
            b"mere.djinn/castellan/records/v1",
        ] {
            assert!(matches!(
                retained.derive_keypair(salt),
                Err(IdentityError::Locked)
            ));
            assert!(matches!(
                retained.attest_derived_key(salt),
                Err(IdentityError::Locked)
            ));
        }
    }
}
