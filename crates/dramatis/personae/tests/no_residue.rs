// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The no-residue instrument (vault lock plan, ruling 6).
//!
//! A tracking global allocator, test-only, owns this binary's heap. Canary
//! keys are planted in a profile; while armed, every freed block is scanned
//! before it reaches the system allocator, and at each scenario's end every
//! block still live is scanned too. A hit is a canary (any 16-byte window of
//! it, raw or as serde_json's decimal array) left in memory nobody owns.
//!
//! The positive control runs first in the same process: an uncleared Vec
//! must be found, a `Zeroizing` one must not. Stack copies and memory the OS
//! allocates (DPAPI's `LocalAlloc`) are outside what this can see.
//!
//! The lock scenarios scan what a locked vault keeps live, with the vault
//! still alive: no seed, payload or storage key may remain. The passphrase
//! one also plants a window of Argon2's final memory block, which with the
//! salt recomputes the passphrase key (ruling 33).
//!
//! No libtest harness: the allocator is process-wide, and one thread keeps
//! the scan free of races.

mod residue;

use personae::sealed_profile_storage::PASSPHRASE_ROOT_FILE;
use personae::{
    CredentialLineage, Ed25519Keypair, IdentitySlot, IdentityStorage, IdentityVault,
    PassphraseEncryptedStorage, Profile, ProfileId, ProtocolKey, SealedProfileStorage,
    SealedRecordStorage, SecretBytes, UnlockMethod, UnlockTier,
};
use zeroize::Zeroizing;

use residue::*;

// ─── Scenarios ────────────────────────────────────────────────────────────

fn canary_profile(seed: [u8; 32], payload: &[u8]) -> Profile {
    let mut profile = Profile::new(
        ProfileId("work".into()),
        "Work",
        Ed25519Keypair::from_seed(seed),
    );
    profile.slots.insert(
        ProtocolKey::new("canary", None),
        IdentitySlot::Direct {
            kind: "canary".into(),
            payload: SecretBytes::new(payload.to_vec()),
            lineage: CredentialLineage::LocallyDerived,
            unlock_tier: UnlockTier::Session,
        },
    );
    profile
}

const STORAGE_PHASES: [&str; 6] = ["setup", "save", "open", "switch", "list", "drop"];

/// Save, load, switch and list through one backend, then drop everything.
fn storage_round_trip<S: IdentityStorage>(storage: S, seed: [u8; 32], payload: &[u8]) {
    let profile = canary_profile(seed, payload);
    phase(1);
    storage.save_profile(&profile).unwrap();
    drop(profile);
    phase(2);
    let mut vault = IdentityVault::open(storage, &ProfileId("work".into())).unwrap();
    assert_eq!(vault.current_profile().unwrap().master.to_seed(), seed);
    phase(3);
    vault.switch_profile(&ProfileId("work".into())).unwrap();
    phase(4);
    let listed = vault.storage().list_profiles().unwrap();
    assert_eq!(listed.len(), 1);
    phase(5);
    drop(listed);
    drop(vault);
}

fn main() {
    let mut report = Report { failures: 0 };
    let seed: [u8; 32] = canary_bytes(0);
    let payload: [u8; 48] = canary_bytes(32);

    // Positive control: the instrument must see what it claims to see.
    positive_control(&seed);

    plant(1, "slot payload", &payload, true);

    // SealedProfileStorage, the AutoOs desktop backend (fixed root here).
    let dir = tempfile::tempdir().unwrap();
    let sealed = SealedProfileStorage::open_with_key(dir.path().join("sealed"), [0x51; 32]);
    arm();
    storage_round_trip(sealed, seed, &payload);
    let (hits, overflow) = disarm();
    report.check("sealed profile storage", &STORAGE_PHASES, &hits, overflow);

    // PassphraseEncryptedStorage, the portable passphrase vault. Reopening
    // checks the passphrase by decrypting a stored profile.
    let path = dir.path().join("vault.json");
    let pass = PassphraseEncryptedStorage::open(&path, b"canary").unwrap();
    arm();
    storage_round_trip(pass, seed, &payload);
    phase(6);
    drop(PassphraseEncryptedStorage::open(&path, b"canary").unwrap());
    let (hits, overflow) = disarm();
    let phases = [&STORAGE_PHASES[..], &["reopen"]].concat();
    report.check("passphrase storage", &phases, &hits, overflow);

    sealed_lock(&mut report, dir.path(), seed, &payload);
    passphrase_lock(&mut report, dir.path(), seed, &payload);
    authoritative_lock(&mut report, dir.path());

    // The DPAPI-held AutoOs root, read back from disk.
    #[cfg(windows)]
    {
        let path = dir.path().join("auto-unlock-root.json");
        let root = Zeroizing::new(
            personae::load_or_create_auto_unlock_root(&path)
                .unwrap()
                .unwrap(),
        );
        plant(2, "DPAPI root", root.as_ref(), false);
        arm();
        phase(1);
        let again = Zeroizing::new(
            personae::load_existing_auto_unlock_root(&path)
                .unwrap()
                .unwrap(),
        );
        assert_eq!(*again, *root);
        drop(again);
        let (hits, overflow) = disarm();
        report.check("DPAPI root", &["setup", "load"], &hits, overflow);
        forget_canary(2);
    }

    // The agent's listing, which mints a certificate from the master.
    #[cfg(feature = "agent")]
    agent_listing(&mut report, dir.path(), seed, &payload);

    clear_canaries();
    if report.failures > 0 {
        println!(
            "no-residue: FAILED ({} scenario(s) left key residue)",
            report.failures
        );
        std::process::exit(1);
    }
    println!("no-residue: ok");
}

/// A sealed vault, its root enrolled under a passphrase, opened and locked
/// with everything allocated while armed; the live scan runs with the
/// locked vault still alive.
fn sealed_lock(report: &mut Report, root: &std::path::Path, seed: [u8; 32], payload: &[u8]) {
    let dir = root.join("sealed-lock");
    std::fs::create_dir_all(&dir).unwrap();
    let key: [u8; 32] = canary_bytes(96);
    personae::save_passphrase_root(dir.join(PASSPHRASE_ROOT_FILE), &key, b"lock").unwrap();
    plant(2, "sealed root key", &key, false);
    arm();
    phase(1);
    let storage = SealedProfileStorage::open_with_key(&dir, key);
    storage
        .save_profile(&canary_profile(seed, payload))
        .unwrap();
    phase(2);
    let mut vault = IdentityVault::open(storage, &ProfileId("work".into())).unwrap();
    phase(3);
    vault.lock().unwrap();
    let (hits, overflow) = disarm();
    report.check(
        "sealed vault locked",
        &["setup", "save", "open", "lock"],
        &hits,
        overflow,
    );
    vault.unlock(UnlockMethod::Passphrase(b"lock")).unwrap();
    assert_eq!(vault.current_profile().unwrap().master.to_seed(), seed);
    drop(vault);
    forget_canary(2);
}

/// A passphrase vault reopened (Argon2 runs) and locked while armed. The
/// test derives the same key and final block first, unarmed, to plant them.
fn passphrase_lock(report: &mut Report, root: &std::path::Path, seed: [u8; 32], payload: &[u8]) {
    let path = root.join("lock-vault.json");
    PassphraseEncryptedStorage::open(&path, b"lock")
        .unwrap()
        .save_profile(&canary_profile(seed, payload))
        .unwrap();
    let file: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let salt: Vec<u8> = file["salt"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b.as_u64().unwrap() as u8)
        .collect();
    let argon = argon2::Argon2::default();
    let mut blocks = vec![argon2::Block::default(); argon.params().block_count()];
    let mut kek = Zeroizing::new([0u8; 32]);
    argon
        .hash_password_into_with_memory(b"lock", &salt, kek.as_mut(), &mut blocks[..])
        .unwrap();
    let last: Vec<u8> = blocks.last().unwrap().as_ref()[..8]
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect();
    drop(blocks);
    plant(2, "passphrase KEK", kek.as_ref(), false);
    plant(3, "argon2 final block", &last, false);
    arm();
    phase(1);
    let storage = PassphraseEncryptedStorage::open(&path, b"lock").unwrap();
    phase(2);
    let mut vault = IdentityVault::open(storage, &ProfileId("work".into())).unwrap();
    phase(7);
    vault
        .remove_slot(&ProtocolKey::new("absent", None))
        .unwrap();
    vault
        .add_slot(
            ProtocolKey::new("second", None),
            IdentitySlot::Direct {
                kind: "second".into(),
                payload: SecretBytes::new(vec![7; 8]),
                lineage: CredentialLineage::LocallyDerived,
                unlock_tier: UnlockTier::Session,
            },
        )
        .unwrap();
    phase(3);
    vault.lock().unwrap();
    phase(4);
    vault
        .storage()
        .unlock(UnlockMethod::Passphrase(b"lock"))
        .unwrap();
    phase(6);
    vault.unlock(UnlockMethod::Passphrase(b"lock")).unwrap();
    phase(5);
    vault.lock().unwrap();
    let (hits, overflow) = disarm();
    report.check(
        "passphrase vault locked",
        &[
            "setup",
            "open storage",
            "open vault",
            "lock",
            "storage unlock",
            "relock",
            "vault unlock",
            "save",
        ],
        &hits,
        overflow,
    );
    drop(vault);
    forget_canary(2);
    forget_canary(3);
}

/// Castellan's shape: an authoritative record store with its own
/// freshness key (ruling 1). After a lock, neither key may stay live.
fn authoritative_lock(report: &mut Report, root: &std::path::Path) {
    let record_key: [u8; 32] = canary_bytes(128);
    let freshness_key: [u8; 32] = canary_bytes(160);
    plant(2, "record key", &record_key, false);
    plant(3, "freshness key", &freshness_key, false);
    arm();
    phase(1);
    let store = SealedRecordStorage::claim_with_file_freshness(
        root.join("auth-records"),
        record_key,
        root.join("auth-freshness"),
        freshness_key,
    )
    .unwrap();
    let clone = store.clone();
    store.save_record("item.json", &7u32).unwrap();
    phase(2);
    clone.lock();
    let (hits, overflow) = disarm();
    report.check(
        "authoritative store locked",
        &["setup", "claim and save", "lock"],
        &hits,
        overflow,
    );
    store.unlock(record_key, Some(freshness_key)).unwrap();
    assert_eq!(store.load_record::<u32>("item.json").unwrap(), Some(7));
    drop((store, clone));
    forget_canary(2);
    forget_canary(3);
}

#[cfg(feature = "agent")]
fn agent_listing(report: &mut Report, root: &std::path::Path, seed: [u8; 32], payload: &[u8]) {
    use ssh_agent_lib::agent::Session;
    use std::future::Future;
    use std::task::{Context, Poll, Waker};

    let storage = SealedProfileStorage::open_with_key(root.join("agent"), [0x52; 32]);
    let mut profile = canary_profile(seed, payload);
    let ssh = ssh_key::private::PrivateKey::from(ssh_key::private::Ed25519Keypair::from_seed(
        &[0x33; 32],
    ));
    profile.slots.insert(
        personae::agent::protocol_key_for(&ssh),
        personae::ssh_slot::slot_for(&ssh, UnlockTier::Session).unwrap(),
    );
    drop(ssh);
    let mut agent = personae::agent::VaultAgent::new(IdentityVault::with_profile(storage, profile));
    arm();
    phase(1);
    {
        let mut listing = std::pin::pin!(agent.request_identities());
        let Poll::Ready(identities) = listing
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        else {
            panic!("the listing awaits nothing");
        };
        // A certified key is listed twice: certificate, then bare key.
        assert_eq!(identities.unwrap().len(), 2);
    }
    phase(2);
    drop(agent);
    let (hits, overflow) = disarm();
    report.check("agent listing", &["setup", "list", "drop"], &hits, overflow);
}
