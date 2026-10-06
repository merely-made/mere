// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The `mere.djinn.receipt/v1` record (harness plan ruling 6), its verifier,
//! and the Markdown summary a lane may commit.
//!
//! A record lives at `<receipts root>/<plan>/<run>/receipt.json`, beside the
//! evidence it hashes; the run directory holds test vaults, so it stays out
//! of the tree.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::load::LoadSample;
use crate::walls::Walls;

pub const RECEIPT_SCHEMA: &str = "mere.djinn.receipt/v1";
pub const RECEIPT_FILE: &str = "receipt.json";
pub const SUMMARY_FILE: &str = "summary.md";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Receipt {
    pub schema: String,
    pub plan: String,
    pub test: String,
    pub run: String,
    pub commit: Option<String>,
    pub dirty: Option<bool>,
    pub binary: BinaryRecord,
    pub machine: MachineRecord,
    /// Local today; the remote runner and attended steps (H5) fill these.
    pub host: HostRecord,
    pub quiet: QuietRecord,
    pub lock_wait_ms: u64,
    pub load: LoadRecord,
    pub walls: WallsRecord,
    pub started_ms: u64,
    pub finished_ms: u64,
    pub steps: Vec<Step>,
    pub assertions: Vec<Assertion>,
    pub controls: Vec<ControlRecord>,
    pub attended: Vec<AttendedStep>,
    pub evidence: Vec<Evidence>,
    pub outcome: Outcome,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BinaryRecord {
    pub path: String,
    pub sha256: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineRecord {
    pub name: String,
    pub os: String,
    pub arch: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostRecord {
    /// `local`, or the remote runner's name.
    pub kind: String,
    pub remote: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuietRecord {
    pub requested: bool,
    pub waited_ms: u64,
    pub achieved: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoadRecord {
    pub before: LoadSample,
    pub after: Option<LoadSample>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WallsRecord {
    pub before: Walls,
    pub after: Option<Walls>,
    pub differences: Vec<String>,
}

/// One timestamped step: `t_ms` from the run's start, `at_ms` wall clock.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub t_ms: u64,
    pub at_ms: u64,
    pub label: String,
    pub detail: Value,
}

/// How an assertion's numbers are meant (ruling 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bound {
    /// Against the run's own events: holds under load.
    Relative,
    /// A wide wall-clock bound, recorded and flagged rather than trusted.
    Absolute,
    /// A state, not a time.
    State,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Assertion {
    pub t_ms: u64,
    pub name: String,
    pub bound: Bound,
    pub expected: Value,
    pub observed: Value,
    pub passed: bool,
}

/// A control: something that must fail, and whether it did.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlRecord {
    pub name: String,
    pub expected_failure: String,
    pub observed: String,
    pub failed_as_expected: bool,
}

/// Reserved for H5: a step a person performs while the harness waits.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttendedStep {
    pub label: String,
    pub armed_ms: u64,
    pub done_ms: Option<u64>,
}

/// One evidence file, relative to the run directory.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    pub passed: bool,
    pub failures: Vec<String>,
}

pub fn sha256_file(path: &Path) -> std::io::Result<(String, u64)> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1 << 16];
    let mut bytes = 0u64;
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        bytes += read as u64;
    }
    Ok((hex(&hasher.finalize()), bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Hash one evidence file for the record.
pub fn evidence(run_dir: &Path, path: &Path) -> std::io::Result<Evidence> {
    let (sha256, bytes) = sha256_file(path)?;
    let relative = path.strip_prefix(run_dir).unwrap_or(path);
    Ok(Evidence {
        path: relative.to_string_lossy().replace('\\', "/"),
        sha256,
        bytes,
    })
}

/// What the verifier found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verification {
    pub receipt: PathBuf,
    pub checked: usize,
    pub problems: Vec<String>,
}

impl Verification {
    pub fn ok(&self) -> bool {
        self.problems.is_empty()
    }
}

/// Recompute every evidence hash in `run_dir/receipt.json`.
pub fn verify(run_dir: &Path) -> std::io::Result<Verification> {
    let path = run_dir.join(RECEIPT_FILE);
    let receipt: Receipt = serde_json::from_slice(&std::fs::read(&path)?)?;
    let mut problems = Vec::new();
    if receipt.schema != RECEIPT_SCHEMA {
        problems.push(format!("schema {} is not {RECEIPT_SCHEMA}", receipt.schema));
    }
    for item in &receipt.evidence {
        let file = run_dir.join(&item.path);
        match sha256_file(&file) {
            Ok((sha256, bytes)) if sha256 == item.sha256 && bytes == item.bytes => {},
            Ok((sha256, bytes)) => problems.push(format!(
                "{}: recorded {} ({} bytes), recomputed {sha256} ({bytes} bytes)",
                item.path, item.sha256, item.bytes
            )),
            Err(error) => problems.push(format!("{}: {error}", item.path)),
        }
    }
    Ok(Verification {
        receipt: path,
        checked: receipt.evidence.len(),
        problems,
    })
}

/// The committed summary: outcome, identity, load, walls, assertions,
/// controls and evidence hashes, one Markdown page.
pub fn summary_markdown(receipt: &Receipt) -> String {
    let mut out = String::new();
    let verdict = if receipt.outcome.passed {
        "passed"
    } else {
        "failed"
    };
    let _ = writeln!(out, "# {} / {}: {verdict}\n", receipt.plan, receipt.test);
    let _ = writeln!(out, "- schema: `{}`", receipt.schema);
    let _ = writeln!(out, "- run: `{}`", receipt.run);
    let _ = writeln!(
        out,
        "- commit: `{}`{}",
        receipt.commit.as_deref().unwrap_or("unknown"),
        if receipt.dirty == Some(true) {
            " (dirty)"
        } else {
            ""
        }
    );
    let _ = writeln!(
        out,
        "- binary sha256: `{}`",
        receipt.binary.sha256.as_deref().unwrap_or("unknown")
    );
    let _ = writeln!(
        out,
        "- machine: {} ({} {}), host {}",
        receipt.machine.name, receipt.machine.os, receipt.machine.arch, receipt.host.kind
    );
    let load = |sample: &LoadSample| {
        format!(
            "rustc {}, cargo {}, queue {:?}, busy {:?}",
            sample.rustc, sample.cargo, sample.cpu_queue, sample.cpu_busy
        )
    };
    let _ = writeln!(out, "- load before: {}", load(&receipt.load.before));
    if let Some(after) = &receipt.load.after {
        let _ = writeln!(out, "- load after: {}", load(after));
    }
    let _ = writeln!(
        out,
        "- duration: {} ms; lock wait {} ms",
        receipt.finished_ms.saturating_sub(receipt.started_ms),
        receipt.lock_wait_ms
    );
    let installed = |walls: &Walls| {
        walls
            .installed
            .iter()
            .map(|p| format!("{} pid {} start {}", p.path, p.pid, p.started))
            .collect::<Vec<_>>()
            .join("; ")
    };
    let _ = writeln!(
        out,
        "- installed before: {}",
        installed(&receipt.walls.before)
    );
    if let Some(after) = &receipt.walls.after {
        let _ = writeln!(out, "- installed after: {}", installed(after));
    }
    let _ = writeln!(
        out,
        "- walls: {}",
        if receipt.walls.differences.is_empty() {
            "unchanged".to_string()
        } else {
            receipt.walls.differences.join("; ")
        }
    );
    if !receipt.outcome.failures.is_empty() {
        let _ = writeln!(out, "\n## Failures\n");
        for failure in &receipt.outcome.failures {
            let _ = writeln!(out, "- {failure}");
        }
    }
    let _ = writeln!(out, "\n## Assertions\n");
    let _ = writeln!(out, "| t (ms) | name | bound | expected | observed | |");
    let _ = writeln!(out, "|---|---|---|---|---|---|");
    for a in &receipt.assertions {
        let _ = writeln!(
            out,
            "| {} | {} | {:?} | {} | {} | {} |",
            a.t_ms,
            a.name,
            a.bound,
            a.expected,
            a.observed,
            if a.passed { "pass" } else { "FAIL" }
        );
    }
    if !receipt.controls.is_empty() {
        let _ = writeln!(out, "\n## Controls\n");
        for c in &receipt.controls {
            let _ = writeln!(
                out,
                "- {}: expected {}; observed {}; {}",
                c.name,
                c.expected_failure,
                c.observed,
                if c.failed_as_expected {
                    "failed as it should"
                } else {
                    "DID NOT FAIL"
                }
            );
        }
    }
    let _ = writeln!(out, "\n## Evidence\n");
    for e in &receipt.evidence {
        let _ = writeln!(
            out,
            "- `{}` {} bytes, sha256 `{}`",
            e.path, e.bytes, e.sha256
        );
    }
    out
}
