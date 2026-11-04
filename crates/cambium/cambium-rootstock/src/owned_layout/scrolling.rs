// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl OwnedLayout {
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
                let range = element_scroll_range(dom, &self.styles, &self.fragments, node);
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

    /// Bring `node` into view through its ancestor scrollports and the document.
    /// Horizontal planes reveal the nearest edge; the first vertical plane
    /// with room moves by `align`, and outer vertical planes follow by nearest.
    /// Returns each plane that moved once, innermost first, even when both axes
    /// moved. A retired node or one without painted bounds moves no planes.
    pub(crate) fn scroll_into_view<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        dom: &D,
        node: NodeId,
        align: ScrollAlign,
    ) -> Vec<ScrollTarget> {
        // Before any walk: a retired node has no parent to ask for.
        if self.painted_rect(dom, node).is_none() {
            return Vec::new();
        }
        let mut moved = Vec::new();
        let mut vertical_align = align;
        let mut ancestor = dom.parent(node);
        while let Some(container) = ancestor {
            let axes = scroll_axes(&self.styles, container);
            if !axes.0 && !axes.1 {
                ancestor = dom.parent(container);
                continue;
            }
            let range = element_scroll_range(dom, &self.styles, &self.fragments, container);
            let mut changed = false;
            if axes.0 && range.0 > 0.0 {
                changed = self.reveal_horizontal(dom, node, Some((container, range.0)));
            }
            if axes.1 && range.1 > 0.0 {
                if let Some((_, top, _, height)) = self.painted_rect(dom, node) {
                    changed |= self
                        .move_plane(dom, Some((container, range.1)), top, height, vertical_align)
                        .is_some();
                }
                vertical_align = ScrollAlign::Nearest;
            }
            if changed {
                moved.push(ScrollTarget::Element(container));
            }
            ancestor = dom.parent(container);
        }
        let mut changed = self.reveal_horizontal(dom, node, None);
        if let Some((_, top, _, height)) = self.painted_rect(dom, node) {
            changed |= self
                .move_plane(dom, None, top, height, vertical_align)
                .is_some();
        }
        if changed {
            moved.push(ScrollTarget::Document);
        }
        moved
    }

    /// Reveal only the part that can escape inner clips. In particular, a
    /// hidden/clip box is not a scroll plane, and its invisible overflow must
    /// not cause an outer plane to chase an unreachable target. Bounds are
    /// re-read after every move, in the same painted coordinate space as the
    /// scrollport; no additional viewport or nested-scroll subtraction applies.
    pub(super) fn reveal_horizontal<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        dom: &D,
        node: NodeId,
        plane: Option<(NodeId, f32)>,
    ) -> bool {
        let Some((left, _, width, _)) = self.painted_rect(dom, node) else {
            return false;
        };
        let (mut left, mut right) = (left, left + width);
        let mut ancestor = dom.parent(node);
        while let Some(container) = ancestor {
            if plane.is_some_and(|(owner, _)| owner == container) {
                break;
            }
            if let Some((x, _, width, _)) = self.content_clip(dom, container) {
                left = left.max(x);
                right = right.min(x + width);
                if right <= left {
                    return false;
                }
            }
            ancestor = dom.parent(container);
        }
        let (area_left, area_width, current, range) = match plane {
            Some((container, range)) => {
                let Some((x, _, width, _)) = self
                    .content_clip(dom, container)
                    .or_else(|| self.painted_rect(dom, container))
                else {
                    return false;
                };
                (
                    x,
                    width,
                    self.element_scroll.get(&container).map_or(0.0, |s| s.0),
                    range,
                )
            },
            None => (
                0.0,
                self.viewport.0,
                self.viewport_scroll.0,
                (self.content_extent.0 - self.viewport.0).max(0.0),
            ),
        };
        let to_left = left - area_left;
        let to_right = right - (area_left + area_width);
        // A target covering both edges already has its nearest visible part
        // on screen. Leave it there rather than alternating between edges.
        let delta = if to_left < 0.0 && to_right > 0.0 {
            0.0
        } else if to_left < 0.0 {
            to_left
        } else if to_right > 0.0 {
            to_right.min(to_left)
        } else {
            0.0
        };
        let next = (current + delta).clamp(0.0, range);
        if next == current {
            return false;
        }
        match plane {
            Some((container, _)) => self.element_scroll.entry(container).or_default().0 = next,
            None => self.viewport_scroll.0 = next,
        }
        true
    }

    /// Walk the vertical planes out from `first`, a scroller or else the
    /// document viewport: `first` moves by `align` to show the span `target`
    /// reports (its painted top and height), then each plane outside it, out
    /// to the viewport, moves only as far as the span needs, so a span in a
    /// scroller that is itself out of sight comes into view. The span is read
    /// again after every move. The planes that moved, innermost first.
    pub(super) fn follow_out<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        dom: &D,
        first: Option<(NodeId, f32)>,
        align: ScrollAlign,
        target: impl Fn(&Self, &D) -> Option<(f32, f32)>,
    ) -> Vec<ScrollTarget> {
        let mut moved = Vec::new();
        let (mut plane, mut align) = (first, align);
        loop {
            let Some((top, height)) = target(self, dom) else {
                return moved;
            };
            moved.extend(self.move_plane(dom, plane, top, height, align));
            let Some((container, _)) = plane else {
                return moved;
            };
            plane = self.vertical_scroll_container(dom, container);
            align = ScrollAlign::Nearest;
        }
    }

    /// Move one plane, `container` or else the document viewport, so the span
    /// from `top` down `height` sits in its scrollport by `align`, clamped to
    /// the plane's range. A container's scrollport is its padding box, inside
    /// its border. `None` when the plane did not move.
    pub(super) fn move_plane<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        dom: &D,
        container: Option<(NodeId, f32)>,
        top: f32,
        height: f32,
        align: ScrollAlign,
    ) -> Option<ScrollTarget> {
        let (area_top, area_height, current, range) = match container {
            Some((container, range)) => {
                let (_, y, _, h) = self
                    .content_clip(dom, container)
                    .or_else(|| self.painted_rect(dom, container))?;
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
    pub(super) fn vertical_scroll_container<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
        node: NodeId,
    ) -> Option<(NodeId, f32)> {
        let mut candidate = dom.parent(node);
        while let Some(ancestor) = candidate {
            if scroll_axes(&self.styles, ancestor).1 {
                let range = element_scroll_range(dom, &self.styles, &self.fragments, ancestor).1;
                if range > 0.0 {
                    return Some((ancestor, range));
                }
            }
            candidate = dom.parent(ancestor);
        }
        None
    }

    pub(super) fn clamp_viewport_scroll(&mut self) {
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
    pub(super) fn clamp_element_scroll<D: LayoutDom<NodeId = NodeId>>(&mut self, dom: &D) {
        let (styles, fragments) = (&self.styles, &self.fragments);
        self.element_scroll.retain(|&node, offset| {
            let (scrolls_x, scrolls_y) = scroll_axes(styles, node);
            if !scrolls_x && !scrolls_y {
                return false;
            }
            let range = element_scroll_range(dom, styles, fragments, node);
            offset.0 = offset.0.clamp(0.0, range.0);
            offset.1 = offset.1.clamp(0.0, range.1);
            true
        });
    }
}

/// How far `node` can scroll per axis. As in CSS, its scrollable overflow
/// ends past its content by its end padding and border, so content scrolled
/// fully in stops at the content edge rather than under the border; that is
/// where a single-line field's caret sits at the end of its text.
pub(super) fn element_scroll_range<D: LayoutDom<NodeId = NodeId>>(
    dom: &D,
    styles: &StylePlane<NodeId>,
    fragments: &LiveryLayout<NodeId>,
    node: NodeId,
) -> (f32, f32) {
    let Some(container) = fragments.get(node) else {
        return (0.0, 0.0);
    };
    let end = (
        computed_px(styles, node, "padding-right")
            + computed_px(styles, node, "border-right-width"),
        computed_px(styles, node, "padding-bottom")
            + computed_px(styles, node, "border-bottom-width"),
    );
    let mut extent = (
        container.x + container.width,
        container.y + container.height,
    );
    if let Some(lines) = fragments.inline_scroll_bounds(node) {
        extent.0 = extent.0.max(lines.x + lines.width + end.0);
        extent.1 = extent.1.max(lines.y + lines.height + end.1);
    }
    for child in dom.dom_children(node) {
        extend_scroll_extent(
            dom,
            styles,
            fragments,
            child,
            end,
            (true, true),
            &mut extent,
        );
    }
    (
        (extent.0 - container.x - container.width).max(0.0),
        (extent.1 - container.y - container.height).max(0.0),
    )
}

/// Descendant border boxes contribute before their clipping boundary. Lines
/// and deeper fragments contribute only on axes still visible to the caller.
/// Shared by element and document extents, in layout coordinates.
pub(super) fn extend_scroll_extent<D: LayoutDom<NodeId = NodeId>>(
    dom: &D,
    styles: &StylePlane<NodeId>,
    fragments: &LiveryLayout<NodeId>,
    node: NodeId,
    end: (f32, f32),
    axes: (bool, bool),
    extent: &mut (f32, f32),
) {
    for fragment in fragments.fragments_for_node(node) {
        if axes.0 {
            extent.0 = extent.0.max(fragment.x + fragment.width + end.0);
        }
        if axes.1 {
            extent.1 = extent.1.max(fragment.y + fragment.height + end.1);
        }
    }
    let visible = |property| {
        styles
            .computed_style(node, property)
            .is_none_or(|value| value == "visible")
    };
    let axes = (
        axes.0 && visible("overflow-x"),
        axes.1 && visible("overflow-y"),
    );
    if !axes.0 && !axes.1 {
        return;
    }
    if let Some(lines) = fragments.inline_scroll_bounds(node) {
        if axes.0 {
            extent.0 = extent.0.max(lines.x + lines.width + end.0);
        }
        if axes.1 {
            extent.1 = extent.1.max(lines.y + lines.height + end.1);
        }
    }
    for child in dom.dom_children(node) {
        extend_scroll_extent(dom, styles, fragments, child, end, axes, extent);
    }
}
