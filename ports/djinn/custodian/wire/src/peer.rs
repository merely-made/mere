// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Who is on the other end, as the OS says so: `SO_PEERCRED` on Linux,
//! `getpeereid` (and `LOCAL_PEERPID`) on macOS and the BSDs, the pipe
//! client's or server's token SID on Windows, and a signed code identity
//! only where the macOS native host established it through XPC. Both ends
//! use it: the custodian to select a principal, the client to verify the
//! custodian before sending anything confidential.

use serde::{Deserialize, Serialize};

/// An OS-established peer identity. Never read from a message.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Peer {
    /// A Unix-domain socket peer.
    Unix {
        uid: u32,
        gid: u32,
        pid: Option<i32>,
    },
    /// A Windows named-pipe peer, by its token's user SID.
    Windows { sid: String, pid: u32 },
    /// A macOS XPC peer whose code signature the native host verified.
    Signed { team: String, identifier: String },
}

impl Peer {
    /// A stable account key for per-principal budgets.
    pub fn account(&self) -> String {
        match self {
            Self::Unix { uid, .. } => format!("uid:{uid}"),
            Self::Windows { sid, .. } => format!("sid:{sid}"),
            Self::Signed { team, identifier } => format!("signed:{team}:{identifier}"),
        }
    }
}

/// What a client accepts as the custodian: the installed service identity.
/// On Linux a socket-activated listener reports systemd (uid 0) as its
/// creator, so root is listed beside the service uid there. On macOS a
/// same-user agent is identified by its code signature, since its uid is
/// everyone's in that session.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerExpectation {
    #[serde(default)]
    pub unix_uids: Vec<u32>,
    #[serde(default)]
    pub windows_sids: Vec<String>,
    #[serde(default)]
    pub signed: Vec<SignedIdentity>,
}

/// A team and code identifier a macOS peer's signature must satisfy.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedIdentity {
    pub team: String,
    pub identifier: String,
}

impl ServerExpectation {
    /// Whether `peer` is an accepted custodian.
    pub fn admits(&self, peer: &Peer) -> bool {
        match peer {
            Peer::Unix { uid, .. } => self.unix_uids.contains(uid),
            Peer::Windows { sid, .. } => self.windows_sids.iter().any(|s| s == sid),
            Peer::Signed { team, identifier } => self
                .signed
                .iter()
                .any(|s| &s.team == team && &s.identifier == identifier),
        }
    }
}

/// The first of `candidates` whose signing requirement the macOS peer on
/// `fd` satisfies, checked against its audit token rather than a pid.
#[cfg(target_os = "macos")]
pub fn signed_peer(fd: std::os::fd::RawFd, candidates: &[SignedIdentity]) -> Option<Peer> {
    let token = macos::audit_token(fd)?;
    candidates
        .iter()
        .find(|c| macos::satisfies(&token, &c.team, &c.identifier))
        .map(|c| Peer::Signed {
            team: c.team.clone(),
            identifier: c.identifier.clone(),
        })
}

#[cfg(target_os = "macos")]
mod macos {
    use std::ffi::c_void;

    type CFTypeRef = *const c_void;
    type OSStatus = i32;

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        static kCFTypeDictionaryKeyCallBacks: c_void;
        static kCFTypeDictionaryValueCallBacks: c_void;
        fn CFDataCreate(allocator: CFTypeRef, bytes: *const u8, length: isize) -> CFTypeRef;
        fn CFStringCreateWithBytes(allocator: CFTypeRef, bytes: *const u8, length: isize, encoding: u32, external: u8) -> CFTypeRef;
        fn CFDictionaryCreate(allocator: CFTypeRef, keys: *const CFTypeRef, values: *const CFTypeRef, count: isize, key_callbacks: *const c_void, value_callbacks: *const c_void) -> CFTypeRef;
        fn CFRelease(object: CFTypeRef);
    }

    #[link(name = "Security", kind = "framework")]
    unsafe extern "C" {
        static kSecGuestAttributeAudit: CFTypeRef;
        fn SecCodeCopyGuestWithAttributes(host: CFTypeRef, attributes: CFTypeRef, flags: u32, guest: *mut CFTypeRef) -> OSStatus;
        fn SecRequirementCreateWithString(text: CFTypeRef, flags: u32, requirement: *mut CFTypeRef) -> OSStatus;
        fn SecCodeCheckValidity(code: CFTypeRef, flags: u32, requirement: CFTypeRef) -> OSStatus;
    }

    const UTF8: u32 = 0x0800_0100;

    struct Owned(CFTypeRef);

    impl Drop for Owned {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // SAFETY: a +1 reference this wrapper owns.
                unsafe { CFRelease(self.0) };
            }
        }
    }

    /// `LOCAL_PEERTOKEN` (6 under `SOL_LOCAL`): the peer's audit token.
    pub fn audit_token(fd: std::os::fd::RawFd) -> Option<[u8; 32]> {
        let mut token = [0u8; 32];
        let mut len = token.len() as libc::socklen_t;
        // SAFETY: `token` is valid for `len` bytes.
        let rc = unsafe { libc::getsockopt(fd, 0, 6, token.as_mut_ptr().cast(), &mut len) };
        (rc == 0 && len as usize == token.len()).then_some(token)
    }

    pub fn satisfies(token: &[u8; 32], team: &str, identifier: &str) -> bool {
        let valid = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || ".-_".contains(c));
        if !valid(team) || !valid(identifier) {
            return false;
        }
        let text = format!("anchor apple generic and identifier \"{identifier}\" and certificate leaf[subject.OU] = \"{team}\"");
        // SAFETY: every CF object is created here, owned by `Owned` and
        // released on every path; the requirement text is validated ASCII.
        unsafe {
            let data = Owned(CFDataCreate(std::ptr::null(), token.as_ptr(), token.len() as isize));
            let key = kSecGuestAttributeAudit;
            let attributes = Owned(CFDictionaryCreate(std::ptr::null(), &key, &data.0, 1, &kCFTypeDictionaryKeyCallBacks, &kCFTypeDictionaryValueCallBacks));
            let mut code: CFTypeRef = std::ptr::null();
            if SecCodeCopyGuestWithAttributes(std::ptr::null(), attributes.0, 0, &mut code) != 0 {
                return false;
            }
            let code = Owned(code);
            let string = Owned(CFStringCreateWithBytes(std::ptr::null(), text.as_ptr(), text.len() as isize, UTF8, 0));
            let mut requirement: CFTypeRef = std::ptr::null();
            if SecRequirementCreateWithString(string.0, 0, &mut requirement) != 0 {
                return false;
            }
            let requirement = Owned(requirement);
            SecCodeCheckValidity(code.0, 0, requirement.0) == 0
        }
    }
}

/// The peer of a connected Unix-domain socket.
#[cfg(unix)]
pub fn unix_peer(fd: std::os::fd::RawFd) -> std::io::Result<Peer> {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        let mut cred = libc::ucred {
            pid: 0,
            uid: 0,
            gid: 0,
        };
        let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
        // SAFETY: `cred` and `len` are valid for writes of the sizes given.
        let rc = unsafe {
            libc::getsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_PEERCRED,
                (&mut cred as *mut libc::ucred).cast(),
                &mut len,
            )
        };
        if rc != 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Peer::Unix {
            uid: cred.uid,
            gid: cred.gid,
            pid: Some(cred.pid),
        })
    }
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    {
        let mut uid: libc::uid_t = 0;
        let mut gid: libc::gid_t = 0;
        // SAFETY: both out-pointers are valid for writes.
        if unsafe { libc::getpeereid(fd, &mut uid, &mut gid) } != 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Peer::Unix {
            uid,
            gid,
            pid: peer_pid(fd),
        })
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn peer_pid(fd: std::os::fd::RawFd) -> Option<i32> {
    // <sys/un.h>: SOL_LOCAL 0, LOCAL_PEERPID 2.
    let mut pid: libc::pid_t = 0;
    let mut len = std::mem::size_of::<libc::pid_t>() as libc::socklen_t;
    // SAFETY: `pid` and `len` are valid for writes of the sizes given.
    let rc = unsafe { libc::getsockopt(fd, 0, 2, (&mut pid as *mut libc::pid_t).cast(), &mut len) };
    (rc == 0).then_some(pid)
}

#[cfg(all(unix, not(any(target_os = "linux", target_os = "android", target_os = "macos", target_os = "ios"))))]
fn peer_pid(_fd: std::os::fd::RawFd) -> Option<i32> {
    None
}

/// This process's effective uid.
#[cfg(unix)]
pub fn effective_uid() -> u32 {
    // SAFETY: geteuid has no preconditions.
    unsafe { libc::geteuid() }
}

#[cfg(windows)]
pub use crate::transport::windows::{current_user_sid, service_sid};
