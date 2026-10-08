// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Linux's lock signals, from systemd-logind and GNOME: the graphical
//! session's `Lock` (`loginctl lock-session`), `PrepareForSleep` under a
//! delay inhibitor released only after the lock (ruling 75), and idle from
//! Mutter's `IdleMonitor`, else logind's `IdleSinceHint`, else unknown
//! (ruling 73).

use std::os::unix::fs::MetadataExt;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use zbus::zvariant::{OwnedFd, OwnedObjectPath};
use zbus::{Connection, Proxy};

use super::LockSignal;
use super::host::TriggerHost;

const LOGIND: &str = "org.freedesktop.login1";
const MANAGER_PATH: &str = "/org/freedesktop/login1";
const MANAGER: &str = "org.freedesktop.login1.Manager";
const USER: &str = "org.freedesktop.login1.User";
const SESSION: &str = "org.freedesktop.login1.Session";

/// The listener task; aborted when the triggers stop.
pub struct Sources(Option<tokio::task::JoinHandle<()>>);

impl Drop for Sources {
    fn drop(&mut self) {
        if let Some(task) = self.0.take() {
            task.abort();
        }
    }
}

/// Listen for the session's lock and the machine's sleep. A failure is
/// logged and that trigger stays off; idle still runs.
pub fn start(host: Arc<TriggerHost>) -> Sources {
    Sources(Some(tokio::spawn(async move {
        if let Err(error) = listen(host).await {
            tracing::warn!(%error, "logind lock and sleep signals unavailable");
        }
    })))
}

fn uid() -> Option<u32> {
    std::fs::metadata("/proc/self").ok().map(|meta| meta.uid())
}

/// This user's graphical session, as logind names it.
async fn display_session(system: &Connection) -> zbus::Result<OwnedObjectPath> {
    let manager = Proxy::new(system, LOGIND, MANAGER_PATH, MANAGER).await?;
    let uid = uid().ok_or_else(|| zbus::Error::Failure("no uid".into()))?;
    let user: OwnedObjectPath = manager.call("GetUser", &(uid,)).await?;
    let user = Proxy::new(system, LOGIND, user, USER).await?;
    let (_, session): (String, OwnedObjectPath) = user.get_property("Display").await?;
    Ok(session)
}

async fn inhibit(manager: &Proxy<'_>) -> Option<OwnedFd> {
    manager
        .call(
            "Inhibit",
            &("sleep", "djinn", "Lock the identity vault before sleep", "delay"),
        )
        .await
        .map_err(|error| tracing::warn!(%error, "no sleep delay; the lock may land after sleep"))
        .ok()
}

async fn listen(host: Arc<TriggerHost>) -> zbus::Result<()> {
    use futures_util::StreamExt;

    let system = Connection::system().await?;
    let manager = Proxy::new(&system, LOGIND, MANAGER_PATH, MANAGER).await?;
    let mut sleeping = manager.receive_signal("PrepareForSleep").await?;
    let mut delay = inhibit(&manager).await;

    let session = match display_session(&system).await {
        Ok(path) => Some(Proxy::new(&system, LOGIND, path, SESSION).await?),
        Err(error) => {
            tracing::warn!(%error, "no graphical session; the session lock will not lock the vault");
            None
        },
    };
    let mut locks = match &session {
        Some(session) => Some(session.receive_signal("Lock").await?),
        None => None,
    };

    loop {
        tokio::select! {
            Some(message) = sleeping.next() => {
                let start: bool = message.body().deserialize().unwrap_or(false);
                if start {
                    host.signal(LockSignal::Suspending);
                    // Locked: let the machine sleep.
                    drop(delay.take());
                } else if delay.is_none() {
                    delay = inhibit(&manager).await;
                }
            },
            Some(_) = async {
                match locks.as_mut() {
                    Some(locks) => locks.next().await,
                    None => std::future::pending().await,
                }
            } => {
                host.signal(LockSignal::SessionLocked);
            },
            else => return Ok(()),
        }
    }
}

/// The time since the last input: Mutter's monitor where GNOME runs, else
/// logind's hint where the desktop sets it, else unknown.
pub async fn idle() -> Option<Duration> {
    if let Some(idle) = mutter_idle().await {
        return Some(idle);
    }
    logind_idle().await
}

async fn mutter_idle() -> Option<Duration> {
    let session = Connection::session().await.ok()?;
    let monitor = Proxy::new(
        &session,
        "org.gnome.Mutter.IdleMonitor",
        "/org/gnome/Mutter/IdleMonitor/Core",
        "org.gnome.Mutter.IdleMonitor",
    )
    .await
    .ok()?;
    let ms: u64 = monitor.call("GetIdletime", &()).await.ok()?;
    Some(Duration::from_millis(ms))
}

/// `IdleSinceHint` is 0 until a desktop first sets the hint: then nothing is
/// known, and unknown counts as idle (ruling 74).
async fn logind_idle() -> Option<Duration> {
    let system = Connection::system().await.ok()?;
    let path = display_session(&system).await.ok()?;
    let session = Proxy::new(&system, LOGIND, path, SESSION).await.ok()?;
    let since: u64 = session.get_property("IdleSinceHint").await.ok()?;
    if since == 0 {
        return None;
    }
    let idle: bool = session.get_property("IdleHint").await.ok()?;
    if !idle {
        return Some(Duration::ZERO);
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?;
    Some(now.saturating_sub(Duration::from_micros(since)))
}
