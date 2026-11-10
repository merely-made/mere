// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Rootstock-owned retained Livery session over Cambium's externally owned DOM.

use std::collections::HashMap;

use genet_livery::{
    Device, IncrementalStyle, InteractionStates, LiveryLayout, LiveryPaintList, StylePlane,
    StyleSet, TextPaintPhase, TextRange, TextSystem, ViewportSizes,
    emit_paint_list_with_text_system_scrolled_with_images, hit_test_with_scroll,
    layout_with_text_system,
};
use genet_render::{VisualAffinity, VisualCaret, VisualMovement, VisualSelection};
use genet_scripted_dom::NodeId;
use layout_dom_api::{DomMutation, LayoutDom, LocalName, Namespace, NodeKind};
use paint_list_api::{
    ColorF, CommonPlacement, DeviceIntSize, LayoutPoint, LayoutRect, LayoutSize, PaintCmd, RectItem,
};

#[cfg(test)]
mod graph_label_tests;
#[cfg(test)]
mod hidden_panel_tests;
#[cfg(test)]
mod incremental_tests;
mod interaction;
mod presented_input;
mod producer;
pub(crate) use presented_input::PresentedInput;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod text_paint_tests;

/// Host-owned focus inputs. Paint emission resolves their geometry in layout
/// coordinates; viewport-space caret APIs remain separate for IME and input.
pub(crate) type FocusedTextPaint = (NodeId, VisualCaret, Option<(usize, usize)>);

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

/// An application's vector image source at one layout scale.
#[derive(Clone)]
pub struct HostVectors {
    pub source: crate::VectorImageSource,
    pub scale: f32,
}

/// The host's image bytes and vector images as one lookup for Livery.
pub(crate) struct HostImageLookup<'a> {
    pub(crate) bytes: &'a HashMap<String, Vec<u8>>,
    pub(crate) vectors: Option<&'a HostVectors>,
}

impl genet_livery::ImageSourceLookup for HostImageLookup<'_> {
    fn vector(&self, key: &str) -> Option<std::sync::Arc<dyn genet_livery::VectorImage>> {
        let vectors = self.vectors?;
        (vectors.source)(key, vectors.scale)
    }

    fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.bytes.get(key)
    }
}

pub struct OwnedLayout {
    style_set: StyleSet,
    interaction_dependencies: interaction::Dependencies,
    device: Device,
    interactions: InteractionStates<NodeId>,
    hovered: Option<NodeId>,
    focused: Option<NodeId>,
    // The cascade result, kept across frames and updated from the drained DOM
    // mutations. Layout adjusts a copy of it into `styles`.
    style: IncrementalStyle<NodeId>,
    styles: StylePlane<NodeId>,
    fragments: LiveryLayout<NodeId>,
    text: TextSystem,
    font_registration_errors: Vec<genet_livery::GenericFontError>,
    viewport: (f32, f32),
    viewport_scroll: (f32, f32),
    element_scroll: HashMap<NodeId, (f32, f32)>,
    content_extent: (f32, f32),
    /// Host-supplied image bytes by URL. Layout resolves intrinsic sizes and
    /// paint decodes textures out of the same ledger, so it is retained here.
    images: HashMap<String, Vec<u8>>,
    /// The application's vector images and the scale they draw at.
    vectors: Option<HostVectors>,
    style_resolve_us: u64,
    layout_with_text_us: u64,
    content_extent_us: u64,
    generation: u64,
    input_lifetime: std::sync::Arc<()>,
}

impl OwnedLayout {
    pub(crate) fn custom_leaf_boxes<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
    ) -> Vec<(u64, (f32, f32))> {
        let mut boxes = Vec::new();
        self.walk_painted_geometry(dom, |node, geometry| {
            if geometry.placement_visible
                && let Some(key) = custom_leaf_key(dom, node)
                && let Some((_, _, width, height)) = geometry.content_rect
            {
                boxes.push((key, (width, height)));
            }
        });
        boxes
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

    /// Visit translated semantic border geometry and retained style facts in
    /// this DOM's flat-tree order. Genet owns ancestor placement; this host
    /// subtracts its viewport scroll once. No per-read cache survives this call.
    ///
    /// This follows the shared translation-only host convention. Full affine
    /// transforms and clipping remain the `element_geometry` contract.
    pub(crate) fn walk_painted_geometry<'a, D: LayoutDom<NodeId = NodeId>>(
        &'a self,
        dom: &D,
        mut visit: impl FnMut(NodeId, genet_livery::TranslatedGeometry<'a>),
    ) {
        genet_livery::walk_translated_geometry(
            dom,
            &self.styles,
            &self.fragments,
            &self.element_scroll,
            |node, mut geometry| {
                if let Some(rect) = &mut geometry.border_rect {
                    rect.0 -= self.viewport_scroll.0;
                    rect.1 -= self.viewport_scroll.1;
                }
                if let Some(rect) = &mut geometry.content_rect {
                    rect.0 -= self.viewport_scroll.0;
                    rect.1 -= self.viewport_scroll.1;
                }
                visit(node, geometry);
            },
        );
    }

    /// The part of `node` a reader can see: its painted rect cut by every
    /// ancestor that clips its content, and by the viewport. `None` when no
    /// part of it shows.
    pub fn visible_rect<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
        node: NodeId,
    ) -> Option<(f32, f32, f32, f32)> {
        let mut rect = self.painted_rect(dom, node)?;
        let mut ancestor = dom.parent(node);
        while let Some(current) = ancestor {
            if let Some(clip) = self.content_clip(dom, current) {
                rect = intersect(rect, clip)?;
            }
            ancestor = dom.parent(current);
        }
        intersect(rect, (0.0, 0.0, self.viewport.0, self.viewport.1))
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
        if !clips_content(&self.styles, node) {
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

    /// The nearest box that scrolls horizontally and has room to, `node`
    /// itself first, with its range.
    fn horizontal_scroll_container<D: LayoutDom<NodeId = NodeId>>(
        &self,
        dom: &D,
        node: NodeId,
    ) -> Option<(NodeId, f32)> {
        let mut candidate = Some(node);
        while let Some(box_node) = candidate {
            if scroll_axes(&self.styles, box_node).0 {
                let range = element_scroll_range(dom, &self.styles, &self.fragments, box_node).0;
                if range > 0.0 {
                    return Some((box_node, range));
                }
            }
            candidate = dom.parent(box_node);
        }
        None
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
}

/// Whether the node clips content, using the existing paint policy on either axis.
fn clips_content(styles: &StylePlane<NodeId>, node: NodeId) -> bool {
    let clips = |property| {
        styles
            .computed_style(node, property)
            .is_some_and(|value| value != "visible")
    };
    clips("overflow-x") || clips("overflow-y")
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

/// A computed length in pixels, `0` for anything else.
fn computed_px(styles: &StylePlane<NodeId>, node: NodeId, property: &str) -> f32 {
    styles
        .computed_style(node, property)
        .and_then(|value| value.strip_suffix("px")?.trim().parse().ok())
        .unwrap_or(0.0)
}

/// `rect` cut to `clip`, or `None` when nothing of it is left.
/// The overlap of two `(x, y, w, h)` rects; `None` when they do not overlap.
fn intersect(
    (x, y, width, height): (f32, f32, f32, f32),
    (clip_x, clip_y, clip_width, clip_height): (f32, f32, f32, f32),
) -> Option<(f32, f32, f32, f32)> {
    let left = x.max(clip_x);
    let top = y.max(clip_y);
    let right = (x + width).min(clip_x + clip_width);
    let bottom = (y + height).min(clip_y + clip_height);
    (right > left && bottom > top).then_some((left, top, right - left, bottom - top))
}

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
fn register_fonts(
    text: &mut TextSystem,
    fonts: &[crate::HostFont],
) -> Vec<genet_livery::GenericFontError> {
    let mut errors = Vec::new();
    for font in fonts {
        match font.family.as_deref() {
            Some(generic @ ("sans-serif" | "serif" | "monospace" | "system-ui")) => {
                match text.register_generic_font_bytes(font.bytes.clone(), generic) {
                    Ok(_) => {
                        // Preserve the prior quoted named-family alias as well.
                        let normal = "normal".parse().expect("`normal` font-feature-settings");
                        text.register_font_face_bytes(font.bytes.clone(), generic, &normal);
                    },
                    Err(error) => errors.push(error),
                }
            },
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
    errors
}

fn text_rect_command(rect: genet_livery::TextRect, color: ColorF) -> PaintCmd {
    PaintCmd::DrawRect(RectItem {
        placement: CommonPlacement::new(LayoutRect::from_origin_and_size(
            LayoutPoint::new(rect.x, rect.y),
            LayoutSize::new(rect.width, rect.height),
        )),
        color,
    })
}

/// Only overflow visible outside nested clip/scroll boundaries can enlarge
/// the document. A boundary's own border box still contributes, so an outer
/// scrollport positioned beyond the viewport remains document-scrollable.
fn content_extent<D: LayoutDom<NodeId = NodeId>>(
    dom: &D,
    styles: &StylePlane<NodeId>,
    fragments: &LiveryLayout<NodeId>,
) -> (f32, f32) {
    let mut extent = (0.0, 0.0);
    extend_scroll_extent(
        dom,
        styles,
        fragments,
        dom.document(),
        (0.0, 0.0),
        (true, true),
        &mut extent,
    );
    extent
}

fn elapsed_us(elapsed: crate::Duration) -> u64 {
    elapsed.as_micros().min(u64::MAX as u128) as u64
}

/// Genet owns flat-tree child selection. Preserve its order while sharing the
/// traversal between producer discovery and semantic routing. Light-DOM text
/// offsets below retain their separate DOM-text contract.
pub(crate) fn flat_nodes<D: LayoutDom<NodeId = NodeId>>(
    dom: &D,
) -> impl Iterator<Item = NodeId> + '_ {
    let mut pending = vec![dom.document()];
    std::iter::from_fn(move || {
        let node = pending.pop()?;
        let start = pending.len();
        pending.extend(dom.flat_children(node));
        pending[start..].reverse();
        Some(node)
    })
}

fn walk<D: LayoutDom<NodeId = NodeId>>(dom: &D, node: NodeId, visit: &mut impl FnMut(NodeId)) {
    visit(node);
    for child in dom.dom_children(node) {
        walk(dom, child, visit);
    }
}

pub(crate) fn custom_leaf_key<D: LayoutDom<NodeId = NodeId>>(dom: &D, node: NodeId) -> Option<u64> {
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

#[path = "owned_layout/text_geometry.rs"]
mod text_geometry;

#[path = "owned_layout/scrolling.rs"]
mod scrolling;

use scrolling::{element_scroll_range, extend_scroll_extent};

#[path = "owned_layout/painting.rs"]
mod painting;

#[path = "owned_layout/construction.rs"]
mod construction;
