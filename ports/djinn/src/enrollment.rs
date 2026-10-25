// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! `djinn --enroll-passphrase` (vault lock ruling 39).
//!
//! Wraps the vault's existing OS-held root under a passphrase typed at the
//! terminal or native masked prompts, so a device without OS presence can lock
//! (ruling 27). The passphrase is never read from the environment (ruling
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

/// First enrollment in native masked prompts, with no terminal fallback.
/// Existing-root and already-enrolled checks run before either prompt.
pub fn enroll_passphrase_native(
    vault_dir: &Path,
    ui: &dyn crate::identity_ui::NativeIdentityUi,
) -> Result<String, String> {
    enroll_passphrase(vault_dir, &mut |message| read_from_native(ui, message))
}

fn read_from_native(
    ui: &dyn crate::identity_ui::NativeIdentityUi,
    message: &str,
) -> io::Result<Zeroizing<String>> {
    match ui.ask_vault_passphrase(message) {
        Ok(Some(secret)) => Ok(secret),
        Ok(None) => Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "Passphrase enrollment cancelled. Nothing changed.",
        )),
        Err(_) => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Native masked passphrase prompts are unavailable. Nothing changed.",
        )),
    }
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

    struct NativeAnswer(
        std::sync::Mutex<
            Option<
                Result<
                    Option<Zeroizing<String>>,
                    crate::door::browser_carrier::NativeIdentityFailure,
                >,
            >,
        >,
    );

    impl crate::identity_ui::NativeIdentityUi for NativeAnswer {
        fn pick_ssh_private_key(
            &self,
        ) -> Result<Option<std::path::PathBuf>, crate::door::browser_carrier::NativeIdentityFailure>
        {
            unreachable!()
        }
        fn prompt_ssh_private_key_passphrase(
            &self,
        ) -> Result<Option<Zeroizing<String>>, crate::door::browser_carrier::NativeIdentityFailure>
        {
            unreachable!()
        }
        fn ask_vault_passphrase(
            &self,
            _: &str,
        ) -> Result<Option<Zeroizing<String>>, crate::door::browser_carrier::NativeIdentityFailure>
        {
            self.0.lock().unwrap().take().unwrap()
        }
    }

    #[test]
    fn native_enrollment_reader_preserves_exact_secret_bytes() {
        let text = " New  native * ' café ";
        let ui = NativeAnswer(std::sync::Mutex::new(Some(Ok(Some(Zeroizing::new(
            text.into(),
        ))))));
        assert_eq!(
            read_from_native(&ui, "New vault passphrase: ")
                .unwrap()
                .as_bytes(),
            text.as_bytes()
        );
    }

    #[test]
    fn native_enrollment_cancellation_and_missing_ui_refuse_without_fallback() {
        let ui = NativeAnswer(std::sync::Mutex::new(Some(Ok(None))));
        assert_eq!(
            read_from_native(&ui, "New vault passphrase: ")
                .unwrap_err()
                .kind(),
            io::ErrorKind::Interrupted
        );
        let ui = NativeAnswer(std::sync::Mutex::new(Some(Err(
            crate::door::browser_carrier::NativeIdentityFailure::UiUnavailable,
        ))));
        assert_eq!(
            read_from_native(&ui, "New vault passphrase: ")
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
    }

    #[cfg(windows)]
    #[test]
    fn native_enrollment_preserves_os_root_identity_and_cold_reopens() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        struct Twice(AtomicUsize);
        impl crate::identity_ui::NativeIdentityUi for Twice {
            fn pick_ssh_private_key(
                &self,
            ) -> Result<
                Option<std::path::PathBuf>,
                crate::door::browser_carrier::NativeIdentityFailure,
            > {
                unreachable!()
            }
            fn prompt_ssh_private_key_passphrase(
                &self,
            ) -> Result<
                Option<Zeroizing<String>>,
                crate::door::browser_carrier::NativeIdentityFailure,
            > {
                unreachable!()
            }
            fn ask_vault_passphrase(
                &self,
                _: &str,
            ) -> Result<
                Option<Zeroizing<String>>,
                crate::door::browser_carrier::NativeIdentityFailure,
            > {
                self.0.fetch_add(1, Ordering::SeqCst);
                Ok(Some(Zeroizing::new(" New  native * ' café ".into())))
            }
        }
        let dir = tempfile::tempdir().unwrap();
        let storage = SealedProfileStorage::open_auto_os(dir.path())
            .unwrap()
            .unwrap();
        let id = ProfileId("work".into());
        storage
            .save_profile(&Profile::new(
                id.clone(),
                "Work",
                Ed25519Keypair::from_seed([4; 32]),
            ))
            .unwrap();
        let root_before =
            std::fs::read(dir.path().join(castellan::custody::AUTO_UNLOCK_ROOT_FILE)).unwrap();
        let ui = Twice(AtomicUsize::new(0));
        enroll_passphrase_native(dir.path(), &ui).unwrap();
        assert_eq!(ui.0.load(Ordering::SeqCst), 2);
        assert_eq!(
            std::fs::read(dir.path().join(castellan::custody::AUTO_UNLOCK_ROOT_FILE)).unwrap(),
            root_before
        );
        drop(storage);
        let storage = SealedProfileStorage::open_locked(dir.path());
        assert!(storage.unlock(UnlockMethod::Passphrase(b"wrong")).is_err());
        storage
            .unlock(UnlockMethod::Passphrase(
                " New  native * ' café ".as_bytes(),
            ))
            .unwrap();
        let vault = IdentityVault::open(storage, &id).unwrap();
        assert_eq!(vault.current_profile().unwrap().master.to_seed(), [4; 32]);
        assert!(
            enroll_passphrase_native(dir.path(), &ui)
                .unwrap_err()
                .contains("already enrolled")
        );
        assert_eq!(ui.0.load(Ordering::SeqCst), 2);
    }

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
            .save_profile(&Profile::new(
                id.clone(),
                "Work",
                Ed25519Keypair::from_seed([4; 32]),
            ))
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
