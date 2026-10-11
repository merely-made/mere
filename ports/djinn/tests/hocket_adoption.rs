// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Real Windows DPAPI migration receipts over disposable synthetic records.
//! No environment override or installed Hocket/djinn directory is used.

#![cfg(windows)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use castellan::authority::PersonaeHost;
use castellan::custody::{
    IdentityStorage, IdentityVault, Profile, SealedIdentityProvider, SealedProfileStorage,
    load_or_create_auto_unlock_root,
};
use djinn::hocket_adoption::{APART_PROFILE, MARKER, RECORD, adopt_from};
use graphshell::identity::VaultProtectionView;
use personae::{Ed25519Keypair, IdentityProvider, ProfileId, SealedRecordStorage};

const FAMILY: [u8; 32] = [0x42; 32];
const VAULT_KEY: [u8; 32] = [0x73; 32];

fn family_host(root: &Path) -> PersonaeHost<SealedProfileStorage> {
    let storage = SealedProfileStorage::open_with_key(root, VAULT_KEY);
    if !root.join("passphrase-root.json").exists() {
        storage
            .enroll_passphrase(b"synthetic adoption receipt")
            .unwrap();
    }
    let id = ProfileId("family".into());
    let profile = match storage.load_profile(&id) {
        Ok(profile) => profile,
        Err(_) => {
            let profile = Profile::new(id, "Family", Ed25519Keypair::from_seed(FAMILY));
            storage.save_profile(&profile).unwrap();
            profile
        },
    };
    PersonaeHost::new(
        IdentityVault::with_profile(storage, profile),
        None,
        VaultProtectionView::Ephemeral,
    )
}

fn legacy_identity(root: &Path, path: &str, seed: [u8; 32]) -> [u8; 32] {
    let wrap = root.join("personae/auto-unlock-root.json");
    let key = load_or_create_auto_unlock_root(&wrap)
        .unwrap()
        .expect("real Windows DPAPI root");
    // Assert that this receipt used DPAPI, not an unsealed stand-in.
    let envelope: serde_json::Value =
        serde_json::from_slice(&std::fs::read(wrap).unwrap()).unwrap();
    assert_eq!(envelope["backend"], "dpapi");
    let records = SealedRecordStorage::open_with_key(root.join("personae/records"), key);
    records
        .save_record(
            path,
            &serde_json::json!({
                "format_version": 1,
                "master_seed": seed,
            }),
        )
        .unwrap();
    let identity = SealedIdentityProvider::load_or_create(&records, path).unwrap();
    identity.master_public_key().to_bytes()
}

fn bytes(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(root: &Path, directory: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, result);
            } else {
                result.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    if root.exists() {
        walk(root, root, &mut result);
    }
    result
}

#[test]
fn dpapi_adoption_preserves_contact_token_family_and_records_across_restart() {
    let temp = tempfile::tempdir().unwrap();
    let data = temp.path().join("hocket-data");
    let vault = temp.path().join("resident-vault");
    let original = legacy_identity(&data, RECORD, [0x61; 32]);
    let before = bytes(&data);
    let host = family_host(&vault);
    let family_before = bytes(&vault);
    let adopted = adopt_from(&host, &data)
        .unwrap()
        .expect("existing legacy identity adopted");
    assert_eq!(adopted.profile, APART_PROFILE);
    let provider = host
        .profile_provider(&ProfileId(adopted.profile.clone()))
        .unwrap();
    assert_eq!(
        provider.master_public_key().to_bytes(),
        original,
        "contact token must not change"
    );
    let token = original
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(adopted.public_key, token);
    assert_eq!(host.current_profile_id(), ProfileId("family".into()));
    assert_eq!(
        host.master_public_key(),
        Ed25519Keypair::from_seed(FAMILY).public_key()
    );
    for (path, content) in &family_before {
        assert_eq!(
            std::fs::read(vault.join(path)).unwrap(),
            *content,
            "existing family vault file must not change"
        );
    }
    for (path, content) in &before {
        assert_eq!(
            std::fs::read(data.join(path)).unwrap(),
            *content,
            "legacy DPAPI wrapper and sealed record must not change"
        );
    }
    let marker: serde_json::Value =
        serde_json::from_slice(&std::fs::read(data.join(MARKER)).unwrap()).unwrap();
    assert_eq!(
        marker.as_object().unwrap().len(),
        2,
        "forwarding note contains only profile and public key"
    );
    assert_eq!(marker["public_key"], token);
    let after_vault = bytes(&vault);
    let after_data = bytes(&data);
    assert_eq!(adopt_from(&host, &data).unwrap().unwrap(), adopted);
    assert_eq!(
        bytes(&vault),
        after_vault,
        "repeated adoption does not rewrite or duplicate profiles"
    );
    assert_eq!(
        bytes(&data),
        after_data,
        "repeated adoption leaves identical public forwarding bytes"
    );
    drop(provider);
    drop(host);
    let restarted = family_host(&vault);
    assert_eq!(adopt_from(&restarted, &data).unwrap().unwrap(), adopted);
    assert_eq!(
        restarted
            .profile_provider(&ProfileId(adopted.profile))
            .unwrap()
            .master_public_key()
            .to_bytes(),
        original
    );
    assert_eq!(
        bytes(&vault),
        after_vault,
        "restart finds the already-adopted persona"
    );
    assert_eq!(bytes(&data), after_data);
}

#[test]
fn dpapi_adoption_reuses_a_persona_already_holding_the_same_identity() {
    let temp = tempfile::tempdir().unwrap();
    let data = temp.path().join("hocket-data");
    let vault = temp.path().join("resident-vault");
    let original = legacy_identity(&data, "strophe/local-identity.json", FAMILY);
    let host = family_host(&vault);
    let before = bytes(&vault);
    let adopted = adopt_from(&host, &data).unwrap().unwrap();
    assert_eq!(adopted.profile, "family");
    assert_eq!(host.master_public_key().to_bytes(), original);
    assert!(!host.has_profile(&ProfileId(APART_PROFILE.into())).unwrap());
    assert_eq!(bytes(&vault), before);
}

#[test]
fn dpapi_adoption_refuses_a_taken_apart_name_without_overwriting_any_identity() {
    let temp = tempfile::tempdir().unwrap();
    let data = temp.path().join("hocket-data");
    let vault = temp.path().join("resident-vault");
    legacy_identity(&data, RECORD, [0x61; 32]);
    let host = family_host(&vault);
    host.import_profile(
        &ProfileId(APART_PROFILE.into()),
        "Already held",
        Ed25519Keypair::from_seed([0x19; 32]),
    )
    .unwrap();
    let before_vault = bytes(&vault);
    let before_data = bytes(&data);
    assert!(adopt_from(&host, &data).is_err());
    assert_eq!(bytes(&vault), before_vault);
    assert_eq!(bytes(&data), before_data);
    assert!(!data.join(MARKER).exists());
}

#[test]
fn dpapi_adoption_while_locked_does_not_open_or_write_the_legacy_identity() {
    let temp = tempfile::tempdir().unwrap();
    let data = temp.path().join("hocket-data");
    let vault = temp.path().join("resident-vault");
    legacy_identity(&data, RECORD, [0x61; 32]);
    let host = family_host(&vault);
    host.lock_vault().unwrap();
    let before_vault = bytes(&vault);
    let before_data = bytes(&data);
    assert!(matches!(
        adopt_from(&host, &data),
        Err(personae::IdentityError::Locked)
    ));
    assert_eq!(bytes(&vault), before_vault);
    assert_eq!(bytes(&data), before_data);
}
