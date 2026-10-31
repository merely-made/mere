// SPDX-License-Identifier: MPL-2.0
use super::vault::{
    CredentialLineage, IdentitySlot, Profile, ProtocolKey, SecretBytes, UnlockTier,
};
use super::{
    IdentityStorage, PassphraseChangeError as E, PassphraseEncryptedStorage, SealedProfileStorage,
    UnlockMethod,
};
use personae::{Ed25519Keypair, ProfileId};
use std::path::Path;
const OLD: &[u8] = b"old public authentication fixture";
const NEW: &[u8] = b"new public authentication fixture";
fn open(
    root: &Path,
    portable: bool,
    password: &[u8],
    seed: bool,
) -> Result<Box<dyn IdentityStorage>, personae::IdentityError> {
    if portable {
        return Ok(Box::new(PassphraseEncryptedStorage::open(
            root.join("vault.json"),
            password,
        )?));
    }
    let storage = if seed {
        SealedProfileStorage::open_with_key(root, [7; 32])
    } else {
        SealedProfileStorage::open_locked(root)
    };
    if seed {
        storage.enroll_passphrase(password)?;
    } else {
        storage.unlock(UnlockMethod::Passphrase(password))?;
    }
    Ok(Box::new(storage))
}
fn seed(storage: &dyn IdentityStorage) {
    for (id, byte) in [("one", 41), ("two", 42)] {
        let mut p = Profile::new(
            ProfileId(id.into()),
            id,
            Ed25519Keypair::from_seed([byte; 32]),
        );
        p.slots.insert(
            ProtocolKey::new("fixture", None),
            IdentitySlot::Direct {
                kind: "fixture".into(),
                payload: SecretBytes::new(vec![byte; 48]),
                lineage: CredentialLineage::LocallyDerived,
                unlock_tier: UnlockTier::Session,
            },
        );
        storage.save_profile(&p).unwrap();
    }
}
fn identities(storage: &dyn IdentityStorage) -> Vec<(String, [u8; 32], Vec<u8>)> {
    let mut profiles = storage.list_profiles().unwrap();
    profiles.sort_by(|a, b| a.id.0.cmp(&b.id.0));
    profiles
        .into_iter()
        .map(|p| {
            let p = storage.load_profile(&p.id).unwrap();
            let IdentitySlot::Direct { payload, .. } =
                p.slots.get(&ProtocolKey::new("fixture", None)).unwrap()
            else {
                panic!()
            };
            (
                p.id.0.clone(),
                p.master.to_seed(),
                payload.as_slice().to_vec(),
            )
        })
        .collect()
}
#[test]
fn both_formats_change_all_profiles_preserve_slots_and_reject_old_after_cold_reopen() {
    for portable in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let s = open(dir.path(), portable, OLD, true).unwrap();
        seed(&*s);
        let before = identities(&*s);
        assert_eq!(s.change_passphrase(b"wrong", NEW), Err(E::IncorrectCurrent));
        assert_eq!(s.change_passphrase(OLD, b""), Err(E::InvalidNew));
        assert_eq!(identities(&*s), before);
        s.change_passphrase(OLD, NEW).unwrap();
        assert_eq!(identities(&*s), before);
        // A subsequent ordinary write must use the replacement key.
        let p = s.load_profile(&ProfileId("two".into())).unwrap();
        s.save_profile(&p).unwrap();
        drop(s);
        assert!(open(dir.path(), portable, OLD, false).is_err());
        let s = open(dir.path(), portable, NEW, false).unwrap();
        assert_eq!(identities(&*s), before);
        s.lock();
        assert_eq!(s.change_passphrase(NEW, OLD), Err(E::Locked));
        s.unlock(UnlockMethod::Passphrase(NEW)).unwrap();
        assert_eq!(identities(&*s), before);
    }
}
#[cfg(unix)]
#[test]
fn failed_replacement_keeps_the_old_credential_and_live_key_for_both_formats() {
    use std::os::unix::fs::PermissionsExt;
    for portable in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let s = open(dir.path(), portable, OLD, true).unwrap();
        seed(&*s);
        let before = identities(&*s);
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o500)).unwrap();
        let result = s.change_passphrase(OLD, NEW);
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!(result, Err(E::Storage));
        assert_eq!(identities(&*s), before);
        drop(s);
        assert!(open(dir.path(), portable, NEW, false).is_err());
        assert_eq!(
            identities(&*open(dir.path(), portable, OLD, false).unwrap()),
            before
        );
    }
}

#[test]
fn wrapped_root_replacement_leaves_sealed_profiles_and_other_records_unchanged() {
    use super::sealed_profile_storage::PASSPHRASE_ROOT_FILE;
    let dir = tempfile::tempdir().unwrap();
    let storage = open(dir.path(), false, OLD, true).unwrap();
    seed(&*storage);
    let records = personae::SealedRecordStorage::open_with_key(dir.path(), [7; 32]);
    records
        .save_record("other/held-record", &vec![83u8; 73])
        .unwrap();
    drop(records);
    fn files(root: &Path) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
        let mut result = std::collections::BTreeMap::new();
        for entry in std::fs::read_dir(root).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                for (n, v) in files(&p) {
                    result.insert(n, v);
                }
            } else if p.file_name().unwrap() != PASSPHRASE_ROOT_FILE {
                result.insert(p.clone(), std::fs::read(p).unwrap());
            }
        }
        result
    }
    let before = files(dir.path());
    storage.change_passphrase(OLD, NEW).unwrap();
    drop(storage);
    assert_eq!(files(dir.path()), before);
    let root = zeroize::Zeroizing::new(
        super::load_passphrase_root(dir.path().join(PASSPHRASE_ROOT_FILE), NEW)
            .unwrap()
            .unwrap(),
    );
    let records = personae::SealedRecordStorage::open_with_key(dir.path(), *root);
    assert_eq!(
        records.load_record::<Vec<u8>>("other/held-record").unwrap(),
        Some(vec![83u8; 73])
    );
}
