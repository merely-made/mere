// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Machine load, sampled into every receipt (harness plan ruling 3): other
//! builds are why absolute times moved 74 s to 312 s while relative ones held.

use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::sys;

/// One load sample. Counts leave out this process's own ancestors, so the
/// `cargo test` running the harness is not counted as load.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoadSample {
    pub at_ms: u64,
    pub rustc: u32,
    pub cargo: u32,
    /// Windows: `\System\Processor Queue Length`; Linux: the 1-minute load.
    pub cpu_queue: Option<f64>,
    /// Busy share of all processors over a quarter second.
    pub cpu_busy: Option<f64>,
}

impl LoadSample {
    pub fn take() -> Self {
        let processes = sys::processes();
        let mut ancestors = BTreeSet::new();
        let mut current = std::process::id();
        while ancestors.insert(current) {
            match processes.iter().find(|p| p.pid == current) {
                Some(process) if process.parent != 0 => current = process.parent,
                _ => break,
            }
        }
        let count = |stem: &str| {
            processes
                .iter()
                .filter(|p| !ancestors.contains(&p.pid))
                .filter(|p| {
                    let name = p.name.to_ascii_lowercase();
                    name == stem || name == format!("{stem}.exe")
                })
                .count() as u32
        };
        Self {
            at_ms: crate::now_ms(),
            rustc: count("rustc"),
            cargo: count("cargo"),
            cpu_queue: sys::cpu_queue(),
            cpu_busy: sys::cpu_busy(Duration::from_millis(250)),
        }
    }

    pub fn quiet(&self) -> bool {
        self.rustc == 0 && self.cargo == 0
    }
}

/// Wait up to `patience` for no other build; the last sample and whether
/// quiet was reached.
pub fn wait_for_quiet(patience: Duration) -> (LoadSample, Duration, bool) {
    let started = Instant::now();
    loop {
        let sample = LoadSample::take();
        if sample.quiet() || started.elapsed() >= patience {
            let quiet = sample.quiet();
            return (sample, started.elapsed(), quiet);
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}
