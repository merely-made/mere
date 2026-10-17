//! Fixture-only diagnostics of stored profile identity and malformed envelopes.
use castellan::custody::{IdentityStorage, PassphraseEncryptedStorage, Profile};
use personae::{Ed25519Keypair, ProfileId};

#[test]
fn encrypted_profile_records_can_be_transplanted_between_profile_names() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("vault.json");
    let store = PassphraseEncryptedStorage::open(&path, b"fixture passphrase").unwrap();
    let first = Profile::new(ProfileId("first".into()), "First", Ed25519Keypair::from_seed([0x61;32]));
    let second = Profile::new(ProfileId("second".into()), "Second", Ed25519Keypair::from_seed([0x62;32]));
    store.save_profile(&first).unwrap();
    store.save_profile(&second).unwrap();
    assert_eq!(store.load_profile(&first.id).unwrap().master.public_key(), first.master.public_key());
    let mut envelope: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let a = envelope["profiles"]["first"].clone();
    let b = envelope["profiles"]["second"].clone();
    envelope["profiles"]["first"] = b;
    envelope["profiles"]["second"] = a;
    std::fs::write(&path, serde_json::to_vec(&envelope).unwrap()).unwrap();
    let transplanted = store.load_profile(&first.id).unwrap();
    assert_eq!(transplanted.id, first.id);
    assert_eq!(transplanted.display_name, second.display_name);
    assert_eq!(transplanted.master.public_key(), second.master.public_key());
    // The correct passphrase also opens this modified envelope successfully.
    let reopened = PassphraseEncryptedStorage::open(&path, b"fixture passphrase").unwrap();
    assert_eq!(reopened.load_profile(&first.id).unwrap().master.public_key(), second.master.public_key());
    // Contrast control: shared sealed-record backend binds records to paths.
    let sealed_root = root.path().join("sealed");
    let sealed = castellan::custody::SealedProfileStorage::open_with_key(&sealed_root, [0x66;32]);
    sealed.save_profile(&first).unwrap();
    sealed.save_profile(&second).unwrap();
    let record = |id: &ProfileId| sealed_root.join("profiles").join(format!("{}.json", blake3::hash(id.0.as_bytes()).to_hex()));
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
fn malformed_nonce_panics_inside_the_fallible_profile_open_api() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("vault.json");
    let store = PassphraseEncryptedStorage::open(&path, b"fixture passphrase").unwrap();
    let profile = Profile::new(ProfileId("first".into()), "First", Ed25519Keypair::from_seed([0x63;32]));
    store.save_profile(&profile).unwrap();
    let mut envelope: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    envelope["profiles"]["first"]["nonce"] = serde_json::json!([0]);
    std::fs::write(&path, serde_json::to_vec(&envelope).unwrap()).unwrap();
    let result = std::panic::catch_unwind(|| PassphraseEncryptedStorage::open(&path, b"fixture passphrase"));
    assert!(result.is_err(), "malformed data panics rather than yielding IdentityError");
}
