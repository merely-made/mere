// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The walls around the installed resident (harness plan ruling 8): what it
//! is, which standard endpoints exist, and canary timestamps under the real
//! roots, captured before a run and compared after it.
//!
//! The installed resident is found by executable path among candidate
//! locations, never by name, and its pipes are enumerated, never opened.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::sys;

/// Overrides the installed-resident candidates, separated like `PATH`.
pub const INSTALLED_ENV: &str = "DJINN_TESTKIT_INSTALLED";

/// One running process at an installed location.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledProcess {
    pub path: String,
    pub pid: u32,
    /// Platform creation time; equality is what matters.
    pub started: u64,
}

/// One canary: a directory's or file's modification time and size.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Canary {
    pub path: String,
    pub kind: String,
    pub modified_ns: Option<u64>,
    pub len: Option<u64>,
}

/// A real root to watch: directories to `depth`, and files too when `files`.
#[derive(Clone, Debug)]
pub struct CanaryRoot {
    pub path: PathBuf,
    pub depth: usize,
    pub files: bool,
}

/// What the walls are made of on this machine.
#[derive(Clone, Debug)]
pub struct WallSpec {
    pub installed: Vec<PathBuf>,
    /// The endpoints only the installed resident may hold.
    pub standard_endpoints: Vec<String>,
    pub canaries: Vec<CanaryRoot>,
}

/// The walls as observed at one moment.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Walls {
    pub installed_candidates: Vec<String>,
    pub installed: Vec<InstalledProcess>,
    /// Each standard endpoint and whether it exists.
    pub standard_endpoints: BTreeMap<String, bool>,
    pub canaries: Vec<Canary>,
    /// The session bus the harness was started under (recorded, never given
    /// to a child it did not start).
    pub session_bus: Option<String>,
}

/// The real user roots, read from this (unredirected) process.
#[derive(Clone, Debug)]
pub struct RealRoots {
    /// djinn's application directory (`Graphshell` / `graphshell`).
    pub app_dir: PathBuf,
    pub vault_dir: PathBuf,
    pub shared_root: PathBuf,
    /// Where an installer would place the resident's binaries.
    pub local_data: PathBuf,
}

impl RealRoots {
    pub fn from_environment() -> Self {
        let var = |name: &str| std::env::var_os(name).map(PathBuf::from);
        #[cfg(windows)]
        let (local_data, app) = (
            var("LOCALAPPDATA").unwrap_or_else(|| PathBuf::from(".")),
            "Graphshell",
        );
        #[cfg(not(windows))]
        let (local_data, app) = (
            var("XDG_DATA_HOME")
                .or_else(|| var("HOME").map(|home| home.join(".local/share")))
                .unwrap_or_else(|| PathBuf::from(".")),
            "graphshell",
        );
        Self {
            app_dir: local_data.join(app),
            vault_dir: local_data.join("personae").join("vault"),
            shared_root: var("MERE_ROOT").unwrap_or_else(|| {
                dirs::data_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("mere")
            }),
            local_data,
        }
    }
}

/// The standard endpoints a test resident must never bind or reach.
pub fn standard_endpoints() -> Vec<String> {
    #[cfg(windows)]
    {
        [
            "openssh-ssh-agent",
            "graphshell-device-browser",
            "graphshell-device-app",
        ]
        .iter()
        .map(|name| format!(r"\\.\pipe\{name}"))
        .collect()
    }
    #[cfg(not(windows))]
    {
        let runtime = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| RealRoots::from_environment().vault_dir);
        let mut endpoints = vec![
            runtime.join("graphshell-app.sock").display().to_string(),
            runtime.join("graphshell-device.sock").display().to_string(),
        ];
        if let Some(sock) = std::env::var("SSH_AUTH_SOCK")
            .ok()
            .filter(|sock| !sock.trim().is_empty())
        {
            endpoints.push(sock);
        }
        endpoints
    }
}

impl WallSpec {
    /// This machine's walls: the installers' default locations (or
    /// [`INSTALLED_ENV`]), the standard endpoints, and the real roots.
    pub fn from_environment() -> Self {
        let roots = RealRoots::from_environment();
        let installed = match std::env::var_os(INSTALLED_ENV) {
            Some(list) => std::env::split_paths(&list).collect(),
            None => default_installed(&roots),
        };
        Self {
            installed,
            standard_endpoints: standard_endpoints(),
            canaries: default_canaries(&roots),
        }
    }

    pub fn capture(&self) -> Walls {
        let processes = sys::processes();
        let mut installed: Vec<InstalledProcess> = processes
            .iter()
            .filter_map(|process| {
                let path = process.path.as_ref()?;
                self.installed
                    .iter()
                    .any(|candidate| same_path(candidate, path))
                    .then(|| InstalledProcess {
                        path: path.display().to_string(),
                        pid: process.pid,
                        started: process.started.unwrap_or(0),
                    })
            })
            .collect();
        installed.sort_by_key(|process| process.pid);
        Walls {
            installed_candidates: self
                .installed
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            installed,
            standard_endpoints: endpoints_present(&self.standard_endpoints),
            canaries: self.canaries.iter().flat_map(canaries_under).collect(),
            session_bus: std::env::var("DBUS_SESSION_BUS_ADDRESS").ok(),
        }
    }
}

#[cfg(windows)]
fn default_installed(roots: &RealRoots) -> Vec<PathBuf> {
    vec![
        // The legacy installer, still the installed resident on the laptop.
        roots
            .local_data
            .join(r"Graphshell\bin\graphshell-device-host.exe"),
        // `install-windows.ps1`'s default `-InstallRoot`.
        roots.local_data.join(r"Djinn\bin\djinn.exe"),
    ]
}

#[cfg(not(windows))]
fn default_installed(_: &RealRoots) -> Vec<PathBuf> {
    // No Unix installer exists yet; H5 names the remote one.
    Vec::new()
}

fn default_canaries(roots: &RealRoots) -> Vec<CanaryRoot> {
    let root = |path: PathBuf, depth, files| CanaryRoot { path, depth, files };
    vec![
        // Directory times change when an entry appears or goes, not when the
        // installed resident appends to its own log or store.
        root(roots.app_dir.clone(), 3, false),
        root(roots.app_dir.join("settings"), 1, true),
        root(roots.vault_dir.clone(), 2, true),
        root(roots.shared_root.clone(), 2, false),
        root(roots.shared_root.join("identity"), 1, true),
    ]
}

fn same_path(a: &Path, b: &Path) -> bool {
    #[cfg(windows)]
    return a
        .as_os_str()
        .to_string_lossy()
        .eq_ignore_ascii_case(&b.as_os_str().to_string_lossy());
    #[cfg(not(windows))]
    return a == b;
}

/// Whether `path` lies under `root`, component-wise (case-insensitive on
/// Windows).
pub fn is_under(path: &Path, root: &Path) -> bool {
    let normalize = |p: &Path| -> Vec<String> {
        p.components()
            .filter(|c| !matches!(c, std::path::Component::CurDir))
            .map(|c| {
                let text = c.as_os_str().to_string_lossy().into_owned();
                if cfg!(windows) {
                    text.to_ascii_lowercase()
                } else {
                    text
                }
            })
            .collect()
    };
    if path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return false;
    }
    let (path, root) = (normalize(path), normalize(root));
    !root.is_empty() && path.len() > root.len() && path[..root.len()] == root[..]
}

fn endpoints_present(endpoints: &[String]) -> BTreeMap<String, bool> {
    #[cfg(windows)]
    let pipes: Vec<String> = sys::pipe_names()
        .into_iter()
        .map(|name| name.to_ascii_lowercase())
        .collect();
    endpoints
        .iter()
        .map(|endpoint| {
            #[cfg(windows)]
            let present = endpoint
                .rsplit('\\')
                .next()
                .is_some_and(|name| pipes.contains(&name.to_ascii_lowercase()));
            #[cfg(not(windows))]
            let present = Path::new(endpoint).exists();
            (endpoint.clone(), present)
        })
        .collect()
}

/// Whether a pipe (Windows) or socket path (Unix) exists, without opening it.
pub fn endpoint_exists(endpoint: &str) -> bool {
    endpoints_present(&[endpoint.to_string()])
        .into_values()
        .next()
        .unwrap_or(false)
}

/// Every pipe whose name contains `token` (Windows); empty elsewhere.
pub fn pipes_containing(token: &str) -> Vec<String> {
    let token = token.to_ascii_lowercase();
    sys::pipe_names()
        .into_iter()
        .filter(|name| name.to_ascii_lowercase().contains(&token))
        .collect()
}

fn canary(path: &Path) -> Canary {
    let meta = std::fs::metadata(path).ok();
    Canary {
        path: path.display().to_string(),
        kind: match &meta {
            None => "missing",
            Some(meta) if meta.is_dir() => "dir",
            Some(_) => "file",
        }
        .into(),
        modified_ns: meta
            .as_ref()
            .and_then(|meta| meta.modified().ok())
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|since| since.as_nanos() as u64),
        len: meta.filter(|meta| meta.is_file()).map(|meta| meta.len()),
    }
}

fn canaries_under(root: &CanaryRoot) -> Vec<Canary> {
    let mut found = vec![canary(&root.path)];
    walk(&root.path, root.depth, root.files, &mut found);
    found
}

fn walk(dir: &Path, depth: usize, files: bool, found: &mut Vec<Canary>) {
    if depth == 0 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| Some(e.ok()?.path())).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            found.push(canary(&path));
            walk(&path, depth - 1, files, found);
        } else if files {
            found.push(canary(&path));
        }
    }
}

impl Walls {
    /// What changed between `self` (before) and `after`, in words.
    pub fn differences(&self, after: &Walls) -> Vec<String> {
        let mut changed = Vec::new();
        if self.installed != after.installed {
            changed.push(format!(
                "installed resident changed: before {:?}, after {:?}",
                self.installed, after.installed
            ));
        }
        if self.standard_endpoints != after.standard_endpoints {
            changed.push(format!(
                "standard endpoints changed: before {:?}, after {:?}",
                self.standard_endpoints, after.standard_endpoints
            ));
        }
        let before: BTreeMap<&str, &Canary> =
            self.canaries.iter().map(|c| (c.path.as_str(), c)).collect();
        let later: BTreeMap<&str, &Canary> = after
            .canaries
            .iter()
            .map(|c| (c.path.as_str(), c))
            .collect();
        for (path, canary) in &before {
            match later.get(path) {
                None => changed.push(format!("canary gone: {path}")),
                Some(now) if now != canary => changed.push(format!(
                    "canary changed: {path} ({:?}/{:?} -> {:?}/{:?})",
                    canary.modified_ns, canary.len, now.modified_ns, now.len
                )),
                Some(_) => {},
            }
        }
        for path in later.keys().filter(|path| !before.contains_key(*path)) {
            changed.push(format!("canary appeared: {path}"));
        }
        if self.session_bus != after.session_bus {
            changed.push("the session bus address changed".into());
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn under_is_component_wise_and_refuses_parent_steps() {
        let root = Path::new("/t/receipts/run");
        assert!(is_under(Path::new("/t/receipts/run/a/vault"), root));
        assert!(!is_under(Path::new("/t/receipts/run"), root));
        assert!(!is_under(Path::new("/t/receipts/run2/a"), root));
        assert!(!is_under(Path::new("/t/receipts/run/../other"), root));
    }

    #[test]
    fn a_changed_installed_identity_is_a_difference() {
        let before = Walls {
            installed_candidates: vec!["x".into()],
            installed: vec![InstalledProcess {
                path: "x".into(),
                pid: 10,
                started: 7,
            }],
            standard_endpoints: BTreeMap::new(),
            canaries: Vec::new(),
            session_bus: None,
        };
        assert!(before.differences(&before.clone()).is_empty());
        let mut moved = before.clone();
        moved.installed[0].pid = 11;
        assert_eq!(before.differences(&moved).len(), 1);
        let mut restarted = before.clone();
        restarted.installed[0].started = 8;
        assert_eq!(before.differences(&restarted).len(), 1);
    }
}
