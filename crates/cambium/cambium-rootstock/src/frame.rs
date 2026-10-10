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
use netrender::{ColorLoad, ExternalTexturePlacement};
use paint_list_api::{DeviceIntSize, PaintEnvelope, PaintList as _};

use crate::input::to_visual_caret;
use crate::meristem_bounds::RootView;
use crate::{AppCtx, FrameProfile, Host};

fn elapsed_us(elapsed: crate::Duration) -> u64 {
    elapsed.as_micros().min(u64::MAX as u128) as u64
}

struct SpriggingSource<'a> {
    rendered: &'a sprigging::RenderedLeaves,
    producers: &'a crate::ProducerRegistry,
}

impl SpriggingSource<'_> {
    fn leaf_commands(&self, key: u64) -> Option<Vec<paint_list_api::PaintCmd>> {
        self.producers
            .commands(key)
            .or_else(|| self.rendered.get(key).map(<[_]>::to_vec))
    }
}

/// The focused text field's paint inputs for this frame: which node, where the
/// caret is, and the selection byte range when one should be drawn.
type FocusedOverlay = (NodeId, VisualCaret, Option<(usize, usize)>);

type EmittedScene = (
    paint_list_render::TranslatedDisplayList,
    Option<PaintEnvelope>,
);

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

    /// Bring the retained layout up to date at `(lw, lh)` logical px: drain the
    /// runner's DOM mutations, rebuild the host-owned Livery frame when its
    /// inputs changed, and re-render custom-paint leaves at their new boxes. No
    /// GPU, no window. Returns whether an animation is still live.
    /// Republish the Window-Controls-Overlay titlebar area when it changes.
    ///
    /// As a stylesheet rule rather than an inline style on the root: the root's
    /// `style` attribute belongs to the application, and a host that
    /// overwrites it each frame would silently drop whatever the app put
    /// there. A `:root` rule declares the same inheriting custom properties
    /// without touching the DOM at all.
    ///
    /// Returns whether the values moved, because a changed sheet has to force
    /// the full relayout path — the incremental one applies DOM mutations, and
    /// this is not one.
    fn publish_titlebar_area(&mut self, lw: f32) -> bool {
        // The platform reports what it reserved in *device*-logical pixels —
        // macOS's traffic lights are a fixed size on screen, not a fixed
        // number of CSS pixels — and the sheet declares CSS ones. Under zoom
        // those are different units, so the strip is divided into layout space
        // before it is published; otherwise the page's content would clear a
        // gap of the wrong height at every zoom but 1.
        let zoom = self.ui_zoom();
        let insets = self
            .s
            .window
            .as_ref()
            .map_or(crate::TitlebarInsets::NONE, |w| {
                let insets = w.titlebar_insets();
                crate::TitlebarInsets {
                    left: insets.left / zoom,
                    right: insets.right / zoom,
                    height: insets.height / zoom,
                }
            });
        if self.s.titlebar_published == Some((insets, lw)) {
            return false;
        }
        self.s.titlebar_published = Some((insets, lw));
        self.s.titlebar_sheet = format!(":root {{ {} }}", insets.declarations(lw));
        true
    }

    pub fn relayout(&mut self, lw: f32, lh: f32) -> bool {
        if self.s.runner.is_none() {
            return false;
        }
        let layout_update_started = crate::Instant::now();
        let now_s = self.s.anim_base.elapsed().as_secs_f64();
        let titlebar_moved = self.publish_titlebar_area(lw);
        // Another window swapped the shared sheet: lay out afresh, as the
        // window that swapped it does.
        if self.s.layout_sheet_generation != self.s.shared.sheet_generation {
            self.s.layout = None;
            self.s.layout_size = (0.0, 0.0);
            self.s.layout_sheet_generation = self.s.shared.sheet_generation;
        }
        let runner = self.s.runner.as_mut().expect("checked above");
        let mut muts: Vec<DomMutation<NodeId>> = Vec::new();
        runner.drain_mutations(&mut muts);
        let dom = runner.dom();
        let mount = runner.mount();
        let dom_ref = dom.borrow();
        let view = crate::WindowDom::new(&dom_ref, mount);
        let sheets: Vec<&str> = vec![self.s.shared.sheet.as_str(), self.s.titlebar_sheet.as_str()];
        let mutation_count = muts.len() as u64;
        let size_changed = self.s.layout_size != (lw, lh);
        let mut tick_us = 0;
        let apply_us = 0;
        let mut rebuild_us = 0;
        let mut style_resolve_us = 0;
        let mut layout_with_text_us = 0;
        let mut content_extent_us = 0;
        let mut rebuilt = false;
        match self.s.layout.as_mut() {
            Some(layout) if muts.is_empty() && !size_changed && !titlebar_moved => {
                let phase = crate::Instant::now();
                let _ = layout.tick_animations(&view, now_s);
                tick_us = elapsed_us(phase.elapsed());
            },
            Some(layout) if !size_changed && !titlebar_moved => {
                rebuilt = true;
                let phase = crate::Instant::now();
                layout.rebuild(&view, lw, lh);
                rebuild_us = elapsed_us(phase.elapsed());
            },
            _ => {
                rebuilt = true;
                let phase = crate::Instant::now();
                let mut layout = crate::OwnedLayout::new(
                    &view,
                    &sheets,
                    lw,
                    lh,
                    &self.s.shared.fonts,
                    &self.s.shared.images,
                );
                // Carry BOTH scroll planes across rebuilds: element scroll and
                // the document scroll. Dropping the latter snaps a scrolled
                // page back to the top on structural re-render. Both setters
                // clamp against the layout this session has already done, so a
                // container that shrank or stopped scrolling cannot carry a
                // stale offset in.
                if let Some(prev) = self.s.layout.as_ref() {
                    layout.set_element_scroll(&view, prev.element_scroll().clone());
                    layout.set_viewport_scroll(prev.viewport_scroll());
                }
                self.s.layout = Some(layout);
                self.s.layout_size = (lw, lh);
                rebuild_us = elapsed_us(phase.elapsed());
            },
        }
        // A caret that moved since the last relayout (an edit, a caret key, a
        // click) is kept in view within its field, as a single-line input
        // keeps it. One that has not moved stays where the user scrolled it.
        let caret = self.focused_overlay().map(|(node, caret, _)| (node, caret));
        match caret {
            Some((node, caret)) if self.s.caret_followed != Some((node, caret.byte)) => {
                let layout = self.s.layout.as_mut().expect("layout just ensured");
                let _ = layout.caret_into_view(&view, node, caret);
                self.s.caret_followed = Some((node, caret.byte));
            },
            Some(_) => {},
            None => self.s.caret_followed = None,
        }
        // Scroll requests resolve against the layout just brought current, so
        // a node the requesting dispatch created already has a box. Each
        // resolves on its own: one that finds nothing drops only itself.
        if !self.s.pending_scroll.is_empty() {
            let layout = self.s.layout.as_mut().expect("layout just ensured");
            let now = crate::Instant::now();
            for request in std::mem::take(&mut self.s.pending_scroll) {
                for target in layout.scroll_into_view(&view, request.node, request.align) {
                    self.s.scrollbar_fade.note(target, now);
                }
            }
        }
        let layout = self.s.layout.as_ref().expect("layout just ensured");
        if rebuilt {
            (style_resolve_us, layout_with_text_us, content_extent_us) = layout.stage_timings();
        }
        let anim_active = layout.has_active_animations();
        let layout_update_us = elapsed_us(layout_update_started.elapsed());
        let leaf_boxes_started = crate::Instant::now();
        let sizes: HashMap<u64, (f32, f32)> = layout.custom_leaf_boxes(&view).into_iter().collect();
        self.s.leaf_keys.clear();
        self.s.leaf_keys.extend(sizes.keys().copied());
        let leaf_boxes_us = elapsed_us(leaf_boxes_started.elapsed());
        let leaf_render_started = crate::Instant::now();
        let leaf_repaints = self.s.shared.leaves.render_into(
            |key| {
                sizes
                    .get(&key)
                    .map(|&(width, height)| sprigging::Size { width, height })
            },
            &mut self.s.shared.rendered,
        );
        self.s.last_layout_update_us = layout_update_us;
        self.s.last_layout_tick_us = tick_us;
        self.s.last_layout_apply_us = apply_us;
        self.s.last_layout_rebuild_us = rebuild_us;
        self.s.last_style_resolve_us = style_resolve_us;
        self.s.last_layout_with_text_us = layout_with_text_us;
        self.s.last_content_extent_us = content_extent_us;
        self.s.last_layout_mutations = mutation_count;
        self.s.last_layout_rebuilt = rebuilt;
        self.s.last_leaf_boxes_us = leaf_boxes_us;
        self.s.last_leaf_render_us = elapsed_us(leaf_render_started.elapsed());
        self.s.last_leaf_repaints = leaf_repaints as u64;
        anim_active
    }

    /// Netrender roadmap E4 — reconcile the renderer's retained-fragment
    /// registry with this frame's rendered leaves. Runs each redraw after
    /// `relayout` (which refreshed `rendered`) and before `emit_scene`, so the
    /// map the emitter consults is current by construction.
    ///
    /// Only Path-A splices free of per-frame state become fragments: a splice
    /// carrying `DrawExternalTexture` or `DrawShadow` keeps the inline path,
    /// because composite textures and blurred shadow masks are rebuilt per
    /// frame by the host painter and cannot be retained in a lowering.
    fn sync_leaf_fragments(&mut self) {
        use paint_list_api::PaintCmd;

        let Some(surface) = self.s.surface.as_ref() else {
            // No surface this turn (suspended, or not yet booted). The
            // renderer lives in the render core, which outlives the surface,
            // so fragments registered before a suspend are still valid after
            // the resume and are kept, not forgotten.
            return;
        };
        let renderer = surface.core().renderer();

        let mut seen: Vec<u64> = Vec::new();
        for (key, epoch, splice) in self.s.shared.rendered.path_a_entries() {
            let fragmentable = !splice.iter().any(|cmd| {
                matches!(
                    cmd,
                    PaintCmd::DrawExternalTexture(_) | PaintCmd::DrawShadow(_)
                )
            });
            if !fragmentable {
                if let Some((id, _)) = self.s.shared.leaf_fragments.remove(&key) {
                    let _ = renderer.remove_fragment(id);
                }
                continue;
            }
            seen.push(key);
            match self.s.shared.leaf_fragments.get(&key) {
                Some((_, e)) if *e == epoch => {},
                Some(&(id, _)) => {
                    let fragment =
                        paint_list_render::translate_paint_cmds_to_fragment(splice, &[], &[]);
                    if renderer.update_fragment(id, fragment) == Some(true) {
                        self.s.shared.leaf_fragments.insert(key, (id, epoch));
                    }
                },
                None => {
                    let fragment =
                        paint_list_render::translate_paint_cmds_to_fragment(splice, &[], &[]);
                    if let Some(id) = renderer.register_fragment(fragment) {
                        self.s.shared.leaf_fragments.insert(key, (id, epoch));
                    }
                },
            }
        }
        // Sweep leaves that no longer render (removed from the registry or
        // not laid out): their retained lowerings go with them.
        let stale: Vec<u64> = self
            .s
            .shared
            .leaf_fragments
            .keys()
            .copied()
            .filter(|k| !seen.contains(k))
            .collect();
        for key in stale {
            if let Some((id, _)) = self.s.shared.leaf_fragments.remove(&key) {
                let _ = renderer.remove_fragment(id);
            }
        }
    }

    /// Where the focused field's caret paints, `(x, y, width, height)`, in the
    /// coordinates the cursor uses. `None` with no focused field, before a
    /// layout, or while the caret is scrolled out of its field.
    pub fn focused_caret_rect(&self) -> Option<(f32, f32, f32, f32)> {
        let (node, caret, _) = self.focused_overlay()?;
        let layout = self.s.layout.as_ref()?;
        let runner = self.s.runner.as_ref()?;
        let dom = runner.dom();
        let dom_ref = dom.borrow();
        let view = crate::WindowDom::new(&dom_ref, runner.mount());
        let rect = layout.caret_rect_for_position(&view, node, caret, 2.0)?;
        Some((rect.x, rect.y, rect.width, rect.height))
    }

    /// Where the focused field's selection paints, one rect per line run, in
    /// the same coordinates, clipped to the field. Empty with no selection.
    pub fn focused_selection_rects(&self) -> Vec<(f32, f32, f32, f32)> {
        let Some((node, _, Some((start, end)))) = self.focused_overlay() else {
            return Vec::new();
        };
        let (Some(layout), Some(runner)) = (self.s.layout.as_ref(), self.s.runner.as_ref()) else {
            return Vec::new();
        };
        let dom = runner.dom();
        let dom_ref = dom.borrow();
        let view = crate::WindowDom::new(&dom_ref, runner.mount());
        layout
            .selection_rects(&view, node, start, end)
            .into_iter()
            .map(|rect| (rect.x, rect.y, rect.width, rect.height))
            .collect()
    }

    /// The focused text field's paint inputs, as the application maps them.
    fn focused_overlay(&self) -> Option<FocusedOverlay> {
        let runner = self.s.runner.as_ref()?;
        let slot = (self.hooks.focused_text)(runner)?;
        let input = (slot.get)(runner.state());
        let mut caret = to_visual_caret(input.caret_position());
        caret.byte = input.caret_byte_in_render();
        let selection = if input.composition().is_none() && input.has_selection() {
            Some(input.selection_bytes())
        } else {
            None
        };
        Some((slot.node, caret, selection))
    }

    /// Tell the platform IME where the caret is, so a candidate window opens
    /// beside the text rather than at the window origin.
    fn sync_ime_area(&self) {
        let (Some(window), Some(layout), Some(runner)) = (
            self.s.window.as_ref(),
            self.s.layout.as_ref(),
            self.s.runner.as_ref(),
        ) else {
            return;
        };
        let Some((node, caret, _)) = self.focused_overlay() else {
            return;
        };
        let dom = runner.dom();
        let dom_ref = dom.borrow();
        let view = crate::WindowDom::new(&dom_ref, runner.mount());
        let Some(rect) = layout.caret_rect_for_position(&view, node, caret, 2.0) else {
            return;
        };
        // The seam takes the *platform's* logical coordinates (winit multiplies
        // them by the device scale, a browser by nothing), and the caret rect
        // is in layout coordinates. Those differ by exactly the zoom, so the
        // rect is carried back across it here — a candidate window that opened
        // at four fifths of the caret's position would be a zoom bug the user
        // sees before any other.
        let zoom = f64::from(self.ui_zoom());
        window.set_ime_cursor_area(
            rect.x as f64 * zoom,
            rect.y as f64 * zoom,
            (rect.width.max(2.0) as f64) * zoom,
            (rect.height.max(1.0) as f64) * zoom,
        );
    }

    /// Emit this frame's paint list with selection and caret inside the text
    /// field's CSS paint context, then any active overlay scrollbars, and
    /// lower it to a netrender scene plus the GPU resources that scene names.
    fn emit_scene(&mut self, lw: f32, lh: f32) -> Option<EmittedScene> {
        let focused_overlay = self.focused_overlay();
        let runner = self.s.runner.as_ref()?;
        let layout = self.s.layout.as_mut()?;
        let dom = runner.dom();
        let dom_ref = dom.borrow();
        let view = crate::WindowDom::new(&dom_ref, runner.mount());
        let source = SpriggingSource {
            rendered: &self.s.shared.rendered,
            producers: &self.s.shared.producers,
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
        let paint_capture = self
            .s
            .pending_paint_capture
            .as_ref()
            .map(|_| PaintEnvelope::from_list(&list));
        let translated = paint_list_render::translate_paint_cmd_stream(
            list.viewport(),
            list.commands(),
            list.fonts(),
            list.images(),
        );
        Some((translated, paint_capture))
    }

    pub fn redraw(&mut self) {
        self.s.last_redraw_presentation = None;
        self.deliver_files();
        let frame_started = crate::Instant::now();
        let mut profile = FrameProfile::default();
        // The application's frame hook first: animation drives, leaf syncs,
        // backend polls. Its return keeps frames coming.
        let phase = crate::Instant::now();
        let animating = self.prepare_frame();
        profile.frame_hook_us = elapsed_us(phase.elapsed());
        // One scale for the whole frame: device times zoom. Layout runs at
        // `physical / layout_scale` and the rasterizer composes that scene
        // under the same factor, so a zoomed frame is laid out at its new size
        // and drawn from outlines at full device resolution — not a smaller
        // frame resampled upward.
        let layout_scale = self.layout_scale() as f32;
        let target_size = self.s.window.as_ref().map(|window| {
            let size = window.inner_size();
            (size.0.max(1), size.1.max(1), layout_scale)
        });
        let (Some((pw, ph, scale)), true) = (target_size, self.s.surface.is_some()) else {
            // No window, or suspended with the surface taken away: there is
            // nothing to present. The layout still advances, so a resume
            // repaints current state rather than a stale one.
            let (lw, lh) = self.logical_size();
            self.suspend_producers();
            let phase = crate::Instant::now();
            self.relayout(lw, lh);
            profile.relayout_us = elapsed_us(phase.elapsed());
            profile.total_us = elapsed_us(frame_started.elapsed());
            self.s.last_frame_profile = Some(profile);
            return;
        };
        let (lw, lh) = (pw as f32 / scale, ph as f32 / scale);

        let phase = crate::Instant::now();
        let anim_active = self.relayout(lw, lh);
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
        profile.producers = self.prepare_producers(scale);
        let phase = crate::Instant::now();
        self.sync_leaf_fragments();
        profile.leaf_fragments_us = elapsed_us(phase.elapsed());
        let phase = crate::Instant::now();
        self.sync_ime_area();
        profile.ime_us = elapsed_us(phase.elapsed());
        let phase = crate::Instant::now();
        let Some((translated, paint_capture)) = self.emit_scene(lw, lh) else {
            profile.emit_scene_us = elapsed_us(phase.elapsed());
            profile.total_us = elapsed_us(frame_started.elapsed());
            self.s.last_frame_profile = Some(profile);
            return;
        };
        profile.emit_scene_us = elapsed_us(phase.elapsed());

        let Some(surface) = self.s.surface.as_ref() else {
            profile.total_us = elapsed_us(frame_started.elapsed());
            self.s.last_frame_profile = Some(profile);
            return;
        };
        // Blurred shadows lower to image ops backed by per-frame GPU masks.
        // Keeping only `translated.scene` drops those masks and leaves the
        // image keys unresolved, which makes every blurred CSS box-shadow
        // disappear in this host even though the paint list is correct.
        let phase = crate::Instant::now();
        for mask in &translated.box_shadow_masks {
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
        let scene = &translated.scene;
        let clear = if self.options.app_frame_is_transparent() {
            wgpu::Color::TRANSPARENT
        } else {
            wgpu::Color::BLACK
        };
        let phase = crate::Instant::now();
        // Keyed by this host: several windows rasterize through one core, and
        // an unkeyed raster diffs against whichever surface drew last (F17).
        let (_tex, view) = surface.core().rasterize_scaled_for(
            self.s.presentation_host,
            scene,
            pw,
            ph,
            ColorLoad::Clear(clear),
            scale,
        );
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
        let Some(frame) = surface.acquire() else {
            profile.acquire_us = elapsed_us(phase.elapsed());
            profile.total_us = elapsed_us(frame_started.elapsed());
            self.s.last_frame_profile = Some(profile);
            return;
        };
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
        profile.compose_us = elapsed_us(phase.elapsed());
        // wgpu 30 moved presentation from SurfaceTexture to Queue.
        let phase = crate::Instant::now();
        surface.queue().present(frame);
        profile.present_us = elapsed_us(phase.elapsed());
        let Some(sequence) = self.s.presentation_sequence.checked_add(1) else {
            eprintln!("[cambium-host] presentation sequence exhausted");
            return;
        };
        self.s.presentation_sequence = sequence;
        let presented = crate::PresentedFrame {
            host: self.s.presentation_host,
            sequence,
            width: pw,
            height: ph,
            layout_scale: scale,
        };
        self.s.last_redraw_presentation = Some(presented);
        // Freeze product state before any capture callback, queued pointer
        // dispatch or platform accessibility action can change the product.
        self.observe_presentation(presented);
        let surface = self.s.surface.as_ref().expect("presented surface exists");
        // A capture armed by the application: run it while the rasterized
        // view is still alive.
        let phase = crate::Instant::now();
        if let Some(envelope) = paint_capture
            && let Some(capture) = self.s.pending_paint_capture.take()
        {
            capture(envelope);
        }
        if let Some(capture) = self.s.pending_capture.take() {
            capture(&**surface, &view, pw, ph);
        }
        if let Some(capture) = self.s.pending_stamped_capture.take() {
            capture(&**surface, &view, presented);
        }
        profile.capture_us = elapsed_us(phase.elapsed());
        if (animating || anim_active)
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

    /// Hand this frame to the accessibility host: build/install/update the
    /// tree, and route the screen reader's requests back into the retained DOM.
    /// Called after `redraw`, once this frame's layout exists. The first sync
    /// reveals the hidden window (install-before-show).
    ///
    /// The two request kinds stay apart, because they mean different things to
    /// the person using the reader: `Click` is "do this control's thing" and
    /// goes through the same dispatch a mouse press does; `Focus` is "put the
    /// cursor here" and only moves focus. Collapsing them would fire every
    /// control a reader navigates across.
    pub fn sync_a11y(&mut self) {
        // A screen reader is told physical client pixels, and the tree it is
        // told them about is laid out in layout pixels: the transform between
        // them is the layout scale, zoom included. A reader that read the
        // device scale alone would point at a control's unzoomed position.
        let layout_scale = self.layout_scale();
        let requests = {
            let (dom, mount, focus) = match self.s.runner.as_ref() {
                Some(runner) => (runner.dom(), runner.mount(), runner.focus()),
                None => return,
            };
            let dom_ref = dom.borrow();
            let view = crate::WindowDom::new(&dom_ref, mount);
            // Focus can change without hover; last_focus belongs to restyling,
            // not to the accessibility adapter's current focus report.
            let focus = focus.map(|node| view.opaque_id(node));
            let (Some(a11y), Some(layout)) = (self.s.a11y.as_mut(), self.s.layout.as_ref()) else {
                return;
            };
            // The window is the adapter's own now, so the seam does not carry it.
            a11y.sync(
                &view,
                layout,
                &mut self.s.shared.leaves,
                &mut self.s.shared.producers,
                focus,
                layout_scale,
            )
        };
        self.apply_a11y_requests(&requests);
    }

    /// Route drained screen-reader requests into the retained DOM. Split out of
    /// [`sync_a11y`] so the routing is exercisable without an OS adapter.
    pub fn apply_a11y_requests(&mut self, requests: &[crate::A11yRequest]) {
        if requests.is_empty() {
            return;
        }
        for request in requests {
            let node = match &request.target {
                crate::A11yTarget::Node(node) => *node,
                // A drawn node's action button: the node lives in its slot,
                // so the app's focus goes to the slot (keys then reach the
                // producer's view, a keyboard move's arrows), and a click is
                // the producer's to carry out.
                crate::A11yTarget::Produced(produced) => {
                    let slot = match (self.s.runner.as_ref(), self.s.layout.as_ref()) {
                        (Some(runner), Some(layout)) => {
                            let dom = runner.dom();
                            let dom = dom.borrow();
                            let view = crate::WindowDom::new(&dom, runner.mount());
                            layout
                                .custom_leaf_nodes(&view)
                                .into_iter()
                                .find_map(|(key, node)| (key == produced.slot).then_some(node))
                        },
                        _ => None,
                    };
                    if let (Some(slot), Some(runner)) = (slot, self.s.runner.as_mut()) {
                        runner.set_focus(Some(slot));
                    }
                    if request.action == A11yAction::Click {
                        self.s.shared.producers.act(produced);
                    }
                    continue;
                },
            };
            let Some(runner) = self.s.runner.as_mut() else {
                break;
            };
            match request.action {
                A11yAction::Click => {
                    // No cursor is involved, so the local point is genuinely the
                    // element's own origin rather than a hit position.
                    runner.dispatch_click(node, PointerClick::at((0.0, 0.0)));
                },
                A11yAction::Focus => runner.set_focus(Some(node)),
                A11yAction::SetValue(value) => {
                    runner.dispatch_value(node, cambium::ValueEvent { value });
                },
            }
        }
        // Focus may have moved without any pointer motion; refresh the
        // `:focus` restyle so the visible state matches what the reader says.
        self.hover();
        self.after_dispatch();
    }

    /// Drive `:hover` / `:focus` restyles on target change. The retained layout
    /// skips the cascade when no rule can depend on that state, and skips
    /// text layout when the cascade leaves the computed styles unchanged.
    pub fn hover(&mut self) {
        let (Some(runner), Some(layout)) = (self.s.runner.as_ref(), self.s.layout.as_mut()) else {
            return;
        };
        let (x, y) = self.s.cursor;
        let dom = runner.dom();
        let dom_ref = dom.borrow();
        let view = crate::WindowDom::new(&dom_ref, runner.mount());
        let hovered_node = layout.hit_test(&view, x, y);
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
        if layout.set_interaction(&view, hovered_node, focused_node) {
            drop(dom_ref);
            if let Some(window) = self.s.window.as_ref() {
                window.request_redraw();
            }
        }
    }

    /// Route Cambium `on_hover` Enter/Leave as the hit node changes. The host
    /// owns transition detection; Move is not routed, so idle motion within a
    /// target stays free. Coordinates are zeroed — a peek only needs which
    /// target.
    pub fn hover_dispatch(&mut self) {
        use cambium::{HoverEvent, HoverPhase};
        let hit = self.hit_at_cursor();
        if hit == self.s.last_hover_hit {
            return;
        }
        let old = self.s.last_hover_hit.take();
        self.s.last_hover_hit = hit;
        let Some(runner) = self.s.runner.as_mut() else {
            return;
        };
        if let Some(old) = old {
            runner.dispatch_hover(
                old,
                HoverEvent::new(HoverPhase::Leave, (0.0, 0.0), (0.0, 0.0)),
            );
        }
        if let Some(new) = hit {
            runner.dispatch_hover(
                new,
                HoverEvent::new(HoverPhase::Enter, (0.0, 0.0), (0.0, 0.0)),
            );
        }
        self.after_dispatch();
    }
}
