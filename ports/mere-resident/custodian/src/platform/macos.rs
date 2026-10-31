// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! macOS: a launchd-managed agent (`packaging/macos`) whose socket launchd
//! creates at 0600 and hands over by name, with `getpeereid` principals.
//! The process refuses debugger attachment and core files before input.
//! Protected mode also requires the running code to be validly signed with
//! the hardened runtime and without `get-task-allow`, which is what keeps
//! another same-user process out of its memory; the signed launch,
//! data-protection Keychain envelope and native consent dialog remain the
//! signed host's part (`testing/macos-castellan-custodian-h1`).

use std::os::fd::FromRawFd;
use std::os::unix::net::UnixListener;

use super::{Applied, Confinement};

unsafe extern "C" {
    fn ptrace(request: libc::c_int, pid: libc::pid_t, addr: *mut libc::c_char, data: libc::c_int) -> libc::c_int;
    fn csops(pid: libc::pid_t, ops: libc::c_uint, useraddr: *mut libc::c_void, usersize: libc::size_t) -> libc::c_int;
    fn launch_activate_socket(name: *const libc::c_char, fds: *mut *mut libc::c_int, count: *mut libc::size_t) -> libc::c_int;
}

const PT_DENY_ATTACH: libc::c_int = 31;
const CS_OPS_STATUS: libc::c_uint = 0;
const CS_VALID: u32 = 0x1;
const CS_GET_TASK_ALLOW: u32 = 0x4;
const CS_RUNTIME: u32 = 0x1_0000;

/// Apply the recipe.
pub fn confine(plan: &Confinement) -> Result<Applied, String> {
    let mut applied = Applied::default();
    // SAFETY: constant arguments; PT_DENY_ATTACH affects only this process.
    unsafe {
        let none = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
        if libc::setrlimit(libc::RLIMIT_CORE, &none) != 0 {
            return Err(format!("no core files: {}", std::io::Error::last_os_error()));
        }
        if ptrace(PT_DENY_ATTACH, 0, std::ptr::null_mut(), 0) != 0 {
            return Err(format!("deny attach: {}", std::io::Error::last_os_error()));
        }
    }
    applied.notes.push("no core files, debugger attachment denied".into());
    let mut flags: u32 = 0;
    // SAFETY: a status query into a u32 for this process.
    let rc = unsafe { csops(0, CS_OPS_STATUS, (&mut flags as *mut u32).cast(), 4) };
    let hardened = rc == 0 && flags & CS_VALID != 0 && flags & CS_RUNTIME != 0 && flags & CS_GET_TASK_ALLOW == 0;
    if hardened {
        applied.notes.push("signed with the hardened runtime".into());
    } else if plan.required {
        return Err("protected mode requires a valid hardened-runtime signature without get-task-allow".into());
    } else {
        applied.notes.push(format!("not hardened-runtime signed (code-signing flags {flags:#x}); development only"));
    }
    Ok(applied)
}

/// The listener launchd created for the plist's `Sockets` entry `name`.
pub fn launchd_listener(name: &str) -> std::io::Result<Option<UnixListener>> {
    let name = std::ffi::CString::new(name).map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
    let mut fds: *mut libc::c_int = std::ptr::null_mut();
    let mut count: libc::size_t = 0;
    // SAFETY: valid out-pointers; the array is freed below.
    let rc = unsafe { launch_activate_socket(name.as_ptr(), &mut fds, &mut count) };
    if rc == libc::ENOENT || rc == libc::ESRCH {
        return Ok(None);
    }
    if rc != 0 {
        return Err(std::io::Error::from_raw_os_error(rc));
    }
    // SAFETY: launchd returned `count` descriptors in a malloc'd array.
    let listener = unsafe {
        let all = std::slice::from_raw_parts(fds, count);
        let first = all.first().copied();
        for extra in all.iter().skip(1) {
            libc::close(*extra);
        }
        libc::free(fds.cast());
        first.map(|fd| UnixListener::from_raw_fd(fd))
    };
    Ok(listener)
}
