// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl<State, Logic, V, T> Host<State, Logic, V, T>
where
    T: crate::HostTree<State>,
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
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
    pub(super) fn publish_titlebar_area(&mut self, lw: f32) -> bool {
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
                layout.rebuild(&view, lw, lh, &muts);
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
                let moved = match request {
                    ScrollRequest::IntoView(request) => {
                        layout.scroll_into_view(&view, request.node, request.align)
                    },
                    ScrollRequest::Text { node, byte, align } => {
                        layout.scroll_text_position(&view, node, byte, align)
                    },
                };
                for target in moved {
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
    pub(crate) fn sync_leaf_fragments(&mut self) -> Result<(), LeafFragmentAdmissionError> {
        use paint_list_api::PaintCmd;

        let Some(surface) = self.s.surface.as_deref().or_else(|| {
            self.s
                .presentation_driver
                .as_ref()
                .and_then(|driver| driver.wgpu_surface())
        }) else {
            // No surface this turn (suspended, or not yet booted). The
            // renderer lives in the render core, which outlives the surface,
            // so fragments registered before a suspend are still valid after
            // the resume and are kept, not forgotten.
            return Ok(());
        };
        // One operation over this window's leaves; nothing below rasterizes
        // or calls a producer while the browser frame gate is held.
        let renderer = crate::try_renderer(surface.core())
            .map_err(LeafFragmentAdmissionError::RendererAccess)?;

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
                        paint_list_render::translate_paint_cmds_to_fragment(splice, &[], &[])
                            .map_err(|error| LeafFragmentAdmissionError::Translation {
                                key,
                                epoch,
                                error,
                            })?;
                    renderer
                        .try_update_fragment(id, fragment)
                        .map_err(|error| LeafFragmentAdmissionError::Retained {
                            key,
                            epoch,
                            error,
                        })?;
                    self.s.shared.leaf_fragments.insert(key, (id, epoch));
                },
                None => {
                    let fragment =
                        paint_list_render::translate_paint_cmds_to_fragment(splice, &[], &[])
                            .map_err(|error| LeafFragmentAdmissionError::Translation {
                                key,
                                epoch,
                                error,
                            })?;
                    let id = renderer.try_register_fragment(fragment).map_err(|error| {
                        LeafFragmentAdmissionError::Retained { key, epoch, error }
                    })?;
                    self.s.shared.leaf_fragments.insert(key, (id, epoch));
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
        Ok(())
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
    pub(super) fn focused_overlay(&self) -> Option<FocusedOverlay> {
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
    pub(crate) fn sync_ime_area(&self) {
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
}
