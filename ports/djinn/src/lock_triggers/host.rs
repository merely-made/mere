// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The triggers running in the resident: the policy, the lock it calls, and
//! the idle sampler that also watches `lock.toml`.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use graphshell::identity::VaultLockView;
use tokio::sync::watch;

use super::{LockReason, LockSignal, Triggers, load_settings, os};

/// How often idle is read and `lock.toml` checked.
pub const SAMPLE_EVERY: Duration = Duration::from_secs(15);

/// The policy and what it acts through. Sources call [`Self::signal`]; it
/// locks before it returns, so a suspend source can hold the sleep until then.
pub struct TriggerHost {
    triggers: Mutex<Triggers>,
    lock: Box<dyn Fn() -> Result<(), String> + Send + Sync>,
    locked: watch::Receiver<VaultLockView>,
    report: Box<dyn Fn(LockReason) + Send + Sync>,
}

impl TriggerHost {
    /// A host locking through `lock`, reporting each trigger to `report`.
    pub fn new(
        triggers: Triggers,
        lock: impl Fn() -> Result<(), String> + Send + Sync + 'static,
        locked: watch::Receiver<VaultLockView>,
        report: impl Fn(LockReason) + Send + Sync + 'static,
    ) -> Self {
        Self {
            triggers: Mutex::new(triggers),
            lock: Box::new(lock),
            locked,
            report: Box::new(report),
        }
    }

    /// Act on `signal`: lock when the policy says so. Nothing happens while
    /// the vault is already locked.
    pub fn signal(&self, signal: LockSignal) -> Option<LockReason> {
        if *self.locked.borrow() == VaultLockView::Locked {
            return None;
        }
        let reason = self.triggers.lock().unwrap().decide(signal, Instant::now())?;
        match (self.lock)() {
            Ok(()) => {
                (self.report)(reason);
                Some(reason)
            },
            Err(error) => {
                tracing::warn!(reason = reason.as_str(), %error, "a lock trigger could not lock");
                None
            },
        }
    }

    fn unlocked(&self) {
        self.triggers.lock().unwrap().unlocked(Instant::now());
    }

    fn reconfigure(&self, settings: super::LockSettings) {
        self.triggers
            .lock()
            .unwrap()
            .reconfigure(settings, Instant::now());
    }
}

/// The running triggers; dropping it stops the sampler and the sources.
pub struct TriggerRun {
    tasks: Vec<tokio::task::JoinHandle<()>>,
    _sources: os::Sources,
}

impl Drop for TriggerRun {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
    }
}

/// Start the triggers: the OS sources, the idle sampler (which reloads
/// `lock.toml` when it changes) and the unlock watch that restarts idle.
pub fn spawn(
    settings_path: PathBuf,
    lock: impl Fn() -> Result<(), String> + Send + Sync + 'static,
    locked: watch::Receiver<VaultLockView>,
    report: impl Fn(LockReason) + Send + Sync + 'static,
    warn: impl Fn(String) + Send + Sync + 'static,
) -> TriggerRun {
    let (settings, warning) = load_settings(&settings_path);
    if let Some(warning) = warning {
        warn(warning);
    }
    tracing::info!(?settings, "lock triggers");
    let host = Arc::new(TriggerHost::new(
        Triggers::new(settings, Instant::now()),
        lock,
        locked.clone(),
        report,
    ));
    let sources = os::start(Arc::clone(&host));

    let sampler = {
        let host = Arc::clone(&host);
        tokio::spawn(async move {
            let modified = |path: &PathBuf| -> Option<SystemTime> {
                std::fs::metadata(path).and_then(|m| m.modified()).ok()
            };
            let mut seen = modified(&settings_path);
            let mut tick = tokio::time::interval(SAMPLE_EVERY);
            loop {
                tick.tick().await;
                let now = modified(&settings_path);
                if now != seen {
                    seen = now;
                    let (settings, warning) = load_settings(&settings_path);
                    if let Some(warning) = warning {
                        warn(warning);
                    }
                    tracing::info!(?settings, "lock triggers reconfigured");
                    host.reconfigure(settings);
                }
                let reading = os::idle().await;
                host.signal(LockSignal::Idle(reading));
            }
        })
    };
    let unlocks = {
        let host = Arc::clone(&host);
        let mut locked = locked;
        tokio::spawn(async move {
            while locked.changed().await.is_ok() {
                if *locked.borrow_and_update() == VaultLockView::Unlocked {
                    host.unlocked();
                }
            }
        })
    };
    TriggerRun {
        tasks: vec![sampler, unlocks],
        _sources: sources,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::super::LockSettings;
    use super::*;

    /// A host over an injected lock: it counts locks and flips the watch,
    /// as the resident's lock does.
    fn host(
        settings: LockSettings,
    ) -> (Arc<TriggerHost>, Arc<AtomicUsize>, Arc<watch::Sender<VaultLockView>>) {
        let (state, locked) = watch::channel(VaultLockView::Unlocked);
        let state = Arc::new(state);
        let count = Arc::new(AtomicUsize::new(0));
        let (lock_state, lock_count) = (Arc::clone(&state), Arc::clone(&count));
        let host = TriggerHost::new(
            Triggers::new(settings, Instant::now()),
            move || {
                lock_count.fetch_add(1, Ordering::SeqCst);
                lock_state.send_replace(VaultLockView::Locked);
                Ok(())
            },
            locked,
            |_| {},
        );
        (Arc::new(host), count, state)
    }

    #[test]
    fn a_session_lock_and_a_suspend_lock_before_signal_returns() {
        let (host, count, state) = host(LockSettings::default());
        assert_eq!(host.signal(LockSignal::SessionLocked), Some(LockReason::SessionLock));
        assert_eq!(count.load(Ordering::SeqCst), 1, "locked before signal returned");
        assert_eq!(host.signal(LockSignal::Suspending), None, "already locked: nothing");
        assert_eq!(count.load(Ordering::SeqCst), 1);

        state.send_replace(VaultLockView::Unlocked);
        assert_eq!(host.signal(LockSignal::Suspending), Some(LockReason::Suspend));
        assert_eq!(count.load(Ordering::SeqCst), 2, "the suspend's lock finished first");
    }

    #[test]
    fn a_trigger_turned_off_does_nothing() {
        let settings = LockSettings {
            session_lock: false,
            suspend: false,
            ..LockSettings::default()
        };
        let (host, count, _state) = host(settings);
        assert_eq!(host.signal(LockSignal::SessionLocked), None);
        assert_eq!(host.signal(LockSignal::Suspending), None);
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }
}
