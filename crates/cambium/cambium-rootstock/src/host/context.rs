// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl<State, Logic, V, T> AppCtx<'_, State, Logic, V, T>
where
    T: HostTree<State>,
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    /// Materialized font instances and explicit generic registration refusals
    /// from the actual retained host layout, suitable for source-bound receipts.
    pub fn retained_font_count(&self) -> usize {
        self.layout.map_or(0, |layout| layout.retained_font_count())
    }
    pub fn font_registration_error_count(&self) -> usize {
        self.layout
            .map_or(0, |layout| layout.font_registration_errors().len())
    }

    /// Where a retained DOM node actually paints, in the logical coordinate
    /// space used by [`HostPointer`]: `(x, y, width, height)`.
    ///
    /// `None` means no layout exists yet or the node does not paint. This is a
    /// read-only observation of host-owned geometry, not a second layout path.
    pub fn painted_rect(&self, node: NodeId) -> Option<(f32, f32, f32, f32)> {
        let layout = self.layout?;
        let dom = self.runner.dom();
        let dom = dom.borrow();
        let view = crate::WindowDom::new(&dom, self.runner.mount());
        layout.painted_rect(&view, node)
    }

    /// The part of `node` inside the viewport and every ancestor content clip,
    /// in logical pointer coordinates. `None` when no part is visible.
    pub fn visible_rect(&self, node: NodeId) -> Option<(f32, f32, f32, f32)> {
        let layout = self.layout?;
        let dom = self.runner.dom();
        let dom = dom.borrow();
        let view = crate::WindowDom::new(&dom, self.runner.mount());
        layout.visible_rect(&view, node)
    }

    /// Current document scroll offset in logical coordinates. This observes
    /// the host-owned scroll plane; no layout yet is distinct from zero.
    pub fn document_scroll(&self) -> Option<(f32, f32)> {
        self.layout.map(OwnedLayout::viewport_scroll)
    }

    /// Observe a rendered element's own scroll offset. A rendered element at
    /// rest returns zero; an absent layout or retired element is unavailable.
    /// Hosts can compare this cheap observation before capturing a semantic
    /// reading anchor, including changes from the scrollbar or caret follow.
    pub fn element_scroll(&self, node: NodeId) -> Option<(f32, f32)> {
        let layout = self.layout?;
        let dom = self.runner.dom();
        let dom = dom.borrow();
        let view = crate::WindowDom::new(&dom, self.runner.mount());
        layout.painted_rect(&view, node)?;
        Some(
            layout
                .element_scroll()
                .get(&node)
                .copied()
                .unwrap_or_default(),
        )
    }

    /// A computed value from the retained layout's own style set. No layout
    /// or absent property is unavailable, rather than a synthesized default.
    /// This observes an existing layout and never triggers style or layout work.
    pub fn painted_style_value(&self, node: NodeId, property: &str) -> Option<String> {
        self.layout?.computed_value(node, property)
    }

    /// The same retained hit test used by this host's input routing, in logical
    /// pointer coordinates. This does not dispatch input or change focus/capture.
    pub fn painted_hit(&self, x: f32, y: f32) -> Option<NodeId> {
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        let layout = self.layout?;
        let dom = self.runner.dom();
        let dom = dom.borrow();
        let view = crate::WindowDom::new(&dom, self.runner.mount());
        layout.hit_test(&view, x, y)
    }

    /// Read the UTF-8 byte position in `node`'s laid-out text at a logical
    /// viewport point. This does not alter focus, caret, or selection.
    pub fn text_position_at(&self, node: NodeId, point: (f32, f32)) -> Option<usize> {
        if !point.0.is_finite() || !point.1.is_finite() {
            return None;
        }
        let layout = self.layout?;
        let dom = self.runner.dom();
        let dom = dom.borrow();
        let view = crate::WindowDom::new(&dom, self.runner.mount());
        layout
            .caret_position_at_point(&view, node, point.0, point.1)
            .map(|caret| caret.byte)
    }

    /// Names and roles from the same retained document projection the host
    /// exposes to accessibility. No layout means no semantic observation yet.
    /// This observes the current DOM and layout without computing another layout
    /// or inferring names for nodes absent from the projection.
    /// Custom leaf contributions are separate; this is not an immutable,
    /// presentation-revision-qualified frame snapshot.
    pub fn a11y_projection(&self) -> Option<crate::DocumentA11yProjection> {
        use layout_dom_api::LayoutDom as _;
        let layout = self.layout?;
        let dom = self.runner.dom();
        let dom = dom.borrow();
        let view = crate::WindowDom::new(&dom, self.runner.mount());
        let focus = self.runner.focus().map(|node| view.opaque_id(node));
        Some(crate::document_projection(&view, layout, focus))
    }

    /// Queue a semantic action through the same current-target admission as
    /// platform accessibility. The host drains it after queued pointer input;
    /// observe the returned routing acknowledgement before claiming delivery.
    /// The queue binds the current host, document, mount and producer
    /// registration, leaf insertion and any authored producer child token.
    /// Existing publication provenance is preserved. Child keys must either
    /// never recur or carry a token assigned by the model, not semantic reads.
    pub fn queue_a11y(&mut self, request: crate::A11yRequest) -> crate::A11yReceipt {
        let (queued, receipt) = crate::QueuedA11y::new(
            request,
            self.a11y_owner,
            &self.runner.dom(),
            self.runner.mount(),
            self.producers,
            self.leaves,
        );
        if self.a11y_owner.get() {
            self.a11y.push(queued);
        }
        // Dropping an unqueued request acknowledges a disposed owner.
        receipt
    }

    /// Latest publisher observation. It does not invoke contributors or certify actor work.
    pub fn a11y_publication(&self) -> crate::A11yPublicationSnapshot {
        self.a11y_publication
    }

    /// Current contributed semantics, using the same composition as native and
    /// browser publication. This shared-borrow read performs no layout, paint,
    /// target allocation or action dispatch. It is not a presentation lease or
    /// an atomic read/act transaction.
    pub fn semantic_projection(&self) -> Option<crate::SemanticProjection> {
        use layout_dom_api::LayoutDom as _;
        let layout = self.layout?;
        let dom = self.runner.dom();
        let dom = dom.borrow();
        let view = crate::WindowDom::new(&dom, self.runner.mount());
        let focus = self.runner.focus().map(|node| view.opaque_id(node));
        Some(crate::project_semantics(
            &view,
            layout,
            self.leaves,
            self.producers,
            focus,
        ))
    }

    /// Ask the host to scroll `node` into view.
    ///
    /// Resolved on the host's next layout against the node's painted rect
    /// there, so a node this dispatch created, or a request made before the
    /// first layout, is found. The nearest plane that can move (an ancestor
    /// that scrolls vertically and has room to, otherwise the window viewport)
    /// moves by `align`; each plane outside it, out to the window, then moves
    /// only as far as the node needs. A container is measured by its
    /// scrollport, inside its border, and each plane is clamped to its range.
    /// Caret follow walks the same planes. Vertical only. A node that no
    /// longer exists or does not paint is a no-op; requests queued together
    /// resolve in order.
    pub fn scroll_into_view(&mut self, node: NodeId, align: ScrollAlign) {
        self.scroll
            .push(ScrollRequest::IntoView(ScrollIntoView { node, align }));
    }

    /// Queue a text byte position to become visible after the next layout.
    /// The host scrolls the retained text geometry and never touches an
    /// editor's caret or selection state.
    pub fn scroll_text_position(&mut self, node: NodeId, byte: usize, align: ScrollAlign) {
        self.scroll.push(ScrollRequest::Text { node, byte, align });
    }
}
