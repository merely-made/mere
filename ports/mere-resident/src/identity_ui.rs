// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Platform dialogs implementing Dramatis's native identity callback.

use std::path::PathBuf;
use zeroize::Zeroizing;
#[cfg(feature = "in-process-custody")]
use castellan::custody::OsPresence;
#[cfg(not(target_os = "linux"))]
use light_file_dialog::dialog::InputBox;
use light_file_dialog::dialog::{Dialog, DialogBackend, OpenFileDialog};
use dramatis_resident::native::NativeIdentityFailure;
use dramatis_resident::identity_ui::NativeIdentityUi;

const PICKER_START_ENV: &str = "GRAPHSHELL_NATIVE_PICKER_START";

#[cfg(all(windows, feature = "in-process-custody"))]
mod windows_presence;

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
    fn confirm_signing(
        &self,
        pending: &personae::signing::PendingSigningRequest,
        decision: personae::signing::SigningDecision,
    ) -> Result<bool, NativeIdentityFailure> {
        use personae::signing::{RememberApproval, SigningDecision};
        self.require_graphical_backend()?;
        let clean = |text: &str| {
            text.chars()
                .filter(|c| !c.is_control())
                .take(256)
                .collect::<String>()
        };
        let scope = match decision {
            SigningDecision::Approve {
                remember: RememberApproval::Once,
            } => "Approve this request once?",
            SigningDecision::Approve {
                remember: RememberApproval::UntilIdle,
            } => "Approve for this key's configured idle window?",
            SigningDecision::Deny => "Deny this signing request?",
        };
        let request = &pending.request;
        let message = format!(
            "Persona: {}\nOperation: {}\nKey: {}\nRequester: {}\nTarget: {}\nPayload digest: {}\n\n{}",
            clean(&request.profile),
            clean(&request.operation),
            clean(&request.public_key_fingerprint),
            clean(
                request
                    .authenticated_requester
                    .as_deref()
                    .unwrap_or("not authenticated")
            ),
            clean(
                request
                    .authenticated_target
                    .as_deref()
                    .unwrap_or("not authenticated")
            ),
            clean(&request.payload_digest),
            scope,
        );
        Ok(
            light_file_dialog::dialog::MessageBox::new("Djinn · Signing confirmation", &message)
                .buttons(light_file_dialog::MessageBoxType::YesNo)
                .default_button(if cfg!(windows) { 1 } else { 0 })
                .show()
                == light_file_dialog::MessageBoxButton::Ok,
        )
    }
    fn confirm_application(&self, app: &str, grant: &str) -> Result<bool, NativeIdentityFailure> {
        self.require_graphical_backend()?;
        let app: String = app.chars().filter(|c| !c.is_control()).take(64).collect();
        let message = format!(
            "An application named \"{app}\" asks for:\n\n{grant}\n\nAllow it? Djinn remembers the answer."
        );
        Ok(
            light_file_dialog::dialog::MessageBox::new("Djinn · New application", &message)
                .buttons(light_file_dialog::MessageBoxType::YesNo)
                .default_button(if cfg!(windows) { 1 } else { 0 })
                .show()
                == light_file_dialog::MessageBoxButton::Ok,
        )
    }
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
    #[cfg(feature = "in-process-custody")]
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
#[cfg(all(test, feature = "in-process-custody"))]
mod tests {
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

}
