// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What locks the vault besides an explicit intent (vault lock rulings 3,
//! 20, 73 to 75, 77): the OS session locking, suspend, and an idle window,
//! each turned on or off per device in `lock.toml`.
//!
//! This is the policy, kept apart from the OS: signals come in as
//! [`LockSignal`]s with the instant they were seen, so every rule runs on an
//! injected clock in tests. The sources that produce them live with each
//! platform.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::Deserialize;

mod host;

pub use host::{SAMPLE_EVERY, TriggerHost, TriggerRun, spawn};

#[cfg(target_os = "linux")]
mod os_linux;
#[cfg(target_os = "linux")]
use os_linux as os;
#[cfg(windows)]
mod os_windows;
#[cfg(windows)]
use os_windows as os;

/// No OS sources here yet: the explicit lock only, and idle unknown.
#[cfg(not(any(windows, target_os = "linux")))]
mod os {
    use std::sync::Arc;
    use std::time::Duration;

    pub struct Sources;

    pub fn start(_host: Arc<super::host::TriggerHost>) -> Sources {
        Sources
    }

    pub async fn idle() -> Option<Duration> {
        None
    }
}

/// The settings file in djinn's app directory (ruling 77).
pub const LOCK_SETTINGS_FILE: &str = "lock.toml";

/// Where `lock.toml` lives for an app directory.
pub fn settings_path(app_dir: &Path) -> PathBuf {
    app_dir.join(LOCK_SETTINGS_FILE)
}

/// Which triggers lock the vault on this device, and the idle window.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LockSettings {
    /// Lock when the OS session locks (`Win+L`, `loginctl lock-session`).
    pub session_lock: bool,
    /// Lock before the machine sleeps.
    pub suspend: bool,
    /// Lock after `idle_minutes` without input.
    pub idle: bool,
    /// The idle window, in minutes (ruling 20).
    pub idle_minutes: u32,
}

impl Default for LockSettings {
    fn default() -> Self {
        Self {
            session_lock: true,
            suspend: true,
            idle: true,
            idle_minutes: 15,
        }
    }
}

impl LockSettings {
    /// The idle window.
    pub fn idle_window(&self) -> Duration {
        Duration::from_secs(u64::from(self.idle_minutes) * 60)
    }
}

/// Read `lock.toml`. Absent means the defaults; malformed means the defaults
/// and a warning, never "no locking" (ruling 77).
pub fn load_settings(path: &Path) -> (LockSettings, Option<String>) {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return (LockSettings::default(), None);
        },
        Err(error) => {
            return (
                LockSettings::default(),
                Some(format!("could not read {}: {error}; using the defaults", path.display())),
            );
        },
    };
    match toml::from_str::<LockSettings>(&text) {
        Ok(settings) if settings.idle_minutes == 0 => (
            LockSettings::default(),
            Some(format!(
                "{}: an idle window of 0 minutes would lock at once; using the defaults \
                 (turn idle off with `idle = false`)",
                path.display()
            )),
        ),
        Ok(settings) => (settings, None),
        Err(error) => (
            LockSettings::default(),
            Some(format!("{} is malformed ({error}); using the defaults", path.display())),
        ),
    }
}

/// The idle rule (rulings 20, 74). A reading of activity moves the instant
/// the user was last active; an unknown reading moves nothing, so unknown
/// counts as idle. It fires once per idle stretch.
#[derive(Clone, Debug)]
pub struct IdleRule {
    window: Duration,
    last_active: Instant,
    fired: bool,
}

impl IdleRule {
    /// A rule that counts from `now`.
    pub fn new(window: Duration, now: Instant) -> Self {
        Self {
            window,
            last_active: now,
            fired: false,
        }
    }

    /// Take one reading at `now`: the time since the last input, or `None`
    /// when it cannot be read. True once the window has passed idle.
    pub fn observe(&mut self, now: Instant, idle: Option<Duration>) -> bool {
        if let Some(idle) = idle {
            let active = now.checked_sub(idle).unwrap_or(self.last_active);
            if active > self.last_active {
                self.last_active = active;
                self.fired = false;
            }
        }
        if !self.fired && now.saturating_duration_since(self.last_active) >= self.window {
            self.fired = true;
            return true;
        }
        false
    }

    /// Count afresh from `now`, after an unlock or a new window.
    pub fn rearm(&mut self, now: Instant, window: Duration) {
        self.window = window;
        self.last_active = now;
        self.fired = false;
    }
}

/// Something the OS said, as the policy sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockSignal {
    /// The OS session locked.
    SessionLocked,
    /// The machine is about to sleep; the lock must finish before the
    /// caller lets it.
    Suspending,
    /// An idle reading: the time since the last input, or unknown.
    Idle(Option<Duration>),
}

/// Why a trigger locked the vault, for the event file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockReason {
    /// The OS session locked.
    SessionLock,
    /// The machine was about to sleep.
    Suspend,
    /// The idle window passed.
    Idle,
}

impl LockReason {
    /// The event file's word for it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SessionLock => "session-lock",
            Self::Suspend => "suspend",
            Self::Idle => "idle",
        }
    }
}

/// The triggers as one policy: settings plus the idle rule.
#[derive(Clone, Debug)]
pub struct Triggers {
    settings: LockSettings,
    idle: IdleRule,
}

impl Triggers {
    /// Triggers under `settings`, counting idle from `now`.
    pub fn new(settings: LockSettings, now: Instant) -> Self {
        let idle = IdleRule::new(settings.idle_window(), now);
        Self { settings, idle }
    }

    /// The settings in force.
    pub fn settings(&self) -> &LockSettings {
        &self.settings
    }

    /// Take new settings; the idle window counts afresh.
    pub fn reconfigure(&mut self, settings: LockSettings, now: Instant) {
        self.idle.rearm(now, settings.idle_window());
        self.settings = settings;
    }

    /// The vault was unlocked: idle counts from here.
    pub fn unlocked(&mut self, now: Instant) {
        self.idle.rearm(now, self.settings.idle_window());
    }

    /// Whether `signal`, seen at `now`, locks the vault.
    pub fn decide(&mut self, signal: LockSignal, now: Instant) -> Option<LockReason> {
        match signal {
            LockSignal::SessionLocked if self.settings.session_lock => Some(LockReason::SessionLock),
            LockSignal::Suspending if self.settings.suspend => Some(LockReason::Suspend),
            LockSignal::Idle(reading) if self.settings.idle => {
                self.idle.observe(now, reading).then_some(LockReason::Idle)
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN: Duration = Duration::from_secs(60);

    #[test]
    fn idle_fires_once_after_the_window_and_activity_resets_it() {
        let t0 = Instant::now();
        let mut rule = IdleRule::new(15 * MIN, t0);
        assert!(!rule.observe(t0 + 5 * MIN, Some(5 * MIN)));
        assert!(!rule.observe(t0 + 14 * MIN, Some(14 * MIN)));
        assert!(rule.observe(t0 + 15 * MIN, Some(15 * MIN)));
        assert!(!rule.observe(t0 + 16 * MIN, Some(16 * MIN)), "once per stretch");
        // Input at 20 minutes: a fresh stretch.
        assert!(!rule.observe(t0 + 21 * MIN, Some(MIN)));
        assert!(!rule.observe(t0 + 34 * MIN, Some(14 * MIN)));
        assert!(rule.observe(t0 + 35 * MIN, Some(15 * MIN)));
    }

    #[test]
    fn an_unknown_reading_counts_as_idle() {
        let t0 = Instant::now();
        let mut rule = IdleRule::new(15 * MIN, t0);
        assert!(!rule.observe(t0 + 2 * MIN, Some(Duration::ZERO)));
        assert!(!rule.observe(t0 + 10 * MIN, None));
        assert!(rule.observe(t0 + 17 * MIN, None), "a window of unknown locks");
    }

    #[test]
    fn rearming_counts_from_the_unlock() {
        let t0 = Instant::now();
        let mut rule = IdleRule::new(15 * MIN, t0);
        assert!(rule.observe(t0 + 15 * MIN, None));
        rule.rearm(t0 + 16 * MIN, 15 * MIN);
        assert!(!rule.observe(t0 + 30 * MIN, None));
        assert!(rule.observe(t0 + 31 * MIN, None));
    }

    #[test]
    fn each_trigger_follows_its_setting() {
        let t0 = Instant::now();
        let mut on = Triggers::new(LockSettings::default(), t0);
        assert_eq!(on.decide(LockSignal::SessionLocked, t0), Some(LockReason::SessionLock));
        assert_eq!(on.decide(LockSignal::Suspending, t0), Some(LockReason::Suspend));
        assert_eq!(on.decide(LockSignal::Idle(None), t0 + 15 * MIN), Some(LockReason::Idle));

        let off = LockSettings {
            session_lock: false,
            suspend: false,
            idle: false,
            idle_minutes: 15,
        };
        let mut off = Triggers::new(off, t0);
        assert_eq!(off.decide(LockSignal::SessionLocked, t0), None);
        assert_eq!(off.decide(LockSignal::Suspending, t0), None);
        assert_eq!(off.decide(LockSignal::Idle(None), t0 + 60 * MIN), None);
    }

    #[test]
    fn settings_absent_malformed_or_zero_mean_the_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = settings_path(dir.path());
        assert_eq!(load_settings(&path), (LockSettings::default(), None));

        std::fs::write(&path, "idle = false\nidle_minutes = 5\n").unwrap();
        let (settings, warning) = load_settings(&path);
        assert!(warning.is_none());
        assert!(!settings.idle && settings.session_lock && settings.suspend);
        assert_eq!(settings.idle_window(), 5 * MIN);

        for bad in ["idle = maybe\n", "idel = false\n", "idle_minutes = 0\n"] {
            std::fs::write(&path, bad).unwrap();
            let (settings, warning) = load_settings(&path);
            assert_eq!(settings, LockSettings::default(), "{bad}");
            assert!(warning.is_some(), "{bad}");
        }
    }
}
