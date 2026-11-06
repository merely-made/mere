// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Input geometry belonging to a published frame, independent of pending DOM work.

use super::{
    HashMap, LayoutDom, LiveryLayout, NodeId, OwnedLayout, StylePlane, VisualAffinity, VisualCaret,
    ancestor_scroll, custom_leaf_key, element_offset,
};
use crate::{WindowDom, producer::ProducerRegistry};

mod dom;
use dom::FrozenDom;
#[cfg(test)]
mod tests;

/// A read-only copy of the DOM facts and layout used by one displayed frame.
/// Original node identities are retained; current routing must separately
/// verify that a hit is still a member of the live window before dispatch.
#[derive(Clone, Debug)]
pub(crate) struct PresentedInput {
    dom: FrozenDom,
    styles: StylePlane<NodeId>,
    fragments: LiveryLayout<NodeId>,
    viewport_scroll: (f32, f32),
    element_scroll: HashMap<NodeId, (f32, f32)>,
    source_document: NodeId,
    dom_sequence: u64,
    layout_generation: u64,
    layout_lifetime: std::sync::Arc<()>,
}

impl OwnedLayout {
    /// Capture alongside a prepared frame, while `dom` and this layout still
    /// describe it. The host promotes this copy only after successful frame
    /// publication; preparing a newer frame does not replace the displayed copy.
    pub(crate) fn presented_input(&self, dom: &WindowDom<'_>) -> PresentedInput {
        PresentedInput {
            dom: FrozenDom::capture(dom),
            styles: self.styles.clone(),
            fragments: self.fragments.clone(),
            viewport_scroll: self.viewport_scroll,
            element_scroll: self.element_scroll.clone(),
            source_document: dom.dom().document(),
            dom_sequence: dom_sequence(dom),
            layout_generation: self.generation,
            layout_lifetime: self.input_lifetime.clone(),
        }
    }
}

impl PresentedInput {
    pub(crate) fn dom_sequence(&self) -> u64 {
        self.dom_sequence
    }

    pub(crate) fn layout_generation(&self) -> u64 {
        self.layout_generation
    }

    /// Current-layout geometry is usable only when it is the exact retained
    /// session and planes captured here. A recreated session may begin at the
    /// same generation; its separate lifetime still refuses the old proof.
    pub(crate) fn matches_layout(&self, layout: &OwnedLayout) -> bool {
        std::sync::Arc::ptr_eq(&self.layout_lifetime, &layout.input_lifetime)
            && self.layout_generation == layout.generation
            && self.viewport_scroll == layout.viewport_scroll
            && self.element_scroll == layout.element_scroll
    }

    /// Conservative text/accessibility admission: even a mutation elsewhere
    /// in the source document invalidates this proof. Draining the mutation
    /// log alone does not invalidate it, and another document cannot match.
    pub(crate) fn matches_dom(&self, dom: &WindowDom<'_>) -> bool {
        self.source_document == dom.dom().document()
            && self.dom.document() == dom.document()
            && self.dom_sequence == dom_sequence(dom)
            && dom.is_live(dom.document())
    }

    pub(crate) fn hit_at(&self, x: f32, y: f32) -> Option<NodeId> {
        genet_livery::hit_test_with_scroll(
            &self.dom,
            &self.styles,
            &self.fragments,
            &self.element_scroll,
            x + self.viewport_scroll.0,
            y + self.viewport_scroll.1,
        )
    }

    pub(crate) fn local_in(
        &self,
        node: NodeId,
        cursor: (f32, f32),
    ) -> Option<((f32, f32), (f32, f32))> {
        let geometry = self.geometry(node)?;
        let point = (
            cursor.0 + self.viewport_scroll.0,
            cursor.1 + self.viewport_scroll.1,
        );
        if custom_leaf_key(&self.dom, node).is_some() {
            Some((
                geometry.map_to_local(point.0, point.1)?,
                geometry.content_size(),
            ))
        } else {
            Some((
                geometry.map_to_border_local(point.0, point.1)?,
                geometry.border_size(),
            ))
        }
    }

    pub(crate) fn admits_producer_pointer(
        &self,
        node: NodeId,
        cursor: (f32, f32),
        producers: &ProducerRegistry,
    ) -> bool {
        if !self.dom.is_live(node) {
            return false;
        }
        if custom_leaf_key(&self.dom, node)
            .filter(|key| producers.contains(*key))
            .is_none()
        {
            return true;
        }
        self.geometry(node).is_some_and(|geometry| {
            geometry
                .map_to_content(
                    cursor.0 + self.viewport_scroll.0,
                    cursor.1 + self.viewport_scroll.1,
                )
                .is_some()
        })
    }

    /// Use the published layout's retained text frames and frozen text-node
    /// offsets. This does not reshape text or read the mutable TextSystem.
    pub(crate) fn caret_position(&self, node: NodeId, x: f32, y: f32) -> Option<VisualCaret> {
        if !self.dom.is_live(node) || !x.is_finite() || !y.is_finite() {
            return None;
        }
        let (nested_x, nested_y) = ancestor_scroll(&self.dom, node, &self.element_scroll);
        let (own_x, own_y) = self.element_scroll.get(&node).copied().unwrap_or_default();
        let (text_node, byte) = self.fragments.text_position_at_point(
            x + self.viewport_scroll.0 + nested_x + own_x,
            y + self.viewport_scroll.1 + nested_y + own_y,
        )?;
        Some(VisualCaret {
            byte: element_offset(&self.dom, node, text_node, byte)?,
            affinity: VisualAffinity::Downstream,
        })
    }

    fn geometry(&self, node: NodeId) -> Option<genet_livery::ElementGeometry> {
        if !self.dom.is_live(node) {
            return None;
        }
        genet_livery::element_geometry(
            &self.dom,
            &self.styles,
            &self.fragments,
            &self.element_scroll,
            node,
        )
    }
}

fn dom_sequence(dom: &WindowDom<'_>) -> u64 {
    let (base, pending) = dom.dom().pending_mutations();
    base.saturating_add(pending.len() as u64)
}
