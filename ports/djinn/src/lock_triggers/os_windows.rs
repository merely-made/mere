// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Windows' lock signals: `Win+L` as a WTS session notification to a
//! message-only window, suspend through `PowerRegisterSuspendResumeNotification`
//! (whose callback locks before it returns, so before the machine sleeps),
//! and idle from `GetLastInputInfo`.

use std::ffi::c_void;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use windows::Win32::Foundation::{HANDLE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Power::{
    DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS, PowerRegisterSuspendResumeNotification,
};
use windows::Win32::System::RemoteDesktop::{NOTIFY_FOR_THIS_SESSION, WTSRegisterSessionNotification};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DEVICE_NOTIFY_CALLBACK, DefWindowProcW, DispatchMessageW, GetMessageW,
    HWND_MESSAGE, MSG, PBT_APMSUSPEND, RegisterClassW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_WTSSESSION_CHANGE,
    WNDCLASSW, WTS_SESSION_LOCK,
};
use windows::core::w;

use super::LockSignal;
use super::host::TriggerHost;

/// One resident per process, so one host the OS callbacks reach.
static HOST: OnceLock<Arc<TriggerHost>> = OnceLock::new();

/// What keeps the sources registered: they live as long as the process.
pub struct Sources;

/// Register for `Win+L` and suspend. A failure is logged and that trigger
/// stays off; the others still run.
pub fn start(host: Arc<TriggerHost>) -> Sources {
    if HOST.set(host).is_err() {
        tracing::warn!("lock trigger sources were already started in this process");
        return Sources;
    }
    if let Err(error) = suspend() {
        tracing::warn!(%error, "suspend will not lock the vault");
    }
    std::thread::Builder::new()
        .name("djinn-session-lock".into())
        .spawn(|| {
            if let Err(error) = session_window() {
                tracing::warn!(%error, "Win+L will not lock the vault");
            }
        })
        .map_err(|error| tracing::warn!(%error, "no session-lock thread"))
        .ok();
    Sources
}

/// The time since the last input in this session, or `None` when Windows
/// cannot say (session 0, some lock-screen states).
pub async fn idle() -> Option<Duration> {
    crate::conditions::input_idle()
}

fn signal(signal: LockSignal) {
    if let Some(host) = HOST.get() {
        host.signal(signal);
    }
}

unsafe extern "system" fn session_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_WTSSESSION_CHANGE && wparam.0 == WTS_SESSION_LOCK as usize {
        signal(LockSignal::SessionLocked);
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// A message-only window registered for this session's changes, pumped on
/// its own thread for the life of the process.
fn session_window() -> windows::core::Result<()> {
    let class = WNDCLASSW {
        lpfnWndProc: Some(session_proc),
        lpszClassName: w!("DjinnLockTriggers"),
        ..Default::default()
    };
    if unsafe { RegisterClassW(&class) } == 0 {
        return Err(windows::core::Error::from_thread());
    }
    let window = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("DjinnLockTriggers"),
            w!("djinn lock triggers"),
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            None,
            None,
        )
    }?;
    unsafe { WTSRegisterSessionNotification(window, NOTIFY_FOR_THIS_SESSION) }?;
    let mut message = MSG::default();
    while unsafe { GetMessageW(&mut message, None, 0, 0) }.as_bool() {
        unsafe { DispatchMessageW(&message) };
    }
    Ok(())
}

unsafe extern "system" fn suspend_callback(_context: *const c_void, kind: u32, _setting: *const c_void) -> u32 {
    if kind == PBT_APMSUSPEND {
        // The lock finishes here, before Windows goes on to sleep (ruling 75).
        signal(LockSignal::Suspending);
    }
    0
}

/// Register the suspend callback for the life of the process.
fn suspend() -> Result<(), String> {
    // Leaked on purpose: Windows reads it for as long as the registration lives.
    let parameters: &'static mut DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS =
        Box::leak(Box::new(DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS {
            Callback: Some(suspend_callback),
            Context: std::ptr::null_mut(),
        }));
    let mut registration: *mut c_void = std::ptr::null_mut();
    let status = unsafe {
        PowerRegisterSuspendResumeNotification(
            DEVICE_NOTIFY_CALLBACK,
            HANDLE(parameters as *mut DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS as *mut c_void),
            &mut registration,
        )
    };
    match status.0 {
        0 => Ok(()),
        code => Err(format!("PowerRegisterSuspendResumeNotification failed ({code})")),
    }
}
