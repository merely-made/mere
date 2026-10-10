// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! `djinn --enroll-passphrase` (vault lock ruling 39).
//!
//! Wraps the vault's existing OS-held root under a passphrase typed at the
//! terminal, so a device without OS presence can lock (ruling 27). The
//! passphrase is read from the terminal only: never the environment (ruling
//! 7), never an argument. Opening never mints a root: a directory without one
//! is refused, so a passphrase vault beside it is never shadowed.

use std::io;
use std::path::Path;

use castellan::custody::{IdentityStorage, SealedProfileStorage};
use zeroize::Zeroizing;

/// Reads one secret line for a prompt. The terminal in production; scripted
/// answers in tests.
pub type ReadSecret<'a> = dyn FnMut(&str) -> io::Result<Zeroizing<String>> + 'a;

/// The terminal reader: `rpassword`, which reads the console or tty, never
/// standard input or the environment.
pub fn read_from_terminal(prompt: &str) -> io::Result<Zeroizing<String>> {
    rpassword::prompt_password(prompt).map(Zeroizing::new)
}

/// Enrol a passphrase over the vault at `vault_dir`, asking twice.
pub fn enroll_passphrase(vault_dir: &Path, read: &mut ReadSecret<'_>) -> Result<String, String> {
    let storage = SealedProfileStorage::open_existing_auto_os(vault_dir)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| {
            format!(
                "no OS-held vault root in {}; start the resident once first (a passphrase \
                 vault, or a platform without one, has nothing to enrol over)",
                vault_dir.display()
            )
        })?;
    if storage.unlock_methods().passphrase {
        return Err("a passphrase is already enrolled for this vault".into());
    }
    let first = read("New vault passphrase: ").map_err(|error| error.to_string())?;
    if first.is_empty() {
        return Err("an empty passphrase cannot be enrolled".into());
    }
    let again = read("Again: ").map_err(|error| error.to_string())?;
    if *first != *again {
        return Err("the passphrases do not match; nothing was enrolled".into());
    }
    storage
        .enroll_passphrase(first.as_bytes())
        .map_err(|error| error.to_string())?;
    Ok(format!(
        "passphrase enrolled for {}; the vault can now be locked and unlocked with it",
        vault_dir.display()
    ))
}

#[cfg(test)]
mod tests {
    use castellan::custody::{IdentityVault, Profile, UnlockMethod};
    use personae::{Ed25519Keypair, ProfileId};

    use super::*;

    fn scripted(answers: &[&str]) -> impl FnMut(&str) -> io::Result<Zeroizing<String>> {
        let mut answers: Vec<String> = answers.iter().rev().map(|a| a.to_string()).collect();
        move |_| {
            answers
                .pop()
                .map(Zeroizing::new)
                .ok_or_else(|| io::Error::other("no more answers"))
        }
    }

    /// Ruling 39 end to end, on a temp data root: the resident's own
    /// OS-held root, enrolled, then locked and unlocked by the passphrase.
    #[test]
    fn enrolment_then_a_passphrase_unlock_works_on_a_temp_root() {
        let dir = tempfile::tempdir().unwrap();
        let Some(storage) = SealedProfileStorage::open_auto_os(dir.path()).unwrap() else {
            // No OS-held root on this platform: refused, never minted.
            let refused = enroll_passphrase(dir.path(), &mut scripted(&["x", "x"]));
            assert!(refused.unwrap_err().contains("no OS-held vault root"));
            return;
        };
        let id = ProfileId("work".into());
        storage
            .save_profile(&Profile::new(id.clone(), "Work", Ed25519Keypair::from_seed([4; 32])))
            .unwrap();
        let mut vault = IdentityVault::open(storage, &id).unwrap();
        // Hello may or may not be set up here; the passphrase is the question.
        assert!(!vault.unlock_methods().passphrase);

        assert!(
            enroll_passphrase(dir.path(), &mut scripted(&["one", "two"]))
                .unwrap_err()
                .contains("do not match")
        );
        assert!(enroll_passphrase(dir.path(), &mut scripted(&[""])).is_err());
        assert!(!vault.unlock_methods().passphrase, "nothing enrolled yet");

        let said = enroll_passphrase(dir.path(), &mut scripted(&["typed", "typed"])).unwrap();
        assert!(said.contains("enrolled"), "{said}");
        assert!(vault.unlock_methods().passphrase);
        assert!(
            enroll_passphrase(dir.path(), &mut scripted(&["again", "again"]))
                .unwrap_err()
                .contains("already enrolled")
        );

        vault.lock().unwrap();
        assert!(vault.unlock(UnlockMethod::Passphrase(b"wrong")).is_err());
        vault.unlock(UnlockMethod::Passphrase(b"typed")).unwrap();
        assert_eq!(vault.current_profile().unwrap().master.to_seed(), [4; 32]);
    }

    #[test]
    fn a_directory_without_a_root_is_refused_and_left_empty() {
        let dir = tempfile::tempdir().unwrap();
        let refused = enroll_passphrase(dir.path(), &mut scripted(&["x", "x"]));
        assert!(refused.unwrap_err().contains("no OS-held vault root"));
        assert!(std::fs::read_dir(dir.path()).unwrap().next().is_none());
    }
}
