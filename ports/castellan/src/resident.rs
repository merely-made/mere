// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The process-wide owner of Castellan's sealed credential records.
//!
//! A desktop host keeps one resident alive and gives applications admitted
//! views or protocol adapters over it. Sandboxed applications may embed the
//! same type. Either shape retains one OS file lock and one external freshness
//! ledger for the lifetime of every clone, and one transaction lock per
//! persona for every multi-record write.
//!
//! ## Locking (vault lock ruling 1)
//!
//! The record and freshness keys derive from the vault, so they follow its
//! lock: [`CastellanResident::lock`] forgets both for every clone and every
//! store handed out, and [`CastellanResident::unlock`] takes them back.
//! [`CredentialLockHolder`] is the hook that does both from the resident
//! host's lock.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use personae::{
    IdentityError, IdentityProvider, PersonaId, ProfileId, SealedRecordChange, SealedRecordStorage,
};
use zeroize::Zeroizing;

use crate::items::ItemStore;
use crate::lock::VaultLockHolder;
use crate::otp::OtpItemStore;

/// The record and freshness keys, cleared when dropped.
type DerivedKeys = (Zeroizing<[u8; 32]>, Zeroizing<[u8; 32]>);

/// The two salts a resident derives its record and freshness keys under.
#[derive(Clone, Copy, Debug)]
pub struct CredentialSalts {
    /// Seals credential contents.
    pub record: &'static [u8],
    /// Authenticates the external freshness ledger.
    pub freshness: &'static [u8],
}

impl CredentialSalts {
    /// Both keys from the unlocked vault, cleared when dropped.
    fn derive<P: IdentityProvider + ?Sized>(
        &self,
        vault: &P,
    ) -> Result<DerivedKeys, IdentityError> {
        let record = Zeroizing::new(vault.derive_keypair(self.record)?.to_seed());
        let freshness = Zeroizing::new(vault.derive_keypair(self.freshness)?.to_seed());
        Ok((record, freshness))
    }
}

/// Exclusive authority over one Castellan credential-record directory.
#[derive(Clone)]
pub struct CastellanResident {
    records: SealedRecordStorage,
    records_root: PathBuf,
    transactions: Arc<Mutex<BTreeMap<PersonaId, Arc<Mutex<()>>>>>,
}

impl CastellanResident {
    /// Claim the record directory and its separately rooted freshness ledger.
    ///
    /// `record_key` seals credential contents. `freshness_key` authenticates
    /// the external generation ledger and should be independently derived by
    /// the already-unlocked Personae authority. A second claim against the
    /// same record directory fails immediately.
    pub fn claim(
        records_root: impl Into<PathBuf>,
        record_key: [u8; 32],
        freshness_root: impl Into<PathBuf>,
        freshness_key: [u8; 32],
    ) -> Result<Self, IdentityError> {
        let records_root = records_root.into();
        Ok(Self {
            records_root: records_root.clone(),
            records: SealedRecordStorage::claim_with_file_freshness(
                records_root,
                record_key,
                freshness_root,
                freshness_key,
            )?,
            transactions: Arc::new(Mutex::new(BTreeMap::new())),
        })
    }

    /// [`Self::claim`] with both keys derived from `vault` under `salts`.
    pub fn claim_derived(
        records_root: impl Into<PathBuf>,
        freshness_root: impl Into<PathBuf>,
        vault: &(impl IdentityProvider + ?Sized),
        salts: CredentialSalts,
    ) -> Result<Self, IdentityError> {
        let (record, freshness) = salts.derive(vault)?;
        Self::claim(records_root, *record, freshness_root, *freshness)
    }

    /// Forget the record and freshness keys for every clone and every store
    /// this authority handed out. Locking twice is harmless.
    pub fn lock(&self) {
        self.records.lock();
    }

    /// Take both keys back.
    pub fn unlock(
        &self,
        record_key: [u8; 32],
        freshness_key: [u8; 32],
    ) -> Result<(), IdentityError> {
        self.records.unlock(record_key, Some(freshness_key))
    }

    /// Whether the keys are forgotten.
    pub fn is_locked(&self) -> bool {
        self.records.is_locked()
    }

    /// The hook that locks this authority with the vault and re-derives its
    /// keys under `salts` on unlock.
    pub fn lock_holder(&self, salts: CredentialSalts) -> Arc<dyn VaultLockHolder> {
        Arc::new(CredentialLockHolder {
            resident: self.clone(),
            salts,
            key_profile: None,
        })
    }

    /// Reopen this store from its original key persona, independent of selection.
    pub fn lock_holder_for_profile(
        &self,
        salts: CredentialSalts,
        profile: ProfileId,
    ) -> Arc<dyn VaultLockHolder> {
        Arc::new(CredentialLockHolder {
            resident: self.clone(),
            salts,
            key_profile: Some(profile),
        })
    }
    /// Bind an item namespace to its explicitly chosen profile, without reassignment.
    pub fn bind_profile(
        &self,
        persona: PersonaId,
        profile: &ProfileId,
    ) -> Result<(), IdentityError> {
        self.bind_profiles(&BTreeMap::from([(persona, profile.clone())]))
    }
    /// Check all destinations before changing the sealed ownership record.
    pub fn bind_profiles(
        &self,
        owners: &BTreeMap<PersonaId, ProfileId>,
    ) -> Result<(), IdentityError> {
        self.records.update_record(
            "castellan/items/v1/profile-bindings.json",
            |old: Option<BTreeMap<String, ProfileId>>| {
                let mut map = old.unwrap_or_default();
                for (persona, profile) in owners {
                    if map
                        .get(&persona.as_uuid().to_string())
                        .is_some_and(|bound| bound != profile)
                    {
                        return Err(IdentityError::Backend(
                            "item namespace already belongs to another profile".into(),
                        ));
                    }
                }
                let mut changed = false;
                for (persona, profile) in owners {
                    if map
                        .insert(persona.as_uuid().to_string(), profile.clone())
                        .is_none()
                    {
                        changed = true;
                    }
                }
                Ok((
                    (),
                    if changed {
                        SealedRecordChange::Replace(map)
                    } else {
                        SealedRecordChange::Keep
                    },
                ))
            },
        )
    }
    /// Only explicitly bound namespaces. Never infer legacy ownership from a name.
    pub fn profile_personas(&self, profile: &ProfileId) -> Result<Vec<PersonaId>, IdentityError> {
        let map: BTreeMap<String, ProfileId> = self
            .records
            .load_record("castellan/items/v1/profile-bindings.json")?
            .unwrap_or_default();
        map.into_iter()
            .filter(|(_, owner)| owner == profile)
            .map(|(id, _)| {
                uuid::Uuid::parse_str(&id)
                    .map(PersonaId::from_uuid)
                    .map_err(|_| IdentityError::Backend("invalid item namespace binding".into()))
            })
            .collect()
    }
    /// Discover older namespaces through authenticated public indexes only.
    /// Directory names do not assign identity ownership.
    #[cfg(feature = "cxf")]
    pub fn unassigned_items(
        &self,
    ) -> Result<Vec<(PersonaId, Vec<chatelaine::Item>)>, IdentityError> {
        let map: BTreeMap<String, ProfileId> = self
            .records
            .load_record("castellan/items/v1/profile-bindings.json")?
            .unwrap_or_default();
        let entries = match std::fs::read_dir(self.records_root.join("castellan/items/v1")) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(_) => {
                return Err(IdentityError::Backend(
                    "older import namespaces are unavailable".into(),
                ));
            },
        };
        let mut namespaces = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|_| {
                IdentityError::Backend("older import namespace is unavailable".into())
            })?;
            if !entry
                .file_type()
                .map_err(|_| {
                    IdentityError::Backend("older import namespace is unavailable".into())
                })?
                .is_dir()
            {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            let Ok(id) = uuid::Uuid::parse_str(&name) else {
                continue;
            };
            if map.contains_key(&id.to_string()) {
                continue;
            }
            let persona = PersonaId::from_uuid(id);
            let items = self.items(persona).list().map_err(|_| {
                IdentityError::Backend("older import metadata is unavailable".into())
            })?;
            if !items.is_empty() {
                namespaces.push((persona, items));
            }
        }
        namespaces.sort_by_key(|(persona, _)| *persona);
        Ok(namespaces)
    }
    /// Associate the precise reviewed namespace, under its transaction lock.
    #[cfg(feature = "cxf")]
    pub fn assign_import(
        &self,
        persona: PersonaId,
        profile: &ProfileId,
        revision: &str,
    ) -> Result<(), IdentityError> {
        let store = self.items(persona);
        let held = store.begin();
        let items = held
            .list()
            .map_err(|_| IdentityError::Backend("older import metadata is unavailable".into()))?;
        if items.is_empty() || crate::review::namespace_revision(&items) != revision {
            return Err(IdentityError::Backend(
                "older import changed; review it again".into(),
            ));
        }
        self.records.update_record(
            "castellan/items/v1/profile-bindings.json",
            |old: Option<BTreeMap<String, ProfileId>>| {
                let mut map = old.unwrap_or_default();
                let key = persona.as_uuid().to_string();
                if map.contains_key(&key) {
                    return Err(IdentityError::Backend(
                        "older import was already assigned".into(),
                    ));
                }
                map.insert(key, profile.clone());
                Ok(((), SealedRecordChange::Replace(map)))
            },
        )
    }
    /// Open one persona's sealed items under this authority.
    pub fn items(&self, persona: PersonaId) -> ItemStore {
        ItemStore::with_transaction(self.records.clone(), persona, self.transaction(persona))
    }

    /// Open one persona's OTP credentials under this authority.
    pub fn otp_items(&self, persona: PersonaId) -> OtpItemStore {
        OtpItemStore::over(self.items(persona))
    }

    /// Open the persona-scoped Freedesktop Secret Service store, a view of
    /// the persona's items.
    #[cfg(feature = "secret-service")]
    pub fn secret_service(
        &self,
        persona: PersonaId,
        limits: crate::secret_service::SecretServiceLimits,
    ) -> crate::secret_service::SecretServiceStore {
        crate::secret_service::SecretServiceStore::new(self.items(persona), limits)
    }

    /// The one transaction lock every store for `persona` shares.
    fn transaction(&self, persona: PersonaId) -> Arc<Mutex<()>> {
        self.transactions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(persona)
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }
}

/// A [`CastellanResident`] as a vault lock holder.
pub struct CredentialLockHolder {
    resident: CastellanResident,
    salts: CredentialSalts,
    key_profile: Option<ProfileId>,
}

impl VaultLockHolder for CredentialLockHolder {
    fn name(&self) -> &str {
        "castellan credentials"
    }
    fn key_profile(&self) -> Option<&ProfileId> {
        self.key_profile.as_ref()
    }

    fn lock(&self) {
        self.resident.lock();
    }

    fn unlock(&self, vault: &dyn IdentityProvider) -> Result<(), IdentityError> {
        let (record, freshness) = self.salts.derive(vault)?;
        self.resident.unlock(*record, *freshness)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::otp::{OtpReleaseGate, OtpReleaseParticipantClaim};
    use tempfile::tempdir;

    const RFC4226_SECRET_BASE32: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";

    fn claim(root: &std::path::Path) -> CastellanResident {
        CastellanResident::claim(
            root.join("records"),
            [0x81; 32],
            root.join("freshness"),
            [0x82; 32],
        )
        .unwrap()
    }

    #[test]
    fn conflicting_batch_does_not_partly_bind_destinations() {
        let dir = tempdir().unwrap();
        let resident = claim(dir.path());
        let bound = PersonaId::new();
        let fresh = PersonaId::new();
        resident
            .bind_profile(bound, &ProfileId("family".into()))
            .unwrap();
        let owners = BTreeMap::from([
            (fresh, ProfileId("work".into())),
            (bound, ProfileId("work".into())),
        ]);
        assert!(resident.bind_profiles(&owners).is_err());
        assert!(
            resident
                .profile_personas(&ProfileId("work".into()))
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            resident
                .profile_personas(&ProfileId("family".into()))
                .unwrap(),
            vec![bound]
        );
    }

    #[test]
    fn independent_gates_under_one_resident_cannot_repeat_an_hotp_counter() {
        let dir = tempdir().unwrap();
        let resident = claim(dir.path());
        let persona = PersonaId::new();
        let items = resident.otp_items(persona);
        let item = items
            .import_otpauth_uri(&format!(
                "otpauth://hotp/Steam:mark?secret={RFC4226_SECRET_BASE32}&issuer=Steam&counter=0"
            ))
            .unwrap();
        let left = OtpReleaseGate::new(resident.otp_items(persona));
        let right = OtpReleaseGate::new(resident.otp_items(persona));
        let participant =
            || OtpReleaseParticipantClaim::unverified("local:test", "resident:test").unwrap();

        let first = left
            .petition(item.item_id(), item.credential_id(), participant())
            .unwrap();
        let first = left.approve(first.id).unwrap();
        let second = right
            .petition(item.item_id(), item.credential_id(), participant())
            .unwrap();
        let second = right.approve(second.id).unwrap();

        assert_eq!(first.tile().code_at_unix_time(0), Some("755224"));
        assert_eq!(second.tile().code_at_unix_time(0), Some("287082"));
    }

    #[test]
    fn resident_rejects_restored_hotp_state_before_releasing_it_again() {
        let dir = tempdir().unwrap();
        let records = dir.path().join("records");
        let resident = claim(dir.path());
        let persona = PersonaId::new();
        let items = resident.otp_items(persona);
        let item = items
            .import_otpauth_uri(&format!(
                "otpauth://hotp/Steam:mark?secret={RFC4226_SECRET_BASE32}&issuer=Steam&counter=0"
            ))
            .unwrap();
        // The HOTP counter lives in the credential's payload record alone.
        let record = records
            .join("castellan/items/v1")
            .join(persona.as_uuid().to_string())
            .join("payloads")
            .join(format!("{}.json", item.credential_id()));
        let counter_zero = std::fs::read(&record).unwrap();
        let gate = OtpReleaseGate::new(items);
        let participant =
            || OtpReleaseParticipantClaim::unverified("local:test", "rollback:test").unwrap();

        let first = gate
            .petition(item.item_id(), item.credential_id(), participant())
            .unwrap();
        assert_eq!(
            gate.approve(first.id).unwrap().tile().code_at_unix_time(0),
            Some("755224")
        );
        let second = gate
            .petition(item.item_id(), item.credential_id(), participant())
            .unwrap();
        std::fs::write(&record, counter_zero).unwrap();

        let error = gate.approve(second.id).unwrap_err();
        assert!(error.to_string().contains("rollback detected"));
    }

    #[cfg(feature = "secret-service")]
    #[test]
    fn independent_secret_service_views_share_one_composite_transaction() {
        use crate::secret_service::{NewSecretItem, SecretServiceLimits};

        let dir = tempdir().unwrap();
        let resident = claim(dir.path());
        let persona = PersonaId::new();
        let left = resident.secret_service(persona, SecretServiceLimits::default());
        let right = resident.secret_service(persona, SecretServiceLimits::default());
        let collection = left.ensure_default_collection("Castellan", 1).unwrap();
        let create = |store: crate::secret_service::SecretServiceStore, prefix: &'static str| {
            std::thread::spawn(move || {
                for index in 0..20 {
                    store
                        .create_item(NewSecretItem {
                            collection: collection.id,
                            label: format!("{prefix} {index}"),
                            attributes: std::collections::BTreeMap::from([(
                                "id".to_string(),
                                format!("{prefix}-{index}"),
                            )]),
                            secret: vec![index],
                            content_type: "application/octet-stream".into(),
                            replace: false,
                            unix_secs: index.into(),
                        })
                        .unwrap();
                }
            })
        };
        let left = create(left, "left");
        let right = create(right, "right");
        left.join().unwrap();
        right.join().unwrap();

        assert_eq!(
            resident
                .secret_service(persona, SecretServiceLimits::default())
                .items(collection.id)
                .unwrap()
                .len(),
            40
        );
    }
}
