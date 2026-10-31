// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Windows Hello as the consent helper's verifier. The helper runs in the
//! owner's interactive session; each petition becomes a
//! `UserConsentVerifier` request through the window interop (this is a
//! desktop process, so the dialog needs an owner window). The OS collects
//! the PIN, fingerprint or face, never this process. Unavailable Hello, a
//! cancel, a retry limit, an error or the petition's own expiry is a refusal.
//!
//! Like everything in `platform::windows`, this is type-checked on other
//! hosts and qualified on a Windows machine, not asserted here.

use std::time::{Duration, Instant};

use custody_wire::Petition;
use windows::Security::Credentials::UI::{UserConsentVerificationResult, UserConsentVerifier, UserConsentVerifierAvailability};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::WinRT::{IUserConsentVerifierInterop, RO_INIT_MULTITHREADED, RoInitialize};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_POPUP, WS_VISIBLE,
};
use windows::core::{HSTRING, factory, w};
use windows_future::{AsyncStatus, IAsyncOperation};

use crate::helper::{Verifier, message};

/// Windows Hello behind a one-pixel owner window kept off the screen.
pub struct Hello {
    owner: HWND,
}

impl Hello {
    /// Join the multithreaded apartment and create the owner window.
    pub fn new() -> Result<Self, String> {
        // SAFETY: plain initialisation of this thread, which then verifies.
        unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.map_err(|e| format!("start the Windows Runtime: {e}"))?;
        // SAFETY: the system's STATIC class needs no registration.
        let owner = unsafe {
            CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
                w!("STATIC"),
                w!("Djinn custodian consent"),
                WS_POPUP | WS_VISIBLE,
                -32000,
                -32000,
                1,
                1,
                None,
                None,
                None,
                None,
            )
        }
        .map_err(|e| format!("create the owner window: {e}"))?;
        Ok(Self { owner })
    }

    fn ask(&self, text: &str, within: Duration) -> windows::core::Result<bool> {
        let available = UserConsentVerifier::CheckAvailabilityAsync()?.join()?;
        if available != UserConsentVerifierAvailability::Available {
            return Ok(false);
        }
        let interop = factory::<UserConsentVerifier, IUserConsentVerifierInterop>()?;
        // SAFETY: `owner` is a live window of this thread.
        let operation: IAsyncOperation<UserConsentVerificationResult> = unsafe { interop.RequestVerificationForWindowAsync(self.owner, &HSTRING::from(text)) }?;
        let until = Instant::now() + within;
        while operation.Status()? == AsyncStatus::Started {
            if Instant::now() >= until {
                let _ = operation.Cancel();
                return Ok(false);
            }
            self.pump();
            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(operation.GetResults()? == UserConsentVerificationResult::Verified)
    }

    /// Let the owner window answer the system while a dialog is up.
    fn pump(&self) {
        let mut message = MSG::default();
        // SAFETY: a message struct of our own and this thread's queue.
        while unsafe { PeekMessageW(&mut message, None, 0, 0, PM_REMOVE) }.as_bool() {
            // SAFETY: the message came from the call above.
            unsafe {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }
}

impl Drop for Hello {
    fn drop(&mut self) {
        // SAFETY: the window this value created.
        let _ = unsafe { DestroyWindow(self.owner) };
    }
}

impl Verifier for Hello {
    fn verify(&self, petition: &Petition) -> bool {
        self.ask(&message(petition), Duration::from_millis(petition.expires_in_ms)).unwrap_or(false)
    }
}
