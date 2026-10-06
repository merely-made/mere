// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One isolated djinn resident: its own roots and endpoints under the run,
//! spawned through the guard into the run's job, ready by its pipes and its
//! status route, and stopped by a kill or by the owner-only stop intent.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::json;

use crate::guard::{self, Refusal, SpawnKind};
use crate::receipt::Bound;
use crate::run::{HarnessError, Shared, Spawned, wait_for};
use crate::status::{self, ResidentEvent, ResidentStatus};
use crate::sys;
use crate::walls::endpoint_exists;

/// How long a one-shot command may run before it is killed.
const COMMAND_PATIENCE: Duration = Duration::from_secs(60);
/// The log filter a resident gets unless the test names one.
pub const DEFAULT_LOG_FILTER: &str = "info";

/// How the resident's vault opens. Lock-enabled residents will not read a
/// passphrase from the environment (vault lock ruling 7); their unlock
/// arrives with H4.
#[derive(Clone, Debug)]
pub enum Unlock {
    Passphrase(String),
}

/// The doors a resident serves.
pub const DOORS: [&str; 3] = ["agent", "browser", "app"];

struct Live {
    child: Child,
    pid: u32,
    spawned_at: Instant,
}

pub struct Resident {
    shared: Arc<Shared>,
    name: String,
    root: PathBuf,
    profile: Option<String>,
    unlock: Option<Unlock>,
    extra_env: BTreeMap<String, String>,
    left_out: BTreeSet<String>,
    unguarded: BTreeSet<String>,
    endpoint_overrides: BTreeMap<String, String>,
    log_filter: String,
    starts: u32,
    live: Option<Live>,
}

impl Resident {
    pub(crate) fn new(shared: Arc<Shared>, name: &str) -> Self {
        let root = shared.dir.join(name);
        Self {
            shared,
            name: name.into(),
            root,
            profile: None,
            unlock: None,
            extra_env: BTreeMap::new(),
            left_out: BTreeSet::new(),
            unguarded: BTreeSet::new(),
            endpoint_overrides: BTreeMap::new(),
            log_filter: DEFAULT_LOG_FILTER.into(),
            starts: 0,
            live: None,
        }
    }

    pub fn profile(mut self, profile: &str) -> Self {
        self.profile = Some(profile.into());
        self
    }

    pub fn unlock(mut self, unlock: Unlock) -> Self {
        self.unlock = Some(unlock);
        self
    }

    /// One more variable for every child of this resident; still guarded.
    pub fn env(mut self, name: &str, value: &str) -> Self {
        self.extra_env.insert(name.into(), value.into());
        self
    }

    /// Leave `name` out of every child's environment. A redirect left out
    /// is refused by the guard, which is what refusal tests rely on.
    pub fn without(mut self, name: &str) -> Self {
        self.left_out.insert(name.into());
        self
    }

    pub fn log_filter(mut self, filter: &str) -> Self {
        self.log_filter = filter.into();
        self
    }

    /// Use `value` for one door instead of the run's own endpoint. The guard
    /// still checks it, which is what refusal tests rely on.
    pub fn endpoint_override(mut self, door: &str, value: &str) -> Self {
        self.endpoint_overrides.insert(door.into(), value.into());
        self
    }

    /// Control switch (C4): point `name` at `value` and exempt it from the
    /// root check, so a canary has something to catch. Recorded as a step.
    #[doc(hidden)]
    pub fn unguarded_root(mut self, name: &str, value: &str) -> Self {
        self.shared.step(
            "control: unguarded root",
            json!({ "resident": self.name, "variable": name, "value": value }),
        );
        self.unguarded.insert(name.into());
        self.extra_env.insert(name.into(), value.into());
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn vault_dir(&self) -> PathBuf {
        self.root.join("vault")
    }

    /// What `LOCALAPPDATA` / `XDG_DATA_HOME` names for this resident.
    pub fn local(&self) -> PathBuf {
        self.root.join("local")
    }

    /// djinn's application directory under [`Self::local`].
    pub fn app_dir(&self) -> PathBuf {
        self.local().join(if cfg!(windows) {
            "Graphshell"
        } else {
            "graphshell"
        })
    }

    /// This resident's endpoint for `door`, unique to the run.
    pub fn endpoint(&self, door: &str) -> String {
        if let Some(value) = self.endpoint_overrides.get(door) {
            return value.clone();
        }
        #[cfg(windows)]
        return format!(r"\\.\pipe\djinn-{}-{}-{door}", self.shared.token, self.name);
        #[cfg(not(windows))]
        return self.root.join(format!("{door}.sock")).display().to_string();
    }

    pub fn starts(&self) -> u32 {
        self.starts
    }

    pub fn log_path(&self) -> PathBuf {
        self.root.join(format!("djinn-{}.log", self.starts))
    }

    pub fn events_path(&self) -> PathBuf {
        self.root.join(format!("events-{}.jsonl", self.starts))
    }

    fn stderr_path(&self) -> PathBuf {
        self.root.join(format!("stderr-{}.txt", self.starts))
    }

    /// The running process's id, when one is running.
    pub fn pid(&self) -> Option<u32> {
        self.live.as_ref().map(|live| live.pid)
    }

    /// When the current process was spawned.
    pub fn spawned_at(&self) -> Option<Instant> {
        self.live.as_ref().map(|live| live.spawned_at)
    }

    fn env_map(&self) -> BTreeMap<String, String> {
        let mut env =
            guard::isolated_env(&self.root, &self.endpoint("app"), &self.endpoint("browser"));
        if let Some(Unlock::Passphrase(passphrase)) = &self.unlock {
            env.insert("PERSONAE_PASSPHRASE".into(), passphrase.clone());
        }
        env.extend(self.extra_env.clone());
        env.retain(|name, _| !self.left_out.contains(name));
        env
    }

    /// A guarded command for `program`, with this resident's environment.
    /// Refused before anything spawns when a wall would be crossed.
    pub fn command(&self, program: &Path, args: &[String]) -> Result<Command, Refusal> {
        self.guarded(SpawnKind::Command, program, args)
    }

    fn guarded(
        &self,
        kind: SpawnKind,
        program: &Path,
        args: &[String],
    ) -> Result<Command, Refusal> {
        self.guarded_env(kind, program, args, self.env_map())
    }

    fn guarded_env(
        &self,
        kind: SpawnKind,
        program: &Path,
        args: &[String],
        env: BTreeMap<String, String>,
    ) -> Result<Command, Refusal> {
        if let Err(refusal) = self.shared.guard.check(kind, &env, args, &self.unguarded) {
            self.shared.step(
                "spawn refused",
                json!({ "resident": self.name, "program": program, "why": refusal.0 }),
            );
            return Err(refusal);
        }
        guard::create_roots(&env).map_err(|error| Refusal(error.to_string()))?;
        let mut command = Command::new(program);
        command
            .env_clear()
            .envs(&env)
            .args(args)
            .stdin(Stdio::null());
        #[cfg(not(windows))]
        sys::die_with_parent(&mut command);
        Ok(command)
    }

    fn adopt(&self, child: &Child, label: String) -> Result<(u32, u64), HarnessError> {
        self.shared.job.adopt(child)?;
        let pid = child.id();
        let started = sys::started(pid).unwrap_or(0);
        self.shared.spawned.lock().unwrap().push(Spawned {
            pid,
            started,
            label,
        });
        Ok((pid, started))
    }

    /// Spawn a guarded process into the run's job and leave it running.
    pub fn spawn(&self, program: &Path, args: &[String]) -> Result<Child, HarnessError> {
        let child = self
            .command(program, args)?
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let label = format!(
            "{} {}",
            program.file_name().unwrap_or_default().to_string_lossy(),
            args.first().map(String::as_str).unwrap_or("")
        );
        self.adopt(&child, label)?;
        Ok(child)
    }

    /// Run a guarded one-shot command in the run's job and collect its output.
    pub fn output(&self, program: &Path, args: &[String]) -> Result<Output, HarnessError> {
        let mut child = self.spawn(program, args)?;
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        let out = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = stdout.read_to_end(&mut bytes);
            bytes
        });
        let err = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = stderr.read_to_end(&mut bytes);
            bytes
        });
        let deadline = Instant::now() + COMMAND_PATIENCE;
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                break child.wait()?;
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        Ok(Output {
            status,
            stdout: out.join().unwrap_or_default(),
            stderr: err.join().unwrap_or_default(),
        })
    }

    /// Run a stock SSH client (`ssh-add`, `ssh-keygen`) with
    /// `SSH_AUTH_SOCK` naming this resident's own agent endpoint, which the
    /// guard checks like any endpoint; `stdin` is written and closed. The
    /// call and what it printed are recorded as a step.
    pub fn ssh_client(
        &self,
        program: &Path,
        args: &[&str],
        stdin: &[u8],
    ) -> Result<Output, HarnessError> {
        self.ssh_client_env(program, args, stdin, &[])
    }

    /// [`Self::ssh_client`] with more variables (an `SSH_ASKPASS`, say);
    /// still guarded.
    pub fn ssh_client_env(
        &self,
        program: &Path,
        args: &[&str],
        stdin: &[u8],
        extra: &[(&str, &str)],
    ) -> Result<Output, HarnessError> {
        let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
        let mut env = self.env_map();
        env.extend(extra.iter().map(|(k, v)| (k.to_string(), v.to_string())));
        env.insert("SSH_AUTH_SOCK".into(), self.endpoint("agent"));
        // Win32-OpenSSH exits 255, silently, without it. A machine root
        // (`C:\ProgramData`), not a user's.
        #[cfg(windows)]
        if let Ok(program_data) = std::env::var("ProgramData") {
            env.insert("ProgramData".into(), program_data);
        }
        let mut child = self
            .guarded_env(SpawnKind::Command, program, &args, env)?
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let label = format!(
            "{} {}",
            program.file_name().unwrap_or_default().to_string_lossy(),
            args.first().map(String::as_str).unwrap_or("")
        );
        self.adopt(&child, label)?;
        {
            use std::io::Write;
            let mut input = child.stdin.take().unwrap();
            let _ = input.write_all(stdin);
        }
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        let out = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = stdout.read_to_end(&mut bytes);
            bytes
        });
        let err = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = stderr.read_to_end(&mut bytes);
            bytes
        });
        let deadline = Instant::now() + COMMAND_PATIENCE;
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                break child.wait()?;
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        let output = Output {
            status,
            stdout: out.join().unwrap_or_default(),
            stderr: err.join().unwrap_or_default(),
        };
        self.shared.step(
            "ssh client",
            json!({
                "resident": self.name,
                "program": program,
                "args": args,
                "exit": output.status.code(),
                "stdout": String::from_utf8_lossy(&output.stdout),
                "stderr": String::from_utf8_lossy(&output.stderr),
            }),
        );
        Ok(output)
    }

    fn management_args(&self, args: &[&str]) -> Vec<String> {
        let mut all = vec!["--dir".to_string(), self.vault_dir().display().to_string()];
        if let Some(profile) = &self.profile {
            all.extend(["--profile".to_string(), profile.clone()]);
        }
        all.extend(args.iter().map(|arg| arg.to_string()));
        all
    }

    /// A djinn management command (`--pairing-facts`, `--pair-node`, ...)
    /// against this resident's vault; panics with its stderr on failure.
    pub fn djinn(&self, args: &[&str]) -> String {
        let output = self
            .output(&self.shared.binary.clone(), &self.management_args(args))
            .unwrap_or_else(|error| panic!("djinn {args:?} on {}: {error}", self.name));
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        assert!(
            output.status.success(),
            "djinn {args:?} on {}: {}\n{stdout}",
            self.name,
            String::from_utf8_lossy(&output.stderr)
        );
        stdout
    }

    /// The arguments a start uses, before the test's own.
    fn resident_args(&self, extra: &[&str]) -> Vec<String> {
        let mut args = self.management_args(&[]);
        for door in DOORS {
            let flag = if door == "agent" {
                "--receipt-agent-endpoint".to_string()
            } else {
                format!("--{door}-endpoint")
            };
            args.extend([flag, self.endpoint(door)]);
        }
        args.extend([
            "--log-file".to_string(),
            self.log_path().display().to_string(),
            "--events-file".to_string(),
            self.events_path().display().to_string(),
            "--log-filter".to_string(),
            self.log_filter.clone(),
        ]);
        args.extend(extra.iter().map(|arg| arg.to_string()));
        args
    }

    /// Spawn the resident (it must not be running). Refused by the guard
    /// before spawning when the start would cross a wall.
    pub fn start(&mut self, extra: &[&str]) -> Result<Instant, HarnessError> {
        assert!(self.live.is_none(), "{} is already running", self.name);
        self.starts += 1;
        let args = self.resident_args(extra);
        let mut command = match self.guarded(SpawnKind::Resident, &self.shared.binary, &args) {
            Ok(command) => command,
            Err(refusal) => {
                self.starts -= 1;
                return Err(refusal.into());
            },
        };
        let stderr = std::fs::File::create(self.stderr_path())?;
        let child = command.stdout(Stdio::null()).stderr(stderr).spawn()?;
        let spawned_at = Instant::now();
        let label = format!("{} start {}", self.name, self.starts);
        let (pid, _) = self.adopt(&child, label)?;
        for path in [self.log_path(), self.events_path(), self.stderr_path()] {
            self.shared.evidence(path);
        }
        self.shared.step(
            "spawned",
            json!({ "resident": self.name, "start": self.starts, "pid": pid, "args": args }),
        );
        self.live = Some(Live {
            child,
            pid,
            spawned_at,
        });
        Ok(spawned_at)
    }

    /// The resident's status route, read through `djinn --resident-status`.
    pub fn status(&self) -> Option<ResidentStatus> {
        let output = self
            .output(&self.shared.binary.clone(), &["--resident-status".into()])
            .ok()?;
        if !output.status.success() {
            return None;
        }
        serde_json::from_slice(&output.stdout).ok()
    }

    /// Ready: every door's endpoint exists (enumerated, not opened) and the
    /// status route, reached on the app door, says ready for this process.
    /// No log is read. Panics, recorded, when the process exits first.
    pub fn wait_ready(&mut self, patience: Duration) -> ResidentStatus {
        let pid = self.pid().expect("start the resident first");
        let label = format!("{} ready (start {})", self.name, self.starts);
        let endpoints: Vec<String> = DOORS.iter().map(|door| self.endpoint(door)).collect();
        let shared = Arc::clone(&self.shared);
        let (status, _) = wait_for(&shared, &label, patience, || {
            if let Some(code) = self.exited() {
                panic!(
                    "{} exited before ready ({code:?}): {}",
                    self.name,
                    std::fs::read_to_string(self.stderr_path()).unwrap_or_default()
                );
            }
            if !endpoints.iter().all(|endpoint| endpoint_exists(endpoint)) {
                return None;
            }
            self.status()
                .filter(|status| status.ready && status.pid == pid)
        });
        let since = self.spawned_at().unwrap().elapsed();
        self.shared.step(
            "ready",
            json!({ "resident": self.name, "start": self.starts, "since_spawn_ms": since.as_millis() as u64 }),
        );
        let path = self
            .shared
            .dir
            .join("evidence")
            .join(format!("{}-status-{}.json", self.name, self.starts));
        let _ = std::fs::create_dir_all(path.parent().unwrap());
        let _ = std::fs::write(
            &path,
            serde_json::to_vec_pretty(&status).unwrap_or_default(),
        );
        self.shared.evidence(path);
        status
    }

    fn exited(&mut self) -> Option<ExitStatus> {
        self.live.as_mut()?.child.try_wait().ok().flatten()
    }

    /// The crash case: terminate the process outright.
    pub fn kill(&mut self) {
        if let Some(mut live) = self.live.take() {
            let _ = live.child.kill();
            let status = live.child.wait().ok();
            self.shared.step(
                "killed",
                json!({ "resident": self.name, "pid": live.pid, "exit": status.and_then(|s| s.code()) }),
            );
        }
    }

    /// The graceful stop: the owner-only stop intent on the app door, then
    /// the wait for the process to leave. Records whether it left in time;
    /// a resident that does not is killed and the stop recorded as failed.
    fn stop_with(&mut self, patience: Duration, judge_exit: bool) -> Option<ExitStatus> {
        let live = self.live.as_ref()?;
        let pid = live.pid;
        let asked = Instant::now();
        let output = self.output(&self.shared.binary.clone(), &["--stop-resident".into()]);
        let accepted = output.as_ref().is_ok_and(|output| output.status.success());
        let deadline = asked + patience;
        let status = loop {
            if let Some(status) = self.exited() {
                break Some(status);
            }
            if Instant::now() >= deadline {
                break None;
            }
            std::thread::sleep(Duration::from_millis(50));
        };
        let took = asked.elapsed();
        let mut live = self.live.take().unwrap();
        let status = match status {
            Some(status) => Some(status),
            None => {
                let _ = live.child.kill();
                live.child.wait().ok();
                None
            },
        };
        // Two facts, kept apart: the stop reached the resident and it left,
        // and the resident's own shutdown reported success.
        self.shared.assertion(
            &format!(
                "{} left on the stop intent (start {})",
                self.name, self.starts
            ),
            Bound::Relative,
            json!({ "accepted": true, "within_ms": patience.as_millis() as u64 }),
            json!({ "accepted": accepted, "took_ms": took.as_millis() as u64, "pid": pid }),
            accepted && status.is_some(),
        );
        if judge_exit {
            let stopped = self.stopped_event();
            self.shared.assertion(
                &format!("{} shut down cleanly (start {})", self.name, self.starts),
                Bound::State,
                json!({ "exit": 0 }),
                json!({
                    "exit": status.and_then(|s| s.code()),
                    "stopped": stopped.map(|event| event.other),
                }),
                status.is_some_and(|s| s.success()),
            );
        }
        status
    }

    /// The graceful stop, judged: the resident must also exit 0.
    pub fn stop(&mut self, patience: Duration) -> Option<ExitStatus> {
        self.stop_with(patience, true)
    }

    /// The graceful stop without judging the exit, for a control that must
    /// exit badly; the caller records what it expected.
    pub fn stop_unjudged(&mut self, patience: Duration) -> Option<ExitStatus> {
        self.stop_with(patience, false)
    }

    /// The last start's `stopped` event, when it wrote one.
    pub fn stopped_event(&self) -> Option<ResidentEvent> {
        self.events()
            .into_iter()
            .rfind(|event| event.event == "stopped")
    }

    /// The lifecycle events of the current (or last) start.
    pub fn events(&self) -> Vec<ResidentEvent> {
        std::fs::read_to_string(self.events_path())
            .map(|text| status::parse_events(&text))
            .unwrap_or_default()
    }
}

impl Drop for Resident {
    fn drop(&mut self) {
        self.kill();
    }
}
