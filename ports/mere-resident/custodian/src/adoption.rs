// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Adopting Djinn's existing vault into the custodian's passphrase vault.
//! A passphrase vault is one file and is copied as it is (the `adopt`
//! command). Djinn's Windows vault is different: sealed records under a root
//! key the OS holds for the owner's logon, which only the owner can open
//! and which the custodian's virtual account never can. This module copies
//! such a vault's profiles, in the owner's own process, into a new
//! passphrase vault; the copy is portable and tested on every OS.

use std::path::Path;

use castellan::custody::bootstrap::PASSPHRASE_VAULT_FILE;
use castellan::custody::roster::choice_path;
use castellan::custody::{AUTO_UNLOCK_ROOT_FILE, IdentityStorage, PassphraseEncryptedStorage, SealedProfileStorage};
use personae::{IdentityError, ProfileId};

/// What a source directory holds.
#[derive(Debug, PartialEq, Eq)]
pub enum VaultKind {
    /// `vault.json`: copied as one file.
    Passphrase,
    /// Sealed records under `auto-unlock-root.json`, no `vault.json`.
    OsRooted,
}

/// Which kind of vault `from` is.
pub fn kind(from: &Path) -> Result<VaultKind, String> {
    if from.join(PASSPHRASE_VAULT_FILE).is_file() {
        Ok(VaultKind::Passphrase)
    } else if from.join(AUTO_UNLOCK_ROOT_FILE).is_file() {
        Ok(VaultKind::OsRooted)
    } else {
        Err(format!("no passphrase vault or OS-rooted vault at {}", from.display()))
    }
}

/// Open the OS-rooted vault in `from` with the key the OS holds for this
/// user. It fails for any other account and where the OS has no such root.
pub fn open_os_rooted(from: &Path) -> Result<SealedProfileStorage, String> {
    match SealedProfileStorage::open_existing_auto_os(from) {
        Ok(Some(storage)) => Ok(storage),
        Ok(None) => Err("the OS-held root of this vault cannot be read on this platform".into()),
        Err(IdentityError::Locked) => Err("the source vault carries Djinn's persisted lock; unlock Djinn once as the owner, close it and run this again".into()),
        Err(error) => Err(format!("open the OS-rooted vault as this user: {error}")),
    }
}

/// Save every profile of `source` into `target`, whole, with its slots.
pub fn copy_profiles(source: &dyn IdentityStorage, target: &dyn IdentityStorage) -> Result<Vec<ProfileId>, String> {
    let summaries = source.list_profiles().map_err(|e| format!("list the source profiles: {e}"))?;
    if summaries.is_empty() {
        return Err("the source vault holds no profile".into());
    }
    let mut copied = Vec::with_capacity(summaries.len());
    for summary in summaries {
        let profile = source.load_profile(&summary.id).map_err(|e| format!("read profile {}: {e}", summary.id.0))?;
        target.save_profile(&profile).map_err(|e| format!("write profile {}: {e}", summary.id.0))?;
        copied.push(summary.id);
    }
    Ok(copied)
}

/// Create the passphrase vault `directory/vault.json` from `source`, and
/// carry the roster's choice file from `from`. The file is staged beside its
/// name and moved into place only when complete; a vault already there is
/// never replaced.
pub fn into_passphrase_vault(source: &dyn IdentityStorage, from: &Path, directory: &Path, passphrase: &[u8]) -> Result<Vec<ProfileId>, String> {
    let vault = directory.join(PASSPHRASE_VAULT_FILE);
    if vault.exists() {
        return Err("a vault already exists in the custodian's state directory".into());
    }
    std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let staged = directory.join(format!("{PASSPHRASE_VAULT_FILE}.adopting"));
    let _ = std::fs::remove_file(&staged);
    let copied: Result<Vec<ProfileId>, String> = (|| {
        let target = PassphraseEncryptedStorage::open(&staged, passphrase).map_err(|e| format!("create the passphrase vault: {e}"))?;
        let copied = copy_profiles(source, &target)?;
        if choice_path(from).is_file() {
            std::fs::copy(choice_path(from), choice_path(directory)).map_err(|e| e.to_string())?;
        }
        std::fs::File::open(&staged).and_then(|f| f.sync_all()).map_err(|e| e.to_string())?;
        Ok(copied)
    })();
    if copied.is_err() {
        let _ = std::fs::remove_file(&staged);
        return copied;
    }
    std::fs::rename(&staged, &vault).map_err(|e| e.to_string())?;
    copied
}

#[cfg(test)]
mod tests {
    use super::*;
    use castellan::custody::vault::Profile;
    use personae::Ed25519Keypair;

    const KEY: [u8; 32] = [7; 32];

    fn sealed(dir: &Path, names: &[&str]) -> Vec<(String, [u8; 32])> {
        let storage = SealedProfileStorage::open_with_key(dir, KEY);
        names
            .iter()
            .map(|name| {
                let profile = Profile::new(ProfileId((*name).into()), format!("{name} display"), Ed25519Keypair::generate());
                storage.save_profile(&profile).unwrap();
                ((*name).to_string(), profile.master.public_key().to_bytes())
            })
            .collect()
    }

    #[test]
    fn a_sealed_record_vault_round_trips_into_a_passphrase_vault() {
        let from = tempfile::tempdir().unwrap();
        let masters = sealed(from.path(), &["work", "home"]);
        std::fs::write(choice_path(from.path()), "home").unwrap();
        std::fs::write(from.path().join(AUTO_UNLOCK_ROOT_FILE), b"{}").unwrap();
        assert_eq!(kind(from.path()), Ok(VaultKind::OsRooted));

        let source = SealedProfileStorage::open_with_key(from.path(), KEY);
        let into = tempfile::tempdir().unwrap();
        let directory = into.path().join("vault");
        let mut copied = into_passphrase_vault(&source, from.path(), &directory, b"a passphrase").unwrap();
        copied.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(copied, vec![ProfileId("home".into()), ProfileId("work".into())]);

        let adopted = PassphraseEncryptedStorage::open(directory.join(PASSPHRASE_VAULT_FILE), b"a passphrase").unwrap();
        for (name, master) in masters {
            let profile = adopted.load_profile(&ProfileId(name.clone())).unwrap();
            assert_eq!(profile.master.public_key().to_bytes(), master, "{name} keeps its master");
            assert_eq!(profile.display_name, format!("{name} display"));
        }
        assert_eq!(std::fs::read_to_string(choice_path(&directory)).unwrap(), "home", "the roster choice travels");
        assert!(PassphraseEncryptedStorage::open(directory.join(PASSPHRASE_VAULT_FILE), b"another").is_err(), "only the new passphrase opens it");
        assert!(!directory.join(format!("{PASSPHRASE_VAULT_FILE}.adopting")).exists(), "nothing is left staged");
        assert!(from.path().join("profiles").is_dir(), "the source stays where it was");
    }

    #[test]
    fn an_existing_vault_or_an_empty_source_is_never_overwritten_or_adopted() {
        let from = tempfile::tempdir().unwrap();
        let source = SealedProfileStorage::open_with_key(from.path(), KEY);
        let into = tempfile::tempdir().unwrap();
        assert_eq!(into_passphrase_vault(&source, from.path(), into.path(), b"p").unwrap_err(), "the source vault holds no profile");
        assert!(!into.path().join(PASSPHRASE_VAULT_FILE).exists(), "a failed copy leaves no vault");
        assert!(!into.path().join(format!("{PASSPHRASE_VAULT_FILE}.adopting")).exists());
        sealed(from.path(), &["work"]);
        into_passphrase_vault(&source, from.path(), into.path(), b"p").unwrap();
        let before = std::fs::read(into.path().join(PASSPHRASE_VAULT_FILE)).unwrap();
        assert!(into_passphrase_vault(&source, from.path(), into.path(), b"p").is_err());
        assert_eq!(std::fs::read(into.path().join(PASSPHRASE_VAULT_FILE)).unwrap(), before);
    }

    #[test]
    fn the_source_kind_follows_the_files_and_other_platforms_cannot_open_an_os_root() {
        let dir = tempfile::tempdir().unwrap();
        assert!(kind(dir.path()).is_err());
        std::fs::write(dir.path().join(AUTO_UNLOCK_ROOT_FILE), b"{}").unwrap();
        assert_eq!(kind(dir.path()), Ok(VaultKind::OsRooted));
        std::fs::write(dir.path().join(PASSPHRASE_VAULT_FILE), b"{}").unwrap();
        assert_eq!(kind(dir.path()), Ok(VaultKind::Passphrase), "a vault.json wins");
        #[cfg(not(windows))]
        {
            std::fs::remove_file(dir.path().join(PASSPHRASE_VAULT_FILE)).unwrap();
            assert!(open_os_rooted(dir.path()).is_err());
        }
    }
}
