//! Bug-observation diagnostic, not a correctness gate. No simultaneous threads
//! are needed: independent opens before the first save are enough.
use castellan::custody::{IdentityStorage, PassphraseEncryptedStorage, Profile};
use personae::{Ed25519Keypair, ProfileId};

fn profile(id: &str, byte: u8) -> Profile {
    Profile::new(ProfileId(id.into()), id, Ed25519Keypair::from_seed([byte; 32]))
}

#[test]
fn independent_first_opens_successfully_save_under_different_keys_in_one_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vault.json");
    let first = PassphraseEncryptedStorage::open(&path, b"synthetic fixture").unwrap();
    let second = PassphraseEncryptedStorage::open(&path, b"synthetic fixture").unwrap();
    assert!(!path.exists(), "open does not create the documented empty file");
    let a = profile("a", 0x31);
    let b = profile("b", 0x32);
    first.save_profile(&a).unwrap();
    let before: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    second.save_profile(&b).unwrap();
    let after: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(before["salt"], after["salt"]);
    assert_eq!(after["profiles"].as_object().unwrap().len(), 2);
    assert_eq!(first.load_profile(&a.id).unwrap().master.public_key(), a.master.public_key());
    assert_eq!(second.load_profile(&b.id).unwrap().master.public_key(), b.master.public_key());
    assert!(first.load_profile(&b.id).is_err());
    assert!(second.load_profile(&a.id).is_err());
    match PassphraseEncryptedStorage::open(&path, b"synthetic fixture") {
        Ok(reopened) => assert!(reopened.load_profile(&b.id).is_err()),
        Err(_) => (), // check_kek can encounter the unreadable entry first.
    }

    // Contrast: opening after a first save reuses its salt/key and both records
    // survive a new backend instance, using the same passphrase.
    let path = dir.path().join("control.json");
    let first = PassphraseEncryptedStorage::open(&path, b"synthetic fixture").unwrap();
    first.save_profile(&a).unwrap();
    let second = PassphraseEncryptedStorage::open(&path, b"synthetic fixture").unwrap();
    second.save_profile(&b).unwrap();
    let reopened = PassphraseEncryptedStorage::open(&path, b"synthetic fixture").unwrap();
    assert_eq!(reopened.load_profile(&a.id).unwrap().master.public_key(), a.master.public_key());
    assert_eq!(reopened.load_profile(&b.id).unwrap().master.public_key(), b.master.public_key());
}
