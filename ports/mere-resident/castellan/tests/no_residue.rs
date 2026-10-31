// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The no-residue instrument over castellan's lock path (vault lock plan,
//! rulings 1 and 6, L2): the residue tracker (`tests/residue`), watching a
//! resident host lock its vault and run its holder hook.
//!
//! Canaries: the master seed, the vault root, and the record and freshness
//! keys the credential authority derives from the seed (computed before
//! arming). After `lock_vault` returns, with the host and the authority
//! still alive, none may be live or freed uncleared.
//!
//! No libtest harness: the allocator is process-wide.

mod residue;

use castellan::authority::PersonaeHost;
use castellan::resident::{CastellanResident, CredentialSalts};
use castellan::view::VaultProtectionView;
use castellan::custody::sealed_profile_storage::PASSPHRASE_ROOT_FILE;
use castellan::custody::{
    IdentityStorage, IdentityVault, Profile, SealedProfileStorage, UnlockMethod,
};
use personae::{Ed25519Keypair, PersonaId, ProfileId};
use residue::*;

const SALTS: CredentialSalts = CredentialSalts {
    record: b"castellan-residue/records",
    freshness: b"castellan-residue/freshness",
};
const TOTP: &str = "otpauth://totp/Example:mark?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=Example";

fn main() {
    let seed: [u8; 32] = canary_bytes(0);
    positive_control(&seed);
    clear_canaries();

    let dir = tempfile::tempdir().unwrap();
    let vault_dir = dir.path().join("vault");
    std::fs::create_dir_all(&vault_dir).unwrap();
    let root: [u8; 32] = canary_bytes(96);
    let master = Ed25519Keypair::from_seed(seed);
    let record_key = master.derive_child(SALTS.record).to_seed();
    let freshness_key = master.derive_child(SALTS.freshness).to_seed();
    drop(master);
    // Argon2 runs before arming; personae's instrument covers its residue.
    castellan::custody::save_passphrase_root(vault_dir.join(PASSPHRASE_ROOT_FILE), &root, b"lock").unwrap();

    plant(0, "master seed", &seed, true);
    plant(1, "vault root", &root, false);
    plant(2, "record key", &record_key, false);
    plant(3, "freshness key", &freshness_key, false);
    arm();
    phase(1);
    let storage = SealedProfileStorage::open_with_key(&vault_dir, root);
    storage
        .save_profile(&Profile::new(
            ProfileId("work".into()),
            "Work",
            Ed25519Keypair::from_seed(seed),
        ))
        .unwrap();
    let host = PersonaeHost::new(
        IdentityVault::open(storage, &ProfileId("work".into())).unwrap(),
        None,
        VaultProtectionView::OsProtected,
    );
    phase(2);
    let resident = CastellanResident::claim_derived(
        dir.path().join("records"),
        dir.path().join("freshness"),
        &host,
        SALTS,
    )
    .unwrap();
    host.register_lock_holder(resident.lock_holder(SALTS));
    let persona = PersonaId::new();
    resident
        .otp_items(persona)
        .import_otpauth_uri(TOTP)
        .unwrap();
    let listed = resident.items(persona).list().unwrap();
    phase(3);
    host.lock_vault().unwrap();
    let (hits, overflow) = disarm();
    let mut report = Report { failures: 0 };
    report.check(
        "castellan resident locked",
        &["setup", "open host", "claim and write items", "lock"],
        &hits,
        overflow,
    );

    // The lock is real and reversible: refused while locked, the same items
    // after a native unlock.
    assert!(resident.is_locked());
    assert!(resident.items(persona).list().is_err());
    host.unlock_vault(UnlockMethod::Passphrase(b"lock")).unwrap();
    assert_eq!(resident.items(persona).list().unwrap(), listed);
    drop((host, resident));
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
