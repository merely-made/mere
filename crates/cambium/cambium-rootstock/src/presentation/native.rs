// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One in-flight native picture. Collection never polls or waits on the GPU.
use super::{
    publication::PreparedCaptures,
    wgpu::{OutputPresentation, WgpuOutput, WgpuPresentation},
    *,
};
use crate::{CaptureRefusal, FrameProfile, Host, HostTree, Surface, meristem_bounds::RootView};
use ::wgpu;
use genet_render_host::{
    NativeFrameStartError, PendingNativeFrame, RasterizeFailure, RecoveryStatus,
};
use std::rc::Rc;

mod attribution;
mod submit;
mod timing;
mod wake;

struct NativePacket {
    prepared: PreparedFrame,
    output: WgpuOutput,
    input: Rc<crate::owned_layout::PresentedInput>,
    observers: Vec<PreparedPresentationObserver>,
}

#[cfg(test)]
mod cpu_tests;
#[cfg(test)]
mod driver_tests;
#[cfg(test)]
mod tests;
pub(crate) struct PendingFrame {
    completion: PendingNativeFrame<NativePacket>,
    captures: PreparedCaptures,
    stamp: crate::PresentedFrame,
    device: wgpu::Device,
    core_generation: u64,
    profile: FrameProfile,
}
impl PendingFrame {
    pub(crate) fn refuse(&mut self, reason: CaptureRefusal) {
        self.captures.refuse(reason);
    }
}
fn micros(elapsed: crate::Duration) -> u64 {
    elapsed.as_micros().min(u64::MAX as u128) as u64
}
fn refusal(failure: &PresentationFailure) -> CaptureRefusal {
    match &failure.native {
        Some(NativePresentationFailure::DeviceLost(loss))
        | Some(NativePresentationFailure::Rasterize(RasterizeFailure::DeviceLost(loss))) => {
            CaptureRefusal::DeviceLost(loss.clone())
        },
        Some(NativePresentationFailure::Rasterize(RasterizeFailure::ProviderError(error))) => {
            CaptureRefusal::ProviderError {
                generation: failure.generation.0,
                source_key: failure.producer_key,
                error: error.clone(),
            }
        },
        _ => CaptureRefusal::PresentationError {
            backend: failure.backend,
            generation: failure.generation,
            source_key: failure.producer_key,
            error: failure.message.clone(),
        },
    }
}
fn raster_failure(generation: ContextGeneration, error: RasterizeFailure) -> PresentationFailure {
    PresentationFailure {
        backend: PresentationBackend::Wgpu,
        generation,
        producer_key: None,
        message: error.to_string(),
        native: Some(NativePresentationFailure::Rasterize(error)),
    }
}
impl<State, Logic, V, T> Host<State, Logic, V, T>
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
    T: HostTree<State>,
{
    fn native_surface(&self) -> Option<&dyn Surface> {
        self.s.surface.as_deref().or_else(|| {
            self.s
                .presentation_driver
                .as_ref()
                .and_then(|d| d.wgpu_surface())
        })
    }
    pub(crate) fn cancel_native(&mut self, reason: CaptureRefusal) {
        self.s.native_waiter = None;
        if let Some(mut pending) = self.s.pending_native.take() {
            pending.refuse(reason);
            drop(pending);
        }
        self.discard_native_staging();
    }
    fn discard_native_staging(&mut self) {
        let surface = self.s.surface.as_deref().or_else(|| {
            self.s
                .presentation_driver
                .as_ref()
                .and_then(|d| d.wgpu_surface())
        });
        self.s
            .shared
            .producers
            .discard_staged(surface.map(|s| s.core()));
    }
    fn request_native_turn(&self) {
        if let Some(window) = self.s.window.as_ref() {
            window.request_redraw();
        }
    }
    pub(crate) fn prepared_input_is_current(
        &self,
        input: &crate::owned_layout::PresentedInput,
    ) -> bool {
        !self.s.native_dirty
            && self.s.runner.as_ref().is_some_and(|runner| {
                let dom = runner.dom();
                let dom = dom.borrow();
                input.matches_dom(&crate::WindowDom::new(&dom, runner.mount()))
                    && self
                        .s
                        .layout
                        .as_ref()
                        .is_some_and(|layout| input.matches_layout(layout))
            })
    }
    /// Returns false only when this surface/source combination uses another
    /// presentation route. Newer model updates stay in the runner while pending.
    pub(crate) fn redraw_native(&mut self) -> bool {
        self.s.last_redraw_presentation = None;
        self.s.last_frame_profile = None;
        let started = crate::Instant::now();
        let work = crate::work_attribution::WorkTimer::start();
        let collecting = self.s.pending_native.is_some();
        let packet = self
            .s
            .pending_native
            .as_ref()
            .and_then(|pending| pending.profile.native_work.packet);
        let handled = self.redraw_native_inner();
        if handled && self.s.last_frame_profile.is_none() {
            // Refusal and cancellation still consumed a UI turn. Their work
            // must remain visible without becoming an accepted-frame sample.
            let elapsed = micros(started.elapsed());
            self.s.last_frame_profile = Some(FrameProfile {
                total_us: elapsed,
                native_pending: self.s.pending_native.is_some(),
                native_submit_us: if collecting { 0 } else { elapsed },
                native_collect_us: if collecting { elapsed } else { 0 },
                ..Default::default()
            });
        }
        if handled {
            if let Some(profile) = self.s.last_frame_profile.as_mut() {
                profile.native_work.packet = profile.native_work.packet.or(packet);
                profile.native_work.finish_call(
                    if collecting {
                        crate::NativeWorkCallKind::Collect
                    } else {
                        crate::NativeWorkCallKind::Submit
                    },
                    work.finish(),
                );
            }
        }
        if !collecting {
            if let Some(work) = self.s.last_frame_profile.as_ref().map(|p| p.native_work) {
                if let Some(pending) = self.s.pending_native.as_mut() {
                    pending.profile.native_work = work;
                }
            }
        }
        handled
    }
    fn redraw_native_inner(&mut self) -> bool {
        if self.s.pending_native.is_some() {
            self.collect_native();
            return true;
        }
        if self.s.close_requested {
            self.refuse_pending_captures(CaptureRefusal::HostDisposed);
            return true;
        }
        if self.s.hidden {
            return false;
        }
        if !self.s.shared.producers.can_stage_native()
            || self.s.presentation_driver.as_ref().is_some_and(|d| {
                d.backend() != PresentationBackend::Wgpu
                    || d.content_mode() != FrameContentMode::NativeTexture
                    || !d.supports_native_receiving()
            })
        {
            return false;
        }
        let Some(core) = self.native_surface().and_then(Surface::shared_core) else {
            return false;
        };
        if !matches!(self.poll_device_recovery(), RecoveryStatus::Ready { .. }) {
            return true;
        }
        if self.s.capture_owner.is_disposed() {
            return true;
        }
        let submission = match core.try_begin_native_frame() {
            Ok(submission) => {
                self.s.native_waiter = None;
                submission
            },
            Err(NativeFrameStartError::Busy) => {
                self.s.last_frame_profile = Some(FrameProfile {
                    native_pending: true,
                    ..Default::default()
                });
                self.s.native_dirty = true;
                if self
                    .s
                    .native_waiter
                    .as_ref()
                    .is_none_or(|waiter| waiter.ready())
                {
                    self.s.native_waiter = Some(
                        core.notify_native_frame_available(
                            self.wake
                                .notification(self.s.native_completion_pending.clone()),
                        ),
                    );
                }
                return true;
            },
            Err(NativeFrameStartError::Refused(error)) => {
                self.s.native_waiter = None;
                self.refuse_rasterization(error, None);
                return true;
            },
        };
        self.submit_native(core, submission)
    }
    fn pending_is_current(&self, pending: &PendingFrame) -> bool {
        !self.s.hidden
            && !self.s.close_requested
            && !self.s.capture_owner.is_disposed()
            && self
                .s
                .window
                .as_ref()
                .is_some_and(|w| w.inner_size() == (pending.stamp.width, pending.stamp.height))
            && self.layout_scale() as f32 == pending.stamp.layout_scale
            && self.native_surface().is_some_and(|s| {
                s.device() == &pending.device
                    && s.core().generation() == pending.core_generation
                    && s.core().device_failure().is_none()
            })
            && self.s.presentation_driver.as_ref().is_none_or(|d| {
                d.state() == PresentationState::Ready
                    && d.generation() == pending.stamp.context_generation
            })
            && self.s.shared.producers.staged_is_live()
    }
    fn collect_native(&mut self) {
        let started = crate::Instant::now();
        let mut pending = self.s.pending_native.take().expect("pending native frame");
        if !self.pending_is_current(&pending) {
            let loss = self
                .native_surface()
                .and_then(|surface| surface.core().device_failure());
            pending
                .captures
                .refuse(loss.map(CaptureRefusal::DeviceLost).unwrap_or(
                    CaptureRefusal::ContextLost {
                        backend: PresentationBackend::Wgpu,
                        generation: pending.stamp.context_generation,
                    },
                ));
            drop(pending);
            self.discard_native_staging();
            self.poll_device_recovery();
            if !self.s.hidden && !self.s.close_requested {
                self.request_native_turn();
            }
            return;
        }
        let accepted = match pending.completion.poll() {
            None => {
                self.s.last_frame_profile = Some(FrameProfile {
                    native_pending: true,
                    native_collect_us: micros(started.elapsed()),
                    total_us: micros(started.elapsed()),
                    ..Default::default()
                });
                self.s.pending_native = Some(pending);
                return;
            },
            Some(Ok(accepted)) => accepted,
            Some(Err(error)) => {
                let failure = raster_failure(pending.stamp.context_generation, error);
                pending.captures.refuse(refusal(&failure));
                self.discard_native_staging();
                self.refuse_presentation(failure);
                return;
            },
        };
        let timing = accepted.timing();
        attribution::capture(
            &mut pending.profile,
            accepted.batch_id(),
            accepted.worker_attribution(),
        );
        let result = accepted.publish(|packet| {
            self.publish_native(
                packet,
                &mut pending.captures,
                pending.profile,
                started,
                timing,
            )
        });
        match result {
            Ok(Ok(OutputPresentation::Presented)) => {},
            Ok(Ok(
                status @ (OutputPresentation::Unavailable | OutputPresentation::Reconfigured),
            )) => {
                pending.captures.refuse(CaptureRefusal::SurfaceUnavailable {
                    generation: pending.core_generation,
                    error: "prepared picture cancelled because the surface was unavailable".into(),
                });
                self.discard_native_staging();
                self.s.native_dirty = true;
                if status == OutputPresentation::Reconfigured {
                    self.request_native_turn();
                }
            },
            Ok(Err(failure)) => {
                pending.captures.refuse(refusal(&failure));
                self.discard_native_staging();
                self.refuse_presentation(failure);
            },
            Err(error) => {
                let failure = raster_failure(pending.stamp.context_generation, error);
                pending.captures.refuse(refusal(&failure));
                self.discard_native_staging();
                self.refuse_presentation(failure);
            },
        }
    }
    fn publish_native(
        &mut self,
        packet: NativePacket,
        captures: &mut PreparedCaptures,
        mut profile: FrameProfile,
        collected: crate::Instant,
        timing: genet_render_host::NativeFrameTiming,
    ) -> Result<OutputPresentation, PresentationFailure> {
        if !packet
            .observers
            .iter()
            .all(PreparedPresentationObserver::is_live)
            || !self.s.shared.producers.staged_is_live()
        {
            return Err(PresentationFailure {
                backend: PresentationBackend::Wgpu,
                generation: packet.prepared.generation,
                producer_key: None,
                message: "prepared source lifetime ended".into(),
                native: None,
            });
        }
        let presented = packet.prepared.presentation_stamp();
        let status = {
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
                .expect("current native surface");
            let presenter = WgpuPresentation::new(surface);
            let status =
                presenter.present_output_status(&packet.prepared, &packet.output, &mut profile)?;
            if status == OutputPresentation::Presented {
                presenter.commit_native_cache(
                    &packet.prepared,
                    &packet.output,
                    &mut self.s.raster_cache,
                );
            }
            status
        };
        if status != OutputPresentation::Presented {
            profile.native_pending = false;
            profile.native_collect_us = micros(collected.elapsed());
            timing::finish(
                &mut profile,
                timing,
                packet.prepared.started,
                crate::Instant::now(),
            );
            profile.total_us = profile
                .native_submit_us
                .saturating_add(profile.native_collect_us);
            self.s.last_frame_profile = Some(profile);
            return Ok(status);
        }
        assert!(
            self.s.shared.producers.publish_staged(),
            "live staged sources must publish atomically on the UI thread"
        );
        let model_changed = !self.prepared_input_is_current(&packet.input);
        let previous_scale = self.input_scale() as f32;
        self.s.cursor.0 *= previous_scale / presented.layout_scale;
        self.s.cursor.1 *= previous_scale / presented.layout_scale;
        self.s.displayed_input = Some(packet.input);
        self.s.displayed_presentation = Some(presented);
        self.s.presentation_sequence = presented.sequence;
        self.s.last_redraw_presentation = Some(presented);
        self.s.last_presentation_failure = None;
        self.s.last_raster_failure = None;
        if let Some(driver) = self.s.presentation_driver.as_mut() {
            driver.native_presented(
                presented,
                NativePresentedImage {
                    output: packet.output.clone(),
                },
            );
        }
        let _ = self
            .s
            .capture_owner
            .resume(PresentationBackend::Wgpu, presented.context_generation);
        for observer in packet.observers {
            observer.publish(presented);
        }
        let phase = crate::Instant::now();
        if let Some(capture) = captures.paint.take() {
            let PreparedPaint::Native(envelope) = &packet.prepared.paint else {
                unreachable!()
            };
            capture(Ok(envelope.clone()));
        }
        if let Some(surface) = self.native_surface() {
            for callback in [captures.capture.take(), captures.stamped.take()]
                .into_iter()
                .flatten()
            {
                callback(
                    self.s
                        .capture_owner
                        .begin(PresentationBackend::Wgpu, presented, || {
                            crate::start_presentation_readback(
                                surface,
                                &packet.output.view,
                                presented,
                            )
                        }),
                );
            }
        }
        profile.capture_us = micros(phase.elapsed());
        if !model_changed {
            self.sync_ime_area();
        }
        let phase = crate::Instant::now();
        self.drain_pointer();
        profile.pointer_us = micros(phase.elapsed());
        profile.native_pending = false;
        profile.native_collect_us = micros(collected.elapsed());
        timing::finish(
            &mut profile,
            timing,
            packet.prepared.started,
            crate::Instant::now(),
        );
        profile.total_us = profile
            .native_submit_us
            .saturating_add(profile.native_collect_us);
        self.s.last_frame_profile = Some(profile);
        // A model update can have coalesced while this frame was pending.
        if packet.prepared.animating || model_changed || self.s.native_dirty {
            self.request_native_turn();
        }
        Ok(OutputPresentation::Presented)
    }
}
