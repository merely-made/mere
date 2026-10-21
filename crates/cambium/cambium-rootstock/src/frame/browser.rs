// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Prepared actual host frames; the browser adapter drives finite qualification.
use super::*;
use std::{any::Any, sync::{Arc, Weak}};
use genet_render_host::{BrowserFrameStartError, PendingBrowserFrame, RasterizeFailure};
use crate::producer::registry::PreparedProducerWindow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserFrameProgress { Idle, Busy, Pending, Presented, Refused }

pub(crate) struct PreparedBrowserFrame {
    quality: PendingBrowserFrame,
    producers: Option<PreparedProducerWindow>,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    masks: Vec<u64>,
    core: Weak<genet_render_host::RenderCore>,
    generation: u64,
    epoch: u64,
    sheet_generation: u64,
    width: u32,
    height: u32,
    scale: f32,
    clear: wgpu::Color,
    paint_capture: Option<PaintEnvelope>,
    animating: bool,
    profile: FrameProfile,
    started: crate::Instant,
}

pub(crate) struct LostBrowserSession {
    pub(crate) core: Option<Arc<genet_render_host::RenderCore>>,
    pub(crate) surface: Option<Box<dyn crate::Surface>>,
    pub(crate) fragments: HashMap<u64,(u64,u64)>,
}
impl Drop for LostBrowserSession {
    fn drop(&mut self) {
        if let Some(core) = &self.core {
            for (id,_) in self.fragments.values() { core.retire_owned_fragment(*id); }
        }
        drop(self.surface.take());
        drop(self.core.take());
    }
}
impl Drop for PreparedBrowserFrame {
    fn drop(&mut self) {
        if let Some(core) = self.core.upgrade() {
            for key in &self.masks { core.retire_owned_image(*key); }
        }
    }
}

fn unavailable(message:&str) -> RasterizeFailure { crate::producer::staged_producer_refusal(message) }

impl<State, Logic, V, T> Host<State, Logic, V, T>
where
    T: crate::HostTree<State>, State:'static,
    Logic:FnMut(&State)->V + 'static, V:RootView<State>,
{
    /// Interior application changes outside routed dispatch must notify this
    /// same epoch, as producer callbacks already notify via revision/identity.
    pub fn invalidate_browser_frame(&mut self) {
        self.s.browser_frame_epoch = self.s.browser_frame_epoch.and_then(|epoch| epoch.checked_add(1));
    }

    pub(crate) fn cancel_browser_frame(&mut self) {
        if let Some(mut frame) = self.s.browser_frame.take() {
            if let Some(plan) = frame.producers.take() { self.s.shared.producers.abort_browser_window(plan); }
            self.s.browser_retirements.push(Box::new(frame));
        }
    }
    pub fn cancel_browser_frame_on_loss(&mut self) {
        if self.s.shared.render_core.as_ref().is_some_and(|core| core.device_failure().is_some()) {
            self.cancel_browser_frame();
        }
    }
    pub(crate) fn has_browser_retirements(&self) -> bool {
        !self.s.browser_retirements.is_empty() || self.s.shared.producers.has_browser_retirements()
    }
    pub fn has_pending_browser_frame(&self) -> bool {
        self.s.browser_frame.is_some() || self.has_browser_retirements()
    }
    pub fn has_submitted_browser_frame(&self) -> bool { self.s.browser_frame.is_some() }

    /// Coalesce a real request observed after the submitted frame's preparation
    /// boundary. Polling that old target cannot satisfy a new redraw request.
    pub fn defer_browser_redraw(&mut self) { self.s.browser_redraw_after_pending = true; }

    /// Call after outside-borrow retirement. Issue at most one follow-up request;
    /// the new preparation consumes it, so it cannot make an idle frame loop.
    pub fn flush_browser_redraw(&mut self) {
        if self.s.browser_redraw_after_pending && !self.has_pending_browser_frame()
            && !self.s.hidden && !self.s.close_requested {
            self.s.browser_redraw_after_pending = false;
            if let Some(window) = &self.s.window { window.request_redraw(); }
        }
    }

    /// Event sources must drop this return value after releasing Host RefMut.
    pub fn take_browser_retirements(&mut self) -> Vec<Box<dyn Any>> {
        let mut retired = std::mem::take(&mut self.s.browser_retirements);
        retired.extend(self.s.shared.producers.take_browser_retirements());
        retired
    }

    /// Returns without waiting. Subsequent browser turns qualify the exact
    /// pending target, then publish on one turn before canvas acquisition.
    pub fn redraw_browser_turn(&mut self, timestamp:Option<std::time::Duration>) -> BrowserFrameProgress {
        self.s.last_redraw_presentation = None;
        self.cancel_browser_frame_on_loss();
        if self.has_browser_retirements() { return BrowserFrameProgress::Pending; }
        if self.s.browser_frame.is_some() { return self.poll_browser_frame(); }
        if self.s.hidden || self.s.close_requested { return BrowserFrameProgress::Idle; }
        if !matches!(self.poll_device_recovery(),genet_render_host::RecoveryStatus::Ready {..}) {
            return BrowserFrameProgress::Pending;
        }
        let Some(core) = self.s.shared.render_core.clone() else { return BrowserFrameProgress::Idle };
        let mut submission = match core.try_begin_browser_frame() {
            Ok(submission) => submission,
            Err(BrowserFrameStartError::Busy) => {
                if let Some(window) = &self.s.window { window.request_redraw(); }
                return BrowserFrameProgress::Busy;
            },
            Err(BrowserFrameStartError::Refused(error)) => { self.refuse_rasterization(error,None); return BrowserFrameProgress::Refused },
        };
        #[cfg(feature = "browser-frame-diagnostics")]
        self.record_browser_work(crate::host::WorkEntry::Preparation);
        // A new actual preparation satisfies the coalesced follow-up intent.
        self.s.browser_redraw_after_pending = false;
        self.s.last_raster_failure = None;
        self.deliver_files();
        let started = crate::Instant::now();
        let mut profile = FrameProfile::default();
        let phase = crate::Instant::now();
        let animating = self.prepare_frame();
        profile.frame_hook_us = elapsed_us(phase.elapsed());
        let scale = self.layout_scale() as f32;
        let Some((width,height)) = self.s.window.as_ref().map(|window| {
            let size = window.inner_size(); (size.0.max(1),size.1.max(1))
        }) else { return BrowserFrameProgress::Idle };
        let (lw,lh) = (width as f32 / scale,height as f32 / scale);
        let limits = paint_list_render::bundle::BundleLimits::default();
        let pixels = (width as usize).checked_mul(height as usize);
        if !scale.is_finite() || scale <= 0.0 || width > limits.max_dimension || height > limits.max_dimension
            || width > core.device().limits().max_texture_dimension_2d || height > core.device().limits().max_texture_dimension_2d
            || pixels.is_none_or(|pixels| pixels > limits.max_pixels) {
            self.refuse_rasterization(unavailable("root target exceeds resource admission limits"),None); return BrowserFrameProgress::Refused;
        }
        if self.s.surface.is_none() { return BrowserFrameProgress::Idle; }
        let phase = crate::Instant::now();
        let anim_active = self.relayout(lw,lh);
        profile.relayout_us = elapsed_us(phase.elapsed());
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
        let Some(epoch) = self.s.browser_frame_epoch else {
            self.refuse_rasterization(unavailable("host frame epoch exhausted"),None); return BrowserFrameProgress::Refused;
        };
        self.s.shared.producers.bind_core(&core);
        self.s.shared.producers.clear_render_failure();
        self.s.shared.producers.set_frame_timestamp(timestamp);
        let clear = if self.options.app_frame_is_transparent() { wgpu::Color::TRANSPARENT } else { wgpu::Color::BLACK };
        let mut plan = None;
        let mut masks = Vec::new();
        let prepared = submission.submit(|commands| {
            let producers = {
                let runner = self.s.runner.as_ref().ok_or_else(|| unavailable("host runner unavailable"))?;
                let layout = self.s.layout.as_ref().ok_or_else(|| unavailable("host layout unavailable"))?;
                let dom = runner.dom(); let dom = dom.borrow();
                let view = crate::WindowDom::new(&dom,runner.mount());
                let shared = &mut self.s.shared; let held = &shared.held_elsewhere;
                shared.producers.prepare_browser_window(self.s.surface.as_deref().expect("prepared surface"),layout,&view,scale,
                    &|key,owner| held.contains(&key) || owner.is_some_and(|node| view.elsewhere(node)),commands)?
            };
            profile.producers = producers.stats;
            plan = Some(producers);
            let phase = crate::Instant::now(); self.sync_leaf_fragments_scoped(commands);
            profile.leaf_fragments_us = elapsed_us(phase.elapsed());
            let phase = crate::Instant::now(); self.sync_ime_area(); profile.ime_us = elapsed_us(phase.elapsed());
            let phase = crate::Instant::now();
            let list = self.emit_paint_list_with_plan(lw,lh,None,plan.as_ref()).ok_or_else(|| unavailable("paint envelope unavailable"))?;
            let paint_capture = self.s.pending_paint_capture.as_ref().map(|_| PaintEnvelope::from_list(&list));
            let mut translated = paint_list_render::translate_paint_cmd_stream(list.viewport(),list.commands(),list.fonts(),list.images());
            plan.as_ref().expect("prepared producers").remap(&mut translated)
                .map_err(|_| unavailable("complete producer references could not be bound"))?;
            profile.emit_scene_us = elapsed_us(phase.elapsed());
            // Plan every mask alias before building any production GPU mask.
            let mut aliases = HashMap::new();
            if translated.box_shadow_masks.len() > limits.max_resources {
                return Err(unavailable("shadow ledger exceeds resource admission limits"));
            }
            let mut payload_bytes = plan.as_ref().expect("prepared producers").payload_bytes
                .checked_add(pixels.expect("admitted pixels") * 4).ok_or_else(|| unavailable("frame payload budget overflow"))?;
            if payload_bytes > limits.max_bytes {
                return Err(unavailable("frame payload exceeds resource admission limits"));
            }
            for mask in &translated.box_shadow_masks {
                if mask.dim == 0 || mask.dim > commands.device().limits().max_texture_dimension_2d
                    || !mask.bounds.iter().all(|value| value.is_finite())
                    || !mask.corner_radius.is_finite() || mask.corner_radius < 0.0
                    || !mask.blur_radius_px.is_finite() || mask.blur_radius_px < 0.0 {
                    return Err(unavailable("invalid production shadow extent"));
                }
                let specification = (mask.dim,mask.bounds,mask.corner_radius,mask.blur_radius_px,mask.invert);
                if let Some((_,previous)) = aliases.get(&mask.key) {
                    if previous != &specification { return Err(unavailable("duplicate inconsistent shadow request")); }
                    continue;
                }
                let private = crate::producer::registry::allocate_frame_image_key().map_err(|_| unavailable("private shadow identity exhausted"))?;
                let mask_pixels = (mask.dim as usize).checked_mul(mask.dim as usize).ok_or_else(|| unavailable("shadow pixel budget overflow"))?;
                payload_bytes = mask_pixels.checked_mul(4).and_then(|bytes| payload_bytes.checked_add(bytes)).ok_or_else(|| unavailable("shadow payload budget overflow"))?;
                if mask_pixels > limits.max_pixels || payload_bytes > limits.max_bytes { return Err(unavailable("frame payload exceeds resource admission limits")); }
                aliases.insert(mask.key,(private,specification)); masks.push(private);
            }
            if aliases.len().checked_add(plan.as_ref().expect("prepared producers").source_count())
                .is_none_or(|count| count > limits.max_resources) {
                return Err(unavailable("complete frame resource ledger exceeds admission limits"));
            }
            for op in &mut translated.scene.ops {
                if let netrender::SceneOp::Image(image) = op {
                    if let Some((key,_)) = aliases.get(&image.key) { image.key = *key; }
                }
            }
            for mask in &mut translated.box_shadow_masks {
                mask.key = aliases[&mask.key].0;
            }
            let phase = crate::Instant::now();
            for (key,spec) in aliases.values() {
                commands.renderer().build_box_shadow_mask(*key,spec.0,spec.1,spec.2,spec.3,spec.4);
            }
            profile.shadows_us = elapsed_us(phase.elapsed());
            let phase = crate::Instant::now();
            let (texture,view) = commands.rasterize_scaled_for(self.s.presentation_host,&translated.scene,width,height,ColorLoad::Clear(clear),scale)?;
            profile.raster_us = elapsed_us(phase.elapsed());
            if let Some(timings) = commands.renderer().last_frame_timings() {
                let span = |name:&str| timings.span(name).map(elapsed_us).unwrap_or_default();
                profile.raster_total_us = elapsed_us(timings.total);
                profile.tile_invalidate_us = span("tile_invalidate"); profile.dirty_tile_rebuild_us = span("dirty_tile_rebuild");
                profile.master_compose_us = span("master_compose"); profile.vello_render_us = span("vello_render");
            }
            profile.dirty_tiles = commands.renderer().vello_last_dirty_count().unwrap_or_default() as u64;
            Ok((texture,view,paint_capture))
        });
        self.s.shared.producers.set_frame_timestamp(None);
        let (texture,view,paint_capture) = match prepared {
            Ok(output) => output,
            Err(error) => {
                if let Some(plan) = plan.take() { self.s.shared.producers.abort_browser_window(plan); }
                for key in masks { core.retire_owned_image(key); }
                self.refuse_rasterization(error,None); return BrowserFrameProgress::Refused;
            },
        };
        let quality = match submission.seal() {
            Ok(quality) => quality,
            Err(error) => {
                if let Some(plan) = plan.take() { self.s.shared.producers.abort_browser_window(plan); }
                for key in masks { core.retire_owned_image(key); }
                self.s.browser_retirements.push(Box::new((texture,view)));
                self.refuse_rasterization(error,None); return BrowserFrameProgress::Refused;
            },
        };
        self.s.browser_frame = Some(PreparedBrowserFrame {
            quality, producers:plan, texture, view, masks, core:Arc::downgrade(&core), generation:core.generation(),
            epoch,sheet_generation:self.s.shared.sheet_generation,width,height,scale,clear,paint_capture,
            animating:animating || anim_active,profile,started,
        });
        BrowserFrameProgress::Pending
    }

    fn poll_browser_frame(&mut self) -> BrowserFrameProgress {
        if let Some(runner) = self.s.runner.as_mut() {
            let before = self.s.browser_pending_mutations.len();
            runner.drain_mutations(&mut self.s.browser_pending_mutations);
            if self.s.browser_pending_mutations.len() != before { self.invalidate_browser_frame(); }
        }
        let stale = self.s.browser_frame.as_ref().is_some_and(|frame| {
            self.s.hidden || self.s.close_requested || self.s.browser_frame_epoch != Some(frame.epoch)
                // A newly armed one-shot needs an envelope from its prepared
                // frame. Keep the callback armed and resubmit after retiring
                // this private target; do not clone every unarmed envelope.
                || (self.s.pending_paint_capture.is_some() && frame.paint_capture.is_none())
                || self.s.shared.sheet_generation != frame.sheet_generation || self.layout_scale() as f32 != frame.scale
                || self.s.window.as_ref().is_none_or(|window| window.inner_size() != (frame.width,frame.height))
                || self.s.shared.render_core.as_ref().is_none_or(|core| core.generation() != frame.generation)
        });
        if stale {
            self.cancel_browser_frame();
            if let Some(window) = &self.s.window { window.request_redraw(); }
            return BrowserFrameProgress::Pending;
        }
        let result = self.s.browser_frame.as_mut().expect("pending frame").quality.poll();
        let Some(result) = result else { return BrowserFrameProgress::Pending };
        let accepted = match result {
            Ok(accepted) => accepted,
            Err(error) => { self.cancel_browser_frame(); self.refuse_rasterization(error,None); return BrowserFrameProgress::Refused },
        };
        let mut prepared = self.s.browser_frame.take().expect("accepted prepared frame");
        let plan = prepared.producers.take().expect("complete producer plan");
        if let Err(error) = self.s.shared.producers.commit_browser_window(plan,&accepted) {
            self.s.browser_retirements.push(Box::new(prepared));
            self.refuse_rasterization(error,None); return BrowserFrameProgress::Refused;
        }
        let progress = self.present_browser_frame(&mut prepared,accepted);
        self.s.browser_retirements.push(Box::new(prepared));
        progress
    }

    fn present_browser_frame(&mut self, prepared:&mut PreparedBrowserFrame, accepted:genet_render_host::AcceptedBrowserFrame) -> BrowserFrameProgress {
        if let Err(error) = accepted.check_current() { self.refuse_rasterization(error,None); return BrowserFrameProgress::Refused; }
        let Some(surface) = self.s.surface.as_ref() else { return BrowserFrameProgress::Idle };
        let phase = crate::Instant::now();
        let frame = match surface.acquire_status() {
            genet_render_host::SurfaceAcquire::Frame(frame) => frame,
            genet_render_host::SurfaceAcquire::DeviceLost(loss) => { self.refuse_rasterization(RasterizeFailure::DeviceLost(loss),None); return BrowserFrameProgress::Refused },
            genet_render_host::SurfaceAcquire::SurfaceLost => {
                self.s.surface = None; self.s.needs_recovery_surface = true;
                if let Some(window) = &self.s.window { window.request_redraw(); }
                return BrowserFrameProgress::Pending;
            },
            genet_render_host::SurfaceAcquire::Unavailable {..} => return BrowserFrameProgress::Idle,
        };
        prepared.profile.acquire_us = elapsed_us(phase.elapsed());
        let target = frame.texture.create_view(&Default::default());
        let phase = crate::Instant::now();
        let mut encoder = surface.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label:Some("cambium accepted surface clear") });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label:Some("cambium accepted surface clear pass"),
                color_attachments:&[Some(wgpu::RenderPassColorAttachment { view:&target,depth_slice:None,resolve_target:None,
                    ops:wgpu::Operations { load:wgpu::LoadOp::Clear(prepared.clear),store:wgpu::StoreOp::Store } })],
                depth_stencil_attachment:None,timestamp_writes:None,occlusion_query_set:None,multiview_mask:None,
            });
        }
        surface.queue().submit([encoder.finish()]); prepared.profile.clear_us = elapsed_us(phase.elapsed());
        let phase = crate::Instant::now();
        if let Err(error) = accepted.compose_external_texture(&prepared.view,&target,surface.format(),prepared.width,prepared.height,
            ExternalTexturePlacement::new([0.0,0.0,prepared.width as f32,prepared.height as f32])) {
            drop(frame); self.refuse_rasterization(error,None); return BrowserFrameProgress::Refused;
        }
        prepared.profile.compose_us = elapsed_us(phase.elapsed());
        let Some(sequence) = self.s.presentation_sequence.checked_add(1) else {
            self.refuse_rasterization(unavailable("presentation sequence exhausted"),None); return BrowserFrameProgress::Refused;
        };
        let phase = crate::Instant::now(); surface.queue().present(frame); prepared.profile.present_us = elapsed_us(phase.elapsed());
        // The owning group is complete and surface work has been submitted.
        // Callbacks may use the legacy renderer accessor for capture blits;
        // release ownership before invoking those user callbacks.
        drop(accepted);
        self.s.presentation_sequence = sequence;
        let presented = crate::PresentedFrame { host:self.s.presentation_host,sequence,width:prepared.width,height:prepared.height,layout_scale:prepared.scale };
        self.s.last_redraw_presentation = Some(presented);
        self.observe_presentation(presented);
        let surface = self.s.surface.as_ref().expect("presented surface");
        let phase = crate::Instant::now();
        if let Some(envelope) = prepared.paint_capture.take() && let Some(capture) = self.s.pending_paint_capture.take() { capture(Ok(envelope)); }
        if let Some(capture) = self.s.pending_capture.take() { capture(Ok(crate::CaptureFrame { surface:&**surface,view:&prepared.view,width:prepared.width,height:prepared.height })); }
        if let Some(capture) = self.s.pending_stamped_capture.take() { capture(Ok(crate::StampedCaptureFrame { surface:&**surface,view:&prepared.view,presentation:presented })); }
        prepared.profile.capture_us = elapsed_us(phase.elapsed());
        if prepared.animating && let Some(window) = &self.s.window { window.request_redraw(); }
        let phase = crate::Instant::now(); self.drain_pointer(); prepared.profile.pointer_us = elapsed_us(phase.elapsed());
        prepared.profile.total_us = elapsed_us(prepared.started.elapsed()); self.s.last_frame_profile = Some(prepared.profile);
        BrowserFrameProgress::Presented
    }
}
