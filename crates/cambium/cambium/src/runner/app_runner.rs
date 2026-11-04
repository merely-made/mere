/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

impl<State, Logic, V, Action> GenetAppRunner<State, Logic, V, Action>
where
    State: 'static,
    Action: 'static,
    Logic: FnMut(&State) -> V,
    V: View<State, Action, GenetCtx, Element = GenetElement>,
{
    /// Build the initial tree from `state` and attach its root under the
    /// document root.
    ///
    /// Mirrors Cambium's test harness: run the logic to
    /// get a view, `View::build` it into the `ScriptedDom`, then
    /// `insert_before(document, root, None)` to append it under the document.
    pub fn new(dom: DomHandle, mut logic: Logic, mut state: State) -> Self {
        let tree = RunnerTree::build(dom, &mut logic, &mut state);
        Self { state, logic, tree }
    }

    /// Build beneath a caller-owned live mount in the same document.
    ///
    /// The mount may move between window subtrees without rebuilding this
    /// runner. Its state, widget tree, focus and pointer capture stay owned by
    /// this runner; the receiving host must withdraw stale window input first.
    pub fn new_at(dom: DomHandle, mount: NodeId, mut logic: Logic, mut state: State) -> Self {
        assert!(dom.borrow().is_live(mount), "runner mount must be live");
        let tree = RunnerTree::build_at(dom, mount, &mut logic, &mut state);
        Self { state, logic, tree }
    }

    /// [`new_at`](Self::new_at) whose listeners register in `events`, an event
    /// registry the host shares with another owner (a script runtime's). Both
    /// owners' listeners on one node then interleave in registration order in one
    /// walk; the other owner's run through
    /// [`set_foreign_invoker`](Self::set_foreign_invoker), and without one they are
    /// skipped. Cambium's own are [`CAMBIUM_OWNER`](crate::CAMBIUM_OWNER)'s.
    pub fn with_events(
        dom: DomHandle,
        mount: NodeId,
        events: Rc<RefCell<Registry>>,
        mut logic: Logic,
        mut state: State,
    ) -> Self {
        assert!(dom.borrow().is_live(mount), "runner mount must be live");
        let ctx = GenetCtx::with_events(dom.clone(), events);
        let tree = RunnerTree::build_in(dom, mount, ctx, &mut logic, &mut state);
        Self { state, logic, tree }
    }

    /// Run the listeners of other owners in a shared registry through
    /// `invoker`, inside the walks of [`dispatch_click`](Self::dispatch_click) and
    /// [`dispatch_key`](Self::dispatch_key) and the focus transitions, at their
    /// place in registration order. A listener the layer marks `once` is
    /// removed before it runs, so without an invoker those would be lost: install
    /// it before the first dispatch.
    pub fn set_foreign_invoker(&mut self, invoker: Option<Box<dyn Invoker>>) {
        self.tree.set_foreign_invoker(invoker);
    }

    /// The endpoint other owners' dispatchers hand [`CAMBIUM_OWNER`](crate::CAMBIUM_OWNER)
    /// calls to. See [`NativeOwner`].
    pub fn native_owner(&self) -> NativeOwner {
        self.tree.native_owner()
    }

    /// Apply a state update, then rebuild the view tree against the new state.
    ///
    /// `f` is the externally-driven "message": it mutates the owned state. The
    /// runner then reruns the logic and diffs the produced view against the
    /// retained one, emitting `DomMutation`s into the `ScriptedDom`.
    pub fn update(&mut self, f: impl FnOnce(&mut State)) {
        f(&mut self.state);
        self.tree.rebuild(&mut self.logic, &mut self.state);
    }

    /// Refresh retained listener routes after an external host changes this
    /// runner's placement path, without evaluating application logic.
    pub fn refresh_routes(&mut self) {
        self.tree.refresh_routes(&mut self.state);
    }

    /// Prepare a subtree for synchronous relocation. Call this before changing
    /// its parent or placement metadata, and forward the returned actions to
    /// the product owner. This can dispatch pointer Cancel and a synthetic IME
    /// Commit through the subtree's current retained routes.
    pub fn prepare_subtree_move(&mut self, subtree: NodeId) -> Vec<Action> {
        self.tree
            .prepare_subtree_move(&mut self.logic, &mut self.state, subtree)
    }

    /// Whether relocating `subtree` now would discard an active Cambium
    /// pointer capture or focused IME preedit. A host can use this as a
    /// fail-closed check when it cannot call [`Self::prepare_subtree_move`].
    pub fn subtree_move_has_active_interaction(&self, subtree: NodeId) -> bool {
        self.tree.subtree_move_has_active_interaction(subtree)
    }

    /// Update retained paint state without rebuilding the semantic view tree.
    ///
    /// Use for host-driven animation whose changing geometry is painted by a
    /// separately refreshed leaf. The caller must refresh that leaf and request
    /// frames while it is active. Changes affecting DOM content or hit targets
    /// still require [`Self::update`], including reconciliation after animation.
    pub fn update_deferred<R>(&mut self, f: impl FnOnce(&mut State) -> R) -> R {
        f(&mut self.state)
    }

    /// Dispatch a native pointer click that hit `target`.
    ///
    /// `target` is the node Genet's hit-test resolved the pointer to (the
    /// `point → NodeId` half lives in the host's `hit_test_node`). This is the
    /// faithful-routing dispatch: no native handler registry of `Rc<dyn Fn>`,
    /// just the `meristem` message cycle the browser path also uses.
    ///
    /// Dispatch runs the full DOM propagation order, **capture → target →
    /// bubble**; a `.capture(true)` ancestor fires before a default (bubble)
    /// descendant/target, yielding the browser/`xilem_web` ordering.
    ///
    /// Returns the `Action`s that bubbled all the way to the root — i.e. any
    /// handler action no parent [`map_action`](meristem::map_action) absorbed.
    ///
    /// After routing and rebuilding, focus is set to the nearest ancestor of
    /// `target` (including `target` itself) that carries
    /// a key handler (is focusable); if none is found, focus is cleared. So
    /// clicking a focusable element focuses it and clicking elsewhere defocuses,
    /// independent of whether any *click* handler fired.
    pub fn dispatch_click(&mut self, target: NodeId, event: PointerClick) -> Vec<Action> {
        self.tree
            .dispatch_click(&mut self.logic, &mut self.state, target, event)
    }

    /// Dispatch a finite numeric value requested by an accessibility host.
    /// Invalid values and nodes without an [`on_value`](crate::on_value) handler
    /// are ignored.
    pub fn dispatch_value(&mut self, target: NodeId, event: ValueEvent) -> Vec<Action> {
        self.tree
            .dispatch_value(&mut self.logic, &mut self.state, target, event)
    }

    /// The file request a view filed since the host last asked, if any. The
    /// host shows its chooser for it and answers with
    /// [`dispatch_file`](Self::dispatch_file).
    pub fn take_file_request(&mut self) -> Option<crate::FileRequest> {
        self.tree.take_file_request()
    }

    /// Dispatch a host's answer to a file request: the chosen files, or none
    /// when the person cancelled. A test or scenario supplies one the same way.
    pub fn dispatch_file(&mut self, target: NodeId, event: crate::FileEvent) -> Vec<Action> {
        self.tree
            .dispatch_file(&mut self.logic, &mut self.state, target, event)
    }

    /// Whether the most recent dispatch had its default action prevented by a
    /// handler (`prevent_default` on the event). The host calls this right after
    /// dispatch to decide whether to run its own default action — navigation,
    /// caret movement, form activation, drag start. This is the native side of
    /// the converged cancellation contract (the JS side returns it from
    /// `dispatchEvent`); a handler that does not prevent leaves it `false`.
    pub fn default_prevented(&self) -> bool {
        self.tree.default_prevented()
    }

    /// The currently focused node, if any.
    ///
    /// Set by [`dispatch_click`](Self::dispatch_click) (click-to-focus) or
    /// [`set_focus`](Self::set_focus); read by [`dispatch_key`](Self::dispatch_key)
    /// as the root of its bubble walk.
    pub fn focus(&self) -> Option<NodeId> {
        self.tree.focus()
    }

    /// Set (or clear, with `None`) the focused node directly.
    ///
    /// The keyboard counterpart of a programmatic `element.focus()`. No
    /// validation that a live `node` is focusable: a test (or a host) may aim
    /// focus at any live node, and [`dispatch_key`](Self::dispatch_key) simply
    /// finds no key handler to route to if it is not focusable. A retired node
    /// clears focus under the DOM dangle contract.
    pub fn set_focus(&mut self, node: Option<NodeId>) {
        self.tree.set_focus(&mut self.logic, &mut self.state, node);
    }

    /// Move focus to the next (`forward`) or previous focusable element in
    /// document order, wrapping unless [focus exits](Self::set_focus_exits)
    /// are on.
    pub fn focus_traverse(&mut self, forward: bool) {
        self.tree
            .focus_traverse(&mut self.logic, &mut self.state, forward);
    }

    /// Stop traversal at the ends of the focusable set instead of wrapping.
    ///
    /// For a host that shows this runner beside other content: a Tab past the
    /// last focusable (or a Shift-Tab before the first) clears focus and
    /// records a [`FocusExit`], which the host takes with
    /// [`take_focus_exit`](Self::take_focus_exit) and answers by moving focus
    /// to whatever comes next in its own order. Without it, Tab cycles inside
    /// the runner and a keyboard user cannot leave. Off by default, which keeps
    /// a standalone app's wrapping order. Turning it off drops a pending exit.
    pub fn set_focus_exits(&mut self, exits: bool) {
        self.tree.set_focus_exits(exits);
    }

    /// The end the latest traversal left by, once. `None` when traversal
    /// stayed inside, or when focus exits are off.
    pub fn take_focus_exit(&mut self) -> Option<FocusExit> {
        self.tree.take_focus_exit()
    }

    /// Every focusable element, in document order — the set
    /// [`focus_traverse`](Self::focus_traverse) walks.
    ///
    /// Public because document order is not the only order worth having.
    /// A host with the laid-out geometry can move focus *spatially* (arrow keys
    /// through a grid), which document order cannot express and which the view
    /// layer cannot compute, having no layout. This is the half the runner
    /// knows; the host supplies the other half.
    pub fn focusables(&self) -> Vec<NodeId> {
        self.tree.focusables()
    }

    /// Dispatch a native key event to the focused node, then apply the
    /// runner-level keyboard defaults (Tab traversal, Enter/Space activation) the
    /// model leaves to the host — each overridable by a handler that calls
    /// `prevent_default` (the G2.3 escape hatches).
    pub fn dispatch_key(&mut self, event: KeyEvent) -> Vec<Action> {
        self.tree
            .dispatch_key(&mut self.logic, &mut self.state, event)
    }

    /// A button went down on `target`. A `Down` [`PointerEvent`] is routed to
    /// the nearest ancestor of `target` (including itself) carrying an
    /// [`on_pointer`](crate::on_pointer) handler. Returns the actions that
    /// bubbled to the root.
    ///
    /// For [`PointerButton::Primary`] that element also becomes the capture:
    /// subsequent [`dispatch_pointer_move`](Self::dispatch_pointer_move) /
    /// [`dispatch_pointer_up`](Self::dispatch_pointer_up) go to it until
    /// release. For [`PointerButton::Secondary`] the press is one-shot — the
    /// capture, whatever it was, is untouched, so a right press during a drag
    /// does not steal it and a right press outside one leaves nothing for a
    /// later release to find.
    ///
    /// `event.local` / `event.size` are the press point + element box in the
    /// captured element's coordinate space; the host computes them from the
    /// laid-out rect (the headless view layer has no layout).
    pub fn dispatch_pointer_down(&mut self, target: NodeId, event: PointerEvent) -> Vec<Action> {
        self.tree
            .dispatch_pointer_down(&mut self.logic, &mut self.state, target, event)
    }

    /// Route a `Move` to the element capturing the drag (if any). No-op when no
    /// drag is active.
    pub fn dispatch_pointer_move(&mut self, event: PointerEvent) -> Vec<Action> {
        self.tree
            .dispatch_pointer_move(&mut self.logic, &mut self.state, event)
    }

    /// Route an `Up` to the capturing element and end the drag (clearing
    /// capture). No-op when no drag is active.
    pub fn dispatch_pointer_up(&mut self, event: PointerEvent) -> Vec<Action> {
        self.tree
            .dispatch_pointer_up(&mut self.logic, &mut self.state, event)
    }

    /// The element currently capturing a pointer drag, if any. The host reads
    /// this between [`dispatch_pointer_down`](Self::dispatch_pointer_down) and
    /// `up` to know which element's rect to measure for the move's local coords.
    pub fn pointer_capture_owner(&self) -> Option<(i32, PointerButton)> {
        self.tree.pointer_owner
    }
    pub fn dispatch_pointer_cancel(&mut self, mut event: PointerEvent) -> Vec<Action> {
        event.phase = crate::PointerPhase::Cancel;
        self.tree
            .dispatch_pointer_up(&mut self.logic, &mut self.state, event)
    }

    pub fn pointer_capture(&self) -> Option<NodeId> {
        self.tree.pointer_capture()
    }

    /// The nearest ancestor of `hit` (including itself) carrying an
    /// [`on_pointer`](crate::on_pointer) handler — the element a press there
    /// would capture, resolved *without* starting a drag.
    pub fn pointer_target(&self, hit: NodeId) -> Option<NodeId> {
        self.tree.pointer_target(hit)
    }

    /// Route a hover Enter, Leave, or Move event from `target` to the nearest
    /// ancestor carrying [`on_hover`](crate::on_hover).
    ///
    /// The host owns hit testing and transition detection. When its hovered
    /// node changes it calls this once for Leave on the old hit and once for
    /// Enter on the new hit; Move uses the current hit.
    pub fn dispatch_hover(&mut self, target: NodeId, event: HoverEvent) -> Vec<Action> {
        self.tree
            .dispatch_hover(&mut self.logic, &mut self.state, target, event)
    }

    /// The nearest ancestor of `hit` (including itself) carrying an
    /// [`on_hover`](crate::on_hover) handler.
    pub fn hover_target(&self, hit: NodeId) -> Option<NodeId> {
        self.tree.hover_target(hit)
    }

    /// Route a wheel/scroll notch to the nearest ancestor of `target` (including
    /// itself) carrying an [`on_wheel`](crate::on_wheel) handler. Unlike a
    /// pointer drag there is no capture: each notch resolves its own target by
    /// the ancestor walk. Returns the actions that bubbled to the root.
    pub fn dispatch_wheel(&mut self, target: NodeId, event: WheelEvent) -> Vec<Action> {
        self.tree
            .dispatch_wheel(&mut self.logic, &mut self.state, target, event)
    }

    /// The nearest ancestor of `hit` (including itself) carrying an
    /// [`on_wheel`](crate::on_wheel) handler — the element a wheel there would
    /// scroll, resolved *without* dispatching.
    pub fn wheel_target(&self, hit: NodeId) -> Option<NodeId> {
        self.tree.wheel_target(hit)
    }

    /// The shared document handle the runner mutates.
    pub fn dom(&self) -> DomHandle {
        self.tree.dom()
    }

    /// The DOM node of the current root element (attached under the document
    /// root). Stable across `update`s unless a view swaps the root node.
    pub fn root(&self) -> NodeId {
        self.tree.root()
    }

    /// The current app state.
    pub fn state(&self) -> &State {
        &self.state
    }
}
