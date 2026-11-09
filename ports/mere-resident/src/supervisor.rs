// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A supervised application (cycle breaks P2 step 3).
//!
//! The resident keeps one application running while the vault is unlocked
//! and ends it on lock, without naming the application: the owner setting
//! gives a command line ([`SupervisedAppSettings`]), and the application
//! reaches the resident only through the door like any other (custody, the
//! blob route), holding what the owner granted. On lock the resident refuses
//! the application new keys (the custody route already does) and ends the
//! process here; on unlock it starts it again.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tokio::process::{Child, Command};

use crate::settings::SupervisedAppSettings;

/// The least time between two starts of a child that keeps exiting.
pub const RESTART_BACKOFF: Duration = Duration::from_secs(5);
/// How long a child gets to exit after being killed before the stop reports it.
const REAP_LIMIT: Duration = Duration::from_secs(10);

pub struct AppSupervisor {
    settings: SupervisedAppSettings,
    data_root: PathBuf,
    child: Option<Child>,
    last_start: Option<Instant>,
}

impl AppSupervisor {
    pub fn new(settings: SupervisedAppSettings, data_root: &Path) -> Self {
        Self {
            settings,
            data_root: data_root.to_path_buf(),
            child: None,
            last_start: None,
        }
    }

    /// The application's id, for logs and events.
    pub fn app(&self) -> &str {
        &self.settings.app
    }

    fn expand(&self, text: &str, endpoint: &str) -> String {
        text.replace("{data_root}", &self.data_root.display().to_string())
            .replace("{endpoint}", endpoint)
    }

    /// Start the child if it is not running. Returns whether one was started.
    /// The child is killed if this supervisor is dropped.
    pub fn start(&mut self, endpoint: &str) -> Result<bool, String> {
        if self.is_running() {
            return Ok(false);
        }
        let mut command = Command::new(self.expand(&self.settings.command, endpoint));
        command
            .args(self.settings.args.iter().map(|arg| self.expand(arg, endpoint)))
            .env(ostiary::app_admission::APP_ENDPOINT_ENV, endpoint)
            .stdin(std::process::Stdio::null())
            .kill_on_drop(true);
        self.last_start = Some(Instant::now());
        let child = command
            .spawn()
            .map_err(|error| format!("could not start supervised {}: {error}", self.settings.app))?;
        self.child = Some(child);
        Ok(true)
    }

    /// Start the child again if it exited, no sooner than [`RESTART_BACKOFF`]
    /// after the last start. Returns whether one was started.
    pub fn ensure_running(&mut self, endpoint: &str) -> Result<bool, String> {
        if self.is_running() {
            return Ok(false);
        }
        if self.last_start.is_some_and(|at| at.elapsed() < RESTART_BACKOFF) {
            return Ok(false);
        }
        self.start(endpoint)
    }

    /// Whether the child is running now. A child that exited is dropped.
    pub fn is_running(&mut self) -> bool {
        match self.child.as_mut().map(Child::try_wait) {
            Some(Ok(None)) => true,
            Some(_) => {
                self.child = None;
                false
            },
            None => false,
        }
    }

    /// The child's process id, while it runs.
    pub fn pid(&self) -> Option<u32> {
        self.child.as_ref().and_then(Child::id)
    }

    /// End the child and wait for it. Returns whether one was running.
    pub async fn stop(&mut self) -> Result<bool, String> {
        let Some(mut child) = self.child.take() else {
            return Ok(false);
        };
        if child.try_wait().map_err(|error| error.to_string())?.is_some() {
            return Ok(false);
        }
        child.start_kill().map_err(|error| error.to_string())?;
        tokio::time::timeout(REAP_LIMIT, child.wait())
            .await
            .map_err(|_| format!("supervised {} did not exit after being killed", self.settings.app))?
            .map_err(|error| error.to_string())?;
        Ok(true)
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn sh(script: &str) -> SupervisedAppSettings {
        SupervisedAppSettings {
            app: "fixture".into(),
            command: "sh".into(),
            args: vec!["-c".into(), script.into()],
        }
    }

    #[tokio::test]
    async fn a_child_runs_until_stopped_and_gets_the_door_endpoint() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("env");
        let script = format!(
            "printf '%s' \"$GRAPHSHELL_APP_ENDPOINT\" > {}; sleep 60",
            marker.display()
        );
        let mut supervisor = AppSupervisor::new(sh(&script), dir.path());
        assert!(supervisor.start("/tmp/door.sock").unwrap());
        assert!(!supervisor.start("/tmp/door.sock").unwrap(), "already running");
        for _ in 0..100 {
            if marker.exists() && std::fs::read_to_string(&marker).unwrap() == "/tmp/door.sock" {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert_eq!(std::fs::read_to_string(&marker).unwrap(), "/tmp/door.sock");
        assert!(supervisor.is_running());
        assert!(supervisor.stop().await.unwrap());
        assert!(!supervisor.is_running());
        assert!(!supervisor.stop().await.unwrap(), "nothing left to stop");
    }

    #[tokio::test]
    async fn placeholders_expand_and_a_crashing_child_waits_out_its_backoff() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("args");
        let mut settings = sh("printf '%s' \"$0\" > \"$1\"");
        settings.args.push("{data_root}".into());
        settings.args.push(marker.display().to_string());
        let mut supervisor = AppSupervisor::new(settings, dir.path());
        assert!(supervisor.start("e").unwrap());
        for _ in 0..100 {
            if !supervisor.is_running() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert!(!supervisor.is_running(), "the child exited");
        assert_eq!(
            std::fs::read_to_string(&marker).unwrap(),
            dir.path().display().to_string()
        );
        assert!(!supervisor.ensure_running("e").unwrap(), "inside the backoff");
    }

    #[tokio::test]
    async fn a_missing_command_is_a_named_error() {
        let dir = tempfile::tempdir().unwrap();
        let mut supervisor = AppSupervisor::new(
            SupervisedAppSettings {
                app: "fixture".into(),
                command: "/nonexistent/app".into(),
                args: Vec::new(),
            },
            dir.path(),
        );
        assert!(supervisor.start("e").unwrap_err().contains("supervised fixture"));
    }
}
