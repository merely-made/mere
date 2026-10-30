// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::custody::PassphraseEncryptedStorage;

fn mixed_profile() -> Profile {
    let mut profile = Profile::new(
        ProfileId("synthetic".into()),
        "Synthetic",
        Ed25519Keypair::from_seed([0x59; 32]),
    );
    for (instance, tier, lineage) in [
        (
            Some("named"),
            UnlockTier::PerUse,
            CredentialLineage::ExternallyIssued,
        ),
        (None, UnlockTier::Session, CredentialLineage::LocallyDerived),
        (
            Some(""),
            UnlockTier::ShortTtl { idle_seconds: 42 },
            CredentialLineage::ExternallyRootedLocallyHeld,
        ),
    ] {
        profile.slots.insert(
            ProtocolKey::new("protocol", instance.map(str::to_owned)),
            IdentitySlot::Direct {
                kind: "synthetic".into(),
                payload: SecretBytes::new(vec![0x5a; 32]),
                lineage,
                unlock_tier: tier,
            },
        );
    }
    profile
}

#[test]
fn public_summaries_preserve_absent_empty_and_named_instances_and_policy() {
    let profile = mixed_profile();
    let public = public_profile_of(&profile);
    assert_eq!(public.slots.len(), 3);
    assert_eq!(
        public
            .slots
            .iter()
            .map(|s| s.key.instance.as_deref().map(String::as_str))
            .collect::<Vec<_>>(),
        vec![None, Some(""), Some("named")]
    );
    for summary in &public.slots {
        let slot = profile
            .slots
            .get(&ProtocolKey::new(
                summary.key.mod_id.clone(),
                summary.key.instance.as_deref().cloned(),
            ))
            .unwrap();
        assert_eq!(summary.kind, slot.kind());
        assert_eq!(summary.lineage, slot.lineage());
        assert_eq!(summary.unlock_tier, slot.unlock_tier());
    }
    assert_eq!(public.master_public_key, profile.master.public_key());
}

#[test]
fn public_summaries_follow_replacement_removal_and_profile_switch() {
    let storage = InMemoryStorage::new();
    let alternate = Profile::new(
        ProfileId("alternate".into()),
        "Alternate",
        Ed25519Keypair::from_seed([0x60; 32]),
    );
    storage.save_profile(&alternate).unwrap();
    let mut vault = IdentityVault::with_profile(storage, mixed_profile());
    let absent = ProtocolKey::new("protocol", None);
    vault
        .add_slot(
            absent.clone(),
            IdentitySlot::Direct {
                kind: "replacement".into(),
                payload: SecretBytes::new(vec![1; 32]),
                lineage: CredentialLineage::LocallyDerived,
                unlock_tier: UnlockTier::PerUse,
            },
        )
        .unwrap();
    assert_eq!(vault.public_profile().slots[0].kind, "replacement");
    vault
        .remove_slot(&ProtocolKey::new("protocol", Some("".into())))
        .unwrap();
    assert_eq!(
        vault
            .public_profile()
            .slots
            .iter()
            .map(|s| s.key.instance.as_deref().map(String::as_str))
            .collect::<Vec<_>>(),
        vec![None, Some("named")]
    );
    vault
        .switch_profile(&ProfileId("alternate".into()))
        .unwrap();
    assert!(vault.public_profile().slots.is_empty());
    assert_eq!(
        vault.public_profile().master_public_key,
        alternate.master.public_key()
    );
}

#[test]
fn passphrase_lock_and_cold_reopen_preserve_public_metadata_identity_and_record_bytes() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("synthetic-vault.json");
    let storage = PassphraseEncryptedStorage::open(&path, b"synthetic-only").unwrap();
    let profile = mixed_profile();
    storage.save_profile(&profile).unwrap();
    let record = std::fs::read(&path).unwrap();
    let mut vault = IdentityVault::open(storage, &profile.id).unwrap();
    let public = vault.public_profile().clone();
    let derived = vault
        .derive_keypair(b"synthetic-continuity")
        .unwrap()
        .public_key();
    vault.lock().unwrap();
    assert_eq!(vault.public_profile(), &public);
    vault
        .unlock(UnlockMethod::Passphrase(b"synthetic-only"))
        .unwrap();
    assert_eq!(vault.public_profile(), &public);
    assert_eq!(
        vault
            .derive_keypair(b"synthetic-continuity")
            .unwrap()
            .public_key(),
        derived
    );
    drop(vault);
    let reopened = IdentityVault::open(
        PassphraseEncryptedStorage::open(&path, b"synthetic-only").unwrap(),
        &profile.id,
    )
    .unwrap();
    assert_eq!(reopened.public_profile(), &public);
    assert_eq!(std::fs::read(&path).unwrap(), record);
}

#[test]
fn retained_absent_instances_have_no_dormant_inline_string_payload() {
    let key = RetainedSlotKey {
        mod_id: "synthetic".into(),
        instance: None,
    };
    assert_eq!(
        std::mem::size_of_val(&key.instance),
        std::mem::size_of::<usize>()
    );
    let cloned = key.clone();
    assert!(cloned.instance.is_none());
    assert_eq!(key, cloned);
}
