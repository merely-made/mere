// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The custodian's hold on Djinn's real Castellan vault: the same
//! passphrase vault file, profiles, slots and derivations, opened only here.
//! Every start is locked before anything is decrypted, whatever the
//! advisory marker says. Unlock reopens the configured or remembered
//! profile and refuses a changed identity; nothing here ever mints a
//! profile or a replacement root. Lock drops the whole owning vault and
//! storage, then reports separately whether the restart barrier persisted.

use std::path::{Path, PathBuf};

use castellan::custody::bootstrap::{self, PASSPHRASE_VAULT_FILE, Unlock};
use castellan::custody::{AUTO_UNLOCK_ROOT_FILE, IdentityStorage, IdentityVault};
use custody_wire::AttestationParts;
use custody_wire::secret::Bytes;
use personae::{IdentityProvider, ProfileId};
use zeroize::Zeroizing;

use crate::Refusal;

type Vault = IdentityVault<Box<dyn IdentityStorage>>;

/// One released derived key, seed in a clearing buffer.
pub struct Released {
    pub salt: Vec<u8>,
    pub seed: Zeroizing<[u8; 32]>,
    pub attestation: AttestationParts,
}

/// A signature made inside custody.
pub struct Signed {
    pub public_key: [u8; 32],
    pub signature: [u8; 64],
    pub attestation: Option<AttestationParts>,
}

/// The vault as the custodian holds it.
pub struct VaultCustody {
    directory: PathBuf,
    configured: Option<ProfileId>,
    vault: Option<Vault>,
    known_profile: Option<ProfileId>,
    known_master: Option<[u8; 32]>,
}

impl VaultCustody {
    /// Hold `directory` locked. Refuses an absent vault (creation is an
    /// explicit administrative act) and an OS-sealed vault, whose DPAPI root
    /// belongs to the desktop user rather than the custodian's identity.
    pub fn locked(directory: &Path, profile: Option<&str>) -> Result<Self, Refusal> {
        if !directory.is_absolute() {
            return Err(Refusal("vault-directory-not-absolute"));
        }
        if !directory.join(PASSPHRASE_VAULT_FILE).is_file() {
            if directory.join(AUTO_UNLOCK_ROOT_FILE).exists() {
                return Err(Refusal("os-sealed-vault-needs-migration"));
            }
            return Err(Refusal("vault-missing"));
        }
        Ok(Self {
            directory: directory.to_path_buf(),
            configured: profile.map(|p| ProfileId(p.to_string())),
            vault: None,
            known_profile: None,
            known_master: None,
        })
    }

    pub fn is_locked(&self) -> bool {
        self.vault.is_none()
    }

    /// The open profile, or the last one opened while locked.
    pub fn profile(&self) -> Option<&str> {
        self.known_profile.as_ref().map(|p| p.0.as_str())
    }

    /// The master public key last seen; public, kept across a lock.
    pub fn master_public_key(&self) -> Option<[u8; 32]> {
        self.known_master
    }

    pub fn protection(&self) -> String {
        format!(
            "passphrase-encrypted vault at {}, held by the custodian",
            self.directory.join(PASSPHRASE_VAULT_FILE).display()
        )
    }

    /// Open with the passphrase. A wrong passphrase, an unresolvable or
    /// changed profile, or a marker that will not clear leaves it locked.
    pub fn unlock(&mut self, passphrase: &[u8]) -> Result<(), Refusal> {
        if self.vault.is_some() {
            return Err(Refusal("already-unlocked"));
        }
        let mut owned = Zeroizing::new(Vec::with_capacity(passphrase.len()));
        owned.extend_from_slice(passphrase);
        let opened = bootstrap::open_storage(&self.directory, Unlock::Passphrase(owned))
            .map_err(|_| Refusal("unlock-refused"))?;
        let profile = self.resolve(&*opened.storage)?;
        let vault = IdentityVault::open(opened.storage, &profile).map_err(|_| Refusal("profile-unavailable"))?;
        let master = vault.master_public_key().to_bytes();
        if self.known_master.is_some_and(|known| known != master) {
            return Err(Refusal("vault-identity-changed"));
        }
        if castellan::custody::clear_persisted_lock(&self.directory).is_err() {
            drop(vault);
            let _ = castellan::custody::persist_lock(&self.directory);
            return Err(Refusal("lock-marker-not-cleared"));
        }
        self.known_master = Some(master);
        self.known_profile = Some(profile);
        self.vault = Some(vault);
        Ok(())
    }

    /// The configured profile, else the remembered choice read directly
    /// (no environment), else the vault's sole profile. Never `default`
    /// minted beside an existing identity.
    fn resolve(&self, storage: &dyn IdentityStorage) -> Result<ProfileId, Refusal> {
        let summaries = storage.list_profiles().map_err(|_| Refusal("vault-storage"))?;
        let present = |id: &ProfileId| summaries.iter().any(|s| &s.id == id);
        if let Some(configured) = &self.configured {
            return present(configured)
                .then(|| configured.clone())
                .ok_or(Refusal("profile-missing"));
        }
        let remembered = std::fs::read_to_string(castellan::custody::roster::choice_path(&self.directory))
            .ok()
            .map(|s| ProfileId(s.trim().to_string()))
            .filter(|id| !id.0.is_empty());
        if let Some(remembered) = remembered {
            return present(&remembered)
                .then_some(remembered)
                .ok_or(Refusal("profile-missing"));
        }
        match summaries.as_slice() {
            [only] => Ok(only.id.clone()),
            _ => Err(Refusal("profile-unresolved")),
        }
    }

    /// Drop the vault and storage; `true` when the restart barrier was
    /// written and synced.
    pub fn lock(&mut self) -> bool {
        self.vault.take();
        castellan::custody::persist_lock(&self.directory).is_ok()
    }

    fn open(&self) -> Result<&Vault, Refusal> {
        self.vault.as_ref().ok_or(Refusal("locked"))
    }

    /// Sign with the key `salt` derives, or with the master for an empty
    /// salt. The derived key never leaves this call.
    pub fn sign(&self, salt: &[u8], message: &[u8]) -> Result<Signed, Refusal> {
        let vault = self.open()?;
        if salt.is_empty() {
            let master = &vault.current_profile().map_err(|_| Refusal("locked"))?.master;
            return Ok(Signed {
                public_key: master.public_key().to_bytes(),
                signature: master.sign(message).to_bytes(),
                attestation: None,
            });
        }
        let key = vault.derived_keypair(salt).map_err(|_| Refusal("derivation-failed"))?;
        let attestation = vault.attest_derived_key(salt).map_err(|_| Refusal("derivation-failed"))?;
        Ok(Signed {
            public_key: key.public_key().to_bytes(),
            signature: key.sign(message).to_bytes(),
            attestation: Some(parts(&attestation)),
        })
    }

    /// Release the derived keys for `salts`, each with its attestation.
    pub fn release(&self, salts: &[Vec<u8>]) -> Result<([u8; 32], Vec<Released>), Refusal> {
        let vault = self.open()?;
        let mut released = Vec::with_capacity(salts.len());
        for salt in salts {
            if salt.is_empty() {
                return Err(Refusal("master-never-released"));
            }
            let key = vault.derive_keypair(salt).map_err(|_| Refusal("derivation-failed"))?;
            let attestation = vault.attest_derived_key(salt).map_err(|_| Refusal("derivation-failed"))?;
            released.push(Released {
                salt: salt.clone(),
                seed: Zeroizing::new(key.to_seed()),
                attestation: parts(&attestation),
            });
        }
        Ok((vault.master_public_key().to_bytes(), released))
    }
}

fn parts(attestation: &insigne::DerivedKeyAttestation) -> AttestationParts {
    AttestationParts {
        master: *attestation.master(),
        derived: *attestation.derived(),
        signature: Bytes(attestation.signature().to_vec()),
    }
}
