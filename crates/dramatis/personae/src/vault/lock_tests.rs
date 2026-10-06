// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Vault lock, L1 (vault lock plan, rulings 13, 14, 25 to 30).
//!
//! Each guard has its own test, set up so that the guard under test is the
//! only thing standing: a vault-level test unlocks the storage behind the
//! vault's back, so a missing vault guard cannot hide behind the storage's.

use std::path::Path;

use tempfile::tempdir;

use super::*;
use crate::sealed_profile_storage::PASSPHRASE_ROOT_FILE;
use crate::unlock::OsPresence;
use crate::{PassphraseEncryptedStorage, SealedProfileStorage, SealedRecordStorage};

const ROOT: [u8; 32] = [0x6b; 32];
const PASSPHRASE: &[u8] = b"correct horse";

fn nostr_slot(byte: u8) -> IdentitySlot {
    IdentitySlot::Direct {
        kind: "nostr".to_string(),
        payload: SecretBytes::new(vec![byte; 32]),
        lineage: CredentialLineage::LocallyGeneratedExternallyRegistered,
        unlock_tier: UnlockTier::Session,
    }
}

fn profile(id: &str, seed: u8) -> Profile {
    let mut profile = Profile::new(
        ProfileId(id.into()),
        id.to_uppercase(),
        Ed25519Keypair::from_seed([seed; 32]),
    );
    profile
        .slots
        .insert(ProtocolKey::new("nostr", None), nostr_slot(seed));
    profile
}

/// A sealed vault in `dir`, its root enrolled under [`PASSPHRASE`], holding
/// `work` and `personal`.
fn sealed_vault(dir: &Path) -> IdentityVault<SealedProfileStorage> {
    std::fs::create_dir_all(dir).unwrap();
    crate::save_passphrase_root(dir.join(PASSPHRASE_ROOT_FILE), &ROOT, PASSPHRASE).unwrap();
    let storage = SealedProfileStorage::open_with_key(dir, ROOT);
    storage.save_profile(&profile("personal", 0x22)).unwrap();
    storage.save_profile(&profile("work", 0x11)).unwrap();
    IdentityVault::open(storage, &ProfileId("work".into())).unwrap()
}

fn locked(dir: &Path) -> IdentityVault<SealedProfileStorage> {
    let mut vault = sealed_vault(dir);
    vault.lock().unwrap();
    vault
}

/// Locked vault, storage unlocked behind its back: only the vault's own
/// guards remain.
fn locked_over_open_storage(dir: &Path) -> IdentityVault<SealedProfileStorage> {
    let vault = locked(dir);
    vault
        .storage()
        .unlock(UnlockMethod::Passphrase(PASSPHRASE))
        .unwrap();
    assert!(!vault.storage().is_locked());
    vault
}

fn is_locked_error<T>(result: Result<T, IdentityError>) -> bool {
    matches!(result, Err(IdentityError::Locked))
}

// ─── The vault's guards ───────────────────────────────────────────────────

#[test]
fn current_profile_is_refused_while_locked() {
    let dir = tempdir().unwrap();
    let vault = locked_over_open_storage(dir.path());
    assert!(is_locked_error(vault.current_profile()));
}

#[test]
fn slot_is_refused_while_locked() {
    let dir = tempdir().unwrap();
    let vault = locked_over_open_storage(dir.path());
    assert!(is_locked_error(
        vault.slot(&ProtocolKey::new("nostr", None))
    ));
}

#[test]
fn add_slot_is_refused_while_locked() {
    let dir = tempdir().unwrap();
    let mut vault = locked_over_open_storage(dir.path());
    assert!(is_locked_error(
        vault.add_slot(ProtocolKey::new("irc", None), nostr_slot(9))
    ));
    let stored = vault
        .storage()
        .load_profile(&ProfileId("work".into()))
        .unwrap();
    assert!(!stored.has_slot(&ProtocolKey::new("irc", None)));
}

#[test]
fn remove_slot_is_refused_while_locked() {
    let dir = tempdir().unwrap();
    let mut vault = locked_over_open_storage(dir.path());
    assert!(is_locked_error(
        vault.remove_slot(&ProtocolKey::new("nostr", None))
    ));
    let stored = vault
        .storage()
        .load_profile(&ProfileId("work".into()))
        .unwrap();
    assert!(stored.has_slot(&ProtocolKey::new("nostr", None)));
}

/// Ruling 14: refused even though the storage could load the target.
#[test]
fn switch_profile_is_refused_while_locked() {
    let dir = tempdir().unwrap();
    let mut vault = locked_over_open_storage(dir.path());
    assert!(is_locked_error(
        vault.switch_profile(&ProfileId("personal".into()))
    ));
    assert_eq!(vault.profile_id().0, "work");
}

#[test]
fn derivation_is_refused_while_locked() {
    let dir = tempdir().unwrap();
    let vault = locked_over_open_storage(dir.path());
    assert!(is_locked_error(vault.derive_keypair(b"salt")));
}

#[test]
fn attestation_is_refused_while_locked() {
    let dir = tempdir().unwrap();
    let vault = locked_over_open_storage(dir.path());
    assert!(is_locked_error(vault.attest_derived_key(b"salt")));
}

/// Ruling 11: the public face stays readable.
#[test]
fn the_public_view_survives_a_lock() {
    let dir = tempdir().unwrap();
    let mut vault = sealed_vault(dir.path());
    let before = vault.public_profile().clone();
    let master = vault.master_public_key();
    vault.lock().unwrap();
    assert!(vault.is_locked());
    assert_eq!(vault.public_profile(), &before);
    assert_eq!(vault.master_public_key(), master);
    assert_eq!(before.slots.len(), 1);
    assert_eq!(before.slots[0].kind, "nostr");
}

/// Production vaults hold `Box<dyn IdentityStorage>` (the bootstrap ladder)
/// or a borrowed storage; the delegates must pass the lock through.
#[test]
fn boxed_and_borrowed_storages_lock_through_their_delegates() {
    let dir = tempdir().unwrap();
    let sealed = sealed_vault(dir.path());
    let id = ProfileId("work".into());
    let storage: Box<dyn IdentityStorage> =
        Box::new(SealedProfileStorage::open_with_key(dir.path(), ROOT));
    let mut boxed = IdentityVault::open(storage, &id).unwrap();
    boxed.lock().unwrap();
    assert!(boxed.storage().is_locked());
    assert!(is_locked_error(boxed.storage().load_profile(&id)));

    let inner = SealedProfileStorage::open_with_key(dir.path(), ROOT);
    let mut borrowed = IdentityVault::open(&inner, &id).unwrap();
    borrowed.lock().unwrap();
    assert!(inner.is_locked());
    assert!(is_locked_error(inner.load_profile(&id)));
    drop(sealed);
}

// ─── Ruling 27: no lock nobody can undo ───────────────────────────────────

#[test]
fn lock_is_refused_without_an_unlock_method() {
    let dir = tempdir().unwrap();
    // A sealed vault with no enrolled passphrase and no OS-held root.
    let storage = SealedProfileStorage::open_with_key(dir.path(), ROOT);
    let mut vault = IdentityVault::with_profile(storage, profile("work", 0x11));
    assert!(!vault.unlock_methods().any());
    assert!(vault.lock().is_err());
    assert!(!vault.is_locked());
    assert!(!vault.storage().is_locked());
    assert!(vault.current_profile().is_ok());

    let mut fixture = IdentityVault::with_profile(InMemoryStorage::new(), profile("work", 1));
    assert!(fixture.lock().is_err());
    assert!(!fixture.is_locked());
}

#[test]
fn lock_and_unlock_are_idempotent() {
    let dir = tempdir().unwrap();
    let mut vault = sealed_vault(dir.path());
    vault.unlock(UnlockMethod::Passphrase(b"ignored")).unwrap();
    vault.lock().unwrap();
    vault.lock().unwrap();
    assert!(vault.is_locked());
}

// ─── Unlock ───────────────────────────────────────────────────────────────

fn slot_bytes(vault: &IdentityVault<SealedProfileStorage>) -> Vec<u8> {
    match vault
        .slot(&ProtocolKey::new("nostr", None))
        .unwrap()
        .unwrap()
    {
        IdentitySlot::Direct { payload, .. } => payload.as_slice().to_vec(),
        IdentitySlot::Bootstrap { bootstrap, .. } => bootstrap.as_slice().to_vec(),
    }
}

#[test]
fn a_passphrase_unlock_restores_the_same_slots_and_signatures() {
    let dir = tempdir().unwrap();
    let mut vault = sealed_vault(dir.path());
    let message = b"the same words, signed twice";
    let slots_before = slot_bytes(&vault);
    let master_signature = vault.current_profile().unwrap().master.sign(message);
    let derived_signature = vault.derive_keypair(b"knot").unwrap().sign(message);

    vault.lock().unwrap();
    assert!(vault.storage().is_locked());
    vault.unlock(UnlockMethod::Passphrase(PASSPHRASE)).unwrap();

    assert!(!vault.is_locked());
    assert_eq!(slot_bytes(&vault), slots_before);
    let master = vault.current_profile().unwrap().master.clone();
    assert_eq!(master.sign(message), master_signature);
    assert!(vault.master_public_key().verify(message, &master_signature));
    let derived = vault.derive_keypair(b"knot").unwrap();
    assert_eq!(derived.sign(message), derived_signature);
    assert!(derived.public_key().verify(message, &derived_signature));
}

#[test]
fn a_wrong_passphrase_leaves_the_vault_locked() {
    let dir = tempdir().unwrap();
    let mut vault = locked(dir.path());
    assert!(vault.unlock(UnlockMethod::Passphrase(b"wrong")).is_err());
    assert!(vault.is_locked());
    assert!(vault.storage().is_locked());
    assert!(is_locked_error(vault.current_profile()));
    // The right one still works afterwards.
    vault.unlock(UnlockMethod::Passphrase(PASSPHRASE)).unwrap();
    assert_eq!(vault.current_profile().unwrap().id.0, "work");
}

#[test]
fn os_presence_reads_the_root_back_from_the_os_store() {
    let dir = tempdir().unwrap();
    let Some(storage) = SealedProfileStorage::open_auto_os(dir.path()).unwrap() else {
        // No AutoOs backend on this platform; nothing to read back.
        return;
    };
    storage.save_profile(&profile("work", 0x33)).unwrap();
    storage.lock();
    assert!(is_locked_error(
        storage.load_profile(&ProfileId("work".into()))
    ));
    storage
        .unlock(UnlockMethod::OsPresence(OsPresence::mint()))
        .unwrap();
    let back = storage.load_profile(&ProfileId("work".into())).unwrap();
    assert_eq!(back.master.to_seed(), [0x33; 32]);
}

#[test]
fn a_passphrase_vault_takes_no_os_presence() {
    let dir = tempdir().unwrap();
    let storage = PassphraseEncryptedStorage::open(dir.path().join("v.json"), PASSPHRASE).unwrap();
    storage.save_profile(&profile("work", 0x44)).unwrap();
    storage.lock();
    assert!(
        storage
            .unlock(UnlockMethod::OsPresence(OsPresence::mint()))
            .is_err()
    );
    assert!(storage.is_locked());
    assert!(storage.unlock(UnlockMethod::Passphrase(b"wrong")).is_err());
    assert!(storage.is_locked());
    storage
        .unlock(UnlockMethod::Passphrase(PASSPHRASE))
        .unwrap();
    assert!(storage.load_profile(&ProfileId("work".into())).is_ok());
}

// ─── The storages' guards ─────────────────────────────────────────────────

fn every_profile_call_is_refused(storage: &dyn IdentityStorage) {
    let id = ProfileId("work".into());
    assert!(is_locked_error(storage.load_profile(&id)), "load");
    assert!(
        is_locked_error(storage.save_profile(&profile("work", 0x55))),
        "save"
    );
    assert!(is_locked_error(storage.list_profiles()), "list");
    assert!(is_locked_error(storage.delete_profile(&id)), "delete");
}

#[test]
fn a_locked_sealed_storage_refuses_every_profile_call() {
    let dir = tempdir().unwrap();
    let vault = locked(dir.path());
    every_profile_call_is_refused(vault.storage());
}

#[test]
fn a_locked_passphrase_storage_refuses_every_profile_call() {
    let dir = tempdir().unwrap();
    let storage = PassphraseEncryptedStorage::open(dir.path().join("v.json"), PASSPHRASE).unwrap();
    storage.save_profile(&profile("work", 0x44)).unwrap();
    storage.lock();
    every_profile_call_is_refused(&storage);
    storage
        .unlock(UnlockMethod::Passphrase(PASSPHRASE))
        .unwrap();
    assert!(storage.load_profile(&ProfileId("work".into())).is_ok());
}

/// Ruling 30: one key cell for every clone, absent records included.
#[test]
fn sealed_record_clones_share_one_lock() {
    let dir = tempdir().unwrap();
    let left = SealedRecordStorage::open_with_key(dir.path(), ROOT);
    let right = left.clone();
    left.save_record("a.json", &1u32).unwrap();
    right.lock();
    assert!(left.is_locked());
    assert!(is_locked_error(left.load_record::<u32>("a.json")));
    assert!(is_locked_error(left.load_record::<u32>("absent.json")));
    assert!(is_locked_error(left.save_record("b.json", &2u32)));
    assert!(is_locked_error(left.delete_record("a.json")));
    left.unlock(ROOT, None).unwrap();
    assert_eq!(right.load_record::<u32>("a.json").unwrap(), Some(1));
}

#[test]
fn an_authoritative_store_needs_its_freshness_key_back() {
    let dir = tempdir().unwrap();
    let store = SealedRecordStorage::claim_with_file_freshness(
        dir.path().join("records"),
        ROOT,
        dir.path().join("freshness"),
        [0x7c; 32],
    )
    .unwrap();
    store.save_record("a.json", &1u32).unwrap();
    store.lock();
    assert!(is_locked_error(store.load_record::<u32>("a.json")));
    assert!(store.unlock(ROOT, None).is_err());
    assert!(store.is_locked());
    store.unlock(ROOT, Some([0x7c; 32])).unwrap();
    assert_eq!(store.load_record::<u32>("a.json").unwrap(), Some(1));

    let plain = SealedRecordStorage::open_with_key(dir.path().join("plain"), ROOT);
    plain.lock();
    assert!(plain.unlock(ROOT, Some([0x7c; 32])).is_err());
}

// ─── Ruling 39: enrolment lifts ruling 27's refusal ───────────────────────

/// A sealed vault with no enrolled passphrase and no OS-held root: the
/// device ruling 39 names, where nothing could unlock it.
fn unenrolled(dir: &Path) -> IdentityVault<SealedProfileStorage> {
    let storage = SealedProfileStorage::open_with_key(dir, ROOT);
    storage.save_profile(&profile("work", 0x11)).unwrap();
    IdentityVault::open(storage, &ProfileId("work".into())).unwrap()
}

#[test]
fn enrolment_then_a_passphrase_unlock_works_end_to_end() {
    let dir = tempdir().unwrap();
    let mut vault = unenrolled(dir.path());
    assert!(vault.lock().is_err(), "ruling 27 refuses before enrolment");
    let before = slot_bytes(&vault);

    vault.enroll_passphrase(PASSPHRASE).unwrap();
    assert!(vault.unlock_methods().passphrase);
    // The same root, wrapped: it unwraps to the key the records use.
    let wrapped = crate::load_passphrase_root(dir.path().join(PASSPHRASE_ROOT_FILE), PASSPHRASE)
        .unwrap()
        .unwrap();
    assert_eq!(wrapped, ROOT);

    vault.lock().unwrap();
    assert!(vault.storage().is_locked());
    assert!(vault.unlock(UnlockMethod::Passphrase(b"wrong")).is_err());
    vault.unlock(UnlockMethod::Passphrase(PASSPHRASE)).unwrap();
    assert_eq!(slot_bytes(&vault), before);
}

#[test]
fn enrolment_is_refused_while_locked() {
    let dir = tempdir().unwrap();
    let vault = locked_over_open_storage(dir.path());
    assert!(is_locked_error(vault.enroll_passphrase(b"another")));
}

#[test]
fn a_locked_storage_cannot_be_enrolled() {
    let dir = tempdir().unwrap();
    let storage = SealedProfileStorage::open_with_key(dir.path(), ROOT);
    storage.lock();
    assert!(is_locked_error(storage.enroll_passphrase(PASSPHRASE)));
    assert!(!dir.path().join(PASSPHRASE_ROOT_FILE).exists());
}

/// An enrolment is never silently replaced; changing it is
/// `change_passphrase`, which needs the old one.
#[test]
fn a_second_enrolment_is_refused_and_the_first_kept() {
    let dir = tempdir().unwrap();
    let vault = unenrolled(dir.path());
    vault.enroll_passphrase(PASSPHRASE).unwrap();
    assert!(vault.enroll_passphrase(b"usurper").is_err());
    assert!(vault.enroll_passphrase(b"").is_err());
    let path = dir.path().join(PASSPHRASE_ROOT_FILE);
    assert!(crate::load_passphrase_root(&path, b"usurper").is_err());
    assert_eq!(
        crate::load_passphrase_root(&path, PASSPHRASE).unwrap(),
        Some(ROOT)
    );
}

#[test]
fn delegates_and_other_backends_answer_enrolment() {
    let dir = tempdir().unwrap();
    let inner = SealedProfileStorage::open_with_key(dir.path().join("boxed"), ROOT);
    let boxed: Box<dyn IdentityStorage> = Box::new(inner);
    boxed.enroll_passphrase(PASSPHRASE).unwrap();
    assert!(boxed.unlock_methods().passphrase);

    let borrowed_inner = SealedProfileStorage::open_with_key(dir.path().join("borrowed"), ROOT);
    let borrowed: &dyn IdentityStorage = &borrowed_inner;
    borrowed.enroll_passphrase(PASSPHRASE).unwrap();
    assert!(borrowed_inner.unlock_methods().passphrase);

    assert!(InMemoryStorage::new().enroll_passphrase(PASSPHRASE).is_err());
    let passphrase_vault =
        PassphraseEncryptedStorage::open(dir.path().join("v.json"), PASSPHRASE).unwrap();
    assert!(passphrase_vault.enroll_passphrase(b"second").is_err());
}

#[test]
fn opening_an_existing_auto_os_root_never_mints_one() {
    let dir = tempdir().unwrap();
    assert!(
        SealedProfileStorage::open_existing_auto_os(dir.path())
            .unwrap()
            .is_none()
    );
    assert!(std::fs::read_dir(dir.path()).unwrap().next().is_none());
    if SealedProfileStorage::open_auto_os(dir.path())
        .unwrap()
        .is_some()
    {
        assert!(
            SealedProfileStorage::open_existing_auto_os(dir.path())
                .unwrap()
                .is_some()
        );
    }
}
