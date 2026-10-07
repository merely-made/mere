// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One receipt run: the machine lock, the walls before and after, the job
//! that owns every spawned process, the record, and the receipt it writes.

use std::collections::BTreeSet;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::{Value, json};

use crate::guard::Guard;
use crate::load::{self, LoadSample};
use crate::receipt::{
    self, Assertion, AttendedStep, BinaryRecord, Bound, ControlRecord, HostRecord, LoadRecord,
    MachineRecord, Outcome, QuietRecord, RECEIPT_FILE, RECEIPT_SCHEMA, Receipt, SUMMARY_FILE, Step,
    WallsRecord,
};
use crate::resident::Resident;
use crate::sys;
use crate::walls::{self, WallSpec, Walls};

/// Where raw records go: `<root>/<plan>/<run>/`. Outside the tree.
pub const RECEIPTS_ROOT_ENV: &str = "DJINN_RECEIPTS_ROOT";
/// Opt-in quiet-machine mode: wait up to this many seconds for no other build.
pub const QUIET_ENV: &str = "DJINN_TESTKIT_QUIET";
/// The machine-wide lock that serializes live receipts.
pub const MACHINE_LOCK: &str = "djinn-testkit-live-receipts.lock";

#[derive(Debug, thiserror::Error)]
pub enum HarnessError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Refused(#[from] crate::guard::Refusal),
    #[error("the machine did not go quiet within {0:?}")]
    NotQuiet(Duration),
    #[error("receipts root {0} is inside the repository; raw records hold test vaults")]
    InsideTree(PathBuf),
    #[error("{0}")]
    Failed(String),
}

/// Compiled-in control switches: each makes the harness catch something.
#[derive(Clone, Debug, Default)]
pub struct Controls {
    /// C1: feed the closing wall check a changed installed identity.
    pub perturb_installed_identity: bool,
}

/// What a run is and where it records.
#[derive(Clone, Debug)]
pub struct RunConfig {
    pub plan: String,
    pub test: String,
    pub binary: PathBuf,
    pub receipts_root: PathBuf,
    pub quiet: Option<Duration>,
    pub walls: WallSpec,
    pub controls: Controls,
}

impl RunConfig {
    pub fn new(plan: &str, test: &str, binary: impl Into<PathBuf>) -> Self {
        Self {
            plan: plan.into(),
            test: test.into(),
            binary: binary.into(),
            receipts_root: std::env::var_os(RECEIPTS_ROOT_ENV)
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::temp_dir().join("djinn-receipts")),
            quiet: std::env::var(QUIET_ENV)
                .ok()
                .and_then(|secs| secs.parse().ok())
                .map(Duration::from_secs),
            walls: WallSpec::from_environment(),
            controls: Controls::default(),
        }
    }
}

#[derive(Default)]
pub(crate) struct Record {
    steps: Vec<Step>,
    assertions: Vec<Assertion>,
    controls: Vec<ControlRecord>,
    evidence: Vec<PathBuf>,
    failures: Vec<String>,
}

/// One spawned process: id, creation time, label.
#[derive(Clone, Debug)]
pub(crate) struct Spawned {
    pub pid: u32,
    pub started: u64,
    pub label: String,
}

/// What a run shares with its residents.
pub(crate) struct Shared {
    pub dir: PathBuf,
    pub token: String,
    pub binary: PathBuf,
    pub guard: Guard,
    pub job: sys::Job,
    pub started: Instant,
    pub record: Mutex<Record>,
    pub spawned: Mutex<Vec<Spawned>>,
}

impl Shared {
    pub fn t_ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }

    pub fn step(&self, label: &str, detail: Value) {
        let step = Step {
            t_ms: self.t_ms(),
            at_ms: crate::now_ms(),
            label: label.into(),
            detail,
        };
        self.record.lock().unwrap().steps.push(step);
    }

    pub fn evidence(&self, path: PathBuf) {
        let mut record = self.record.lock().unwrap();
        if !record.evidence.contains(&path) {
            record.evidence.push(path);
        }
    }

    pub fn assertion(
        &self,
        name: &str,
        bound: Bound,
        expected: Value,
        observed: Value,
        passed: bool,
    ) -> bool {
        let mut record = self.record.lock().unwrap();
        if !passed {
            record
                .failures
                .push(format!("{name}: expected {expected}, observed {observed}"));
        }
        record.assertions.push(Assertion {
            t_ms: self.started.elapsed().as_millis() as u64,
            name: name.into(),
            bound,
            expected,
            observed,
            passed,
        });
        passed
    }
}

/// A live receipt run. Finish it to write the record; dropped unfinished (a
/// panic), it still writes one, failed.
pub struct Run {
    shared: Arc<Shared>,
    config: RunConfig,
    id: String,
    started_ms: u64,
    lock_wait_ms: u64,
    quiet: QuietRecord,
    load_before: LoadSample,
    walls_before: Walls,
    commit: Option<String>,
    dirty: Option<bool>,
    _lock: File,
    finished: bool,
}

impl Run {
    /// Take the machine lock, sample load, capture the walls, open the run
    /// directory and the job.
    pub fn begin(config: RunConfig) -> Result<Self, HarnessError> {
        if let Some(top) = git(&["rev-parse", "--show-toplevel"]) {
            let top = PathBuf::from(top);
            if walls::is_under(&config.receipts_root, &top) || config.receipts_root == top {
                return Err(HarnessError::InsideTree(config.receipts_root));
            }
        }
        let waiting = Instant::now();
        let lock = File::create(std::env::temp_dir().join(MACHINE_LOCK))?;
        lock.lock()?;
        let lock_wait_ms = waiting.elapsed().as_millis() as u64;
        let quiet = match config.quiet {
            None => QuietRecord {
                requested: false,
                waited_ms: 0,
                achieved: None,
            },
            Some(patience) => {
                let (_, waited, achieved) = load::wait_for_quiet(patience);
                if !achieved {
                    return Err(HarnessError::NotQuiet(patience));
                }
                QuietRecord {
                    requested: true,
                    waited_ms: waited.as_millis() as u64,
                    achieved: Some(true),
                }
            },
        };
        let load_before = LoadSample::take();
        let walls_before = config.walls.capture();
        let token = token();
        let id = format!("{}-{token}", utc_stamp(crate::now_ms() / 1000));
        let dir = config.receipts_root.join(&config.plan).join(&id);
        std::fs::create_dir_all(&dir)?;
        let guard = Guard {
            run_dir: dir.clone(),
            run_token: token.clone(),
            standard_endpoints: config.walls.standard_endpoints.clone(),
            owned_buses: BTreeSet::new(),
        };
        let shared = Arc::new(Shared {
            dir,
            token,
            binary: config.binary.clone(),
            guard,
            job: sys::Job::new()?,
            started: Instant::now(),
            record: Mutex::new(Record::default()),
            spawned: Mutex::new(Vec::new()),
        });
        let run = Self {
            shared,
            id,
            started_ms: crate::now_ms(),
            lock_wait_ms,
            quiet,
            load_before,
            walls_before,
            commit: git(&["rev-parse", "HEAD"]),
            dirty: git(&["status", "--porcelain", "--untracked-files=no"]).map(|s| !s.is_empty()),
            _lock: lock,
            finished: false,
            config,
        };
        run.step(
            "run begun",
            json!({ "dir": run.dir(), "lock_wait_ms": lock_wait_ms }),
        );
        Ok(run)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn dir(&self) -> &Path {
        &self.shared.dir
    }

    /// Short and unique to the run; every endpoint name carries it.
    pub fn token(&self) -> &str {
        &self.shared.token
    }

    pub fn walls_before(&self) -> &Walls {
        &self.walls_before
    }

    /// A resident under `<run>/<name>`, not yet started.
    pub fn resident(&self, name: &str) -> Resident {
        Resident::new(Arc::clone(&self.shared), name)
    }

    /// Milliseconds since the run began.
    pub fn t_ms(&self) -> u64 {
        self.shared.t_ms()
    }

    pub fn step(&self, label: &str, detail: Value) {
        self.shared.step(label, detail);
    }

    /// Record an assertion; returns whether it passed.
    pub fn assertion(
        &self,
        name: &str,
        bound: Bound,
        expected: impl Serialize,
        observed: impl Serialize,
        passed: bool,
    ) -> bool {
        self.shared.assertion(
            name,
            bound,
            serde_json::to_value(expected).unwrap_or(Value::Null),
            serde_json::to_value(observed).unwrap_or(Value::Null),
            passed,
        )
    }

    /// Record an assertion and panic when it failed; the record survives.
    pub fn require(
        &self,
        name: &str,
        bound: Bound,
        expected: impl Serialize,
        observed: impl Serialize + std::fmt::Debug,
        passed: bool,
    ) {
        let shown = format!("{observed:?}");
        if !self.assertion(name, bound, expected, observed, passed) {
            panic!("{name}: observed {shown}");
        }
    }

    /// Record a control: something that had to fail, and whether it did.
    pub fn control(&self, name: &str, expected_failure: &str, observed: &str, failed: bool) {
        let mut record = self.shared.record.lock().unwrap();
        if !failed {
            record
                .failures
                .push(format!("control {name} did not fail: {observed}"));
        }
        record.controls.push(ControlRecord {
            name: name.into(),
            expected_failure: expected_failure.into(),
            observed: observed.into(),
            failed_as_expected: failed,
        });
    }

    /// Hash this file into the record at finish.
    pub fn evidence(&self, path: impl Into<PathBuf>) {
        self.shared.evidence(path.into());
    }

    /// Write `value` as `<run>/evidence/<name>.json` and record it.
    pub fn write_evidence(&self, name: &str, value: &impl Serialize) -> PathBuf {
        let path = self.dir().join("evidence").join(format!("{name}.json"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
        self.evidence(path.clone());
        path
    }

    /// Poll `probe` until it answers, recording the wait as a step; past
    /// `patience` the failure is recorded and the test panics.
    pub fn wait_for<T>(
        &self,
        what: &str,
        patience: Duration,
        probe: impl FnMut() -> Option<T>,
    ) -> (T, Duration) {
        wait_for(&self.shared, what, patience, probe)
    }

    /// Write the record and return it; does not panic on failure.
    pub fn finish_record(mut self) -> Receipt {
        self.finished = true;
        self.write(None)
    }

    /// Write the record, and panic when the run failed.
    pub fn finish(self) -> Receipt {
        let receipt = self.finish_record();
        assert!(
            receipt.outcome.passed,
            "the run failed: {:?}",
            receipt.outcome.failures
        );
        receipt
    }

    fn write(&mut self, unfinished: Option<String>) -> Receipt {
        let shared = &self.shared;
        let alive: Vec<String> = shared
            .spawned
            .lock()
            .unwrap()
            .iter()
            .filter(|spawned| sys::alive(spawned.pid, spawned.started))
            .map(|spawned| format!("{} (pid {})", spawned.label, spawned.pid))
            .collect();
        if unfinished.is_none() {
            self.assertion(
                "every spawned process has exited",
                Bound::State,
                Vec::<String>::new(),
                &alive,
                alive.is_empty(),
            );
            let left = walls::pipes_containing(&shared.token);
            self.assertion(
                "no endpoint of this run remains",
                Bound::State,
                Vec::<String>::new(),
                &left,
                left.is_empty(),
            );
        }
        let mut walls_after = self.config.walls.capture();
        if self.config.controls.perturb_installed_identity {
            match walls_after.installed.first_mut() {
                Some(process) => process.started = process.started.wrapping_add(1),
                None => walls_after.installed.push(walls::InstalledProcess {
                    path: "control".into(),
                    pid: 0,
                    started: 0,
                }),
            }
        }
        let differences = self.walls_before.differences(&walls_after);
        let load_after = LoadSample::take();
        let mut record = std::mem::take(&mut *shared.record.lock().unwrap());
        record
            .failures
            .extend(differences.iter().map(|d| format!("wall: {d}")));
        if let Some(why) = unfinished {
            record.failures.push(why);
        }
        let evidence = record
            .evidence
            .iter()
            .filter_map(|path| receipt::evidence(&shared.dir, path).ok())
            .collect();
        let receipt = Receipt {
            schema: RECEIPT_SCHEMA.into(),
            plan: self.config.plan.clone(),
            test: self.config.test.clone(),
            run: self.id.clone(),
            commit: self.commit.clone(),
            dirty: self.dirty,
            binary: BinaryRecord {
                path: self.config.binary.display().to_string(),
                sha256: receipt::sha256_file(&self.config.binary)
                    .ok()
                    .map(|(sha, _)| sha),
            },
            machine: machine(),
            host: HostRecord {
                kind: "local".into(),
                remote: None,
            },
            quiet: self.quiet.clone(),
            lock_wait_ms: self.lock_wait_ms,
            load: LoadRecord {
                before: self.load_before.clone(),
                after: Some(load_after),
            },
            walls: WallsRecord {
                before: self.walls_before.clone(),
                after: Some(walls_after),
                differences,
            },
            started_ms: self.started_ms,
            finished_ms: crate::now_ms(),
            steps: record.steps,
            assertions: record.assertions,
            controls: record.controls,
            attended: Vec::<AttendedStep>::new(),
            evidence,
            outcome: Outcome {
                passed: record.failures.is_empty(),
                failures: record.failures,
            },
        };
        let dir = &shared.dir;
        let _ = std::fs::write(
            dir.join(RECEIPT_FILE),
            serde_json::to_vec_pretty(&receipt).unwrap_or_default(),
        );
        let _ = std::fs::write(dir.join(SUMMARY_FILE), receipt::summary_markdown(&receipt));
        println!(
            "RECEIPT {} {} at {}",
            receipt.run,
            if receipt.outcome.passed {
                "passed"
            } else {
                "failed"
            },
            dir.join(RECEIPT_FILE).display()
        );
        receipt
    }
}

impl Drop for Run {
    fn drop(&mut self) {
        if !self.finished {
            self.finished = true;
            let why = if std::thread::panicking() {
                "the test panicked before the run finished"
            } else {
                "the run was dropped without finish"
            };
            self.write(Some(why.into()));
        }
    }
}

pub(crate) fn wait_for<T>(
    shared: &Shared,
    what: &str,
    patience: Duration,
    mut probe: impl FnMut() -> Option<T>,
) -> (T, Duration) {
    let started = Instant::now();
    loop {
        if let Some(found) = probe() {
            let took = started.elapsed();
            shared.step(what, json!({ "waited_ms": took.as_millis() as u64 }));
            return (found, took);
        }
        if started.elapsed() >= patience {
            shared.assertion(
                what,
                Bound::Absolute,
                json!({ "within_ms": patience.as_millis() as u64 }),
                json!("not observed"),
                false,
            );
            panic!("no {what} within {patience:?}");
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn git(args: &[&str]) -> Option<String> {
    let output = std::process::Command::new("git").args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn token() -> String {
    use std::hash::{BuildHasher, Hasher};
    let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
    hasher.write_u128(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    );
    hasher.write_u32(std::process::id());
    format!("{:08x}", hasher.finish() as u32)
}

/// `YYYYMMDDTHHMMSSZ` from Unix seconds (civil-from-days).
fn utc_stamp(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}{month:02}{day:02}T{:02}{:02}{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

fn machine() -> MachineRecord {
    let name = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .ok()
        .or_else(|| {
            std::fs::read_to_string("/etc/hostname")
                .ok()
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "unknown".into());
    MachineRecord {
        name,
        os: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn utc_stamps_are_civil_dates() {
        assert_eq!(super::utc_stamp(0), "19700101T000000Z");
        // 2026-10-05 15:14:00 UTC
        assert_eq!(super::utc_stamp(1_791_213_240), "20261005T151400Z");
    }
}
