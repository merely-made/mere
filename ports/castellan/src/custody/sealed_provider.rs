// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A provider whose master lives in a versioned sealed record: custody,
//! moved from personae's `provider` (dramatis repo plan, DR-B).

use std::path::Path;

use insigne::DerivedKeyAttestation;
use personae::{
    Ed25519Keypair, Ed25519PublicKey, IdentityError, IdentityProvider, SealedRecordStorage,
    attest_derived_key,
};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

const SEALED_IDENTITY_FORMAT_VERSION: u8 = 1;

#[derive(Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
struct SealedIdentityRecord {
    format_version: u8,
    master_seed: [u8; 32],
}

/// An identity provider loaded from a versioned sealed record.
///
/// The caller owns unlock policy and path selection. This provider owns key
/// generation, the seed record format, and immediate scrubbing of temporary
/// seed copies. Its master key remains in memory only while the provider lives.
pub struct SealedIdentityProvider {
    master: Ed25519Keypair,
}

impl SealedIdentityProvider {
    /// Load an existing identity from `record_path`, or create and seal one.
    pub fn load_or_create(
        storage: &SealedRecordStorage,
        record_path: impl AsRef<Path>,
    ) -> Result<Self, IdentityError> {
        let record_path = record_path.as_ref();
        let mut record = match storage.load_record::<SealedIdentityRecord>(record_path)? {
            Some(record) => {
                if record.format_version != SEALED_IDENTITY_FORMAT_VERSION {
                    return Err(IdentityError::Backend(format!(
                        "unsupported sealed identity version {} at {:?}",
                        record.format_version, record_path
                    )));
                }
                record
            },
            None => {
                let master = Ed25519Keypair::generate();
                let record = SealedIdentityRecord {
                    format_version: SEALED_IDENTITY_FORMAT_VERSION,
                    master_seed: master.to_seed(),
                };
                storage.save_record(record_path, &record)?;
                record
            },
        };
        let master = Ed25519Keypair::from_seed(record.master_seed);
        record.master_seed.zeroize();
        Ok(Self { master })
    }

    /// Borrow the master keypair for transport implementations that require it.
    pub fn master_keypair(&self) -> &Ed25519Keypair {
        &self.master
    }
}

impl IdentityProvider for SealedIdentityProvider {
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
    fn sealed_provider_is_stable_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let storage = SealedRecordStorage::open_with_key(dir.path(), [0x45; 32]);
        let first =
            SealedIdentityProvider::load_or_create(&storage, "identity/default.json").unwrap();
        let first_public = first.master_public_key();
        drop(first);

        let second =
            SealedIdentityProvider::load_or_create(&storage, "identity/default.json").unwrap();
        assert_eq!(second.master_public_key(), first_public);
    }

    #[test]
    fn sealed_provider_rejects_the_wrong_record_root() {
        let dir = tempfile::tempdir().unwrap();
        let storage = SealedRecordStorage::open_with_key(dir.path(), [0x45; 32]);
        SealedIdentityProvider::load_or_create(&storage, "identity/default.json").unwrap();

        let wrong_storage = SealedRecordStorage::open_with_key(dir.path(), [0x46; 32]);
        let error = SealedIdentityProvider::load_or_create(&wrong_storage, "identity/default.json")
            .err()
            .expect("wrong root should fail");
        assert!(error.to_string().contains("decrypt sealed record"));
    }
}
