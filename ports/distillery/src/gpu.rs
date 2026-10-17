// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Process-local GPU ownership. Mesh leases remain the job authority.
//!
//! A foreground request withdraws the GPU offer immediately. The supervisor
//! asks leased work to stop; this gate waits for the actual blocking worker,
//! including a worker detached after losing its lease, to drop its guard.

use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

#[derive(Default)]
struct State {
    foreground: usize,
    occupied: bool,
}

#[derive(Default)]
struct Inner {
    state: Mutex<State>,
    changed: Condvar,
}

/// Shared by the local mesh supervisor, GPU trainer and inference lane.
#[derive(Clone, Default)]
pub struct GpuAccess(Arc<Inner>);

impl GpuAccess {
    /// Actual tensor-work ownership, including a detached blocking worker.
    pub fn is_occupied(&self) -> bool {
        self.0.state.lock().unwrap().occupied
    }

    /// True while foreground work is waiting or owns the device.
    pub fn foreground_requested(&self) -> bool {
        self.0.state.lock().unwrap().foreground != 0
    }

    /// Reserve priority before waiting, so background work cannot jump ahead.
    pub fn request(&self) -> GpuRequest {
        self.0.state.lock().unwrap().foreground += 1;
        GpuRequest {
            access: self.clone(),
        }
    }

    /// Called inside the trainer's blocking closure, before loading tensors.
    /// Refuse instead of waiting: the mesh supervisor will report the stop.
    pub fn try_background(&self) -> Option<GpuGuard> {
        let mut state = self.0.state.lock().unwrap();
        if state.foreground != 0 || state.occupied {
            return None;
        }
        state.occupied = true;
        Some(GpuGuard {
            access: self.clone(),
            foreground: false,
        })
    }
}

/// Foreground priority remains advertised until this request and its guard drop.
pub struct GpuRequest {
    access: GpuAccess,
}

impl GpuRequest {
    /// Poll cancellation even when the supervisor has not started or a worker
    /// is still inside a kernel. Never grant ownership based on a job table.
    pub fn acquire(&self, cancelled: &mut dyn FnMut() -> bool) -> Option<GpuGuard> {
        loop {
            if cancelled() {
                return None;
            }
            let mut state = self.access.0.state.lock().unwrap();
            if !state.occupied {
                state.occupied = true;
                state.foreground += 1;
                return Some(GpuGuard {
                    access: self.access.clone(),
                    foreground: true,
                });
            }
            let _wait = self
                .access
                .0
                .changed
                .wait_timeout(state, Duration::from_millis(20))
                .unwrap();
        }
    }
}

impl Drop for GpuRequest {
    fn drop(&mut self) {
        self.access.0.state.lock().unwrap().foreground -= 1;
        self.access.0.changed.notify_all();
    }
}

/// Must live inside the blocking computation, never beside its JoinHandle.
pub struct GpuGuard {
    access: GpuAccess,
    foreground: bool,
}

impl Drop for GpuGuard {
    fn drop(&mut self) {
        let mut state = self.access.0.state.lock().unwrap();
        state.occupied = false;
        if self.foreground {
            state.foreground -= 1;
        }
        self.access.0.changed.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_cancellation_and_release_follow_the_worker() {
        let access = GpuAccess::default();
        let background = access.try_background().unwrap();
        assert!(access.is_occupied());
        let request = access.request();
        assert!(access.foreground_requested());
        assert!(access.try_background().is_none());
        assert!(request.acquire(&mut || true).is_none());
        // A cancelled waiter does not evict an actual worker.
        drop(request);
        assert!(!access.foreground_requested());
        assert!(access.try_background().is_none());
        drop(background);
        assert!(!access.is_occupied());
        let request = access.request();
        let guard = request.acquire(&mut || false).unwrap();
        assert!(access.try_background().is_none());
        drop(guard);
        assert!(access.try_background().is_none());
        drop(request);
        assert!(access.try_background().is_some());
    }
    #[test]
    fn a_waiter_cancels_without_releasing_a_detached_workers_guard() {
        let access = GpuAccess::default();
        let guard = access.try_background().unwrap();
        let (release, held) = std::sync::mpsc::channel();
        // Dropping the JoinHandle detaches the actual owner, just like a lost
        // mesh lease. Its guard continues to prevent another device user.
        std::thread::spawn(move || {
            let _guard = guard;
            let _ = held.recv();
        });
        let request = access.request();
        let mut polls = 0;
        assert!(
            request
                .acquire(&mut || {
                    polls += 1;
                    polls == 3
                })
                .is_none()
        );
        assert_eq!(polls, 3);
        drop(request);
        assert!(!access.foreground_requested());
        assert!(access.try_background().is_none());
        release.send(()).unwrap();
        for _ in 0..100 {
            if access.try_background().is_some() {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("worker did not release the GPU");
    }
}
