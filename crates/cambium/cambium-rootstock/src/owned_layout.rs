// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Rootstock-owned retained Livery session over Cambium's externally owned DOM.

use std::collections::HashMap;

use genet_livery::{
    Device, InteractionStates, LiveryLayout, LiveryPaintList, StylePlane, StyleSet, TextRange,
    TextSystem, ViewportSizes, emit_paint_list_with_text_system_scrolled_with_images,
    hit_test_with_scroll, layout_with_text_system, resolve_styles,
};
use genet_render::{VisualAffinity, VisualCaret, VisualMovement, VisualSelection};
use genet_scripted_dom::NodeId;
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};
use paint_list_api::{ColorF, DeviceIntSize, LayoutPoint, LayoutRect, LayoutSize};

mod interaction;
mod producer;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ScrollTarget {
    Document,
    Element(NodeId),
}

/// Where a scroll request places its element in the scroll area. No animation:
/// the offset moves in one step.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ScrollAlign {
    /// The element's top edge at the top of the scroll area.
    #[default]
    Start,
    /// Move only if the element is not fully visible, and only as far as it
    /// needs: its top edge to the area's top when it sits above, its bottom
    /// edge to the area's bottom when below. An element taller than the area
    /// aligns its top.
    Nearest,
}

pub struct OwnedLayout {
    style_set: StyleSet,
    interaction_dependencies: interaction::Dependencies,
    device: Device,
    interactions: InteractionStates<NodeId>,
    hovered: Option<NodeId>,
    focused: Option<NodeId>,
    // Keep the cascade result separately: layout can adjust its output styles.
    resolved_styles: StylePlane<NodeId>,
    styles: StylePlane<NodeId>,
    fragments: LiveryLayout<NodeId>,
    text: TextSystem,
    viewport: (f32, f32),
    viewport_scroll: (f32, f32),
    element_scroll: HashMap<NodeId, (f32, f32)>,
    content_extent: (f32, f32),
    /// Host-supplied image bytes by URL. Layout resolves intrinsic sizes and
    /// paint decodes textures out of the same ledger, so it is retained here.
    images: HashMap<String, Vec<u8>>,
    style_resolve_us: u64,
    layout_with_text_us: u64,
    content_extent_us: u64,
    generation: u64,
}

impl OwnedLayout {
    /// A retained session over the host's resource ledger: font faces to
    /// register into its text system, and image bytes to resolve `url()`
    /// against. The session is rebuilt whenever the ledger changes, so
    /// registration cannot outlive a face the host withdrew.
    pub(crate) fn new<D: LayoutDom<NodeId = NodeId>>(
        dom: &D,
        sheets: &[&str],
        width: f32,
        height: f32,
        fonts: &[crate::HostFont],
        images: &HashMap<String, Vec<u8>>,
    ) -> Self {
        let style_set = StyleSet::cambium(sheets);
        let interaction_dependencies = interaction::Dependencies::new(&style_set);
        let device = Device::screen(width, height);
        let interactions = InteractionStates::default();
        let phase = crate::Instant::now();
        let resolved_styles = resolve_styles(dom, &style_set, &device, &interactions);
        let style_resolve_us = elapsed_us(phase.elapsed());
        let mut text = TextSystem::new();
        register_fonts(&mut text, fonts);
        let phase = crate::Instant::now();
        let (styles, fragments) = layout_with_text_system(
            dom,
            &resolved_styles,
            width,
            height,
            ViewportSizes::uniform(width, height),
            &mut text,
            images,
        )
        .expect("Cambium's authored Livery layout must resolve");
        let layout_with_text_us = elapsed_us(phase.elapsed());
        let phase = crate::Instant::now();
        let content_extent = content_extent(dom, &fragments);
        let content_extent_us = elapsed_us(phase.elapsed());
        Self {
            style_set,
            interaction_dependencies,
            device,
            interactions,
            hovered: None,
            focused: None,
            resolved_styles,
            styles,
            fragments,
            text,
            viewport: (width, height),
            viewport_scroll: (0.0, 0.0),
            element_scroll: HashMap::new(),
            content_extent,
            images: images.clone(),
            style_resolve_us,
            layout_with_text_us,
            content_extent_us,
            generation: 1,
        }
    }

    pub(crate) fn rebuild<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        dom: &D,
        width: f32,
        height: f32,
    ) {
        self.viewport = (width, height);
        self.device.set_viewport_size(width, height);
        let phase = crate::Instant::now();
        let styles = resolve_styles(dom, &self.style_set, &self.device, &self.interactions);
        self.style_resolve_us = elapsed_us(phase.elapsed());
        self.layout_resolved(dom, styles);
    }

    fn layout_resolved<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        dom: &D,
        resolved_styles: StylePlane<NodeId>,
    ) {
        let (width, height) = self.viewport;
        let phase = crate::Instant::now();
        let (styles, fragments) = layout_with_text_system(
            dom,
            &resolved_styles,
            width,
            height,
            self.device.viewport_sizes,
            &mut self.text,
            &self.images,
        )
        .expect("Cambium's authored Livery layout must resolve");
        self.layout_with_text_us = elapsed_us(phase.elapsed());
        self.resolved_styles = resolved_styles;
        self.styles = styles;
        self.fragments = fragments;
        let phase = crate::Instant::now();
        self.content_extent = content_extent(dom, &self.fragments);
        self.content_extent_us = elapsed_us(phase.elapsed());
        self.clamp_viewport_scroll();
        self.clamp_element_scroll(dom);
        self.generation = self.generation.saturating_add(1);
    }

    pub(crate) fn stage_timings(&self) -> (u64, u64, u64) {
        (
            self.style_resolve_us,
            self.layout_with_text_us,
            self.content_extent_us,
        )
    }

    pub fn fragments(&self) -> &LiveryLayout<NodeId> {
        &self.fragments
    }

    /// Font instances this session's text system has materialised. The
    /// observable end of the host font seam.
    pub fn retained_font_count(&self) -> usize {
        self.text.retained_font_count()
    }

    pub(crate) fn has_active_animations(&self) -> bool {
        false
    }

    pub(crate) fn tick_animations<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        _dom: &D,
        _now: f64,
    ) -> bool {
        false
    }

    pub fn element_scroll(&self) -> &HashMap<NodeId, (f32, f32)> {
        &self.element_scroll
    }

    /// Adopt a scroll plane taken against an older layout. A carried plane
    /// arrives *after* the new session has laid out, so it is clamped here
    /// rather than waiting for the next `layout_resolved`.
    pub(crate) fn set_element_scroll<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        dom: &D,
        scroll: HashMap<NodeId, (f32, f32)>,
    ) {
        self.element_scroll = scroll;
        self.clamp_element_scroll(dom);
    }

    pub fn viewport_scroll(&self) -> (f32, f32) {
        self.viewport_scroll
    }

    pub(crate) fn set_viewport_scroll(&mut self, scroll: (f32, f32)) {
        self.viewport_scroll = scroll;
        self.clamp_viewport_scroll();
    }

    pub(crate) fn custom_leaf_boxes<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
    ) -> Vec<(u64, (f32, f32))> {
        self.custom_leaf_nodes(dom)
            .into_iter()
            .filter_map(|(key, node)| Some((key, self.element_geometry(dom, node)?.content_size())))
            .collect()
    }

    pub(crate) fn hit_test<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
        x: f32,
        y: f32,
    ) -> Option<NodeId> {
        hit_test_with_scroll(
            dom,
            &self.styles,
            &self.fragments,
            &self.element_scroll,
            x + self.viewport_scroll.0,
            y + self.viewport_scroll.1,
        )
    }

    pub fn painted_rect<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
        node: NodeId,
    ) -> Option<(f32, f32, f32, f32)> {
        let fragment = self.fragments.get(node)?;
        let (nested_x, nested_y) = ancestor_scroll(dom, node, &self.element_scroll);
        Some((
            fragment.x - self.viewport_scroll.0 - nested_x,
            fragment.y - self.viewport_scroll.1 - nested_y,
            fragment.width,
            fragment.height,
        ))
    }

    pub(crate) fn caret_position_at_point<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
        node: NodeId,
        x: f32,
        y: f32,
    ) -> Option<VisualCaret> {
        let (scroll_x, scroll_y) = self.content_scroll(dom, node);
        let (text_node, byte) = self
            .fragments
            .text_position_at_point(x + scroll_x, y + scroll_y)?;
        Some(VisualCaret {
            byte: element_offset(dom, node, text_node, byte)?,
            affinity: VisualAffinity::Downstream,
        })
    }

    /// The caret at byte `offset` of `node`'s text, in document coordinates
    /// before any scroll, from the first of [`text_positions`] the text frame
    /// can place. A node with no measurable text shows it at the start of its
    /// content box, a line tall.
    fn caret_rect_at<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
        node: NodeId,
        offset: usize,
    ) -> Option<genet_livery::TextRect> {
        if let Some(rect) = text_positions(dom, node, offset)
            .into_iter()
            .find_map(|(text_node, local)| self.fragments.caret_rect(text_node, local))
        {
            return Some(rect);
        }
        let fragment = self.fragments.get(node)?;
        let px = |property| computed_px(&self.styles, node, property);
        let line = match px("line-height") {
            height if height > 0.0 => height,
            _ => px("font-size") * 1.2,
        };
        Some(genet_livery::TextRect {
            x: fragment.x + px("border-left-width") + px("padding-left"),
            y: fragment.y + px("border-top-width") + px("padding-top"),
            width: 1.0,
            height: line,
        })
    }

    /// The caret's painted rect in `node`, moved with every scroll that moves
    /// the node's text and clipped to the node's own box when it clips. `None`
    /// when the caret is scrolled out of that box.
    pub(crate) fn caret_rect_for_position<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
        node: NodeId,
        caret: VisualCaret,
        width: f32,
    ) -> Option<genet_livery::TextRect> {
        let mut rect = self.caret_rect_at(dom, node, caret.byte)?;
        let (scroll_x, scroll_y) = self.content_scroll(dom, node);
        rect.x -= scroll_x;
        rect.y -= scroll_y;
        rect.width = width;
        match self.content_clip(dom, node) {
            Some(clip) => clip_text_rect(rect, clip),
            None => Some(rect),
        }
    }

    /// Every scroll that moves `node`'s own content: the viewport's, each
    /// enclosing container's, and `node`'s own when it scrolls.
    fn content_scroll<D: LayoutDom<NodeId = NodeId>>(&self, dom: &D, node: NodeId) -> (f32, f32) {
        let (nested_x, nested_y) = ancestor_scroll(dom, node, &self.element_scroll);
        let (own_x, own_y) = self.element_scroll.get(&node).copied().unwrap_or_default();
        (
            self.viewport_scroll.0 + nested_x + own_x,
            self.viewport_scroll.1 + nested_y + own_y,
        )
    }

    /// Where `node` shows its content when it clips overflow: its padding box,
    /// painted. `None` when it lets content show outside it.
    fn content_clip<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
        node: NodeId,
    ) -> Option<(f32, f32, f32, f32)> {
        let clips = |property| {
            self.styles
                .computed_style(node, property)
                .is_some_and(|value| value != "visible")
        };
        if !clips("overflow-x") && !clips("overflow-y") {
            return None;
        }
        let (x, y, width, height) = self.painted_rect(dom, node)?;
        let px = |property| computed_px(&self.styles, node, property);
        let (left, right) = (px("border-left-width"), px("border-right-width"));
        let (top, bottom) = (px("border-top-width"), px("border-bottom-width"));
        Some((
            x + left,
            y + top,
            (width - left - right).max(0.0),
            (height - top - bottom).max(0.0),
        ))
    }

    pub(crate) fn selection_rects<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
        node: NodeId,
        start: usize,
        end: usize,
    ) -> Vec<genet_livery::TextRect> {
        let (scroll_x, scroll_y) = self.content_scroll(dom, node);
        let clip = self.content_clip(dom, node);
        // The selection starts where a caret at `start` would, and ends at
        // the close of the text before `end`.
        let (Some((anchor_node, anchor_offset)), Some((focus_node, focus_offset))) = (
            text_positions(dom, node, start).into_iter().next(),
            text_positions(dom, node, end).into_iter().last(),
        ) else {
            return Vec::new();
        };
        self.fragments
            .text_selection(TextRange {
                anchor_node,
                anchor_offset,
                focus_node,
                focus_offset,
            })
            .map(|selection| {
                selection
                    .rects
                    .into_iter()
                    .filter_map(|mut rect| {
                        rect.x -= scroll_x;
                        rect.y -= scroll_y;
                        match clip {
                            Some(clip) => clip_text_rect(rect, clip),
                            None => Some(rect),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(crate) fn selection_visual_move<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
        node: NodeId,
        selection: VisualSelection,
        movement: VisualMovement,
        extend: bool,
    ) -> Option<VisualSelection> {
        let text = node_text(dom, node);
        let len = text.len();
        let mut byte = selection.focus.byte.min(len);
        byte = match movement {
            VisualMovement::PreviousCluster => previous_boundary(&text, byte),
            VisualMovement::NextCluster => next_boundary(&text, byte),
            VisualMovement::PreviousWord => previous_word(&text, byte),
            VisualMovement::NextWord => next_word(&text, byte),
            VisualMovement::LineStart => 0,
            VisualMovement::LineEnd => len,
            VisualMovement::PreviousLine | VisualMovement::NextLine => return None,
        };
        let focus = VisualCaret {
            byte,
            affinity: VisualAffinity::Downstream,
        };
        Some(VisualSelection {
            anchor: if extend { selection.anchor } else { focus },
            focus,
        })
    }

    pub fn computed_value(&self, node: NodeId, property: &str) -> Option<String> {
        self.styles.computed_style(node, property)
    }

    pub fn computed_custom_property(&self, node: NodeId, name: &str) -> Option<String> {
        let properties = self.styles.custom_properties(node)?;
        properties
            .get(&format!("--{name}"))
            .or_else(|| properties.get(name))
            .cloned()
    }

    pub(crate) fn selection_style<D: LayoutDom<NodeId = NodeId>>(
        &self,
        _dom: &D,
        _node: NodeId,
    ) -> Option<([f32; 4], [f32; 4])> {
        None
    }

    /// The caret paints in its field's text colour, as `caret-color: auto`
    /// does, so it shows on a light field as well as a dark one.
    pub(crate) fn caret_color<D: LayoutDom<NodeId = NodeId>>(
        &self,
        _dom: &D,
        node: NodeId,
    ) -> Option<[f32; 4]> {
        self.styles.used_color(node)
    }

    pub(crate) fn emit_paint_list_with_leaves<D, F, G>(
        &mut self,
        dom: &D,
        viewport: DeviceIntSize,
        mut commands: F,
        mut fragment: G,
    ) -> LiveryPaintList
    where
        D: LayoutDom<NodeId = NodeId>,
        F: FnMut(u64) -> Option<Vec<paint_list_api::PaintCmd>>,
        G: FnMut(u64) -> Option<u64>,
    {
        let mut list = emit_paint_list_with_text_system_scrolled_with_images(
            dom,
            &self.styles,
            &self.fragments,
            viewport,
            self.generation,
            &mut self.text,
            &self.element_scroll,
            &self.images,
        );
        // Slots were recorded against the un-translated list. Fill them before
        // the document viewport transform is added so their indices and their
        // CSS paint context stay aligned.
        list.splice_host_leaf_slots(&mut commands, &mut fragment);
        list.translated(-self.viewport_scroll.0, -self.viewport_scroll.1)
    }

    pub(crate) fn push_rect(
        list: &mut LiveryPaintList,
        rect: genet_livery::TextRect,
        color: ColorF,
    ) {
        list.push_overlay_rect(
            LayoutRect::from_origin_and_size(
                LayoutPoint::new(rect.x, rect.y),
                LayoutSize::new(rect.width, rect.height),
            ),
            color,
        );
    }

    pub(crate) fn append_scrollbars<D: LayoutDom<NodeId = NodeId>>(
        &self,
        _dom: &D,
        _list: &mut LiveryPaintList,
        _alpha: &impl Fn(ScrollTarget) -> f32,
    ) {
    }

    pub(crate) fn scroll_at_target<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        dom: &D,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
    ) -> Option<ScrollTarget> {
        let mut candidate = self.hit_test(dom, x, y);
        while let Some(node) = candidate {
            let (scrolls_x, scrolls_y) = scroll_axes(&self.styles, node);
            if scrolls_x || scrolls_y {
                let range = element_scroll_range(dom, &self.fragments, node);
                let current = self.element_scroll.get(&node).copied().unwrap_or_default();
                let next = (
                    if scrolls_x {
                        (current.0 + dx).clamp(0.0, range.0)
                    } else {
                        current.0
                    },
                    if scrolls_y {
                        (current.1 + dy).clamp(0.0, range.1)
                    } else {
                        current.1
                    },
                );
                if next != current {
                    self.element_scroll.insert(node, next);
                    return Some(ScrollTarget::Element(node));
                }
            }
            candidate = dom.parent(node);
        }
        let before = self.viewport_scroll;
        self.viewport_scroll.0 += dx;
        self.viewport_scroll.1 += dy;
        self.clamp_viewport_scroll();
        (self.viewport_scroll != before).then_some(ScrollTarget::Document)
    }

    /// Bring `node` into view on the vertical axis by moving one plane: the
    /// nearest ancestor that scrolls vertically and has room to, otherwise the
    /// document viewport, clamped to that plane's range. `None` when nothing
    /// moved, which includes a node that is gone or does not paint.
    pub(crate) fn scroll_into_view<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        dom: &D,
        node: NodeId,
        align: ScrollAlign,
    ) -> Option<ScrollTarget> {
        let (_, top, _, height) = self.painted_rect(dom, node)?;
        let container = self.vertical_scroll_container(dom, node);
        let (area_top, area_height, current, range) = match container {
            Some((container, range)) => {
                let (_, y, _, h) = self.painted_rect(dom, container)?;
                let current = self.element_scroll.get(&container).map_or(0.0, |s| s.1);
                (y, h, current, range)
            },
            None => (
                0.0,
                self.viewport.1,
                self.viewport_scroll.1,
                (self.content_extent.1 - self.viewport.1).max(0.0),
            ),
        };
        let to_top = top - area_top;
        let to_bottom = top + height - (area_top + area_height);
        let delta = match align {
            ScrollAlign::Start => to_top,
            ScrollAlign::Nearest if to_top < 0.0 => to_top,
            ScrollAlign::Nearest if to_bottom > 0.0 => to_bottom.min(to_top),
            ScrollAlign::Nearest => 0.0,
        };
        let next = (current + delta).clamp(0.0, range);
        if next == current {
            return None;
        }
        Some(match container {
            Some((container, _)) => {
                self.element_scroll.entry(container).or_default().1 = next;
                ScrollTarget::Element(container)
            },
            None => {
                self.viewport_scroll.1 = next;
                ScrollTarget::Document
            },
        })
    }

    /// The nearest ancestor of `node` that scrolls vertically and has room to,
    /// with its range. An `overflow: auto` box that grows to its content has
    /// none, so a request passes over it, as the wheel does.
    fn vertical_scroll_container<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
        node: NodeId,
    ) -> Option<(NodeId, f32)> {
        let mut candidate = dom.parent(node);
        while let Some(ancestor) = candidate {
            if scroll_axes(&self.styles, ancestor).1 {
                let range = element_scroll_range(dom, &self.fragments, ancestor).1;
                if range > 0.0 {
                    return Some((ancestor, range));
                }
            }
            candidate = dom.parent(ancestor);
        }
        None
    }

    fn clamp_viewport_scroll(&mut self) {
        self.viewport_scroll.0 = self
            .viewport_scroll
            .0
            .clamp(0.0, (self.content_extent.0 - self.viewport.0).max(0.0));
        self.viewport_scroll.1 = self
            .viewport_scroll
            .1
            .clamp(0.0, (self.content_extent.1 - self.viewport.1).max(0.0));
    }

    /// A nested offset outlives the layout it was taken against. Drop the ones
    /// whose node is gone or no longer scrolls — an absent computed overflow
    /// reads the same as a non-scrolling one — and clamp the rest to the range
    /// the current fragments give. The nested half of `clamp_viewport_scroll`,
    /// and the same shape as genet-livery's nested clamp.
    fn clamp_element_scroll<D: LayoutDom<NodeId = NodeId>>(&mut self, dom: &D) {
        let (styles, fragments) = (&self.styles, &self.fragments);
        self.element_scroll.retain(|&node, offset| {
            let (scrolls_x, scrolls_y) = scroll_axes(styles, node);
            if !scrolls_x && !scrolls_y {
                return false;
            }
            let range = element_scroll_range(dom, fragments, node);
            offset.0 = offset.0.clamp(0.0, range.0);
            offset.1 = offset.1.clamp(0.0, range.1);
            true
        });
    }
}

/// Whether the node's computed overflow scrolls, per axis.
fn scroll_axes(styles: &StylePlane<NodeId>, node: NodeId) -> (bool, bool) {
    let scrolls = |property| {
        styles
            .computed_style(node, property)
            .is_some_and(|value| matches!(value.as_str(), "auto" | "scroll"))
    };
    (scrolls("overflow-x"), scrolls("overflow-y"))
}

fn element_scroll_range<D: LayoutDom<NodeId = NodeId>>(
    dom: &D,
    fragments: &LiveryLayout<NodeId>,
    node: NodeId,
) -> (f32, f32) {
    let Some(container) = fragments.get(node) else {
        return (0.0, 0.0);
    };
    let mut extent = (
        container.x + container.width,
        container.y + container.height,
    );
    for child in dom.dom_children(node) {
        walk(dom, child, &mut |descendant| {
            if let Some(fragment) = fragments.get(descendant) {
                extent.0 = extent.0.max(fragment.x + fragment.width);
                extent.1 = extent.1.max(fragment.y + fragment.height);
            }
        });
    }
    (
        (extent.0 - container.x - container.width).max(0.0),
        (extent.1 - container.y - container.height).max(0.0),
    )
}

/// A computed length in pixels, `0` for anything else.
fn computed_px(styles: &StylePlane<NodeId>, node: NodeId, property: &str) -> f32 {
    styles
        .computed_style(node, property)
        .and_then(|value| value.strip_suffix("px")?.trim().parse().ok())
        .unwrap_or(0.0)
}

/// `rect` cut to `clip`, or `None` when nothing of it is left.
fn clip_text_rect(
    mut rect: genet_livery::TextRect,
    (x, y, width, height): (f32, f32, f32, f32),
) -> Option<genet_livery::TextRect> {
    let left = rect.x.max(x);
    let top = rect.y.max(y);
    let right = (rect.x + rect.width).min(x + width);
    let bottom = (rect.y + rect.height).min(y + height);
    if right <= left || bottom <= top {
        return None;
    }
    rect.x = left;
    rect.y = top;
    rect.width = right - left;
    rect.height = bottom - top;
    Some(rect)
}

/// Register the host's faces into a freshly built text system. Every
/// construction site calls this, so a face survives any relayout that rebuilds
/// the session.
fn register_fonts(text: &mut TextSystem, fonts: &[crate::HostFont]) {
    for font in fonts {
        match font.family.as_deref() {
            // `@font-face` semantics: the sheet's family name wins over the one
            // the face's own name table declares. The feature-settings value
            // comes through its parser because its CSS type lives in `livery`,
            // which rootstock reaches only through genet-livery.
            Some(family) => {
                let normal = "normal".parse().expect("`normal` font-feature-settings");
                text.register_font_face_bytes(font.bytes.clone(), family, &normal)
            },
            None => text.register_font_bytes(font.bytes.clone()),
        }
    }
}

fn content_extent<D: LayoutDom<NodeId = NodeId>>(
    dom: &D,
    fragments: &LiveryLayout<NodeId>,
) -> (f32, f32) {
    let mut extent: (f32, f32) = (0.0, 0.0);
    walk(dom, dom.document(), &mut |node| {
        if let Some(fragment) = fragments.get(node) {
            extent.0 = extent.0.max(fragment.x + fragment.width);
            extent.1 = extent.1.max(fragment.y + fragment.height);
        }
    });
    extent
}

fn elapsed_us(elapsed: crate::Duration) -> u64 {
    elapsed.as_micros().min(u64::MAX as u128) as u64
}

fn walk<D: LayoutDom<NodeId = NodeId>>(dom: &D, node: NodeId, visit: &mut impl FnMut(NodeId)) {
    visit(node);
    for child in dom.dom_children(node) {
        walk(dom, child, visit);
    }
}

fn custom_leaf_key<D: LayoutDom<NodeId = NodeId>>(dom: &D, node: NodeId) -> Option<u64> {
    if dom.kind(node) != NodeKind::Element
        || !matches!(
            dom.element_name(node)?.local.as_ref(),
            "custom-leaf" | "chisel-leaf"
        )
    {
        return None;
    }
    dom.attribute(node, &Namespace::default(), &LocalName::from("key"))?
        .parse()
        .ok()
}

fn ancestor_scroll<D: LayoutDom<NodeId = NodeId>>(
    dom: &D,
    node: NodeId,
    scroll: &HashMap<NodeId, (f32, f32)>,
) -> (f32, f32) {
    let mut total = (0.0, 0.0);
    let mut current = dom.parent(node);
    while let Some(parent) = current {
        if let Some((x, y)) = scroll.get(&parent) {
            total.0 += x;
            total.1 += y;
        }
        current = dom.parent(parent);
    }
    total
}

/// The text nodes that can hold byte `offset` of `node`'s text, each with the
/// offset inside it, best first: the node the offset starts or falls inside,
/// then the node it ends. A field counts its bytes across all its text, as
/// [`node_text`] reads it, and the text frame per text node; on a boundary the
/// later node comes first, since a caret after a line break sits on the next
/// line. Empty nodes hold nothing, and an offset past the end lands at the end
/// of the last node.
fn text_positions<D: LayoutDom<NodeId = NodeId>>(
    dom: &D,
    node: NodeId,
    offset: usize,
) -> Vec<(NodeId, usize)> {
    let mut start = 0;
    let (mut inside, mut ending, mut last) = (None, None, None);
    walk(dom, node, &mut |descendant| {
        let Some(text) = dom.text(descendant).filter(|text| !text.is_empty()) else {
            return;
        };
        let end = start + text.len();
        if inside.is_none() && (start..end).contains(&offset) {
            inside = Some((descendant, offset - start));
        }
        if ending.is_none() && offset == end {
            ending = Some((descendant, text.len()));
        }
        last = Some((descendant, text.len()));
        start = end;
    });
    match (inside, ending) {
        (None, None) => last.into_iter().collect(),
        (inside, ending) => inside.into_iter().chain(ending).collect(),
    }
}

/// [`text_positions`] read backwards: byte `offset` of the text node
/// `text_node`, counted across all of `node`'s text. `None` when `text_node`
/// is not `node`'s.
fn element_offset<D: LayoutDom<NodeId = NodeId>>(
    dom: &D,
    node: NodeId,
    text_node: NodeId,
    offset: usize,
) -> Option<usize> {
    let (mut before, mut found) = (0, false);
    walk(dom, node, &mut |descendant| {
        if found {
            return;
        }
        if descendant == text_node {
            found = true;
        } else if let Some(text) = dom.text(descendant) {
            before += text.len();
        }
    });
    found.then_some(before + offset)
}

fn node_text<D: LayoutDom<NodeId = NodeId>>(dom: &D, node: NodeId) -> String {
    if let Some(text) = dom.text(node) {
        return text.to_owned();
    }
    let mut text = String::new();
    for child in dom.dom_children(node) {
        text.push_str(&node_text(dom, child));
    }
    text
}

fn previous_boundary(text: &str, byte: usize) -> usize {
    text[..byte]
        .char_indices()
        .next_back()
        .map_or(0, |(index, _)| index)
}

fn next_boundary(text: &str, byte: usize) -> usize {
    text[byte..]
        .char_indices()
        .nth(1)
        .map_or(text.len(), |(index, _)| byte + index)
}

fn previous_word(text: &str, byte: usize) -> usize {
    let prefix = &text[..byte];
    let trimmed = prefix.trim_end_matches(char::is_whitespace);
    trimmed
        .rfind(char::is_whitespace)
        .map_or(0, |index| index + 1)
}

fn next_word(text: &str, byte: usize) -> usize {
    let suffix = &text[byte..];
    let skipped = suffix
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace())
        .map_or(suffix.len(), |(index, _)| index);
    let rest = &suffix[skipped..];
    byte + skipped
        + rest
            .char_indices()
            .find(|(_, ch)| ch.is_whitespace())
            .map_or(rest.len(), |(index, _)| index)
}
