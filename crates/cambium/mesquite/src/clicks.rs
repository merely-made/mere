// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Selector clicks shared by Cambium's scenario runners.

use cambium_rootstock::{AppCtx, HostPointer, NodeId, ScrollAlign, meristem_bounds::RootView};
use taproot::Selector;

/// Holds a clipped target until the host has laid out its scroll request.
/// Pump [`after_frame`](Self::after_frame) before ticking the scenario. A
/// handled frame must not tick: queued input is dispatched after this hook.
#[derive(Default)]
pub struct Clicks {
    pending: Option<(NodeId, Selector)>,
}

impl Clicks {
    /// Match against the retained DOM. Click a fully visible target now, or
    /// ask the host to reveal it and retain its identity for the next frame.
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
        let nodes = taproot::matching(&ctx.runner.dom().borrow(), selector);
        let Some((node, rect)) = nodes.into_iter().find_map(|node| {
            ctx.painted_rect(node)
                .filter(|r| r.2 > 0.0 && r.3 > 0.0)
                .map(|rect| (node, rect))
        }) else {
            return false;
        };
        if ctx.visible_rect(node) == Some(rect) {
            let (x, y) = point(ctx, node, [rect.0, rect.1, rect.2, rect.3]);
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
        let Some((node, selector)) = self.pending.take() else {
            return Ok(false);
        };
        let rect = ctx.visible_rect(node).ok_or_else(|| {
            format!("click {selector:?}: target never came into view after scrolling")
        })?;
        let (x, y) = point(ctx, node, [rect.0, rect.1, rect.2, rect.3]);
        ctx.pointer
            .extend([HostPointer::Press(x, y), HostPointer::Release(x, y)]);
        Ok(true)
    }
}
