// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The spawn guard (harness plan ruling 8): every child gets a complete
//! environment the harness built, and the guard refuses it before spawning
//! when any root or endpoint is not redirected under the run, a standard
//! endpoint appears, or a variable it does not know would reach the child.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::walls::is_under;

/// Roots every child must have, each pointing under the run's directory.
pub const REDIRECTED_ROOTS: &[&str] = &[
    "LOCALAPPDATA",
    "APPDATA",
    "USERPROFILE",
    "HOME",
    "XDG_DATA_HOME",
    "XDG_CONFIG_HOME",
    "XDG_CACHE_HOME",
    "XDG_RUNTIME_DIR",
    "TEMP",
    "TMP",
    "TMPDIR",
    "MERE_ROOT",
];

/// Endpoint variables every child must have, none of them standard.
pub const REDIRECTED_ENDPOINTS: &[&str] =
    &["GRAPHSHELL_APP_ENDPOINT", "GRAPHSHELL_DEVICE_ENDPOINT"];

/// Variables that would point a child at the user's own state; never passed.
pub const FORBIDDEN: &[&str] = &[
    "SSH_AUTH_SOCK",
    "DJINN_DATA_ROOT",
    "GRAPHSHELL_PROFILE",
    "PERSONAE_PROFILE",
    "DBUS_SESSION_BUS_ADDRESS",
];

/// Copied from the harness's own environment: what a process needs to run,
/// and nothing that names a user root.
#[cfg(windows)]
pub const PASSED_THROUGH: &[&str] = &[
    "SystemRoot",
    "windir",
    "SystemDrive",
    "PATH",
    "PATHEXT",
    "ComSpec",
    "NUMBER_OF_PROCESSORS",
    "PROCESSOR_ARCHITECTURE",
    "OS",
    "COMPUTERNAME",
    "USERNAME",
    "USERDOMAIN",
];
#[cfg(not(windows))]
pub const PASSED_THROUGH: &[&str] = &["PATH", "LANG", "LC_ALL", "USER", "LOGNAME", "TZ"];

/// Flags only the installed resident is launched with.
const FORBIDDEN_FLAGS: &[&str] = &["--installed", "--agent-endpoint"];
/// Endpoint flags a resident must carry, each redirected.
pub const ENDPOINT_FLAGS: &[&str] = &[
    "--receipt-agent-endpoint",
    "--browser-endpoint",
    "--app-endpoint",
];
/// Path flags that must point under the run.
const PATH_FLAGS: &[&str] = &[
    "--dir",
    "--data-root",
    "--log-file",
    "--events-file",
    "--sync-store",
];

/// What is spawned: a resident must carry every endpoint flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnKind {
    Resident,
    Command,
}

/// Why a spawn was refused, before anything started.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("spawn refused: {0}")]
pub struct Refusal(pub String);

/// What the guard checks a spawn against.
#[derive(Clone, Debug)]
pub struct Guard {
    pub run_dir: PathBuf,
    /// Pipe names must carry it, so two runs cannot share an endpoint.
    pub run_token: String,
    pub standard_endpoints: Vec<String>,
    /// Session buses the harness itself started (Linux).
    pub owned_buses: BTreeSet<String>,
}

impl Guard {
    /// Check a child's complete environment and arguments.
    pub fn check(
        &self,
        kind: SpawnKind,
        env: &BTreeMap<String, String>,
        args: &[String],
        unguarded: &BTreeSet<String>,
    ) -> Result<(), Refusal> {
        let refuse = |why: String| Err(Refusal(why));
        for name in REDIRECTED_ROOTS {
            match env.get(*name) {
                None => return refuse(format!("{name} is not redirected")),
                Some(_) if unguarded.contains(*name) => {},
                Some(value) if !is_under(Path::new(value), &self.run_dir) => {
                    return refuse(format!("{name}={value} is outside the run"));
                },
                Some(_) => {},
            }
        }
        for name in REDIRECTED_ENDPOINTS {
            match env.get(*name) {
                None => return refuse(format!("{name} is not redirected")),
                Some(value) => self.endpoint(name, value)?,
            }
        }
        for (name, value) in env {
            if FORBIDDEN.contains(&name.as_str()) {
                let owned = name == "DBUS_SESSION_BUS_ADDRESS" && self.owned_buses.contains(value);
                // An SSH client may name this run's own agent endpoint, and
                // nothing else (the lock's agent receipts, harness H4).
                let own_agent = name == "SSH_AUTH_SOCK" && self.endpoint(name, value).is_ok();
                if !owned && !own_agent {
                    return refuse(format!("{name} would reach the user's own state"));
                }
            }
        }
        let mut seen = BTreeSet::new();
        let mut args = args.iter();
        while let Some(arg) = args.next() {
            if FORBIDDEN_FLAGS.contains(&arg.as_str()) {
                return refuse(format!("{arg} is the installed resident's"));
            }
            if ENDPOINT_FLAGS.contains(&arg.as_str()) {
                let value = args
                    .next()
                    .ok_or_else(|| Refusal(format!("{arg} has no value")))?;
                self.endpoint(arg, value)?;
                seen.insert(arg.as_str());
            } else if PATH_FLAGS.contains(&arg.as_str()) {
                let value = args
                    .next()
                    .ok_or_else(|| Refusal(format!("{arg} has no value")))?;
                if !is_under(Path::new(value), &self.run_dir) {
                    return refuse(format!("{arg} {value} is outside the run"));
                }
            }
        }
        if kind == SpawnKind::Resident
            && let Some(missing) = ENDPOINT_FLAGS.iter().find(|flag| !seen.contains(*flag))
        {
            return refuse(format!("a resident needs {missing}"));
        }
        Ok(())
    }

    fn endpoint(&self, name: &str, value: &str) -> Result<(), Refusal> {
        let standard = self.standard_endpoints.iter().any(|standard| {
            if cfg!(windows) {
                standard.eq_ignore_ascii_case(value)
            } else {
                standard == value
            }
        });
        if standard {
            return Err(Refusal(format!("{name} {value} is a standard endpoint")));
        }
        #[cfg(windows)]
        let isolated = value.to_ascii_lowercase().starts_with(r"\\.\pipe\")
            && value
                .to_ascii_lowercase()
                .contains(&self.run_token.to_ascii_lowercase());
        #[cfg(not(windows))]
        let isolated = is_under(Path::new(value), &self.run_dir);
        if !isolated {
            return Err(Refusal(format!(
                "{name} {value} is not this run's endpoint"
            )));
        }
        Ok(())
    }
}

/// A child environment: the pass-through set from this process, the
/// redirects under `root`, and the endpoint variables.
pub fn isolated_env(
    root: &Path,
    app_endpoint: &str,
    browser_endpoint: &str,
) -> BTreeMap<String, String> {
    let mut env: BTreeMap<String, String> = PASSED_THROUGH
        .iter()
        .filter_map(|name| Some((name.to_string(), std::env::var(name).ok()?)))
        .collect();
    let at = |sub: &str| root.join(sub).display().to_string();
    for (name, value) in [
        ("LOCALAPPDATA", at("local")),
        ("XDG_DATA_HOME", at("local")),
        ("APPDATA", at("roaming")),
        ("USERPROFILE", at("home")),
        ("HOME", at("home")),
        ("XDG_CONFIG_HOME", at("config")),
        ("XDG_CACHE_HOME", at("cache")),
        ("XDG_RUNTIME_DIR", at("runtime")),
        ("TEMP", at("tmp")),
        ("TMP", at("tmp")),
        ("TMPDIR", at("tmp")),
        ("MERE_ROOT", at("mere")),
    ] {
        env.insert(name.into(), value);
    }
    env.insert("GRAPHSHELL_APP_ENDPOINT".into(), app_endpoint.into());
    env.insert("GRAPHSHELL_DEVICE_ENDPOINT".into(), browser_endpoint.into());
    env
}

/// The directories [`isolated_env`] names, created so the child finds them.
pub fn create_roots(env: &BTreeMap<String, String>) -> std::io::Result<()> {
    for name in REDIRECTED_ROOTS {
        if let Some(value) = env.get(*name) {
            std::fs::create_dir_all(value)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guard(dir: &Path) -> Guard {
        Guard {
            run_dir: dir.to_path_buf(),
            run_token: "run1234".into(),
            standard_endpoints: crate::walls::standard_endpoints(),
            owned_buses: BTreeSet::new(),
        }
    }

    fn endpoint(dir: &Path, door: &str) -> String {
        if cfg!(windows) {
            format!(r"\\.\pipe\djinn-test-run1234-{door}")
        } else {
            dir.join(format!("{door}.sock")).display().to_string()
        }
    }

    fn resident_args(dir: &Path) -> Vec<String> {
        vec![
            "--dir".into(),
            dir.join("a/vault").display().to_string(),
            "--receipt-agent-endpoint".into(),
            endpoint(dir, "agent"),
            "--browser-endpoint".into(),
            endpoint(dir, "browser"),
            "--app-endpoint".into(),
            endpoint(dir, "app"),
        ]
    }

    fn good(dir: &Path) -> BTreeMap<String, String> {
        isolated_env(
            &dir.join("a"),
            &endpoint(dir, "app"),
            &endpoint(dir, "browser"),
        )
    }

    #[test]
    fn a_complete_isolated_spawn_passes() {
        let dir = std::env::temp_dir().join("djinn-testkit-guard");
        let none = BTreeSet::new();
        guard(&dir)
            .check(
                SpawnKind::Resident,
                &good(&dir),
                &resident_args(&dir),
                &none,
            )
            .unwrap();
    }

    /// C1: each missing redirect, each standard endpoint and each leak is
    /// refused by name.
    #[test]
    fn every_missing_or_standard_redirect_is_refused() {
        let dir = std::env::temp_dir().join("djinn-testkit-guard");
        let guard = guard(&dir);
        let none = BTreeSet::new();
        for name in REDIRECTED_ROOTS.iter().chain(REDIRECTED_ENDPOINTS) {
            let mut env = good(&dir);
            env.remove(*name);
            let refused = guard.check(SpawnKind::Resident, &env, &resident_args(&dir), &none);
            assert!(
                refused.as_ref().is_err_and(|r| r.0.contains(name)),
                "{name}: {refused:?}"
            );
        }
        // A deliberately unredirected root (C4): the real value leaks through.
        let mut env = good(&dir);
        env.insert(
            "LOCALAPPDATA".into(),
            std::env::temp_dir().display().to_string(),
        );
        assert!(
            guard
                .check(SpawnKind::Resident, &env, &resident_args(&dir), &none)
                .is_err()
        );
        for name in FORBIDDEN {
            let mut env = good(&dir);
            env.insert(name.to_string(), "anything".into());
            assert!(
                guard
                    .check(SpawnKind::Resident, &env, &resident_args(&dir), &none)
                    .is_err(),
                "{name}"
            );
        }
        for standard in crate::walls::standard_endpoints() {
            let mut args = resident_args(&dir);
            args[3] = standard.clone();
            assert!(
                guard
                    .check(SpawnKind::Resident, &good(&dir), &args, &none)
                    .is_err(),
                "{standard}"
            );
            let mut env = good(&dir);
            env.insert("GRAPHSHELL_APP_ENDPOINT".into(), standard.clone());
            assert!(guard.check(SpawnKind::Command, &env, &[], &none).is_err());
        }
        for flag in ENDPOINT_FLAGS {
            let args: Vec<String> = resident_args(&dir)
                .chunks(2)
                .filter(|pair| pair[0] != *flag)
                .flatten()
                .cloned()
                .collect();
            assert!(
                guard
                    .check(SpawnKind::Resident, &good(&dir), &args, &none)
                    .is_err()
            );
        }
        for flag in ["--installed", "--agent-endpoint"] {
            let mut args = resident_args(&dir);
            args.push(flag.into());
            assert!(
                guard
                    .check(SpawnKind::Resident, &good(&dir), &args, &none)
                    .is_err()
            );
        }
        let mut args = resident_args(&dir);
        args[1] = std::env::temp_dir().join("vault").display().to_string();
        assert!(
            guard
                .check(SpawnKind::Resident, &good(&dir), &args, &none)
                .is_err()
        );
    }

    /// An SSH client may reach this run's agent endpoint, and only that:
    /// the standard agent and any other path are refused.
    #[test]
    fn ssh_auth_sock_is_only_ever_this_runs_agent() {
        let dir = std::env::temp_dir().join("djinn-testkit-guard");
        let guard = guard(&dir);
        let none = BTreeSet::new();
        let with = |value: &str| {
            let mut env = good(&dir);
            env.insert("SSH_AUTH_SOCK".into(), value.into());
            guard.check(SpawnKind::Command, &env, &[], &none)
        };
        with(&endpoint(&dir, "agent")).unwrap();
        for standard in crate::walls::standard_endpoints() {
            assert!(with(&standard).is_err(), "{standard}");
        }
        assert!(with("agent.sock").is_err());
        assert!(with(r"\\.\pipe\someone-elses-agent").is_err());
    }
}
