// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Unix: `/proc` reads on Linux; elsewhere the identity walls are empty until
//! the remote runner (H5) brings its own.

#![allow(unsafe_code)]

use std::process::{Child, Command};
use std::time::Duration;

use super::ProcessEntry;

#[cfg(target_os = "linux")]
fn stat_fields(pid: u32) -> Option<(String, Vec<String>)> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let open = stat.find('(')?;
    let close = stat.rfind(')')?;
    let name = stat[open + 1..close].to_string();
    let rest = stat[close + 1..]
        .split_whitespace()
        .map(str::to_string)
        .collect();
    Some((name, rest))
}

pub fn processes() -> Vec<ProcessEntry> {
    #[cfg(target_os = "linux")]
    {
        let Ok(entries) = std::fs::read_dir("/proc") else {
            return Vec::new();
        };
        entries
            .filter_map(|entry| entry.ok()?.file_name().to_str()?.parse::<u32>().ok())
            .filter_map(|pid| {
                let (name, rest) = stat_fields(pid)?;
                // After the name: state, ppid, ..., starttime is field 22 (index 19).
                Some(ProcessEntry {
                    pid,
                    parent: rest.get(1)?.parse().ok()?,
                    name,
                    path: std::fs::read_link(format!("/proc/{pid}/exe")).ok(),
                    started: rest.get(19).and_then(|value| value.parse().ok()),
                })
            })
            .collect()
    }
    #[cfg(not(target_os = "linux"))]
    Vec::new()
}

pub fn started(pid: u32) -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let (_, rest) = stat_fields(pid)?;
        // A zombie has exited; it is not alive for the harness's purposes.
        if rest.first().map(String::as_str) == Some("Z") {
            return None;
        }
        rest.get(19)?.parse().ok()
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        None
    }
}

pub fn pipe_names() -> Vec<String> {
    Vec::new()
}

/// Off Windows the parent-death signal does the job object's work on Linux.
pub struct Job;

impl Job {
    pub fn new() -> std::io::Result<Self> {
        Ok(Self)
    }

    pub fn adopt(&self, _child: &Child) -> std::io::Result<()> {
        Ok(())
    }
}

/// Kill the child when the thread that spawned it dies (Linux).
pub fn die_with_parent(command: &mut Command) {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            command.pre_exec(|| {
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    #[cfg(not(target_os = "linux"))]
    let _ = command;
}

/// The one-minute load average: the run queue's Unix measure.
pub fn cpu_queue() -> Option<f64> {
    std::fs::read_to_string("/proc/loadavg")
        .ok()?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

pub fn cpu_busy(window: Duration) -> Option<f64> {
    let sample = || -> Option<(u64, u64)> {
        let stat = std::fs::read_to_string("/proc/stat").ok()?;
        let line = stat.lines().next()?;
        let values: Vec<u64> = line
            .split_whitespace()
            .skip(1)
            .filter_map(|v| v.parse().ok())
            .collect();
        let idle = values.get(3)? + values.get(4).copied().unwrap_or(0);
        Some((idle, values.iter().sum()))
    };
    let (idle0, total0) = sample()?;
    std::thread::sleep(window);
    let (idle1, total1) = sample()?;
    let total = total1.saturating_sub(total0);
    (total > 0).then(|| 1.0 - idle1.saturating_sub(idle0) as f64 / total as f64)
}
