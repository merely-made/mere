// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl OwnedLayout {
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
    pub(super) fn caret_rect_at<D: LayoutDom<NodeId = NodeId>>(
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

    /// Keep a caret that moved in view on both axes: across its field, and
    /// down the planes that scroll vertically. The planes that moved, the
    /// horizontal one first.
    pub(crate) fn caret_into_view<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        dom: &D,
        node: NodeId,
        caret: VisualCaret,
    ) -> Vec<ScrollTarget> {
        let mut moved: Vec<_> = self
            .caret_across_into_view(dom, node, caret)
            .into_iter()
            .collect();
        moved.extend(self.caret_down_into_view(dom, node, caret));
        moved
    }

    /// Scroll the nearest box that scrolls horizontally, `node` itself first,
    /// just far enough to show the caret inside its content box: what a
    /// single-line field does as its caret moves past an edge. `None` when
    /// nothing moved, which includes a field whose text fits.
    pub(super) fn caret_across_into_view<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        dom: &D,
        node: NodeId,
        caret: VisualCaret,
    ) -> Option<ScrollTarget> {
        let rect = self.caret_rect_at(dom, node, caret.byte)?;
        let (container, range) = self.horizontal_scroll_container(dom, node)?;
        let fragment = self.fragments.get(container)?;
        let left = fragment.x
            + computed_px(&self.styles, container, "border-left-width")
            + computed_px(&self.styles, container, "padding-left");
        let right = fragment.x + fragment.width
            - computed_px(&self.styles, container, "border-right-width")
            - computed_px(&self.styles, container, "padding-right");
        // Where the scrolls between the caret and the container, the
        // container's own included, leave the caret.
        let mut shown = rect.x;
        let mut current = Some(node);
        while let Some(box_node) = current {
            shown -= self.element_scroll.get(&box_node).map_or(0.0, |s| s.0);
            if box_node == container {
                break;
            }
            current = dom.parent(box_node);
        }
        let width = rect.width.max(1.0);
        let delta = if shown < left {
            shown - left
        } else if shown + width > right {
            (shown + width - right).min(shown - left)
        } else {
            0.0
        };
        let offset = self.element_scroll.get(&container).map_or(0.0, |s| s.0);
        let next = (offset + delta).clamp(0.0, range);
        if next == offset {
            return None;
        }
        self.element_scroll.entry(container).or_default().0 = next;
        Some(ScrollTarget::Element(container))
    }

    /// Scroll just far enough to show the caret's line: `node` itself when it
    /// scrolls vertically, else the nearest ancestor that scrolls and has
    /// room to, else the document viewport, and then the planes outside it
    /// (see [`follow_out`](Self::follow_out)). This keeps typing in view in a
    /// field that grows with its text inside a scrolling pane, as well as in
    /// one that scrolls its own text.
    pub(super) fn caret_down_into_view<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        dom: &D,
        node: NodeId,
        caret: VisualCaret,
    ) -> Vec<ScrollTarget> {
        let own = scroll_axes(&self.styles, node)
            .1
            .then(|| element_scroll_range(dom, &self.styles, &self.fragments, node).1)
            .filter(|range| *range > 0.0);
        let first = match own {
            Some(range) => Some((node, range)),
            None => self.vertical_scroll_container(dom, node),
        };
        self.follow_out(dom, first, ScrollAlign::Nearest, |layout, dom| {
            let rect = layout.caret_rect_at(dom, node, caret.byte)?;
            Some((
                rect.y - layout.content_scroll(dom, node).1,
                rect.height.max(1.0),
            ))
        })
    }

    /// Reveal a byte position in retained text without changing an editor's
    /// selection. Unlike `caret_rect_at` callers, this accepts only a real
    /// UTF-8 boundary in text that has a laid-out caret rectangle; it does not
    /// use the empty-field fallback rectangle.
    pub(crate) fn scroll_text_position<D: LayoutDom<NodeId = NodeId>>(
        &mut self,
        dom: &D,
        node: NodeId,
        byte: usize,
        align: ScrollAlign,
    ) -> Vec<ScrollTarget> {
        if self.fragments.get(node).is_none() {
            return Vec::new();
        }
        let text = node_text(dom, node);
        if byte > text.len() || !text.is_char_boundary(byte) {
            return Vec::new();
        }
        let positions = text_positions(dom, node, byte);
        if positions.is_empty()
            || !positions
                .iter()
                .any(|(text_node, local)| self.fragments.caret_rect(*text_node, *local).is_some())
        {
            return Vec::new();
        }

        let own = scroll_axes(&self.styles, node)
            .1
            .then(|| element_scroll_range(dom, &self.styles, &self.fragments, node).1)
            .filter(|range| *range > 0.0);
        let first = match own {
            Some(range) => Some((node, range)),
            None => self.vertical_scroll_container(dom, node),
        };
        self.follow_out(dom, first, align, |layout, dom| {
            let rect = layout.caret_rect_at(dom, node, byte)?;
            Some((
                rect.y - layout.content_scroll(dom, node).1,
                rect.height.max(1.0),
            ))
        })
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
        self.selection_rects_at(dom, node, start, end)
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
    }

    /// Unscrolled layout geometry for insertion inside the field's CSS paint
    /// context. That context applies every ancestor clip, scroll and transform.
    pub(super) fn selection_rects_at<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
        node: NodeId,
        start: usize,
        end: usize,
    ) -> Vec<genet_livery::TextRect> {
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
            .map(|selection| selection.rects)
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
}
