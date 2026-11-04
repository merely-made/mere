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
    /// Route a focus change through the retained view before the caller's
    /// publication rebuild. Lost is delivered first, then Gained. Each is a
    /// `blur` or `focus` event walked through the layer to its node; neither
    /// bubbles, so only that node's own observer (and any other owner's listeners
    /// there) hears it.
    pub(super) fn route_focus_transition(
        &mut self,
        state: &mut State,
        old: Option<NodeId>,
        new: Option<NodeId>,
    ) -> (Vec<Action>, bool) {
        if old == new {
            return (Vec::new(), false);
        }
        let mut actions = Vec::new();
        let mut routed = false;
        for (node, event_type, phase) in [
            (old, "blur", FocusPhase::Lost),
            (new, "focus", FocusPhase::Gained),
        ] {
            let Some(node) = node else {
                continue;
            };
            let walk = self.walk(state, node, event_type, false, None, &|| {
                DynMessage::new(FocusEvent::new(phase))
            });
            routed |= walk.invoked > 0;
            actions.extend(walk.actions);
        }
        (actions, routed)
    }

    pub(crate) fn set_focus_exits(&mut self, exits: bool) {
        self.focus_exits = exits;
        if !exits {
            self.focus_exit = None;
        }
    }

    pub(crate) fn take_focus_exit(&mut self) -> Option<FocusExit> {
        self.focus_exit.take()
    }

    pub(crate) fn focus(&self) -> Option<NodeId> {
        self.focus
    }

    pub(crate) fn set_focus(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        node: Option<NodeId>,
    ) -> Vec<Action> {
        let next = node.filter(|&node| self.node_is_admitted(node));
        let old = self.focus;
        self.focus = next;
        if old != next {
            self.prepared_ime_commit = None;
            self.ime_preedit = None;
        }
        let (actions, _) = self.route_focus_transition(state, old, next);
        if old != next {
            self.rebuild(logic, state);
        }
        actions
    }

    /// Every focusable element, in document order.
    pub(crate) fn focusables(&self) -> Vec<NodeId> {
        let dom = self.dom.borrow();
        let mut out = Vec::new();
        collect_focusables(&dom, &self.ctx, self.mount, &mut out);
        out
    }

    /// Move focus to the next (`forward`) or previous focusable element in
    /// document order. The Tab-traversal default: a document engine has no
    /// built-in tab order, so the runner provides one over the focusable set
    /// (elements carrying a key handler, per [`GenetCtx::is_focusable`]) in DOM
    /// pre-order. With nothing focused, `forward` focuses the first focusable and
    /// backward the last. Rebuilds after (focus may drive `:focus` styling
    /// later).
    ///
    /// Past the last focusable (forward) or the first (backward), traversal
    /// wraps, or, with focus exits on, clears focus and records the exit for
    /// the host. With no focusable elements it is a no-op, or an exit.
    pub(crate) fn focus_traverse(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        forward: bool,
    ) {
        let exit = if forward {
            FocusExit::Forward
        } else {
            FocusExit::Backward
        };
        let focusables: Vec<NodeId> = self.focusables();
        if focusables.is_empty() {
            if self.focus_exits {
                self.focus_exit = Some(exit);
            }
            return;
        }
        let next = match self
            .focus
            .and_then(|f| focusables.iter().position(|&n| n == f))
        {
            Some(i) => {
                let len = focusables.len();
                let at_edge = if forward { i + 1 == len } else { i == 0 };
                if self.focus_exits && at_edge {
                    self.focus_exit = Some(exit);
                    self.set_focus(logic, state, None);
                    return;
                }
                if forward {
                    (i + 1) % len
                } else {
                    (i + len - 1) % len
                }
            },
            None => {
                if forward {
                    0
                } else {
                    focusables.len() - 1
                }
            },
        };
        self.set_focus(logic, state, Some(focusables[next]));
    }

    /// Dispatch a native key event to the focused node, then apply the
    /// runner-level keyboard defaults (Tab traversal, Enter/Space activation).
    /// See [`GenetAppRunner::dispatch_key`] for the full contract.
    pub(crate) fn dispatch_key(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        event: KeyEvent,
    ) -> Vec<Action> {
        // A fresh pass: the routing tail records the real cancellation value; a
        // pass that routes nothing leaves it false.
        self.last_default_prevented = false;
        event.prop.take_tab_focus();
        event.prop.set_key_target(None);
        self.scrub_dead_interaction_handles();
        let focus_request_serial = self.focus_request_serial;

        // With nothing focused, the only key carrying a runner default is Tab,
        // which enters the focusable set (no element to route to). Everything else
        // no-ops — no focused node means nowhere to send keys.
        let Some(focus) = self.focus else {
            if matches!(event.key, Key::Named(NamedKey::Tab)) {
                self.focus_traverse(logic, state, !event.mods.shift);
            }
            return Vec::new();
        };

        if let Key::Composition(CompositionEvent::Commit(text)) = &event.key
            && self
                .prepared_ime_commit
                .as_ref()
                .is_some_and(|(target, committed)| *target == focus && committed == text)
        {
            self.prepared_ime_commit = None;
            return Vec::new();
        }
        if !matches!(&event.key, Key::Composition(CompositionEvent::Commit(_))) {
            self.prepared_ime_commit = None;
        }

        // 1. Collect the ancestor chain (focus → … → document) once.
        let chain: Vec<NodeId> = {
            let dom = self.dom.borrow();
            let mut chain = Vec::new();
            let mut current = Some(focus);
            while let Some(node) = current {
                chain.push(node);
                current = dom.parent(node);
            }
            chain
        };

        // 2. Walk the key from the focused element through the event layer in
        //    propagation order (capture then bubble), exactly as `dispatch_click`.
        //    Tab is delivered here too (no longer swallowed pre-routing), so a
        //    view can handle it and `prevent_default` to override the traversal
        //    in step 3. Nothing runs for a focusable node with no `on_key` — a
        //    plain `focusable(button(..))`.
        let walk = self.walk(state, focus, "keydown", true, Some(&event.prop), &|| {
            DynMessage::new(event.clone())
        });
        let routed = walk.invoked > 0;
        let mut actions = walk.actions;
        if routed {
            match &event.key {
                Key::Composition(CompositionEvent::Preedit { text, .. }) => {
                    self.ime_preedit = Some((focus, text.clone()));
                },
                Key::Composition(CompositionEvent::Commit(_))
                    if self.preparing_ime_commit && event.prop.default_prevented() =>
                {
                    // Keep the tracked preedit live through the following
                    // rebuild. RetainedComposition must still refuse a move
                    // while a handler prevented this synthetic commit.
                },
                Key::Composition(
                    CompositionEvent::Enabled
                    | CompositionEvent::Commit(_)
                    | CompositionEvent::Disabled,
                ) => self.ime_preedit = None,
                _ => self.ime_preedit = None,
            }
        } else if !matches!(
            &event.key,
            Key::Composition(CompositionEvent::Preedit { .. })
        ) {
            self.ime_preedit = None;
        }
        if routed {
            // Record whether a handler prevented the default (gates step 3 + host).
            self.last_default_prevented = event.prop.default_prevented();
        }

        // 3. Runner-level key defaults, each suppressed when a handler prevented it
        //    (the escape hatches, G2.3).
        if !event.prop.default_prevented() {
            let activates = {
                let dom = self.dom.borrow();
                semantic_activation_matches(&dom, focus, &event.key)
            };
            match event.key {
                // Tab traverses focus across the focusable set, the runner-level
                // default a document engine otherwise lacks.
                Key::Named(NamedKey::Tab) => {
                    self.focus_traverse(logic, state, !event.mods.shift);
                    return actions; // focus_traverse rebuilt
                },
                // Enter/Space activate a focusable control that has a click handler
                // according to its native tag or ARIA role. Checkbox, switch,
                // roles use Space; buttons use Enter and Space. Radio groups
                // own their authored-widget behavior in Cambium.
                Key::Named(NamedKey::Enter | NamedKey::Space)
                    if self.ctx.has_click(focus) && !self.ctx.has_key(focus) && activates =>
                {
                    actions.extend(self.dispatch_click(
                        logic,
                        state,
                        focus,
                        PointerClick::at((0.0, 0.0)),
                    ));
                    // The key was consumed as activation: tell the host not to run
                    // its own default (e.g. Space scrolling the page).
                    self.last_default_prevented = true;
                    return actions; // dispatch_click rebuilt
                },
                _ => {},
            }
        }

        // 4. A handler ran but no runner default fired: rebuild so the handler's
        //    state change reaches the DOM. When nothing routed and no default
        //    fired, nothing changed, so the rebuild is skipped.
        if routed {
            self.rebuild(logic, state);
            let request = event.prop.take_tab_focus();
            if self.focus == Some(focus) && self.focus_request_serial == focus_request_serial {
                let next = request.and_then(|(origin, group, index)| {
                    let dom = self.dom.borrow();
                    let owned = |node| {
                        let mut current = Some(node);
                        while let Some(node) = current {
                            if !dom.is_live(node) {
                                return false;
                            }
                            if node == self.root.node {
                                return true;
                            }
                            current = dom.parent(node);
                        }
                        false
                    };
                    // The group was captured by the actual handler before diff.
                    // A nested runner or foreign host cannot contribute a seat.
                    if origin != focus
                        || !chain.contains(&origin)
                        || !owned(origin)
                        || !owned(group)
                        || dom.parent(origin) != Some(group)
                        || !self.ctx.has_key(origin)
                        || attr(&dom, origin, "role") != Some("tab")
                        || attr(&dom, group, "role") != Some("tablist")
                    {
                        return None;
                    }
                    let candidate = dom.dom_children(group).into_iter().nth(index)?;
                    (owned(candidate)
                        && self.ctx.is_focusable(candidate)
                        && self.ctx.has_key(candidate)
                        && attr(&dom, candidate, "role") == Some("tab")
                        && attr(&dom, candidate, "aria-selected") == Some("true")
                        && attr(&dom, candidate, "tabindex") == Some("0"))
                    .then_some(candidate)
                });
                if let Some(next) = next {
                    actions.extend(self.set_focus(logic, state, Some(next)));
                }
            }
        }

        actions
    }
}
