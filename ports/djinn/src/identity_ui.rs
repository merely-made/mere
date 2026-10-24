// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Native-only identity interactions.
//!
//! The browser may request one of these operations after admission, but it
//! never supplies or receives a local path, private-key byte, or passphrase.
//! A desktop UI implementation selects and unlocks the key inside the native
//! host process, then returns only a public mutation receipt.
//!
//! Moved from graphshell in DR-C (dramatis repo plan, D15): the dialogs act
//! on castellan's keeper, which only djinn links. Graphshell's doors reach
//! them through `ResidentIdentity::native_action`, which [`crate::keeper`]
//! answers here.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use castellan::authority::PersonaeHost;
use castellan::custody::{IdentityStorage, OsPresence, UnlockMethod};
#[cfg(not(target_os = "linux"))]
use light_file_dialog::dialog::InputBox;
use light_file_dialog::dialog::{Dialog, DialogBackend, OpenFileDialog};
use ssh_key::PrivateKey;
use zeroize::Zeroizing;

use crate::door::browser_carrier::{
    NativeIdentityAction, NativeIdentityFailure, NativeIdentityResult,
};
use dramatis::intents::ImportSshKeyNativeIntentV1;

const MAX_SSH_PRIVATE_KEY_BYTES: u64 = 1024 * 1024;
const PICKER_START_ENV: &str = "GRAPHSHELL_NATIVE_PICKER_START";

#[cfg(windows)]
mod windows_presence;

/// Secret-bearing desktop interactions required by native identity actions.
///
/// Implementations own the selected path and passphrase. Neither appears in
/// the browser request or response types.
pub trait NativeIdentityUi: Send + Sync {
    /// Resident-owned confirmation; apps cannot supply the answer.
    fn confirm_quarantine(
        &self,
        _profile: &str,
        _title: &str,
        _action: &str,
    ) -> Result<bool, NativeIdentityFailure> {
        Err(NativeIdentityFailure::UiUnavailable)
    }
    fn pick_ssh_private_key(&self) -> Result<Option<PathBuf>, NativeIdentityFailure>;

    fn prompt_ssh_private_key_passphrase(
        &self,
    ) -> Result<Option<Zeroizing<String>>, NativeIdentityFailure>;

    /// Ask the OS to verify the user (Windows Hello). `Ok(None)` when it is
    /// unavailable, cancelled or not verified: the passphrase box follows.
    fn verify_presence(&self) -> Result<Option<OsPresence>, NativeIdentityFailure> {
        Ok(None)
    }

    /// Ask for the vault passphrase in a native box; `Ok(None)` if cancelled.
    fn prompt_vault_passphrase(&self) -> Result<Option<Zeroizing<String>>, NativeIdentityFailure> {
        Err(NativeIdentityFailure::UiUnavailable)
    }

    /// The same box in the host's own words: a resident's start prompt, a
    /// first-run choice, a retry after an error (vault lock rulings 63, 64).
    fn ask_vault_passphrase(
        &self,
        _message: &str,
    ) -> Result<Option<Zeroizing<String>>, NativeIdentityFailure> {
        Err(NativeIdentityFailure::UiUnavailable)
    }
}

/// Cross-platform system-dialog implementation used by the installed host.
///
/// `light-file-dialog` uses Win32 dialogs on Windows, AppleScript on macOS,
/// and an available graphical dialog provider on Linux. Graphshell refuses
/// its console fallback because stdout belongs exclusively to native
/// messaging.
#[derive(Clone, Debug, Default)]
pub struct SystemNativeIdentityUi {
    picker_start: Option<PathBuf>,
    prefer_passphrase: bool,
}

impl SystemNativeIdentityUi {
    /// Override the initial picker location for an embedding host or receipt.
    pub fn with_picker_start(path: PathBuf) -> Self {
        Self {
            picker_start: Some(path),
            ..Self::default()
        }
    }

    /// Choose the enrolled passphrase directly for native vault unlocks.
    /// This changes prompt selection only; the storage still verifies the credential.
    pub fn with_passphrase_preference(mut self, prefer: bool) -> Self {
        self.prefer_passphrase = prefer;
        self
    }

    fn dialog_backend() -> DialogBackend {
        light_file_dialog::set_verbose(0);
        light_file_dialog::set_silent(1);
        light_file_dialog::set_force_console(0);
        DialogBackend::query()
    }

    fn require_graphical_backend(&self) -> Result<(), NativeIdentityFailure> {
        Self::dialog_backend()
            .graphic
            .then_some(())
            .ok_or(NativeIdentityFailure::UiUnavailable)
    }

    fn password_input(
        &self,
        title: &str,
        message: &str,
    ) -> Result<Option<Zeroizing<String>>, NativeIdentityFailure> {
        let backend = Self::dialog_backend();
        if !masked_input_backend(&backend) {
            return Err(NativeIdentityFailure::UiUnavailable);
        }
        #[cfg(target_os = "linux")]
        {
            linux_password_input(&backend.name, title, message)
        }
        #[cfg(not(target_os = "linux"))]
        {
            Ok(InputBox::new(title, message)
                .password()
                .show()
                .map(Zeroizing::new))
        }
    }
}

// light-file-dialog 3.21.3's Unix input query delegates to its message-box
// query, which can report dialog/whiptail as graphical. Its xmessage input
// fallback also ignores password mode. Only admit providers with an actual
// masked input implementation; an unknown provider must not collect a secret.
fn masked_input_backend(backend: &DialogBackend) -> bool {
    backend.graphic
        && matches!(
            backend.name.as_str(),
            "windows_wchar" | "applescript" | "kdialog" | "zenity" | "yad"
        )
}

// The library's Unix helpers return `echo 1$szAnswer` through a shell. That
// changes spaces and expands globs in the secret. Invoke the selected masked
// helper with separate arguments and read its exact output instead.
#[cfg(target_os = "linux")]
fn linux_password_input(
    backend: &str,
    title: &str,
    message: &str,
) -> Result<Option<Zeroizing<String>>, NativeIdentityFailure> {
    let mut command = match backend {
        "zenity" | "yad" => {
            let mut command = std::process::Command::new(backend);
            command.args([
                "--entry",
                "--hide-text",
                "--title",
                title,
                "--text",
                message,
            ]);
            command
        },
        "kdialog" => {
            let mut command = std::process::Command::new("kdialog");
            command.args(["--password", message, "--title", title]);
            command
        },
        _ => return Err(NativeIdentityFailure::UiUnavailable),
    };
    let output = command
        .output()
        .map_err(|_| NativeIdentityFailure::UiUnavailable)?;
    let _stderr = Zeroizing::new(output.stderr);
    linux_password_answer(output.status.code(), output.stdout)
}

#[cfg(target_os = "linux")]
fn linux_password_answer(
    status: Option<i32>,
    bytes: Vec<u8>,
) -> Result<Option<Zeroizing<String>>, NativeIdentityFailure> {
    let mut bytes = Zeroizing::new(bytes);
    match status {
        Some(0) => {},
        Some(1) => return Ok(None),
        _ => return Err(NativeIdentityFailure::UiUnavailable),
    }
    // These single-line input helpers append one LF. Preserve every other byte.
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
    }
    let value = std::str::from_utf8(&bytes).map_err(|_| NativeIdentityFailure::UiUnavailable)?;
    Ok(Some(Zeroizing::new(value.to_owned())))
}

impl NativeIdentityUi for SystemNativeIdentityUi {
    fn confirm_quarantine(
        &self,
        profile: &str,
        title: &str,
        action: &str,
    ) -> Result<bool, NativeIdentityFailure> {
        self.require_graphical_backend()?;
        let question = if action == "Assign all items to this persona" {
            "Assign this older import namespace to the persona above?\nThis assigns every item without accepting quarantined credentials.".to_string()
        } else if action == "Accept" {
            "Accept this quarantined item?\nIts supported credentials become available for use."
                .to_string()
        } else {
            format!("{action} this quarantined item?")
        };
        let message = format!(
            "Persona: {profile}\nReview: {title}\n\n{question}\nThis decision is made by Djinn."
        );
        Ok(
            light_file_dialog::dialog::MessageBox::new("Djinn · Quarantine review", &message)
                .buttons(light_file_dialog::MessageBoxType::YesNo)
                // Windows indexes visible buttons; Unix backends use 0 for No.
                .default_button(if cfg!(windows) { 1 } else { 0 })
                .show()
                == light_file_dialog::MessageBoxButton::Ok,
        )
    }
    fn pick_ssh_private_key(&self) -> Result<Option<PathBuf>, NativeIdentityFailure> {
        self.require_graphical_backend()?;
        let mut dialog = OpenFileDialog::new("Import SSH private key")
            .filter_description("OpenSSH private keys");
        let configured_start = self
            .picker_start
            .as_deref()
            .map(|path| path.as_os_str().to_owned())
            .or_else(|| std::env::var_os(PICKER_START_ENV));
        if let Some(start) = configured_start {
            dialog = dialog.default_path(start.to_string_lossy());
        }
        Ok(dialog.show().map(PathBuf::from))
    }

    fn prompt_ssh_private_key_passphrase(
        &self,
    ) -> Result<Option<Zeroizing<String>>, NativeIdentityFailure> {
        self.password_input(
            "Graphshell",
            "Enter the passphrase for the selected SSH private key.",
        )
    }

    /// Windows Hello through personae's gate, which alone mints the proof.
    fn verify_presence(&self) -> Result<Option<OsPresence>, NativeIdentityFailure> {
        if self.prefer_passphrase {
            return Ok(None);
        }
        tracing::debug!("checking native presence availability");
        let availability = castellan::custody::unlock::presence_availability();
        tracing::debug!(?availability, "native presence availability answered");
        if !availability.is_available() {
            return Ok(None);
        }
        #[cfg(windows)]
        let verified = windows_presence::verify();
        #[cfg(not(windows))]
        let verified = castellan::custody::unlock::verify_presence("Unlock your identity vault");
        match verified {
            Ok(proof) => Ok(Some(proof)),
            Err(error) => {
                tracing::info!(%error, "presence not verified; asking for the passphrase");
                Ok(None)
            },
        }
    }

    fn prompt_vault_passphrase(&self) -> Result<Option<Zeroizing<String>>, NativeIdentityFailure> {
        self.ask_vault_passphrase("Enter the vault passphrase.")
    }

    fn ask_vault_passphrase(
        &self,
        message: &str,
    ) -> Result<Option<Zeroizing<String>>, NativeIdentityFailure> {
        self.password_input("Identity vault", message)
    }
}

/// Refusal implementation for hosts that deliberately expose no desktop UI.
#[derive(Clone, Copy, Debug, Default)]
pub struct UnavailableNativeIdentityUi;

impl NativeIdentityUi for UnavailableNativeIdentityUi {
    fn pick_ssh_private_key(&self) -> Result<Option<PathBuf>, NativeIdentityFailure> {
        Err(NativeIdentityFailure::UiUnavailable)
    }

    fn prompt_ssh_private_key_passphrase(
        &self,
    ) -> Result<Option<Zeroizing<String>>, NativeIdentityFailure> {
        Err(NativeIdentityFailure::UiUnavailable)
    }
}

/// Apply one fixed native interaction against the resident Personae host.
pub fn apply_native_identity_action<S, U>(
    host: &Arc<PersonaeHost<S>>,
    ui: &U,
    action: NativeIdentityAction,
) -> NativeIdentityResult
where
    S: IdentityStorage + 'static,
    U: NativeIdentityUi + ?Sized,
{
    match action {
        NativeIdentityAction::ImportSshPrivate { unlock_policy } => {
            import_ssh_private(host, ui, ImportSshKeyNativeIntentV1 { unlock_policy })
        },
        NativeIdentityAction::UnlockVault => unlock_vault(host, ui),
    }
}

/// Local owner action. No caller supplies a credential, path, or prompt text.
pub(crate) fn change_passphrase<S: IdentityStorage + 'static, U: NativeIdentityUi + ?Sized>(
    host: &Arc<PersonaeHost<S>>,
    ui: &U,
) -> dramatis::authentication::PassphraseChangeReceiptV1 {
    use castellan::custody::PassphraseChangeError as E;
    use dramatis::authentication::{
        PassphraseChangeFailureV1 as F, PassphraseChangeReceiptV1 as R,
    };
    let rejected = |reason| R::Rejected { reason };
    if host.is_locked() {
        return rejected(F::Locked);
    }
    let prompt = |message| match ui.ask_vault_passphrase(message) {
        Ok(Some(secret)) => Ok(secret),
        Ok(None) => Err(R::Cancelled),
        Err(_) => Err(rejected(F::UiUnavailable)),
    };
    let old = match prompt("Change vault passphrase: enter the current passphrase.") {
        Ok(v) => v,
        Err(r) => return r,
    };
    let new = match prompt("Enter the new vault passphrase.") {
        Ok(v) => v,
        Err(r) => return r,
    };
    if new.is_empty() {
        return rejected(F::InvalidNewPassphrase);
    }
    let confirmation = match prompt("Enter the new vault passphrase again.") {
        Ok(v) => v,
        Err(r) => return r,
    };
    if *new != *confirmation {
        return rejected(F::ConfirmationMismatch);
    }
    match host.change_passphrase(old.as_bytes(), new.as_bytes()) {
        Ok(()) => R::Changed,
        Err(e) => rejected(match e {
            E::Locked => F::Locked,
            E::Unsupported => F::Unsupported,
            E::IncorrectCurrent => F::CurrentPassphraseRejected,
            E::InvalidNew => F::InvalidNewPassphrase,
            E::Storage => F::StorageFailure,
        }),
    }
}

/// OS presence first, then the passphrase box; the credential stays here.
fn unlock_vault<S, U>(host: &Arc<PersonaeHost<S>>, ui: &U) -> NativeIdentityResult
where
    S: IdentityStorage + 'static,
    U: NativeIdentityUi + ?Sized,
{
    if !host.is_locked() {
        return NativeIdentityResult::UnlockedVault;
    }
    if let Ok(Some(proof)) = ui.verify_presence() {
        match host.unlock_vault(UnlockMethod::OsPresence(proof)) {
            Ok(()) => return NativeIdentityResult::UnlockedVault,
            Err(error) => {
                tracing::warn!(%error, "presence unlock refused; asking for the passphrase")
            },
        }
    }
    let passphrase = match ui.prompt_vault_passphrase() {
        Ok(Some(passphrase)) => passphrase,
        Ok(None) => return NativeIdentityResult::Cancelled,
        Err(reason) => return NativeIdentityResult::Rejected { reason },
    };
    let unlocked = host.unlock_vault(UnlockMethod::Passphrase(passphrase.as_bytes()));
    drop(passphrase);
    match unlocked {
        Ok(()) => NativeIdentityResult::UnlockedVault,
        Err(_) if host.is_locked() => NativeIdentityResult::Rejected {
            reason: NativeIdentityFailure::IncorrectPassphrase,
        },
        Err(_) => NativeIdentityResult::Rejected {
            reason: NativeIdentityFailure::UnlockRejected,
        },
    }
}

fn import_ssh_private<S, U>(
    host: &Arc<PersonaeHost<S>>,
    ui: &U,
    options: ImportSshKeyNativeIntentV1,
) -> NativeIdentityResult
where
    S: IdentityStorage + 'static,
    U: NativeIdentityUi + ?Sized,
{
    let path = match ui.pick_ssh_private_key() {
        Ok(Some(path)) => path,
        Ok(None) => return NativeIdentityResult::Cancelled,
        Err(reason) => return NativeIdentityResult::Rejected { reason },
    };
    let metadata = match fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() => metadata,
        _ => {
            return NativeIdentityResult::Rejected {
                reason: NativeIdentityFailure::SelectedFileUnreadable,
            };
        },
    };
    if metadata.len() > MAX_SSH_PRIVATE_KEY_BYTES {
        return NativeIdentityResult::Rejected {
            reason: NativeIdentityFailure::SelectedFileTooLarge,
        };
    }
    let bytes = match fs::read(&path) {
        Ok(bytes) => Zeroizing::new(bytes),
        Err(_) => {
            return NativeIdentityResult::Rejected {
                reason: NativeIdentityFailure::SelectedFileUnreadable,
            };
        },
    };
    if bytes.len() as u64 > MAX_SSH_PRIVATE_KEY_BYTES {
        return NativeIdentityResult::Rejected {
            reason: NativeIdentityFailure::SelectedFileTooLarge,
        };
    }
    let mut private = match PrivateKey::from_openssh(bytes.as_slice()) {
        Ok(private) => private,
        Err(_) => {
            return NativeIdentityResult::Rejected {
                reason: NativeIdentityFailure::InvalidPrivateKey,
            };
        },
    };
    if private.is_encrypted() {
        let passphrase = match ui.prompt_ssh_private_key_passphrase() {
            Ok(Some(passphrase)) => passphrase,
            Ok(None) => return NativeIdentityResult::Cancelled,
            Err(reason) => return NativeIdentityResult::Rejected { reason },
        };
        private = match private.decrypt(passphrase.as_bytes()) {
            Ok(private) => private,
            Err(_) => {
                return NativeIdentityResult::Rejected {
                    reason: NativeIdentityFailure::IncorrectPassphrase,
                };
            },
        };
    }

    match host.import_ssh_private(private, options) {
        Ok(receipt) => NativeIdentityResult::ImportedSshPrivate {
            fingerprint: receipt.fingerprint,
            comment: receipt.comment,
            unlock_policy: receipt.unlock_policy,
            replaced_existing: receipt.replaced_existing,
        },
        Err(_) => NativeIdentityResult::Rejected {
            reason: NativeIdentityFailure::ImportRejected,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use castellan::custody::{IdentityVault, InMemoryStorage, Profile};
    use dramatis::intents::SshUnlockPolicyIntentV1;
    use dramatis::view::VaultProtectionView;
    use personae::{Ed25519Keypair, ProfileId};
    use ssh_key::{Algorithm, LineEnding};

    use super::*;

    #[test]
    fn password_input_refuses_console_unmasked_and_unknown_backends() {
        for name in [
            "dialog",
            "whiptail",
            "basicinput",
            "gxmessage",
            "xmessage",
            "python3-tkinter",
            "python2-tkinter",
            "unknown",
        ] {
            assert!(
                !masked_input_backend(&DialogBackend {
                    graphic: true,
                    name: name.into()
                }),
                "{name}"
            );
        }
        for name in ["windows_wchar", "applescript", "kdialog", "zenity", "yad"] {
            assert!(
                masked_input_backend(&DialogBackend {
                    graphic: true,
                    name: name.into()
                }),
                "{name}"
            );
            assert!(
                !masked_input_backend(&DialogBackend {
                    graphic: false,
                    name: name.into()
                }),
                "{name}"
            );
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_password_output_preserves_spaces_quotes_globs_and_unicode() {
        let value = "  New  local fixture * ' \" $() \\ café  ";
        let answer = linux_password_answer(Some(0), format!("{value}\n").into_bytes())
            .unwrap()
            .unwrap();
        assert_eq!(answer.as_str(), value);
        assert_eq!(
            linux_password_answer(Some(0), b"\n".to_vec())
                .unwrap()
                .unwrap()
                .as_str(),
            ""
        );
        assert!(
            linux_password_answer(Some(1), b"discarded".to_vec())
                .unwrap()
                .is_none()
        );
        for status in [Some(2), None] {
            assert!(matches!(
                linux_password_answer(status, vec![]),
                Err(NativeIdentityFailure::UiUnavailable)
            ));
        }
        assert!(matches!(
            linux_password_answer(Some(0), vec![255]),
            Err(NativeIdentityFailure::UiUnavailable)
        ));
    }

    #[test]
    fn preferred_passphrase_does_not_wait_for_os_consent() {
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = SystemNativeIdentityUi::default()
                .with_passphrase_preference(true)
                .verify_presence();
            send.send(result).unwrap();
        });
        assert!(
            receive
                .recv_timeout(std::time::Duration::from_secs(2))
                .expect("passphrase selection must not wait for an OS consent dialog")
                .unwrap()
                .is_none()
        );
    }

    struct ScriptedUi {
        path: PathBuf,
        passphrases: Mutex<Vec<String>>,
    }

    impl NativeIdentityUi for ScriptedUi {
        fn pick_ssh_private_key(&self) -> Result<Option<PathBuf>, NativeIdentityFailure> {
            Ok(Some(self.path.clone()))
        }

        fn prompt_ssh_private_key_passphrase(
            &self,
        ) -> Result<Option<Zeroizing<String>>, NativeIdentityFailure> {
            Ok(self.passphrases.lock().unwrap().pop().map(Zeroizing::new))
        }
    }

    fn host() -> Arc<PersonaeHost<InMemoryStorage>> {
        let profile = Profile::new(
            ProfileId("native-ui".to_string()),
            "Native UI",
            Ed25519Keypair::from_seed([0x81; 32]),
        );
        Arc::new(PersonaeHost::new(
            IdentityVault::with_profile(InMemoryStorage::new(), profile),
            None,
            VaultProtectionView::Ephemeral,
        ))
    }

    fn scratch_key(tag: &str, passphrase: &str) -> (PathBuf, String) {
        let mut private = PrivateKey::random(&mut rand_core::OsRng, Algorithm::Ed25519).unwrap();
        private.set_comment("native picker receipt");
        let fingerprint = private.fingerprint(ssh_key::HashAlg::Sha256).to_string();
        let private = private.encrypt(&mut rand_core::OsRng, passphrase).unwrap();
        let encoded = private.to_openssh(LineEnding::LF).unwrap();
        let path = std::env::temp_dir().join(format!(
            "graphshell-native-import-{tag}-{}",
            std::process::id()
        ));
        fs::write(&path, encoded.as_bytes()).unwrap();
        (path, fingerprint)
    }

    #[test]
    fn encrypted_key_is_unlocked_and_imported_without_a_serialized_secret() {
        let (path, fingerprint) = scratch_key("ok", "correct horse");
        let ui = ScriptedUi {
            path: path.clone(),
            passphrases: Mutex::new(vec!["correct horse".to_string()]),
        };
        let host = host();
        let result = apply_native_identity_action(
            &host,
            &ui,
            NativeIdentityAction::ImportSshPrivate {
                unlock_policy: SshUnlockPolicyIntentV1::PerUse,
            },
        );
        assert!(matches!(
            &result,
            NativeIdentityResult::ImportedSshPrivate {
                fingerprint: imported,
                ..
            } if imported == &fingerprint
        ));
        assert_eq!(host.snapshot().unwrap().ssh_keys.len(), 1);

        let serialized = serde_json::to_string(&result).unwrap();
        assert!(!serialized.contains("correct horse"));
        assert!(!serialized.contains("BEGIN OPENSSH PRIVATE KEY"));
        assert!(!serialized.contains(path.to_string_lossy().as_ref()));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn wrong_passphrase_imports_nothing_and_returns_a_bounded_reason() {
        let (path, _) = scratch_key("wrong", "correct horse");
        let ui = ScriptedUi {
            path: path.clone(),
            passphrases: Mutex::new(vec!["wrong horse".to_string()]),
        };
        let host = host();
        let result = apply_native_identity_action(
            &host,
            &ui,
            NativeIdentityAction::ImportSshPrivate {
                unlock_policy: SshUnlockPolicyIntentV1::ShortTtl { idle_seconds: 90 },
            },
        );
        assert_eq!(
            result,
            NativeIdentityResult::Rejected {
                reason: NativeIdentityFailure::IncorrectPassphrase
            }
        );
        assert!(host.snapshot().unwrap().ssh_keys.is_empty());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn unavailable_ui_cannot_downgrade_to_browser_secret_input() {
        let result = apply_native_identity_action(
            &host(),
            &UnavailableNativeIdentityUi,
            NativeIdentityAction::ImportSshPrivate {
                unlock_policy: SshUnlockPolicyIntentV1::Session,
            },
        );
        assert_eq!(
            result,
            NativeIdentityResult::Rejected {
                reason: NativeIdentityFailure::UiUnavailable
            }
        );
    }
}
