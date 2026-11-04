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
    /// Build the initial tree from `state` and attach its root under the
    /// document root of `dom` (the ordinary single-tree / N-doms path).
    pub(crate) fn build(
        dom: DomHandle,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
    ) -> Self {
        let doc_root = dom.borrow().document();
        Self::build_at(dom, doc_root, logic, state)
    }

    /// Build the initial tree and attach its root under `mount` — the
    /// **forest-dom** path (F1: a runner tree mounts at a node, not the
    /// document root), so several projections can share one document as
    /// sibling window-root subtrees. `build` is the `mount == document()`
    /// case. The retained root, rebuild, and teardown all stay scoped to
    /// `mount`, so a mutation or teardown in one window never reaches another.
    pub(crate) fn build_at(
        dom: DomHandle,
        mount: NodeId,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
    ) -> Self {
        let ctx = GenetCtx::new(dom.clone());
        Self::build_in(dom, mount, ctx, logic, state)
    }

    /// [`build_at`](Self::build_at) over a context the caller made, whose event
    /// registry a host may share with another owner.
    pub(crate) fn build_in(
        dom: DomHandle,
        mount: NodeId,
        mut ctx: GenetCtx,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
    ) -> Self {
        let view = logic(state);
        let (root, view_state) = view.build(&mut ctx, state);

        // Attach the produced root under the mount node (append).
        dom.borrow_mut().insert_before(mount, root.node, None);
        let focus = match ctx.take_focus_request() {
            Some(FocusRequest::Focus(node)) if node_is_admitted_in_dom(&dom.borrow(), node) => {
                Some(node)
            },
            _ => None,
        };
        let file_request = ctx
            .take_file_request()
            .filter(|request| dom.borrow().is_live(request.node));
        ctx.set_interaction_snapshot(focus, None, None);

        Self {
            dom,
            ctx,
            view,
            view_state,
            root,
            mount,
            focus,
            focus_request_serial: 0,
            pointer_capture: None,
            pointer_owner: None,
            ime_preedit: None,
            preparing_ime_commit: false,
            prepared_ime_commit: None,
            last_default_prevented: false,
            focus_exits: false,
            focus_exit: None,
            file_request,
            foreign: None,
            phantom: PhantomData,
        }
    }

    /// The node this tree's root attaches under (its window-root, or the
    /// document root for a non-forest tree).
    pub(crate) fn mount(&self) -> NodeId {
        self.mount
    }

    /// Re-run the logic against the current state and diff the produced view
    /// into the retained one.
    ///
    /// The disjoint-field borrows matter: `rebuild` needs `&prev_view`,
    /// `&mut view_state`, `&mut ctx`, the root `Mut`, and `&mut state` all at
    /// once, so the fields are destructured into separate `&mut`s. The root
    /// `GenetElementMut` is constructed exactly as `Harness::rebuild` does —
    /// borrowing `root.node` so a view *could* swap the root node, and cloning
    /// the shared `dom` handle.
    pub(crate) fn rebuild(&mut self, logic: &mut impl FnMut(&State) -> V, state: &mut State) {
        self.sync_interaction_snapshot();
        let next = logic(state);

        // Disjoint borrows: each field separately so `rebuild`'s argument set
        // does not alias `self`.
        let Self {
            ctx,
            view,
            view_state,
            root,
            dom,
            mount,
            ..
        } = self;

        let mut_ref = GenetElementMut {
            node: &mut root.node,
            dom: dom.clone(),
            // The root is attached under `mount` (the document root, or a
            // window-root in the forest dom), so a type-changing `AnyView`
            // root can swap its node there via `replace_inner`.
            parent: Some(*mount),
        };
        next.rebuild(view, view_state, ctx, mut_ref, state);

        // The freshly diffed view becomes the retained `prev` for the next tick.
        *view = next;

        // Tear down any portable child parked during this rebuild and not
        // claimed within it — its key really left every portable list in this
        // tree, so it gets a real teardown and its (still-attached) node is
        // removed. Per-tree draining also confines parked elements to their
        // own dom: a cross-dom adoption is impossible by construction.
        // (moveBefore plan S5.)
        self.ctx.drain_nursery();

        // A node can remain live while an ordinary view hides or inerts an
        // ancestor. Deliver lifecycle cleanup through the retained old route
        // before the generic scrub drops those handles. The callbacks run only
        // after the root rebuild has unwound; if one routes, rebuild once to
        // publish any local-state changes.
        if self.cleanup_inadmitted_interactions(state) {
            self.rebuild(logic, state);
            return;
        }

        // Focus and pointer capture are retained NodeIds, so a rebuild that
        // replaces or removes their element can retire either handle. The DOM
        // dangle contract makes `is_live` the only safe read in that state.
        // Clear them at the publication boundary before the host can ask for
        // the next dispatch target or captured-element rect.
        self.scrub_dead_interaction_handles();
        if let Some(request) = self.ctx.take_focus_request() {
            self.focus_request_serial = self.focus_request_serial.wrapping_add(1);
            match request {
                FocusRequest::Focus(node) if self.node_is_admitted(node) => {
                    if self.focus != Some(node) {
                        self.prepared_ime_commit = None;
                        self.ime_preedit = None;
                    }
                    self.focus = Some(node);
                },
                FocusRequest::Blur(node) if self.focus == Some(node) => {
                    self.focus = None;
                    self.prepared_ime_commit = None;
                    self.ime_preedit = None;
                },
                _ => {},
            }
        }
        if let Some(request) = self.ctx.take_file_request()
            && self.node_is_live(request.node)
        {
            self.file_request = Some(request);
        }
        self.sync_interaction_snapshot();
    }

    pub(super) fn sync_interaction_snapshot(&mut self) {
        let preedit = self
            .ime_preedit
            .as_ref()
            .map(|(node, _)| *node)
            .filter(|node| self.focus == Some(*node));
        self.ctx
            .set_interaction_snapshot(self.focus, preedit, self.pointer_capture);
    }

    /// Refresh retained event routes after a host moves this subtree to a new
    /// placement path. This deliberately uses the stored view and state; it
    /// does not call the logic closure or recreate the component tree.
    pub(crate) fn refresh_routes(&mut self, state: &mut State) {
        self.sync_interaction_snapshot();
        let Self {
            ctx,
            view,
            view_state,
            root,
            dom,
            mount,
            ..
        } = self;
        let mut_ref = GenetElementMut {
            node: &mut root.node,
            dom: dom.clone(),
            parent: Some(*mount),
        };
        ctx.with_route_refresh(|ctx| {
            view.rebuild(view, view_state, ctx, mut_ref, state);
        });
        self.scrub_dead_interaction_handles();
        self.sync_interaction_snapshot();
    }

    /// Finish Cambium-owned interaction state before a host reparents a live
    /// subtree. Pointer capture receives a real Cancel, and an active focused
    /// IME preedit receives a synthetic Commit through its old retained route.
    /// This does not ask the operating system to finalize its IME candidate;
    /// hosts must still suppress a matching late native commit. A prevented
    /// synthetic Commit leaves the preedit active, so the host must check
    /// [`subtree_move_has_active_interaction`](Self::subtree_move_has_active_interaction)
    /// and refuse the relocation.
    pub(crate) fn prepare_subtree_move(
        &mut self,
        logic: &mut impl FnMut(&State) -> V,
        state: &mut State,
        subtree: NodeId,
    ) -> Vec<Action> {
        if !self.node_is_live(subtree) {
            return Vec::new();
        }
        self.sync_interaction_snapshot();
        let mut actions = Vec::new();
        if let Some((target, text)) = self.ime_preedit.clone()
            && self.focus == Some(target)
            && self.is_descendant_or_self(subtree, target)
            && self.node_is_admitted(target)
        {
            let commit = KeyEvent::new(Key::Composition(CompositionEvent::Commit(text.clone())));
            let was_preparing = self.preparing_ime_commit;
            self.preparing_ime_commit = true;
            actions.extend(self.dispatch_key(logic, state, commit.clone()));
            self.preparing_ime_commit = was_preparing;
            if commit.prop.default_prevented() {
                self.ime_preedit = Some((target, text));
                self.prepared_ime_commit = None;
            } else {
                self.ime_preedit = None;
                self.prepared_ime_commit = Some((target, text));
            }
        }
        if self
            .pointer_capture
            .is_some_and(|node| self.is_descendant_or_self(subtree, node))
        {
            let (pointer_id, button) = self.pointer_owner.unwrap_or((0, PointerButton::Primary));
            let mut cancel = PointerEvent::new(crate::PointerPhase::Cancel, (0.0, 0.0), (0.0, 0.0))
                .with_button(button);
            cancel.pointer_id = pointer_id;
            actions.extend(self.dispatch_pointer_up(logic, state, cancel));
        }
        self.sync_interaction_snapshot();
        actions
    }

    pub(crate) fn subtree_move_has_active_interaction(&self, subtree: NodeId) -> bool {
        self.pointer_capture
            .is_some_and(|node| self.is_descendant_or_self(subtree, node))
            || self.ime_preedit.as_ref().is_some_and(|(node, _)| {
                self.focus == Some(*node) && self.is_descendant_or_self(subtree, *node)
            })
    }

    pub(super) fn is_descendant_or_self(&self, ancestor: NodeId, node: NodeId) -> bool {
        let dom = self.dom.borrow();
        let mut current = Some(node);
        while let Some(candidate) = current {
            if candidate == ancestor {
                return true;
            }
            current = dom.parent(candidate);
        }
        false
    }

    pub(super) fn node_is_live(&self, node: NodeId) -> bool {
        self.dom.borrow().is_live(node)
    }

    pub(super) fn node_is_admitted(&self, node: NodeId) -> bool {
        node_is_admitted_in_dom(&self.dom.borrow(), node)
    }

    pub(super) fn cleanup_inadmitted_interactions(&mut self, state: &mut State) -> bool {
        let mut routed = false;
        if let Some(node) = self.focus
            && self.node_is_live(node)
            && !self.node_is_admitted(node)
        {
            self.focus = None;
            self.prepared_ime_commit = None;
            self.ime_preedit = None;
            let (_, focus_routed) = self.route_focus_transition(state, Some(node), None);
            routed |= focus_routed;
        }
        if let Some(node) = self.pointer_capture
            && self.node_is_live(node)
            && !self.node_is_admitted(node)
        {
            let (pointer_id, button) = self.pointer_owner.unwrap_or((0, PointerButton::Primary));
            // Clear the old handles before invoking user code so a cleanup
            // callback cannot cause this publication boundary to cancel twice.
            self.pointer_capture = None;
            self.pointer_owner = None;
            routed |= self.route_pointer_without_rebuild(
                state,
                node,
                PointerEvent::new(crate::PointerPhase::Cancel, (0.0, 0.0), (0.0, 0.0))
                    .with_button(button)
                    .with_pointer_id(pointer_id),
            );
        }
        self.sync_interaction_snapshot();
        routed
    }

    pub(super) fn scrub_dead_interaction_handles(&mut self) {
        let dom = self.dom.borrow();
        if self
            .focus
            .is_some_and(|node| !dom.is_live(node) || !node_is_admitted_in_dom(&dom, node))
        {
            self.focus = None;
            self.prepared_ime_commit = None;
            self.ime_preedit = None;
        }
        if self
            .pointer_capture
            .is_some_and(|node| !dom.is_live(node) || !node_is_admitted_in_dom(&dom, node))
        {
            self.pointer_capture = None;
            self.pointer_owner = None;
        }
    }

    /// Tear the whole tree down (a projection being removed): run the retained
    /// view's teardown, then remove the root node from the document.
    pub(crate) fn teardown(mut self) {
        let Self {
            ctx,
            view,
            view_state,
            root,
            dom,
            mount,
            ..
        } = &mut self;
        let mut_ref = GenetElementMut {
            node: &mut root.node,
            dom: dom.clone(),
            parent: Some(*mount),
        };
        view.teardown(view_state, ctx, mut_ref);
        let root_node = root.node;
        dom.borrow_mut().remove(root_node);
        // A teardown can park portable children; nothing can ever claim them
        // (the tree is gone), so drain immediately.
        ctx.drain_nursery();
    }
}
