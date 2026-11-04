/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

impl<State, V, Action> RunnerTree<State, V, Action>
where
    State: 'static,
    Action: 'static,
    V: View<State, Action, GenetCtx, Element = GenetElement>,
{
    pub(super) fn route_pointer_without_rebuild(
        &mut self,
        state: &mut State,
        node: NodeId,
        event: PointerEvent,
    ) -> bool {
        let Some(path) = self.ctx.pointer_handler(node).map(<[ViewId]>::to_vec) else {
            return false;
        };
        let Self {
            view,
            view_state,
            root,
            dom,
            mount,
            ..
        } = self;
        let mut message = MessageCtx::new(path, DynMessage::new(event));
        let element = GenetElementMut {
            node: &mut root.node,
            dom: dom.clone(),
            parent: Some(*mount),
        };
        let _ = view.message(view_state, &mut message, element, state);
        true
    }

    /// Dispatch a native pointer click that hit `target`. See
    /// [`GenetAppRunner::dispatch_click`] for the full contract; this is the
    /// per-tree implementation with state and logic threaded in.
    pub(crate) fn dispatch_click(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        target: NodeId,
        event: PointerClick,
    ) -> Vec<Action> {
        self.scrub_dead_interaction_handles();
        if !self.node_is_admitted(target) {
            self.last_default_prevented = false;
            return Vec::new();
        }

        // 1. Find the nearest focusable (key-handler-bearing or marked) ancestor
        //    for click-to-focus. Done under a short-lived shared borrow of
        //    `dom`, fully released before routing.
        let new_focus: Option<NodeId> = {
            let dom = self.dom.borrow();
            let mut current = Some(target);
            let mut focus = None;
            while let Some(node) = current {
                // The first (nearest) focusable ancestor wins; the walk runs
                // target → root, so the first hit is the deepest. Focusability
                // is phase-independent.
                if self.ctx.is_focusable(node) {
                    focus = Some(node);
                    break;
                }
                current = dom.parent(node);
            }
            focus
        };

        // Click-to-focus: focus the nearest focusable ancestor, or clear it when
        // the click landed outside any focusable element. Independent of whether
        // a click handler fired below.
        let old_focus = self.focus;
        self.focus = new_focus;
        if old_focus != new_focus {
            self.prepared_ime_commit = None;
            self.ime_preedit = None;
        }
        let (mut actions, focus_routed) = self.route_focus_transition(state, old_focus, new_focus);

        // 2. Walk the click through the event layer: capture pass first
        //    (root → target, capture listeners only), then bubble pass
        //    (target → root, the rest). A listener sits in exactly one phase,
        //    so it routes once. Any action that reaches the root (a
        //    `MessageResult::Action`) is collected; `Action = ()` collects
        //    nothing meaningful. A handler's stop_propagation ends the walk, and
        //    its prevent_default is read below.
        let walk = self.walk(state, target, "click", true, Some(&event.prop), &|| {
            DynMessage::new(event.clone())
        });
        actions.extend(walk.actions);

        // Record whether a handler prevented the default action — the host reads
        // this (default_prevented()) to gate its own default (the cancellation
        // seam's consumption point).
        self.last_default_prevented = event.prop.default_prevented();

        if walk.invoked == 0 {
            // Focus observers may still have mutated state even when the hit had
            // no click handler, so publish that transition before returning.
            if focus_routed {
                self.rebuild(logic, state);
            }
            return actions;
        }

        // 3. Rebuild so the handler's state mutation reaches the DOM — the same
        //    tail `update` runs.
        self.rebuild(logic, state);

        actions
    }

    /// A button went down on `target`. Primary begins a drag capture;
    /// secondary routes one `Down` and may explicitly request capture. See
    /// [`GenetAppRunner::dispatch_pointer_down`].
    pub(crate) fn dispatch_pointer_down(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        target: NodeId,
        event: PointerEvent,
    ) -> Vec<Action> {
        // A fresh pass: reset the cancellation flag; route_pointer records the
        // real value when a handler runs, and a no-capture press leaves it false.
        self.last_default_prevented = false;
        self.scrub_dead_interaction_handles();
        if !self.node_is_admitted(target) {
            return Vec::new();
        }
        let captured = {
            let dom = self.dom.borrow();
            let mut current = Some(target);
            let mut found = None;
            while let Some(node) = current {
                if self.ctx.pointer_handler(node).is_some() {
                    found = Some(node);
                    break;
                }
                current = dom.parent(node);
            }
            found
        };
        // Primary and auxiliary gestures capture before routing. Secondary
        // capture is opt-in after routing and cannot steal an existing gesture.
        if matches!(
            event.button,
            PointerButton::Primary | PointerButton::Auxiliary
        ) {
            if self
                .pointer_owner
                .is_some_and(|owner| owner != (event.pointer_id, event.button))
            {
                return Vec::new();
            }
            self.pointer_capture = captured;
            self.pointer_owner = captured.map(|_| (event.pointer_id, event.button));
        }
        match captured {
            Some(node) => {
                let request = event.clone();
                let actions = self.route_pointer(logic, state, node, event);
                if request.button == PointerButton::Secondary
                    && request.capture_requested()
                    && self.pointer_capture.is_none()
                    && self.node_is_admitted(node)
                    && self.pointer_target(node) == Some(node)
                {
                    self.pointer_capture = Some(node);
                    self.pointer_owner = Some((request.pointer_id, request.button));
                }
                actions
            },
            None => Vec::new(),
        }
    }

    /// Route a `Move` to the element capturing the drag (if any).
    pub(crate) fn dispatch_pointer_move(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        event: PointerEvent,
    ) -> Vec<Action> {
        self.last_default_prevented = false;
        self.scrub_dead_interaction_handles();
        if self
            .pointer_owner
            .is_some_and(|(id, _)| id != event.pointer_id)
        {
            return Vec::new();
        }
        match self.pointer_capture {
            Some(node) => self.route_pointer(logic, state, node, event),
            None => Vec::new(),
        }
    }

    /// Route an `Up` to the capturing element and end the drag.
    pub(crate) fn dispatch_pointer_up(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        event: PointerEvent,
    ) -> Vec<Action> {
        self.last_default_prevented = false;
        self.scrub_dead_interaction_handles();
        if self.pointer_owner.is_some_and(|(id, button)| {
            id != event.pointer_id
                || (event.phase != crate::PointerPhase::Cancel && button != event.button)
        }) {
            return Vec::new();
        }
        self.pointer_owner = None;
        match self.pointer_capture.take() {
            Some(node) => self.route_pointer(logic, state, node, event),
            None => Vec::new(),
        }
    }

    pub(crate) fn pointer_capture_owner(&self) -> Option<(i32, PointerButton)> {
        self.pointer_owner
    }

    pub(crate) fn pointer_capture(&self) -> Option<NodeId> {
        self.pointer_capture
    }

    /// The nearest ancestor of `hit` (including itself) carrying an
    /// [`on_pointer`](crate::on_pointer) handler.
    pub(crate) fn pointer_target(&self, hit: NodeId) -> Option<NodeId> {
        if !self.node_is_admitted(hit) {
            return None;
        }
        let dom = self.dom.borrow();
        let mut current = Some(hit);
        while let Some(node) = current {
            if self.ctx.pointer_handler(node).is_some() {
                return Some(node);
            }
            current = dom.parent(node);
        }
        None
    }

    /// Route one host-computed hover transition to the nearest registered
    /// ancestor of `target`.
    pub(crate) fn dispatch_hover(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        target: NodeId,
        event: HoverEvent,
    ) -> Vec<Action> {
        self.last_default_prevented = false;
        self.scrub_dead_interaction_handles();
        if !self.node_is_admitted(target) {
            return Vec::new();
        }
        match self.hover_target(target) {
            Some(node) => self.route_hover(logic, state, node, event),
            None => Vec::new(),
        }
    }

    /// The nearest ancestor of `hit` (including itself) carrying a hover
    /// handler.
    pub(crate) fn hover_target(&self, hit: NodeId) -> Option<NodeId> {
        if !self.node_is_admitted(hit) {
            return None;
        }
        let dom = self.dom.borrow();
        let mut current = Some(hit);
        while let Some(node) = current {
            if self.ctx.hover_handler(node).is_some() {
                return Some(node);
            }
            current = dom.parent(node);
        }
        None
    }

    /// Route a hover event down one registered view path, then rebuild.
    pub(super) fn route_hover(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        node: NodeId,
        event: HoverEvent,
    ) -> Vec<Action> {
        let Some(path) = self.ctx.hover_handler(node).map(<[ViewId]>::to_vec) else {
            return Vec::new();
        };
        let mut actions = Vec::new();
        {
            let Self {
                view,
                view_state,
                root,
                dom,
                mount,
                ..
            } = self;
            let mut message = MessageCtx::new(path, DynMessage::new(event.clone()));
            let element = GenetElementMut {
                node: &mut root.node,
                dom: dom.clone(),
                parent: Some(*mount),
            };
            if let MessageResult::Action(action) =
                view.message(view_state, &mut message, element, state)
            {
                actions.push(action);
            }
        }
        self.last_default_prevented = event.prop.default_prevented();
        if !event.rebuild_is_deferred() {
            self.rebuild(logic, state);
        }
        actions
    }

    /// Route a [`PointerEvent`] down `node`'s registered pointer path through the
    /// faithful message cycle, then rebuild.
    pub(super) fn route_pointer(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        node: NodeId,
        event: PointerEvent,
    ) -> Vec<Action> {
        let Some(path) = self.ctx.pointer_handler(node).map(<[ViewId]>::to_vec) else {
            return Vec::new();
        };
        let mut actions = Vec::new();
        {
            let Self {
                view,
                view_state,
                root,
                dom,
                mount,
                ..
            } = self;
            // Clone into the message: the handler mutates its clone's shared
            // `Propagation` cell, and the original below reads back what it set.
            let mut msg = MessageCtx::new(path, DynMessage::new(event.clone()));
            let mut_ref = GenetElementMut {
                node: &mut root.node,
                dom: dom.clone(),
                parent: Some(*mount),
            };
            if let MessageResult::Action(a) = view.message(view_state, &mut msg, mut_ref, state) {
                actions.push(a);
            }
        }
        // Record this pointer pass's own cancellation (the host gates its default
        // drag behavior on it), mirroring dispatch_click / dispatch_key — not the
        // stale value left by an earlier click/key.
        self.last_default_prevented = event.prop.default_prevented();
        if !event.rebuild_is_deferred() {
            self.rebuild(logic, state);
        }
        actions
    }
}
