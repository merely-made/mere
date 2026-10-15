// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The existing same-device raster/acquire/submit path, behind frame preparation.
use super::{
    NativePresentationFailure, PreparedFrame, PreparedPaint, PresentationBackend,
    PresentationFailure,
};
use crate::{FrameProfile, Surface};
use netrender::{ColorLoad, ExternalTexturePlacement};
fn elapsed_us(elapsed: crate::Duration) -> u64 {
    elapsed.as_micros().min(u64::MAX as u128) as u64
}
pub(crate) struct WgpuOutput {
    pub _texture: wgpu::Texture,
    pub view: wgpu::TextureView,
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
            trace,
        )
    }

    /// Rasterize one complete scene at `raster_scale` and present it. Native
    /// frames pass the layout scale; a materialized portable scene is already
    /// in physical pixels and passes 1.
    pub fn present_scene(
        &self,
        packet: &PreparedFrame,
        scene: &netrender::Scene,
        box_shadow_masks: &[paint_list_render::BoxShadowMaskRequest],
        raster_scale: f32,
        profile: &mut FrameProfile,
        trace: Option<&crate::frame::FrameTrace>,
    ) -> Result<Option<WgpuOutput>, PresentationFailure> {
        macro_rules! trace { ($phase:expr, $($args:tt)*) => { if let Some(trace) = trace { trace.mark($phase,format_args!($($args)*)); } }; }
        let frame = packet;
        let surface = self.surface;
        let (pw, ph, scale) = (packet.width, packet.height, raster_scale);
        let frame_started = packet.started;
        // Blurred shadows lower to image ops backed by per-frame GPU masks.
        // Keeping only `translated.scene` drops those masks and leaves the
        // image keys unresolved, which makes every blurred CSS box-shadow
        // disappear in this host even though the paint list is correct.
        let phase = crate::Instant::now();
        trace!("shadows-begin", "");
        for mask in box_shadow_masks {
            surface.renderer().build_box_shadow_mask(
                mask.key,
                mask.dim,
                mask.bounds,
                mask.corner_radius,
                mask.blur_radius_px,
                mask.invert,
            );
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
        if let Some(timings) = surface.renderer().last_frame_timings() {
            let span = |name: &str| timings.span(name).map(elapsed_us).unwrap_or_default();
            profile.raster_total_us = elapsed_us(timings.total);
            profile.tile_invalidate_us = span("tile_invalidate");
            profile.dirty_tile_rebuild_us = span("dirty_tile_rebuild");
            profile.master_compose_us = span("master_compose");
            profile.vello_render_us = span("vello_render");
        }
        profile.dirty_tiles = surface
            .renderer()
            .vello_last_dirty_count()
            .unwrap_or_default() as u64;
        let phase = crate::Instant::now();
        trace!("acquire-begin", "");
        let frame = match surface.acquire_status() {
            genet_render_host::SurfaceAcquire::Frame(frame) => frame,
            genet_render_host::SurfaceAcquire::DeviceLost(loss) => {
                trace!("acquire-device-lost", "");
                // Release this frame's old-device resources before the host
                // acknowledges detachment and can open the replacement factory.
                drop(view);
                drop(_tex);
                return Err(self.failure(packet, NativePresentationFailure::DeviceLost(loss)));
            },
            genet_render_host::SurfaceAcquire::SurfaceLost => {
                trace!("acquire-surface-lost", "");
                return Err(self.failure(packet, NativePresentationFailure::SurfaceLost));
            },
            genet_render_host::SurfaceAcquire::Unavailable { reason } => {
                trace!("acquire-unavailable", "{reason}");
                profile.acquire_us = elapsed_us(phase.elapsed());
                profile.total_us = elapsed_us(frame_started.elapsed());
                return Ok(None);
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
        let phase = crate::Instant::now();
        surface.renderer().compose_external_texture(
            &view,
            &target,
            surface.format(),
            pw,
            ph,
            ExternalTexturePlacement::new([0.0, 0.0, pw as f32, ph as f32]),
        );
        trace!("compose-end", "");
        profile.compose_us = elapsed_us(phase.elapsed());
        if let Some(loss) = surface.core().device_failure() {
            trace!("compose-device-lost", "");
            drop(frame);
            drop(target);
            drop(view);
            drop(_tex);
            return Err(self.failure(packet, NativePresentationFailure::DeviceLost(loss)));
        }
        // wgpu 30 moved presentation from SurfaceTexture to Queue.
        let phase = crate::Instant::now();
        trace!("present-begin", "");
        surface.queue().present(frame);
        trace!("present-end", "");
        profile.present_us = elapsed_us(phase.elapsed());
        Ok(Some(WgpuOutput {
            _texture: _tex,
            view,
        }))
    }
}
