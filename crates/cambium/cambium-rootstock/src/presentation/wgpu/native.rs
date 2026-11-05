// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Encoding borrows the host's native batch; acceptance belongs to its caller.

use super::*;

impl WgpuPresentation<'_> {
    /// Encode a native candidate without waiting for qualification or adding
    /// it to the accepted output cache. The caller seals the owning batch.
    pub fn encode_native(
        &self,
        packet: &PreparedFrame,
        submission: &genet_render_host::NativeFrameSubmission,
        profile: &mut FrameProfile,
        cache: &mut RasterCache,
    ) -> Result<WgpuOutput, PresentationFailure> {
        let (PreparedPaint::Native(_), Some(translated)) = (&packet.paint, &packet.native_scene)
        else {
            return Err(PresentationFailure {
                backend: PresentationBackend::Wgpu,
                generation: packet.generation,
                producer_key: None,
                message: "native presenter requires bound native paint".into(),
                native: None,
            });
        };
        reset_raster_profile(profile);
        let surface = self.surface;
        if let Some(loss) = surface.core().device_failure() {
            cache.clear();
            return Err(self.failure(packet, NativePresentationFailure::DeviceLost(loss)));
        }
        let policy_allows_reuse = surface.core().renderer().vello_output_reuse_allowed();
        if !policy_allows_reuse {
            cache.clear();
        }
        if let Some(output) = cache.lookup(surface.device(), packet, policy_allows_reuse) {
            // The caller already owns native exclusive admission. Acquiring
            // legacy checked admission here would reject our own batch.
            let output = output.clone();
            if let Some(loss) = surface.core().device_failure() {
                cache.clear();
                return Err(self.failure(packet, NativePresentationFailure::DeviceLost(loss)));
            }
            profile.raster_reused = true;
            return Ok(output);
        }
        let phase = crate::Instant::now();
        if !translated.box_shadow_masks.is_empty() {
            let renderer = surface.core().renderer();
            for mask in &translated.box_shadow_masks {
                renderer.build_box_shadow_mask(
                    mask.key,
                    mask.dim,
                    mask.bounds,
                    mask.corner_radius,
                    mask.blur_radius_px,
                    mask.invert,
                );
            }
        }
        profile.shadows_us = elapsed_us(phase.elapsed());
        let clear = wgpu::Color {
            r: f64::from(packet.clear[0]),
            g: f64::from(packet.clear[1]),
            b: f64::from(packet.clear[2]),
            a: f64::from(packet.clear[3]),
        };
        let phase = crate::Instant::now();
        let result = submission.rasterize(
            packet.host,
            &translated.scene,
            packet.width,
            packet.height,
            ColorLoad::Clear(clear),
            packet.scale,
        );
        let (texture, view) = result.map_err(|failure| {
            if matches!(failure, genet_render_host::RasterizeFailure::DeviceLost(_)) {
                cache.clear();
            }
            self.failure(packet, refused(failure))
        })?;
        profile.raster_us = elapsed_us(phase.elapsed());
        let renderer = surface.core().renderer();
        if let Some(timings) = renderer.last_frame_timings() {
            let span = |name: &str| timings.span(name).map(elapsed_us).unwrap_or_default();
            profile.raster_total_us = elapsed_us(timings.total);
            profile.tile_invalidate_us = span("tile_invalidate");
            profile.dirty_tile_rebuild_us = span("dirty_tile_rebuild");
            profile.master_compose_us = span("master_compose");
            profile.vello_render_us = span("vello_render");
        }
        profile.dirty_tiles = renderer.vello_last_dirty_count().unwrap_or_default() as u64;
        Ok(WgpuOutput {
            _texture: texture,
            view,
        })
    }

    /// Accept retention only after the caller has qualified this candidate
    /// and successfully presented the same packet/output. Never call on an
    /// unavailable surface, cancellation or failed publication.
    pub fn commit_native_cache(
        &self,
        packet: &PreparedFrame,
        output: &WgpuOutput,
        cache: &mut RasterCache,
    ) {
        if self.surface.core().device_failure().is_some()
            || !self.surface.core().renderer().vello_output_reuse_allowed()
        {
            cache.clear();
            return;
        }
        cache.insert(self.surface.device().clone(), packet, output.clone());
    }
}
