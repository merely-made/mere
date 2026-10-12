// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shared-core loss handling in the actual host, before producer/GPU work.
use crate::meristem_bounds::RootView;
use crate::{CaptureRefusal, Host, HostTree};
use genet_render_host::{RecoveryStatus, SharedRenderCore};
use std::sync::Arc;

impl<State, Logic, V, T> Host<State, Logic, V, T>
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
    T: HostTree<State>,
{
    fn refuse_pending_captures(&mut self, refusal: CaptureRefusal) {
        if let Some(capture) = self.s.pending_capture.take() {
            capture(Err(refusal.clone()));
        }
        if let Some(capture) = self.s.pending_stamped_capture.take() {
            capture(Err(refusal.clone()));
        }
        if let Some(capture) = self.s.pending_paint_capture.take() {
            capture(Err(refusal));
        }
        self.s.pending_presentation_observer = None;
    }
    pub fn attach_shared_recovery(&mut self, owner: Arc<SharedRenderCore>) {
        if self
            .s
            .shared
            .render_recovery
            .as_ref()
            .is_some_and(|old| Arc::ptr_eq(old, &owner))
            && self.s.recovery_subscription.is_some()
        {
            return;
        }
        self.s.recovery_subscription = Some(owner.subscribe(self.wake.callback()));
        self.s.shared.render_recovery = Some(owner);
    }

    /// Each live host drops its own GPU resources before acknowledging. The
    /// shared owner boots once only after every live host has acknowledged.
    /// Application runners, layout, CPU leaves and portable sources survive.
    pub fn poll_device_recovery(&mut self) -> RecoveryStatus {
        if self.s.shared.render_recovery.is_none()
            && let Some(core) = self.s.shared.render_core.clone()
        {
            self.attach_shared_recovery(core.shared_recovery());
        }
        let Some(owner) = self.s.shared.render_recovery.clone() else {
            return RecoveryStatus::Ready {
                generation: self
                    .s
                    .shared
                    .render_core
                    .as_ref()
                    .map_or(0, |core| core.generation()),
            };
        };
        if self.s.recovery_subscription.is_none() {
            self.attach_shared_recovery(owner.clone());
        }
        let status = owner.status();
        if let Some(loss) = status.loss() {
            if self.s.detached_generation != Some(loss.generation) {
                self.s.last_redraw_presentation = None;
                // Clear every producer in this shared core, including leaves
                // held by another window. Per-window suspension is insufficient.
                self.s.shared.producers.suspend_all(None);
                self.s.shared.leaf_fragments.clear();
                self.s.surface = None;
                self.s.shared.render_core = None;
                self.s.needs_recovery_surface = true;
                self.s.detached_generation = Some(loss.generation);
            }
            self.refuse_pending_captures(CaptureRefusal::DeviceLost(loss.clone()));
            owner.acknowledge_detached(
                self.s
                    .recovery_subscription
                    .as_ref()
                    .expect("subscribed host"),
                loss.generation,
            );
        }
        let status = owner.recover();
        if let RecoveryStatus::Ready { generation } = status {
            if let Some(RecoveryStatus::SurfaceUnavailable {
                generation: failed, ..
            }) = &self.s.recovery_status
                && *failed == generation
            {
                let status = self.s.recovery_status.clone().expect("surface refusal");
                if let RecoveryStatus::SurfaceUnavailable { generation, error } = &status {
                    self.refuse_pending_captures(CaptureRefusal::SurfaceUnavailable {
                        generation: *generation,
                        error: error.clone(),
                    });
                }
                return status;
            }
            if let Ok(core) = owner.current() {
                self.s.shared.render_core = Some(core);
            }
        }
        self.s.recovery_status = Some(status.clone());
        status
    }

    /// A new device is ready but this window's surface could not be configured.
    /// Keep the failure typed and explicit; never reattach the lost surface.
    pub fn refuse_recovery_surface(&mut self, generation: u64, error: String) {
        self.s.surface = None;
        self.s.last_redraw_presentation = None;
        self.s.recovery_status = Some(RecoveryStatus::SurfaceUnavailable {
            generation,
            error: error.clone(),
        });
        let refusal = CaptureRefusal::SurfaceUnavailable { generation, error };
        self.refuse_pending_captures(refusal);
    }
}
