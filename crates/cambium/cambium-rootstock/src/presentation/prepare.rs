// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Shared model, layout and producer preparation, before either graphics route.
use super::*;
use crate::meristem_bounds::RootView;
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
        self.s.last_redraw_presentation = None;
        self.s.last_raster_failure = None;
        let (backend, generation, mode) = match &self.s.presentation_driver {
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
        let phase = crate::Instant::now();
        let animating = self.prepare_frame();
        profile.frame_hook_us = micros(phase.elapsed());
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
        let phase = crate::Instant::now();
        let anim_active = self.relayout(lw, lh);
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
            profile.producers = self.prepare_producers(scale);
            if let Some(refusal) = self.s.shared.producers.render_failure().cloned() {
                return Err(failure(
                    Some(refusal.source_key),
                    refusal.failure.to_string(),
                    Some(NativePresentationFailure::Rasterize(refusal.failure)),
                ));
            }
            let phase = crate::Instant::now();
            self.sync_leaf_fragments();
            profile.leaf_fragments_us = micros(phase.elapsed());
        }
        let phase = crate::Instant::now();
        self.sync_ime_area();
        profile.ime_us = micros(phase.elapsed());
        let phase = crate::Instant::now();
        let paint = match mode {
            FrameContentMode::NativeTexture => {
                let Some(list) = self.emit_paint_list(lw, lh, None) else {
                    return Ok(None);
                };
                PreparedPaint::Native(paint_list_api::PaintEnvelope::from_list(&list))
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
        let sequence = self
            .s
            .presentation_sequence
            .checked_add(1)
            .ok_or_else(|| failure(None, "presentation sequence exhausted".into(), None))?;
        Ok(Some(PreparedFrame {
            paint,
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
