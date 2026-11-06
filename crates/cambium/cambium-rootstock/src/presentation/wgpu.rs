// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The existing same-device raster/acquire/submit path, behind frame preparation.
use super::{
    NativePresentationFailure, PreparedFrame, PreparedPaint, PresentationBackend,
    PresentationFailure,
};
use crate::work_attribution::WorkTimer;
use crate::{FrameProfile, Surface};
use netrender::{ColorLoad, ExternalTexturePlacement};
fn elapsed_us(elapsed: crate::Duration) -> u64 {
    elapsed.as_micros().min(u64::MAX as u128) as u64
}

fn reset_raster_profile(profile: &mut FrameProfile) {
    profile.raster_reused = false;
    profile.shadows_us = 0;
    profile.raster_us = 0;
    profile.raster_total_us = 0;
    profile.tile_invalidate_us = 0;
    profile.dirty_tile_rebuild_us = 0;
    profile.master_compose_us = 0;
    profile.vello_render_us = 0;
    profile.dirty_tiles = 0;
}
#[derive(Clone)]
pub(crate) struct WgpuOutput {
    pub _texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}
pub(crate) type RasterCache = super::raster_cache::QualifiedRasterCache<wgpu::Device, WgpuOutput>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OutputPresentation {
    Presented,
    Reconfigured,
    Unavailable,
}

pub(crate) struct WgpuPresentation<'a> {
    surface: &'a dyn Surface,
}
impl<'a> WgpuPresentation<'a> {
    pub fn new(surface: &'a dyn Surface) -> Self {
        Self { surface }
    }
    fn failure(
        &self,
        frame: &PreparedFrame,
        cause: NativePresentationFailure,
    ) -> PresentationFailure {
        PresentationFailure {
            backend: PresentationBackend::Wgpu,
            generation: frame.generation,
            producer_key: None,
            message: format!("{cause:?}"),
            native: Some(cause),
        }
    }
    pub fn present(
        &self,
        packet: &PreparedFrame,
        profile: &mut FrameProfile,
        cache: &mut RasterCache,
        trace: Option<&crate::frame::FrameTrace>,
    ) -> Result<Option<WgpuOutput>, PresentationFailure> {
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
        self.present_scene(
            packet,
            &translated.scene,
            &translated.box_shadow_masks,
            packet.scale,
            profile,
            cache,
            trace,
        )
    }

    /// Produce a qualified complete image before any surface acquisition.
    /// Kept separate so output lifetime can be checked on a real offscreen GPU.
    fn rasterize_scene(
        &self,
        packet: &PreparedFrame,
        scene: &netrender::Scene,
        box_shadow_masks: &[paint_list_render::BoxShadowMaskRequest],
        raster_scale: f32,
        profile: &mut FrameProfile,
        cache: &mut RasterCache,
        trace: Option<&crate::frame::FrameTrace>,
    ) -> Result<WgpuOutput, PresentationFailure> {
        macro_rules! trace { ($phase:expr, $($args:tt)*) => { if let Some(trace) = trace { trace.mark($phase,format_args!($($args)*)); } }; }
        // A retained image performs none of these stages. Do not report the
        // renderer's previous frame timings as work done by this request.
        reset_raster_profile(profile);
        let frame = packet;
        let surface = self.surface;
        let (pw, ph, scale) = (packet.width, packet.height, raster_scale);
        if let Some(loss) = surface.core().device_failure() {
            cache.clear();
            return Err(self.failure(packet, NativePresentationFailure::DeviceLost(loss)));
        }
        let policy_allows_reuse = cfg!(not(target_arch = "wasm32"))
            && surface
                .try_renderer()
                .map_err(|failure| self.failure(packet, refused(failure)))?
                .vello_output_reuse_allowed();
        if !policy_allows_reuse {
            cache.clear();
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(output) = cache.lookup(surface.device(), packet, policy_allows_reuse) {
            let output = surface
                .core()
                .try_reuse_native_raster(|| output.clone())
                .map_err(|failure| self.failure(packet, refused(failure)))?;
            profile.raster_reused = true;
            trace!("raster-reused", "qualified output retained");
            return Ok(output);
        }
        // Blurred shadows lower to image ops backed by per-frame GPU masks.
        // Keeping only `translated.scene` drops those masks and leaves the
        // image keys unresolved, which makes every blurred CSS box-shadow
        // disappear in this host even though the paint list is correct.
        let phase = crate::Instant::now();
        trace!("shadows-begin", "");
        if !box_shadow_masks.is_empty() {
            // One renderer access for the masks, released before rasterizing.
            let renderer = surface
                .try_renderer()
                .map_err(|failure| self.failure(packet, refused(failure)))?;
            for mask in box_shadow_masks {
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
        trace!("shadows-end", "");
        let clear = wgpu::Color {
            r: f64::from(frame.clear[0]),
            g: f64::from(frame.clear[1]),
            b: f64::from(frame.clear[2]),
            a: f64::from(frame.clear[3]),
        };
        let phase = crate::Instant::now();
        // Keyed by this host: several windows rasterize through one core, and
        // an unkeyed raster diffs against whichever surface drew last (F17).
        trace!("raster-begin", "");
        let raster = surface.core().try_rasterize_scaled_for(
            frame.host,
            scene,
            pw,
            ph,
            ColorLoad::Clear(clear),
            scale,
        );
        let (_tex, view) = match raster {
            Ok(output) => output,
            Err(failure) => {
                trace!("raster-refused", "{failure}");
                return Err(self.failure(packet, NativePresentationFailure::Rasterize(failure)));
            },
        };
        trace!("raster-end", "");
        profile.raster_us = elapsed_us(phase.elapsed());
        // Timing is observation only; a refused access leaves it unrecorded.
        if let Ok(renderer) = surface.try_renderer() {
            if let Some(timings) = renderer.last_frame_timings() {
                let span = |name: &str| timings.span(name).map(elapsed_us).unwrap_or_default();
                profile.raster_total_us = elapsed_us(timings.total);
                profile.tile_invalidate_us = span("tile_invalidate");
                profile.dirty_tile_rebuild_us = span("dirty_tile_rebuild");
                profile.master_compose_us = span("master_compose");
                profile.vello_render_us = span("vello_render");
            }
            profile.dirty_tiles = renderer.vello_last_dirty_count().unwrap_or_default() as u64;
        }
        let output = WgpuOutput {
            _texture: _tex,
            view,
        };
        if policy_allows_reuse {
            cache.insert(surface.device().clone(), packet, output.clone());
        }
        Ok(output)
    }

    /// Rasterize one complete scene and present it through the shared surface path.
    pub fn present_scene(
        &self,
        packet: &PreparedFrame,
        scene: &netrender::Scene,
        box_shadow_masks: &[paint_list_render::BoxShadowMaskRequest],
        raster_scale: f32,
        profile: &mut FrameProfile,
        cache: &mut RasterCache,
        trace: Option<&crate::frame::FrameTrace>,
    ) -> Result<Option<WgpuOutput>, PresentationFailure> {
        let output = self.rasterize_scene(
            packet,
            scene,
            box_shadow_masks,
            raster_scale,
            profile,
            cache,
            trace,
        )?;
        if self.present_output_traced(packet, &output, profile, trace)?
            == OutputPresentation::Presented
        {
            Ok(Some(output))
        } else {
            Ok(None)
        }
    }

    /// Acquire, compose and present an already qualified output. This method
    /// performs no scene rasterization and never accepts a cache candidate.
    pub fn present_output(
        &self,
        packet: &PreparedFrame,
        output: &WgpuOutput,
        profile: &mut FrameProfile,
    ) -> Result<bool, PresentationFailure> {
        self.present_output_status(packet, output, profile)
            .map(|status| status == OutputPresentation::Presented)
    }

    pub fn present_output_status(
        &self,
        packet: &PreparedFrame,
        output: &WgpuOutput,
        profile: &mut FrameProfile,
    ) -> Result<OutputPresentation, PresentationFailure> {
        self.present_output_traced(packet, output, profile, None)
    }

    fn present_output_traced(
        &self,
        packet: &PreparedFrame,
        output: &WgpuOutput,
        profile: &mut FrameProfile,
        trace: Option<&crate::frame::FrameTrace>,
    ) -> Result<OutputPresentation, PresentationFailure> {
        macro_rules! trace { ($phase:expr, $($args:tt)*) => { if let Some(trace) = trace { trace.mark($phase,format_args!($($args)*)); } }; }
        let frame = packet;
        let surface = self.surface;
        let (pw, ph) = (packet.width, packet.height);
        let frame_started = packet.started;
        if let Some(loss) = surface.core().device_failure() {
            return Err(self.failure(packet, NativePresentationFailure::DeviceLost(loss)));
        }
        let clear = wgpu::Color {
            r: f64::from(frame.clear[0]),
            g: f64::from(frame.clear[1]),
            b: f64::from(frame.clear[2]),
            a: f64::from(frame.clear[3]),
        };
        let phase = crate::Instant::now();
        trace!("acquire-begin", "");
        let acquire_work = profile.native_work.packet.is_some().then(WorkTimer::start);
        let acquired = surface.acquire_status();
        profile.native_work.acquire = acquire_work.and_then(WorkTimer::finish);
        let frame = match acquired {
            genet_render_host::SurfaceAcquire::Frame(frame) => frame,
            genet_render_host::SurfaceAcquire::DeviceLost(loss) => {
                trace!("acquire-device-lost", "");
                // The caller drops its candidate before acknowledging
                // detachment and opening the replacement factory.
                return Err(self.failure(packet, NativePresentationFailure::DeviceLost(loss)));
            },
            genet_render_host::SurfaceAcquire::SurfaceLost => {
                trace!("acquire-surface-lost", "");
                return Err(self.failure(packet, NativePresentationFailure::SurfaceLost));
            },
            genet_render_host::SurfaceAcquire::Reconfigured => {
                trace!("acquire-reconfigured", "");
                profile.acquire_us = elapsed_us(phase.elapsed());
                profile.total_us = elapsed_us(frame_started.elapsed());
                return Ok(OutputPresentation::Reconfigured);
            },
            genet_render_host::SurfaceAcquire::Unavailable { reason } => {
                trace!("acquire-unavailable", "{reason}");
                profile.acquire_us = elapsed_us(phase.elapsed());
                profile.total_us = elapsed_us(frame_started.elapsed());
                return Ok(OutputPresentation::Unavailable);
            },
        };
        trace!("acquire-end", "");
        profile.acquire_us = elapsed_us(phase.elapsed());
        let target = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        // The composition pass blends and therefore loads its target. Clear a
        // fresh swapchain texture first so transparent app-frame margins do
        // not preserve undefined pixels from the compositor-owned image.
        let phase = crate::Instant::now();
        let clear_work = profile.native_work.packet.is_some().then(WorkTimer::start);
        let mut encoder =
            surface
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("cambium surface clear"),
                });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("cambium surface clear pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        surface.queue().submit([encoder.finish()]);
        profile.clear_us = elapsed_us(phase.elapsed());
        profile.native_work.clear = clear_work.and_then(WorkTimer::finish);
        let phase = crate::Instant::now();
        let compose_work = profile.native_work.packet.is_some().then(WorkTimer::start);
        match surface.try_renderer() {
            Ok(renderer) => renderer.compose_external_texture(
                &output.view,
                &target,
                surface.format(),
                pw,
                ph,
                ExternalTexturePlacement::new([0.0, 0.0, pw as f32, ph as f32]),
            ),
            Err(failure) => {
                trace!("compose-refused", "{failure}");
                drop(frame);
                drop(target);
                return Err(self.failure(packet, refused(failure)));
            },
        }
        trace!("compose-end", "");
        profile.compose_us = elapsed_us(phase.elapsed());
        profile.native_work.compose = compose_work.and_then(WorkTimer::finish);
        if let Some(loss) = surface.core().device_failure() {
            trace!("compose-device-lost", "");
            drop(frame);
            drop(target);
            return Err(self.failure(packet, NativePresentationFailure::DeviceLost(loss)));
        }
        // wgpu 30 moved presentation from SurfaceTexture to Queue.
        let phase = crate::Instant::now();
        trace!("present-begin", "");
        let present_work = profile.native_work.packet.is_some().then(WorkTimer::start);
        surface.queue().present(frame);
        profile.native_work.present = present_work.and_then(WorkTimer::finish);
        trace!("present-end", "");
        profile.present_us = elapsed_us(phase.elapsed());
        Ok(OutputPresentation::Presented)
    }
}

/// A refused renderer access keeps device loss distinct from other refusals.
fn refused(failure: genet_render_host::RasterizeFailure) -> NativePresentationFailure {
    match failure {
        genet_render_host::RasterizeFailure::DeviceLost(loss) => {
            NativePresentationFailure::DeviceLost(loss)
        },
        failure => NativePresentationFailure::Rasterize(failure),
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(test)]
mod tests;
