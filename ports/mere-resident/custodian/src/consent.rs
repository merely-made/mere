// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Consent mechanisms the custodian runs itself. A same-user process cannot
//! answer these: on Linux the custodian asks polkit to authenticate the
//! requesting process's session user (`auth_self`, never cached), and
//! polkit's authentication agent, not a socket client, collects the
//! password. The macOS host answers through its own native dialog via the
//! C ABI instead. Cancellation, a missing agent, a timeout or any failure
//! to run the mechanism is a refusal.

use crate::core::Prompt;

/// One way to obtain the owner's decision on a petition.
pub trait Mechanism: Send + Sync {
    /// Block until the owner decides; `false` unless they approved.
    fn present(&self, prompt: &Prompt) -> bool;
}

/// polkit's `pkcheck`, at a fixed installed path, for one action.
pub struct Polkit {
    pub pkcheck: std::path::PathBuf,
    pub action: String,
}

/// The action the installed policy file declares.
pub const POLKIT_ACTION: &str = "com.merely.djinn.custody.petition";

impl Polkit {
    pub fn installed() -> Self {
        Self {
            pkcheck: "/usr/bin/pkcheck".into(),
            action: POLKIT_ACTION.into(),
        }
    }

    fn subject(prompt: &Prompt) -> Option<String> {
        let custody_wire::Peer::Unix { uid, pid: Some(pid), .. } = &prompt.requester else {
            return None;
        };
        // pid,start-time,uid binds the subject against pid reuse.
        let start = process_start_time(*pid)?;
        Some(format!("{pid},{start},{uid}"))
    }
}

impl Mechanism for Polkit {
    fn present(&self, prompt: &Prompt) -> bool {
        let Some(subject) = Self::subject(prompt) else {
            return false;
        };
        let petition = &prompt.petition;
        let mut command = std::process::Command::new(&self.pkcheck);
        command
            .env_clear()
            .arg("--action-id")
            .arg(&self.action)
            .arg("--process")
            .arg(subject)
            .arg("--allow-user-interaction");
        for (key, value) in [
            ("principal", petition.principal.as_str()),
            ("operation", petition.operation.as_str()),
            ("profile", petition.profile.as_str()),
            ("namespace", petition.namespace.as_str()),
            ("payload", petition.payload_digest.as_str()),
        ] {
            command.arg("--detail").arg(key).arg(value);
        }
        command
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        let Ok(mut child) = command.spawn() else {
            return false;
        };
        let deadline = std::time::Instant::now() + custody_wire::limits::CONSENT;
        loop {
            match child.try_wait() {
                Ok(Some(status)) => return status.success(),
                Ok(None) if std::time::Instant::now() < deadline => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                },
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return false;
                },
            }
        }
    }
}

/// Field 22 of `/proc/<pid>/stat`, after the parenthesised command name.
fn process_start_time(pid: i32) -> Option<u64> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let after = stat.rsplit_once(')')?.1;
    after.split_whitespace().nth(19)?.parse().ok()
}

/// A scripted mechanism for tests.
pub struct Scripted(pub std::sync::Mutex<std::collections::VecDeque<bool>>);

impl Mechanism for Scripted {
    fn present(&self, _prompt: &Prompt) -> bool {
        self.0.lock().map(|mut q| q.pop_front().unwrap_or(false)).unwrap_or(false)
    }
}
