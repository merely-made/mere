// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Selector clicks shared by Cambium's scenario runners.

use cambium_rootstock::{AppCtx, HostPointer, NodeId, ScrollAlign, meristem_bounds::RootView};
use taproot::{Selector, SemanticTarget};

/// Holds a clipped target until the host has laid out its scroll request.
/// Pump [`after_frame`](Self::after_frame) before ticking the scenario. A
/// handled frame must not tick: queued input is dispatched after this hook.
#[derive(Default)]
pub struct Clicks {
    pending: Option<(NodeId, Selector)>,
    produced: Option<(Selector, cambium_rootstock::A11yReceipt)>,
}

impl Clicks {
    pub(crate) fn is_pending(&self) -> bool {
        self.pending.is_some() || self.produced.is_some()
    }

    /// Match roles/names against the host's semantic projection; explicit
    /// class/text selectors keep their DOM meaning. Click a visible target or
    /// substantial visible part of a tall textbox. Otherwise reveal it and
    /// retain its identity for the next frame.
    /// `point` preserves the product's coordinate transform, if it has one.
    pub fn click<State, Logic, V>(
        &mut self,
        ctx: &mut AppCtx<'_, State, Logic, V>,
        selector: &Selector,
        point: impl FnOnce(&AppCtx<'_, State, Logic, V>, NodeId, [f32; 4]) -> (f32, f32),
    ) -> bool
    where
        State: 'static,
        Logic: FnMut(&State) -> V,
        V: RootView<State>,
    {
        if self.is_pending() {
            return false;
        }
        let Some(projection) = ctx.semantic_projection() else {
            return false;
        };
        let lookup = taproot::AccessKitLookup::new(&projection.tree, |id| projection.dom_node(id));
        let nodes = taproot::matching_prefer_exact_with_lookup(
            &ctx.runner.dom().borrow(),
            selector,
            &lookup,
        );
        // A virtual action button has no physical pointer target. Queue its
        // explicit route and require the host's acknowledgement next frame.
        if let Some(SemanticTarget::Contributed(id)) = nodes.first() {
            let Some(action) = projection.produced_action(*id) else {
                return false;
            };
            self.produced = Some((
                selector.clone(),
                ctx.queue_a11y(cambium_rootstock::A11yRequest::produced(
                    cambium_rootstock::A11yAction::Click,
                    action.clone(),
                )),
            ));
            return true;
        }
        let Some((node, rect)) = nodes.into_iter().find_map(|node| {
            let SemanticTarget::Dom(node) = node else {
                return None;
            };
            ctx.painted_rect(node)
                .filter(|r| r.2 > 0.0 && r.3 > 0.0)
                .map(|rect| (node, rect))
        }) else {
            return false;
        };
        let visible = ctx.visible_rect(node);
        let is_textbox = taproot::matching_with_lookup(
            &ctx.runner.dom().borrow(),
            &Selector::role("textbox"),
            &lookup,
        )
        .contains(&SemanticTarget::Dom(node));
        // A tall editor already has enough visible space to focus without
        // scrolling its toolbar away. Other controls retain reveal behavior.
        const USABLE_HEIGHT: f32 = 44.0;
        if let Some(visible) = visible.filter(|visible| {
            *visible == rect
                || (is_textbox
                    && rect.3 >= USABLE_HEIGHT * 2.0
                    && visible.2 >= USABLE_HEIGHT
                    && visible.3 >= USABLE_HEIGHT)
        }) {
            let (x, y) = point(ctx, node, [visible.0, visible.1, visible.2, visible.3]);
            ctx.pointer
                .extend([HostPointer::Press(x, y), HostPointer::Release(x, y)]);
        } else {
            ctx.scroll_into_view(node, ScrollAlign::Nearest);
            self.pending = Some((node, selector.clone()));
        }
        true
    }

    /// Land the held click at the visible part of the target after layout.
    /// `Ok(true)` and `Err` both consume this frame; `Ok(false)` allows the
    /// scenario to advance. An unrevealable or removed target fails once.
    pub fn after_frame<State, Logic, V>(
        &mut self,
        ctx: &mut AppCtx<'_, State, Logic, V>,
        point: impl FnOnce(&AppCtx<'_, State, Logic, V>, NodeId, [f32; 4]) -> (f32, f32),
    ) -> Result<bool, String>
    where
        State: 'static,
        Logic: FnMut(&State) -> V,
        V: RootView<State>,
    {
        if let Some((selector, receipt)) = self.produced.take() {
            return match receipt.result() {
                Some(cambium_rootstock::A11yDispatch::Dispatched) => Ok(true),
                Some(result) => Err(format!("click {selector:?}: semantic routing {result:?}")),
                None => {
                    // One host drain is required before the next presented frame.
                    Err(format!(
                        "click {selector:?}: host did not acknowledge semantic routing"
                    ))
                },
            };
        }
        let Some((node, selector)) = self.pending.take() else {
            return Ok(false);
        };
        if matches!(selector.matcher, taproot::Match::Role(_)) {
            let still_matches = ctx.semantic_projection().is_some_and(|projection| {
                let lookup =
                    taproot::AccessKitLookup::new(&projection.tree, |id| projection.dom_node(id));
                taproot::matching_prefer_exact_with_lookup(
                    &ctx.runner.dom().borrow(),
                    &selector,
                    &lookup,
                )
                .contains(&SemanticTarget::Dom(node))
            });
            if !still_matches {
                return Err(format!(
                    "click {selector:?}: held target no longer matches the semantic projection"
                ));
            }
        }
        let rect = ctx.visible_rect(node).ok_or_else(|| {
            format!("click {selector:?}: target never came into view after scrolling")
        })?;
        let (x, y) = point(ctx, node, [rect.0, rect.1, rect.2, rect.3]);
        ctx.pointer
            .extend([HostPointer::Press(x, y), HostPointer::Release(x, y)]);
        Ok(true)
    }
}
