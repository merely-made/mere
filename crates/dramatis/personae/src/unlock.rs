// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! How a locked vault is unlocked (vault lock plan, rulings 4, 21, 27, 28).
//!
//! Two methods: a passphrase, which unwraps the vault root sealed under it
//! ([`crate::passphrase_root`]) or derives the passphrase vault's key; and OS
//! presence, a gate rather than a key: the OS confirms the user is at the
//! device (Windows Hello today), and the root is then read back from the OS
//! store. [`OsPresence`] is the proof of that check. Only [`verify_presence`]
//! mints one, after the OS reports the user verified.
//!
//! Failed attempts are throttled by Argon2id's cost alone (ruling 19).

use crate::IdentityError;

/// A user act that unlocks the vault.
pub enum UnlockMethod<'a> {
    /// The vault passphrase.
    Passphrase(&'a [u8]),
    /// The OS verified the user's presence. Consumed by the unlock.
    OsPresence(OsPresence),
}

/// Which unlock methods a storage could accept on this device now.
///
/// A vault refuses to lock while this is empty (ruling 27): a lock nobody
/// can undo is not offered.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UnlockMethods {
    /// A passphrase can unlock it (a wrapped root is enrolled, or the
    /// storage is passphrase-keyed).
    pub passphrase: bool,
    /// OS presence can unlock it (the gate is built and available, and the
    /// OS store holds the root).
    pub os_presence: bool,
}

impl UnlockMethods {
    /// Whether any method is available.
    pub fn any(&self) -> bool {
        self.passphrase || self.os_presence
    }
}

/// Proof that the OS verified the user's presence just now.
///
/// It has no public constructor: [`verify_presence`] mints it after the OS
/// says the user verified, and unlocking consumes it.
pub struct OsPresence {
    _minted: (),
}

impl OsPresence {
    #[cfg(any(test, all(windows, feature = "os-presence")))]
    pub(crate) fn mint() -> Self {
        Self { _minted: () }
    }
}

/// Whether OS presence verification can be asked for on this device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PresenceAvailability {
    /// The OS can verify the user (Windows Hello is set up).
    Available,
    /// It cannot, and why.
    Unavailable(String),
}

impl PresenceAvailability {
    /// Whether presence can be verified.
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }
}

/// Ask the OS whether presence verification is available. Never prompts.
pub fn presence_availability() -> PresenceAvailability {
    imp::availability()
}

/// Prompt the user to verify their presence, unparented.
///
/// A desktop process with a window should prefer
/// [`verify_presence_for_window`], which parents the prompt.
pub fn verify_presence(message: &str) -> Result<OsPresence, IdentityError> {
    imp::verify(None, message)
}

/// Prompt the user to verify their presence over a window (`HWND` on
/// Windows).
pub fn verify_presence_for_window(
    window: isize,
    message: &str,
) -> Result<OsPresence, IdentityError> {
    imp::verify(Some(window), message)
}

#[cfg(all(windows, feature = "os-presence"))]
mod imp {
    use windows::Security::Credentials::UI::{
        UserConsentVerificationResult, UserConsentVerifier, UserConsentVerifierAvailability,
    };
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::WinRT::IUserConsentVerifierInterop;
    use windows::core::{HSTRING, Interface};

    use super::{OsPresence, PresenceAvailability};
    use crate::IdentityError;

    pub(super) fn availability() -> PresenceAvailability {
        match UserConsentVerifier::CheckAvailabilityAsync().and_then(|op| op.join()) {
            Ok(UserConsentVerifierAvailability::Available) => PresenceAvailability::Available,
            Ok(other) => PresenceAvailability::Unavailable(format!("Windows Hello: {other:?}")),
            Err(error) => PresenceAvailability::Unavailable(format!("Windows Hello: {error}")),
        }
    }

    /// The parented request, typed like the unparented one: the async
    /// operation's type is taken from `RequestVerificationAsync`'s signature
    /// (never called here), so it need not be named.
    fn for_window<T: Interface>(
        _typed_like: fn(&HSTRING) -> windows::core::Result<T>,
        window: isize,
        message: &HSTRING,
    ) -> windows::core::Result<T> {
        let interop = windows::core::factory::<UserConsentVerifier, IUserConsentVerifierInterop>()?;
        unsafe {
            interop
                .RequestVerificationForWindowAsync(HWND(window as *mut core::ffi::c_void), message)
        }
    }

    pub(super) fn verify(
        window: Option<isize>,
        message: &str,
    ) -> Result<OsPresence, IdentityError> {
        let message = HSTRING::from(message);
        let operation = match window {
            None => UserConsentVerifier::RequestVerificationAsync(&message),
            Some(window) => for_window(
                UserConsentVerifier::RequestVerificationAsync,
                window,
                &message,
            ),
        };
        let result = operation
            .and_then(|op| op.join())
            .map_err(|error| IdentityError::Backend(format!("Windows Hello: {error}")))?;
        if result == UserConsentVerificationResult::Verified {
            Ok(OsPresence::mint())
        } else {
            Err(IdentityError::Backend(format!(
                "Windows Hello did not verify: {result:?}"
            )))
        }
    }
}

#[cfg(not(all(windows, feature = "os-presence")))]
mod imp {
    use super::{OsPresence, PresenceAvailability};
    use crate::IdentityError;

    fn reason() -> String {
        if cfg!(windows) {
            "built without personae's os-presence feature".to_string()
        } else {
            "no OS presence method on this platform yet".to_string()
        }
    }

    pub(super) fn availability() -> PresenceAvailability {
        PresenceAvailability::Unavailable(reason())
    }

    pub(super) fn verify(
        _window: Option<isize>,
        _message: &str,
    ) -> Result<OsPresence, IdentityError> {
        Err(IdentityError::Backend(reason()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Asks the OS, which never prompts. Hello may or may not be set up on
    /// the machine running this, so either answer is accepted; the check is
    /// that the call completes and says which.
    #[test]
    fn availability_answers_without_prompting() {
        let answer = presence_availability();
        if let PresenceAvailability::Unavailable(reason) = &answer {
            assert!(!reason.is_empty());
        }
        println!("presence availability: {answer:?}");
    }

    #[cfg(not(all(windows, feature = "os-presence")))]
    #[test]
    fn without_the_gate_presence_is_unavailable_and_never_minted() {
        assert!(!presence_availability().is_available());
        assert!(verify_presence("test").is_err());
    }

    #[test]
    fn no_methods_means_none() {
        assert!(!UnlockMethods::default().any());
        assert!(
            UnlockMethods {
                passphrase: true,
                os_presence: false
            }
            .any()
        );
    }
}
