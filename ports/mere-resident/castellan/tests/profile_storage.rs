// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Stored profile identity and malformed-envelope regressions.
use castellan::custody::{IdentityStorage, PassphraseEncryptedStorage, Profile};
use personae::{Ed25519Keypair, ProfileId};

#[test]
fn a_stale_first_open_refuses_before_mixing_profile_keys_or_deleting_data() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("vault.json");
    let first = PassphraseEncryptedStorage::open(&path, b"fixture passphrase").unwrap();
    let stale = PassphraseEncryptedStorage::open(&path, b"fixture passphrase").unwrap();
    assert!(!path.exists());
    let first_profile = Profile::new(
        ProfileId("first".into()),
        "First",
        Ed25519Keypair::from_seed([0x71; 32]),
    );
    let second_profile = Profile::new(
        ProfileId("second".into()),
        "Second",
        Ed25519Keypair::from_seed([0x72; 32]),
    );
    first.save_profile(&first_profile).unwrap();
    let before = std::fs::read(&path).unwrap();
    let error = stale.save_profile(&second_profile).unwrap_err();
    assert!(error.to_string().contains("salt changed"), "{error}");
    assert!(stale.delete_profile(&first_profile.id).is_err());
    assert!(stale.list_profiles().is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    drop(stale);
    let reopened = PassphraseEncryptedStorage::open(&path, b"fixture passphrase").unwrap();
    reopened.save_profile(&second_profile).unwrap();
    let fresh = PassphraseEncryptedStorage::open(&path, b"fixture passphrase").unwrap();
    assert_eq!(
        fresh
            .load_profile(&first_profile.id)
            .unwrap()
            .master
            .public_key(),
        first_profile.master.public_key()
    );
    assert_eq!(
        fresh
            .load_profile(&second_profile.id)
            .unwrap()
            .master
            .public_key(),
        second_profile.master.public_key()
    );
}

#[test]
fn encrypted_profile_records_cannot_be_transplanted_between_profile_names() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("vault.json");
    let store = PassphraseEncryptedStorage::open(&path, b"fixture passphrase").unwrap();
    let first = Profile::new(
        ProfileId("first".into()),
        "First",
        Ed25519Keypair::from_seed([0x61; 32]),
    );
    let second = Profile::new(
        ProfileId("second".into()),
        "Second",
        Ed25519Keypair::from_seed([0x62; 32]),
    );
    store.save_profile(&first).unwrap();
    store.save_profile(&second).unwrap();
    assert_eq!(
        store.load_profile(&first.id).unwrap().master.public_key(),
        first.master.public_key()
    );
    let mut envelope: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let a = envelope["profiles"]["first"].clone();
    let b = envelope["profiles"]["second"].clone();
    envelope["profiles"]["first"] = b;
    envelope["profiles"]["second"] = a;
    std::fs::write(&path, serde_json::to_vec(&envelope).unwrap()).unwrap();
    assert!(store.load_profile(&first.id).is_err());
    assert!(PassphraseEncryptedStorage::open(&path, b"fixture passphrase").is_err());
    // Contrast control: shared sealed-record backend binds records to paths.
    let sealed_root = root.path().join("sealed");
    let sealed = castellan::custody::SealedProfileStorage::open_with_key(&sealed_root, [0x66; 32]);
    sealed.save_profile(&first).unwrap();
    sealed.save_profile(&second).unwrap();
    let record = |id: &ProfileId| {
        sealed_root
            .join("profiles")
            .join(format!("{}.json", blake3::hash(id.0.as_bytes()).to_hex()))
    };
    let first_path = record(&first.id);
    let second_path = record(&second.id);
    let first_bytes = std::fs::read(&first_path).unwrap();
    let second_bytes = std::fs::read(&second_path).unwrap();
    std::fs::write(&first_path, second_bytes).unwrap();
    std::fs::write(&second_path, first_bytes).unwrap();
    assert!(sealed.load_profile(&first.id).is_err());
    assert!(sealed.load_profile(&second.id).is_err());
}

#[test]
fn malformed_nonce_returns_an_error_without_panicking() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("vault.json");
    let store = PassphraseEncryptedStorage::open(&path, b"fixture passphrase").unwrap();
    let profile = Profile::new(
        ProfileId("first".into()),
        "First",
        Ed25519Keypair::from_seed([0x63; 32]),
    );
    store.save_profile(&profile).unwrap();
    let mut envelope: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    envelope["profiles"]["first"]["nonce"] = serde_json::json!([0]);
    std::fs::write(&path, serde_json::to_vec(&envelope).unwrap()).unwrap();
    let result =
        std::panic::catch_unwind(|| PassphraseEncryptedStorage::open(&path, b"fixture passphrase"));
    assert!(result.is_ok(), "malformed data must not panic");
    assert!(result.unwrap().is_err());
}
