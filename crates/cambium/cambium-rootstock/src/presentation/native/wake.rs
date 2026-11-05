// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Completion is a channel drain, separate from new application/render demand.

use super::*;
use std::sync::atomic::Ordering;

impl<State, Logic, V, T> Host<State, Logic, V, T>
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
    T: HostTree<State>,
{
    /// Drain a native notification and collect only an existing candidate.
    /// True means a collection attempt ran, even if an older notification
    /// raced with a newer channel that is not ready yet. Without a candidate,
    /// only a ready Core waiter may request the previously owed picture.
    pub fn collect_presentation_wake(&mut self) -> bool {
        let notified = self
            .s
            .native_completion_pending
            .swap(false, Ordering::AcqRel);
        if !notified {
            return false;
        }
        if self.s.pending_native.is_none() {
            self.wake_native_waiter();
            return false;
        }
        // The guarded pending branch cannot enter fresh frame preparation.
        self.redraw_native();
        true
    }

    /// A previously requested picture can retry when another host releases
    /// the Core. This is resource availability, never application work or a
    /// completed frame, so it runs neither application nor frame hooks.
    pub(crate) fn wake_native_waiter(&mut self) -> bool {
        if !self
            .s
            .native_waiter
            .as_ref()
            .is_some_and(|waiter| waiter.ready())
        {
            return false;
        }
        self.s.native_waiter.take();
        if !self.s.hidden && !self.s.close_requested && !self.s.capture_owner.is_disposed() {
            if let Some(window) = self.s.window.as_ref() {
                window.request_redraw();
                return true;
            }
        }
        false
    }

    /// An OS redraw while receiving records newer demand and returns without
    /// collecting, preparing, requesting another redraw or running hooks.
    pub fn coalesce_presentation_redraw(&mut self) -> bool {
        if self.s.pending_native.is_none() && self.s.native_waiter.is_none() {
            return false;
        }
        self.s.native_dirty = true;
        true
    }
}
