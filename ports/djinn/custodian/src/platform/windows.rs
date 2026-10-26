// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Windows: an SCM-managed service under its virtual account
//! (`NT SERVICE\djinn-custodian`), serving a named pipe whose DACL admits
//! only the vault owner and whose peers are identified by token SID
//! (`custody-wire`). Before input the process disables extension points,
//! dynamic code and remote or low-label image loads. The service account,
//! ACLs on its executable, configuration and state, and the CNG wrapping
//! key are installation facts (`install-windows.ps1`) and are qualified on
//! a Windows host, not asserted here.

use std::sync::OnceLock;

use windows_sys::Win32::System::Services::{
    RegisterServiceCtrlHandlerExW, SERVICE_ACCEPT_SHUTDOWN, SERVICE_ACCEPT_STOP,
    SERVICE_CONTROL_SHUTDOWN, SERVICE_CONTROL_STOP, SERVICE_RUNNING, SERVICE_STATUS,
    SERVICE_STATUS_HANDLE, SERVICE_STOPPED, SERVICE_TABLE_ENTRYW, SERVICE_WIN32_OWN_PROCESS,
    SetServiceStatus, StartServiceCtrlDispatcherW,
};
use windows_sys::Win32::System::Threading::{
    ProcessDynamicCodePolicy, ProcessExtensionPointDisablePolicy, ProcessImageLoadPolicy,
    SetProcessMitigationPolicy,
};

use super::{Applied, Confinement};

/// The service name, and the virtual account clients expect as the pipe's
/// server.
pub const SERVICE_NAME: &str = "djinn-custodian";

/// Apply the recipe.
pub fn confine(plan: &Confinement) -> Result<Applied, String> {
    let mut applied = Applied::default();
    for (policy, flags, name) in [
        (ProcessExtensionPointDisablePolicy, 0b1u32, "extension points disabled"),
        (ProcessImageLoadPolicy, 0b111u32, "remote and low-label image loads refused"),
        (ProcessDynamicCodePolicy, 0b1u32, "dynamic code prohibited"),
    ] {
        // SAFETY: each policy's structure is a single u32 of flag bits.
        let ok = unsafe { SetProcessMitigationPolicy(policy, (&flags as *const u32).cast(), 4) } != 0;
        if ok {
            applied.notes.push(name.into());
        } else if plan.required {
            return Err(format!("{name}: {}", std::io::Error::last_os_error()));
        } else {
            applied.notes.push(format!("{name}: not applied"));
        }
    }
    Ok(applied)
}

type Body = Box<dyn FnOnce(Box<dyn Fn() + Send + Sync>) + Send>;

static BODY: OnceLock<std::sync::Mutex<Option<Body>>> = OnceLock::new();
static STATUS: OnceLock<usize> = OnceLock::new();
static STOP: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn report(state: u32) {
    if let Some(handle) = STATUS.get() {
        let status = SERVICE_STATUS {
            dwServiceType: SERVICE_WIN32_OWN_PROCESS,
            dwCurrentState: state,
            dwControlsAccepted: if state == SERVICE_RUNNING { SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN } else { 0 },
            dwWin32ExitCode: 0,
            dwServiceSpecificExitCode: 0,
            dwCheckPoint: 0,
            dwWaitHint: 0,
        };
        // SAFETY: a handle SCM returned for this service.
        unsafe { SetServiceStatus(*handle as SERVICE_STATUS_HANDLE, &status) };
    }
}

unsafe extern "system" fn control(code: u32, _: u32, _: *mut core::ffi::c_void, _: *mut core::ffi::c_void) -> u32 {
    if code == SERVICE_CONTROL_STOP || code == SERVICE_CONTROL_SHUTDOWN {
        if let Some(stop) = STOP.get() {
            stop();
        }
        report(SERVICE_STOPPED);
        std::process::exit(0);
    }
    0
}

unsafe extern "system" fn service_main(_: u32, _: *mut *mut u16) {
    let name = wide(SERVICE_NAME);
    // SAFETY: a static handler and a NUL-terminated name.
    let handle = unsafe { RegisterServiceCtrlHandlerExW(name.as_ptr(), Some(control), std::ptr::null()) };
    if handle.is_null() {
        return;
    }
    let _ = STATUS.set(handle as usize);
    report(SERVICE_RUNNING);
    let body = BODY.get().and_then(|b| b.lock().ok()?.take());
    if let Some(body) = body {
        body(Box::new(|| {}));
    }
    report(SERVICE_STOPPED);
}

/// Run `body` as the SCM service. `on_stop` runs on a stop or shutdown
/// control before the process exits; the custodian locks there.
pub fn run_service(body: Body, on_stop: Box<dyn Fn() + Send + Sync>) -> Result<(), String> {
    let _ = STOP.set(on_stop);
    BODY.get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .map_err(|_| "service body lock poisoned".to_string())?
        .replace(body);
    let mut name = wide(SERVICE_NAME);
    let table = [
        SERVICE_TABLE_ENTRYW {
            lpServiceName: name.as_mut_ptr(),
            lpServiceProc: Some(service_main),
        },
        SERVICE_TABLE_ENTRYW {
            lpServiceName: std::ptr::null_mut(),
            lpServiceProc: None,
        },
    ];
    // SAFETY: a NULL-terminated table whose strings outlive the call.
    if unsafe { StartServiceCtrlDispatcherW(table.as_ptr()) } == 0 {
        return Err(format!("not started by the service control manager: {}", std::io::Error::last_os_error()));
    }
    Ok(())
}
