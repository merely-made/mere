// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shared model, layout and producer preparation, before either graphics route.
use super::*;
use crate::meristem_bounds::RootView;
use crate::work_attribution::WorkTimer;
use crate::{FrameProfile, Host, HostTree, PortableCaptureError, ProducerFramePurpose};

fn micros(elapsed: crate::Duration) -> u64 {
    elapsed.as_micros().min(u64::MAX as u128) as u64
}
impl<State, Logic, V, T> Host<State, Logic, V, T>
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
    T: HostTree<State>,
{
    pub fn prepare_presentation_frame(
        &mut self,
    ) -> Result<Option<PreparedFrame>, PresentationFailure> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.prepare_with_native(None)
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.prepare_with_native()
        }
    }

    pub(crate) fn prepare_with_native(
        &mut self,
        #[cfg(not(target_arch = "wasm32"))] submission: Option<
            &genet_render_host::NativeFrameSubmission,
        >,
    ) -> Result<Option<PreparedFrame>, PresentationFailure> {
        #[cfg(not(target_arch = "wasm32"))]
        if self.s.pending_native.is_some() {
            return Ok(None);
        }
        #[cfg(not(target_arch = "wasm32"))]
        let measured = submission.is_some();
        #[cfg(target_arch = "wasm32")]
        let measured = false;
        let whole = measured.then(WorkTimer::start);
        let validation = measured.then(WorkTimer::start);
        let mut work = crate::NativePreparationWork::default();
        self.s.last_redraw_presentation = None;
        self.s.last_raster_failure = None;
        if self.s.capture_owner.is_disposed() {
            self.suspend_producers();
            return Ok(None);
        }
        let (backend, mut generation, mode) = match &self.s.presentation_driver {
            Some(driver) => (driver.backend(), driver.generation(), driver.content_mode()),
            None => (
                PresentationBackend::Wgpu,
                ContextGeneration(
                    self.s
                        .surface
                        .as_ref()
                        .map_or(0, |surface| surface.core().generation()),
                ),
                FrameContentMode::NativeTexture,
            ),
        };
        if backend == PresentationBackend::Wgpu
            && !matches!(
                self.poll_device_recovery(),
                genet_render_host::RecoveryStatus::Ready { .. }
            )
        {
            return Ok(None);
        }
        if backend == PresentationBackend::Wgpu
            && let Some(driver) = self.s.presentation_driver.as_mut()
        {
            driver.restore()?;
            generation = driver.generation();
        }
        if self
            .s
            .presentation_driver
            .as_ref()
            .is_some_and(|driver| driver.state() != PresentationState::Ready)
        {
            self.suspend_producers();
            self.refuse_pending_captures(crate::CaptureRefusal::ContextLost {
                backend,
                generation,
            });
            return Ok(None);
        }
        if self
            .s
            .surface
            .as_ref()
            .is_some_and(|surface| surface.core().device_failure().is_some())
        {
            self.poll_device_recovery();
            return Ok(None);
        }
        self.deliver_files();
        let started = crate::Instant::now();
        let mut profile = FrameProfile::default();
        work.validation = validation.and_then(WorkTimer::finish);
        let hook = measured.then(WorkTimer::start);
        let phase = crate::Instant::now();
        let animating = self.prepare_frame();
        if self.s.capture_owner.is_disposed() {
            self.suspend_producers();
            return Ok(None);
        }
        profile.frame_hook_us = micros(phase.elapsed());
        work.application_hook = hook.and_then(WorkTimer::finish);
        let setup = measured.then(WorkTimer::start);
        let scale = self.layout_scale() as f32;
        let extent = self.s.window.as_ref().map(|window| window.inner_size());
        let presentable = self.s.presentation_driver.is_some() || self.s.surface.is_some();
        let Some((width, height)) =
            extent.filter(|(w, h)| *w > 0 && *h > 0 && !self.s.hidden && presentable)
        else {
            self.suspend_producers();
            let (lw, lh) = self.logical_size();
            let phase = crate::Instant::now();
            self.relayout(lw, lh);
            profile.relayout_us = micros(phase.elapsed());
            profile.total_us = micros(started.elapsed());
            self.s.last_frame_profile = Some(profile);
            return Ok(None);
        };
        let (lw, lh) = (width as f32 / scale, height as f32 / scale);
        work.frame_setup = setup.and_then(WorkTimer::finish);
        let layout = measured.then(WorkTimer::start);
        let phase = crate::Instant::now();
        let anim_active = self.relayout(lw, lh);
        work.model_layout = layout.and_then(WorkTimer::finish);
        profile.relayout_us = micros(phase.elapsed());
        profile.layout_update_us = self.s.last_layout_update_us;
        profile.layout_tick_us = self.s.last_layout_tick_us;
        profile.layout_apply_us = self.s.last_layout_apply_us;
        profile.layout_rebuild_us = self.s.last_layout_rebuild_us;
        profile.style_resolve_us = self.s.last_style_resolve_us;
        profile.layout_with_text_us = self.s.last_layout_with_text_us;
        profile.content_extent_us = self.s.last_content_extent_us;
        profile.layout_mutations = self.s.last_layout_mutations;
        profile.layout_rebuilt = self.s.last_layout_rebuilt;
        profile.leaf_boxes_us = self.s.last_leaf_boxes_us;
        profile.leaf_render_us = self.s.last_leaf_render_us;
        profile.leaf_repaints = self.s.last_leaf_repaints;
        let failure = |producer_key, message, native| PresentationFailure {
            backend,
            generation,
            producer_key,
            message,
            native,
        };
        if mode == FrameContentMode::NativeTexture {
            let producers = measured.then(WorkTimer::start);
            #[cfg(not(target_arch = "wasm32"))]
            {
                profile.producers = if let Some(submission) = submission {
                    if !self.s.shared.producers.can_stage_native() {
                        return Err(failure(
                            None,
                            "producer changed to a source without native staging".into(),
                            None,
                        ));
                    }
                    self.prepare_producers_staged(scale, submission)
                } else {
                    self.prepare_producers(scale)
                };
            }
            #[cfg(target_arch = "wasm32")]
            {
                profile.producers = self.prepare_producers(scale);
            }
            work.producers = producers.and_then(WorkTimer::finish);
            if let Some(error) = self.s.shared.producers.gpu_remap_failure() {
                self.refuse_producer_resource(error);
                return Ok(None);
            }
            if let Some(refusal) = self.s.shared.producers.render_failure().cloned() {
                return Err(failure(
                    Some(refusal.source_key),
                    refusal.failure.to_string(),
                    Some(NativePresentationFailure::Rasterize(refusal.failure)),
                ));
            }
            let fragments = measured.then(WorkTimer::start);
            let phase = crate::Instant::now();
            self.sync_leaf_fragments().map_err(|error| {
                failure(
                    None,
                    format!("retained leaf admission refused: {error}"),
                    None,
                )
            })?;
            profile.leaf_fragments_us = micros(phase.elapsed());
            work.leaf_fragments = fragments.and_then(WorkTimer::finish);
        }
        let phase = crate::Instant::now();
        #[cfg(not(target_arch = "wasm32"))]
        if submission.is_none() {
            self.sync_ime_area();
        }
        #[cfg(target_arch = "wasm32")]
        self.sync_ime_area();
        profile.ime_us = micros(phase.elapsed());
        let phase = crate::Instant::now();
        let emission = measured.then(WorkTimer::start);
        let mut native_scene = None;
        let paint = match mode {
            FrameContentMode::NativeTexture => {
                let Some((envelope, translated)) =
                    self.emit_native_scene(lw, lh).map_err(|error| {
                        failure(None, format!("paint translation refused: {error}"), None)
                    })?
                else {
                    return Ok(None);
                };
                native_scene = Some(translated);
                PreparedPaint::Native(envelope)
            },
            FrameContentMode::PortableScene => PreparedPaint::Portable(
                self.capture_portable_for(ProducerFramePurpose::Presentation)
                    .map_err(|error| {
                        let key = match &error {
                            PortableCaptureError::Producer { texture_key, .. }
                            | PortableCaptureError::DuplicateDomKey { texture_key }
                            | PortableCaptureError::InvalidExtent { texture_key } => {
                                Some(*texture_key)
                            },
                            _ => None,
                        };
                        failure(
                            key,
                            format!("portable preparation refused: {error:?}"),
                            None,
                        )
                    })?,
            ),
        };
        profile.emit_scene_us = micros(phase.elapsed());
        work.emit_scene = emission.and_then(WorkTimer::finish);
        let binding = measured.then(WorkTimer::start);
        let sequence = self
            .s
            .presentation_sequence
            .checked_add(1)
            .ok_or_else(|| failure(None, "presentation sequence exhausted".into(), None))?;
        let native_producer_identities = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if submission.is_some() {
                    self.s.shared.producers.staged_image_identities()
                } else {
                    self.s.shared.producers.image_identities()
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                self.s.shared.producers.image_identities()
            }
        };
        work.packet_binding = binding.and_then(WorkTimer::finish);
        work.finish(whole.and_then(WorkTimer::finish));
        profile.native_work.preparation = measured.then_some(work);
        Ok(Some(PreparedFrame {
            paint,
            native_scene,
            native_producer_identities,
            host: self.s.presentation_host,
            sequence,
            width,
            height,
            scale,
            generation,
            clear: if self.options.app_frame_is_transparent() {
                [0., 0., 0., 0.]
            } else {
                [0., 0., 0., 1.]
            },
            profile,
            started,
            animating: animating || anim_active,
        }))
    }
}
