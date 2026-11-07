// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The frame pipeline: retained layout, paint emission, presentation, and
//! accessibility synchronization. Extracted from the woodshed-genet donor.
//!
//! A frame is deliberately in two halves. [`Host::relayout`] brings the
//! retained layout up to date and needs no GPU and no window; everything after
//! it rasterizes and presents. The split is what lets [`Harness`](crate::Harness)
//! run an application's real layout, hit testing, and input routing in an
//! ordinary `cargo test`.

use std::collections::HashMap;

use crate::A11yAction;
use cambium::PointerClick;
use genet_render::VisualCaret;
use genet_scripted_dom::NodeId;
use layout_dom_api::{DomMutation, LayoutDom as _};
use paint_list_api::{DeviceIntSize, PaintEnvelope};

use crate::input::to_visual_caret;
use crate::meristem_bounds::RootView;
use crate::{AppCtx, Host, ScrollRequest};

fn elapsed_us(elapsed: crate::Duration) -> u64 {
    elapsed.as_micros().min(u64::MAX as u128) as u64
}

#[derive(Debug)]
pub(crate) enum LeafFragmentAdmissionError {
    RendererAccess(genet_render_host::RasterizeFailure),
    Translation {
        key: u64,
        epoch: u64,
        error: paint_list_render::TranslationError,
    },
    Retained {
        key: u64,
        epoch: u64,
        error: netrender::RetainedFragmentError,
    },
}

impl std::fmt::Display for LeafFragmentAdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RendererAccess(error) => {
                write!(f, "retained fragment renderer access refused: {error}")
            },
            Self::Translation { key, epoch, error } => {
                write!(
                    f,
                    "leaf {key} epoch {epoch} paint translation refused: {error}"
                )
            },
            Self::Retained { key, epoch, error } => {
                write!(
                    f,
                    "leaf {key} epoch {epoch} retained fragment refused: {error}"
                )
            },
        }
    }
}

#[cfg(test)]
#[path = "frame/gpu_binding_controls.rs"]
mod gpu_binding_controls;

/// Opt-in startup attribution, including attempts that never present. Stores
/// only CPU timing and host identity; it does not retain render resources.
pub(crate) struct FrameTrace {
    started: crate::Instant,
    host: u64,
    last_presented: u64,
}

impl FrameTrace {
    fn enabled() -> bool {
        static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ENABLED.get_or_init(|| {
            std::env::var_os("CAMBIUM_HOST_FRAME_TRACE").is_some_and(|value| value != "0")
        })
    }

    fn new(host: u64, last_presented: u64) -> Option<Self> {
        Self::enabled().then(|| Self {
            started: crate::Instant::now(),
            host,
            last_presented,
        })
    }

    pub(crate) fn mark(&self, phase: &str, detail: std::fmt::Arguments<'_>) {
        eprintln!(
            "[cambium-frame] host={} last-presented={} elapsed={}us phase={phase} {detail}",
            self.host,
            self.last_presented,
            elapsed_us(self.started.elapsed()),
        );
    }
}

impl Drop for FrameTrace {
    fn drop(&mut self) {
        self.mark(
            "return",
            format_args!("unwinding={}", std::thread::panicking()),
        );
    }
}

struct SpriggingSource<'a> {
    rendered: &'a sprigging::RenderedLeaves,
    producers: &'a crate::ProducerRegistry,
    portable: Option<&'a crate::producer::registry::PortablePlan>,
}

impl SpriggingSource<'_> {
    fn leaf_commands(&self, key: u64) -> Option<Vec<paint_list_api::PaintCmd>> {
        if let Some(plan) = self.portable {
            return plan
                .commands(key)
                .or_else(|| self.producers.contains(key).then(Vec::new))
                .or_else(|| self.rendered.get(key).map(<[_]>::to_vec));
        }
        self.producers
            .commands(key)
            .or_else(|| self.rendered.get(key).map(<[_]>::to_vec))
    }
}

/// The focused text field's paint inputs for this frame: which node, where the
/// caret is, and the selection byte range when one should be drawn.
type FocusedOverlay = (NodeId, VisualCaret, Option<(usize, usize)>);

impl<State, Logic, V, T> Host<State, Logic, V, T>
where
    T: crate::HostTree<State>,
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    /// Run the application's per-frame hook. Returns `true` when it wants more
    /// frames (an animation is live).
    pub fn prepare_frame(&mut self) -> bool {
        let animating = {
            let logical_size = self.logical_size();
            let (ui_zoom, zoom_changed) = self.take_zoom_edge();
            let geometry = self.s.geometry;
            let frame_profile = self.s.last_frame_profile;
            let commands = self.s.commands.clone();
            let window = self.s.window.as_deref();
            let layout = self.s.layout.as_ref();
            let Some(runner) = self.s.runner.as_mut() else {
                return false;
            };
            let mut ctx = AppCtx {
                runner,
                layout,
                window,
                logical_size,
                ui_zoom,
                zoom_changed,
                leaves: &mut self.s.shared.leaves,
                files: &mut self.s.files,
                producers: &mut self.s.shared.producers,
                set_sheet: &mut self.s.pending_sheet,
                set_ui_zoom: &mut self.s.pending_ui_zoom,
                close: &mut self.s.close_requested,
                wake: &self.wake,
                capture: &mut self.s.pending_capture,
                presentation_observer: &mut self.s.pending_presentation_observer,
                capture_stamped: &mut self.s.pending_stamped_capture,
                capture_paint: &mut self.s.pending_paint_capture,
                pointer: &mut self.s.pending_pointer,
                a11y: &mut self.s.pending_a11y,
                a11y_owner: &self.s.a11y_owner,
                a11y_publication: crate::A11yPublicationSnapshot {
                    host: self.s.presentation_host,
                    sequence: self.s.a11y_publication_sequence,
                    publication: self.s.a11y_publication,
                },
                scroll: &mut self.s.pending_scroll,
                window_commands: &commands,
                render_core: self.s.shared.render_core.as_ref(),
                geometry,
                frame_profile,
                presentation: self.s.last_redraw_presentation,
            };
            (self.hooks.frame)(&mut ctx)
        };
        if let Some(sheet) = self.s.pending_sheet.take() {
            self.swap_sheet(sheet);
        }
        if let Some(zoom) = self.s.pending_ui_zoom.take() {
            self.set_ui_zoom(zoom);
        }
        animating
    }

    /// Emit this frame's paint list with selection and caret inside the text
    /// field's CSS paint context, then any active overlay scrollbars, and
    /// lower it to a netrender scene plus the GPU resources that scene names.
    pub(crate) fn emit_paint_list(
        &mut self,
        lw: f32,
        lh: f32,
        portable: Option<&crate::producer::registry::PortablePlan>,
    ) -> Option<genet_livery::LiveryPaintList> {
        let focused_overlay = self.focused_overlay();
        let runner = self.s.runner.as_ref()?;
        let layout = self.s.layout.as_mut()?;
        let dom = runner.dom();
        let dom_ref = dom.borrow();
        let view = crate::WindowDom::new(&dom_ref, runner.mount());
        let source = SpriggingSource {
            rendered: &self.s.shared.rendered,
            producers: &self.s.shared.producers,
            portable,
        };
        let mut list = layout.emit_paint_list_with_leaves(
            &view,
            DeviceIntSize::new(lw as i32, lh as i32),
            focused_overlay,
            |key| source.leaf_commands(key),
            // A retained fragment has no CSS clip or layer identity in the
            // renderer yet. Keep custom leaves in the recorded Livery slot as
            // ordinary commands until that composition boundary is real.
            // This preserves their stacking relation to DOM overlays.
            |_| None,
        );
        // Overlay scrollbar thumbs mid-hold/mid-fade: the engine draws the
        // geometry, the shared fade clock supplies alpha.
        let now = crate::Instant::now();
        let fade = &self.s.scrollbar_fade;
        layout.append_scrollbars(&view, &mut list, &|t| fade.alpha(t, now));
        Some(list)
    }

    /// Capture the current retained layout and actual producer callbacks without
    /// acquiring a device. Call frame preparation and relayout before this, as
    /// the host does for a presented frame. No cache/lifecycle commit precedes
    /// complete envelope admission.
    pub fn capture_portable(
        &mut self,
    ) -> Result<crate::PortablePaintCapture, crate::PortableCaptureError> {
        self.capture_portable_for(crate::ProducerFramePurpose::Export)
    }

    pub(crate) fn capture_portable_for(
        &mut self,
        purpose: crate::ProducerFramePurpose,
    ) -> Result<crate::PortablePaintCapture, crate::PortableCaptureError> {
        use crate::PortableCaptureError;
        let scale = self.layout_scale() as f32;
        let plan = {
            let runner = self
                .s
                .runner
                .as_ref()
                .ok_or(PortableCaptureError::MissingLayout)?;
            let layout = self
                .s
                .layout
                .as_ref()
                .ok_or(PortableCaptureError::MissingLayout)?;
            let dom = runner.dom();
            let dom = dom.borrow();
            let view = crate::WindowDom::new(&dom, runner.mount());
            let shared = &mut self.s.shared;
            let held = &shared.held_elsewhere;
            shared.producers.plan_portable_for(
                layout,
                &view,
                scale,
                &|key, owner| held.contains(&key) || owner.is_some_and(|node| view.elsewhere(node)),
                purpose,
            )?
        };
        let (width, height) = self.s.layout_size;
        let list = self
            .emit_paint_list(width, height, Some(&plan))
            .ok_or(PortableCaptureError::MissingLayout)?;
        self.s
            .shared
            .producers
            .admit_portable(plan, PaintEnvelope::from_list(&list))
    }

    /// Translate a native frame once and bind its producer draws to this
    /// registry's physical renderer identities. A refused binding refuses the
    /// frame and its captures; it never presents unbound producer content.
    pub(crate) fn emit_native_scene(
        &mut self,
        lw: f32,
        lh: f32,
    ) -> Result<
        Option<(PaintEnvelope, paint_list_render::TranslatedDisplayList)>,
        paint_list_render::TranslationError,
    > {
        let Some(list) = self.emit_paint_list(lw, lh, None) else {
            return Ok(None);
        };
        let envelope = PaintEnvelope::from_list(&list);
        let mut translated =
            paint_list_render::translate_envelope_with_external_textures(&envelope)?;
        if let Err(error) = self
            .s
            .shared
            .producers
            .remap_gpu_display_list(&mut translated)
        {
            self.refuse_producer_resource(error);
            return Ok(None);
        }
        Ok(Some((envelope, translated)))
    }

    pub(crate) fn refuse_producer_resource(&mut self, error: crate::ProducerGpuRemapError) {
        if FrameTrace::enabled() {
            eprintln!(
                "[cambium-frame] host={} last-presented={} phase=producer-resource-refused {error}",
                self.s.presentation_host, self.s.presentation_sequence,
            );
        }
        self.s.shared.producers.record_gpu_remap_failure(error);
        let refusal = crate::CaptureRefusal::ProducerResource {
            generation: self
                .s
                .shared
                .render_core
                .as_ref()
                .map_or(0, |core| core.generation()),
            error,
        };
        self.s.last_redraw_presentation = None;
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

    pub fn redraw(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if self.redraw_native() {
            return;
        }
        let trace = FrameTrace::new(self.s.presentation_host, self.s.presentation_sequence);
        macro_rules! trace {
            ($phase:expr, $($args:tt)*) => {
                if let Some(trace) = &trace {
                    trace.mark($phase, format_args!($($args)*));
                }
            };
        }
        trace!("begin", "");
        let prepared = match self.prepare_presentation_frame() {
            Ok(Some(frame)) => frame,
            Ok(None) => return,
            Err(failure) => {
                self.refuse_presentation(failure);
                return;
            },
        };
        if self.s.presentation_driver.is_some() {
            self.present_with_driver(prepared);
            return;
        }
        let frame_started = prepared.started;
        let mut profile = prepared.profile;
        let Some(surface) = self.s.surface.as_deref() else {
            return;
        };
        let output = crate::presentation::wgpu::WgpuPresentation::new(surface).present(
            &prepared,
            &mut profile,
            &mut self.s.raster_cache,
            trace.as_ref(),
        );
        let output = match output {
            Ok(Some(output)) => output,
            Ok(None) => {
                profile.total_us = elapsed_us(frame_started.elapsed());
                self.s.last_frame_profile = Some(profile);
                return;
            },
            Err(failure) => {
                self.refuse_presentation(failure);
                return;
            },
        };
        let crate::presentation::wgpu::WgpuOutput { _texture, view } = output;
        let paint_capture = match &prepared.paint {
            crate::PreparedPaint::Native(envelope) if self.s.pending_paint_capture.is_some() => {
                Some(envelope.clone())
            },
            _ => None,
        };
        let presented = prepared.presentation_stamp();
        self.s.native_receiving = false;
        self.s.displayed_input = None;
        self.s.displayed_presentation = Some(presented);
        self.s.presentation_sequence = presented.sequence;
        let sequence = presented.sequence;
        self.s.last_redraw_presentation = Some(presented);
        trace!("presentation-recorded", "sequence={sequence}");
        // Freeze product state before any capture callback, queued pointer
        // dispatch or platform accessibility action can change the product.
        let _ = self.s.capture_owner.resume(
            crate::PresentationBackend::Wgpu,
            presented.context_generation,
        );
        self.observe_presentation(presented);
        let surface = self.s.surface.as_ref().expect("presented surface exists");
        // A capture armed by the application: run it while the rasterized
        // view is still alive.
        let phase = crate::Instant::now();
        if let Some(envelope) = paint_capture
            && let Some(capture) = self.s.pending_paint_capture.take()
        {
            capture(Ok(envelope));
        }
        if let Some(capture) = self.s.pending_capture.take() {
            capture(self.s.capture_owner.begin(
                crate::PresentationBackend::Wgpu,
                presented,
                || crate::capture::start_presentation_readback(&**surface, &view, presented),
            ));
        }
        if let Some(capture) = self.s.pending_stamped_capture.take() {
            capture(self.s.capture_owner.begin(
                crate::PresentationBackend::Wgpu,
                presented,
                || crate::capture::start_presentation_readback(&**surface, &view, presented),
            ));
        }
        profile.capture_us = elapsed_us(phase.elapsed());
        if prepared.animating
            && let Some(window) = self.s.window.as_ref()
        {
            window.request_redraw();
        }
        // A `frame` hook runs before this frame's layout, so anything it queued
        // is delivered here instead — hit-testing against the layout that was
        // just built rather than the previous one.
        let phase = crate::Instant::now();
        self.drain_pointer();
        profile.pointer_us = elapsed_us(phase.elapsed());
        profile.total_us = elapsed_us(frame_started.elapsed());
        self.s.last_frame_profile = Some(profile);
    }

    /// Drive `:hover` / `:focus` restyles on target change. The retained layout
    /// skips the cascade when no rule can depend on that state, and skips
    /// text layout when the cascade leaves the computed styles unchanged.
    pub fn hover(&mut self) {
        let hovered_node = self.hit_at_cursor();
        let (Some(runner), Some(layout)) = (self.s.runner.as_ref(), self.s.layout.as_mut()) else {
            return;
        };
        let dom = runner.dom();
        let dom_ref = dom.borrow();
        let view = crate::WindowDom::new(&dom_ref, runner.mount());
        let focused_node = runner.focus();
        let hovered = hovered_node.map(|n| layout_dom_api::LayoutDom::opaque_id(&view, n));
        let focused = focused_node.map(|n| layout_dom_api::LayoutDom::opaque_id(&view, n));
        if (hovered, focused) == (self.s.last_hover, self.s.last_focus) {
            return;
        }
        self.s.last_hover = hovered;
        self.s.last_focus = focused;
        // Bring the transition clock to now before the flip, so a
        // hover/focus transition runs from now rather than a stale
        // idle-frozen clock.
        let now_s = self.s.anim_base.elapsed().as_secs_f64();
        let _ = layout.tick_animations(&view, now_s);
        // A dispatch since the last frame may have changed the DOM, and the
        // retained style learns of that only when relayout drains the
        // mutations. Cascading now would lay out nodes it never styled, so
        // the state waits for that relayout, which applies both.
        let settled = dom_ref.pending_mutations().1.is_empty();
        let restyled = if settled {
            layout.set_interaction(&view, hovered_node, focused_node)
        } else {
            layout.note_interaction(hovered_node, focused_node)
        };
        if restyled {
            drop(dom_ref);
            if let Some(window) = self.s.window.as_ref() {
                window.request_redraw();
            }
        }
    }

    /// Route Cambium `on_hover` Enter/Leave as the hit node changes. The host
    /// owns transition detection. Registered hover handlers also receive local
    /// coordinates on Move; motion over unregistered content does no dispatch.
    pub fn hover_dispatch(&mut self) {
        use cambium::{HoverEvent, HoverPhase};
        let target = self
            .hit_at_cursor()
            .and_then(|hit| self.s.runner.as_ref()?.hover_target(hit));
        if target == self.s.last_hover_hit {
            if let Some(target) = target {
                let Some((local, size)) = self.input_local(target) else {
                    self.s.last_hover_hit = None;
                    return;
                };
                if let Some(runner) = self.s.runner.as_mut() {
                    runner.dispatch_hover(target, HoverEvent::new(HoverPhase::Move, local, size));
                }
                self.after_dispatch();
            }
            return;
        }
        let old = self.s.last_hover_hit.take();
        self.s.last_hover_hit = target;
        for (node, phase) in [(old, HoverPhase::Leave), (target, HoverPhase::Enter)] {
            if let Some(node) = node {
                // A view replacement can retire the cached Leave target; its
                // handler can also retire the prospective Enter target. Neither
                // has geometry or an event route in the current document.
                let live = self.s.runner.as_ref().is_some_and(|runner| {
                    let dom = runner.dom();
                    let dom = dom.borrow();
                    if self.s.native_receiving {
                        crate::WindowDom::new(&dom, runner.mount()).contains(node)
                    } else {
                        dom.is_live(node)
                    }
                });
                if !live {
                    if self.s.last_hover_hit == Some(node) {
                        self.s.last_hover_hit = None;
                    }
                    continue;
                }
                let Some((local, size)) = self.input_local(node) else {
                    if self.s.last_hover_hit == Some(node) {
                        self.s.last_hover_hit = None;
                    }
                    continue;
                };
                if let Some(runner) = self.s.runner.as_mut() {
                    runner.dispatch_hover(node, HoverEvent::new(phase, local, size));
                }
            }
        }
        self.after_dispatch();
    }
}

#[path = "frame/layout.rs"]
mod layout;

#[path = "frame/accessibility.rs"]
mod accessibility;
