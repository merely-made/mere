// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One native submission, with packet-bound UI capture and encoding scopes.

use super::*;
use crate::work_attribution::WorkTimer;
use std::sync::Arc;

impl<State, Logic, V, T> Host<State, Logic, V, T>
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
    T: HostTree<State>,
{
    pub(super) fn submit_native(
        &mut self,
        core: Arc<genet_render_host::RenderCore>,
        submission: genet_render_host::NativeFrameSubmission,
    ) -> bool {
        self.s.native_receiving = true;
        self.s.native_dirty = false;
        let prepared = match self.prepare_with_native(Some(&submission)) {
            Ok(Some(prepared)) => prepared,
            Ok(None) => {
                self.discard_native_staging();
                return true;
            },
            Err(failure) => {
                self.discard_native_staging();
                self.refuse_presentation(failure);
                return true;
            },
        };
        let stamp = prepared.presentation_stamp();
        let mut captures = PreparedCaptures {
            capture: self.s.pending_capture.take(),
            stamped: self.s.pending_stamped_capture.take(),
            paint: self.s.pending_paint_capture.take(),
        };
        let observer_work = WorkTimer::start();
        let observers = self.prepare_presentation_observers(stamp);
        let observer_work = observer_work.finish();
        let input_work = WorkTimer::start();
        let input =
            {
                let runner = self.s.runner.as_ref().expect("prepared runner");
                let dom = runner.dom();
                let dom = dom.borrow();
                let view = crate::WindowDom::new(&dom, runner.mount());
                let layout = self.s.layout.as_ref().expect("prepared layout");
                match self.s.displayed_input.as_ref().filter(|snapshot| {
                    snapshot.matches_dom(&view) && snapshot.matches_layout(layout)
                }) {
                    Some(input) => input.clone(),
                    None => Rc::new(layout.presented_input(&view)),
                }
            };
        let input_work = input_work.finish();
        let mut profile = prepared.profile;
        profile.native_work.packet = Some(crate::NativeWorkPacket {
            host: stamp.host,
            generation: core.generation(),
            batch_id: submission.batch_id(),
            proposed_sequence: stamp.sequence,
        });
        profile.native_work.observer_capture = observer_work;
        profile.native_work.input_capture = input_work;
        if self.native_surface().is_none_or(|surface| {
            surface.device() != core.device() || surface.core().generation() != core.generation()
        }) {
            self.discard_native_staging();
            captures.refuse(CaptureRefusal::ContextLost {
                backend: PresentationBackend::Wgpu,
                generation: stamp.context_generation,
            });
            return true;
        }
        let encode_work = WorkTimer::start();
        let result = {
            let surface = self
                .s
                .surface
                .as_deref()
                .or_else(|| {
                    self.s
                        .presentation_driver
                        .as_ref()
                        .and_then(|d| d.wgpu_surface())
                })
                .expect("prepared surface");
            WgpuPresentation::new(surface).encode_native(
                &prepared,
                &submission,
                &mut profile,
                &mut self.s.raster_cache,
            )
        };
        profile.native_work.encode = encode_work.finish();
        let output = match result {
            Ok(output) => output,
            Err(failure) => {
                self.discard_native_staging();
                captures.refuse(refusal(&failure));
                self.refuse_presentation(failure);
                return true;
            },
        };
        profile.native_submit_us = micros(prepared.started.elapsed());
        profile.native_pending = true;
        profile.total_us = profile.native_submit_us;
        let seal_work = WorkTimer::start();
        let completion = submission.seal_with_wake(
            NativePacket {
                prepared,
                output,
                input,
                observers,
            },
            self.wake
                .notification(self.s.native_completion_pending.clone()),
        );
        profile.native_work.seal = seal_work.finish();
        self.s.last_frame_profile = Some(profile);
        self.s.pending_native = Some(PendingFrame {
            stamp,
            device: core.device().clone(),
            core_generation: core.generation(),
            profile,
            captures,
            completion,
        });
        true
    }
}
