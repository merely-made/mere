// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One state, N windows (stack seams S24, S28, S29): a forest document with a
//! window-root per window, each window driving the same per-window pipeline as
//! a single-window host.
//!
//! The windows take turns. A window's turn borrows what the application
//! shares, the multi-window runner and the hooks, so the pipeline runs on
//! plain borrows exactly as it does for one window; the turn hands them back
//! when it ends.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use cambium::{
    DomHandle, FileEvent, FileRequest, GenetMultiRunner, HoverEvent, KeyEvent, PointerClick,
    PointerEvent, ProjectionId, ValueEvent, WheelEvent,
};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{DomMutation, LayoutDom as _, LayoutDomMut as _};

use crate::meristem_bounds::RootView;
use crate::tree::sealed;
use crate::{AppShared, Host, HostHooks, HostOptions, HostTree};

/// The runner every window shares: one state, one window projection each.
pub type MultiRunner<State, Logic, V> = GenetMultiRunner<State, Logic, V, ()>;

/// One window's tree during its turn: the shared runner, lent, and the window
/// it is lent to. A hook's `ctx.runner` under a multi-window host.
pub struct WindowTree<State: 'static, Logic, V>
where
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    runner: MultiRunner<State, Logic, V>,
    router: MutationRouter,
    requests: WindowRequests,
    window: ProjectionId,
}

/// What turns asked of the event source, acted on when they end.
#[derive(Default)]
pub struct WindowRequests {
    /// Windows to open, each with its options; their projections exist.
    pub opened: Vec<(ProjectionId, HostOptions)>,
    /// Windows to close.
    pub closed: Vec<ProjectionId>,
    /// Windows to redraw: a change they show that no document mutation
    /// carries, such as a leaf's or a producer's own state.
    pub redraws: Vec<ProjectionId>,
}

impl<State, Logic, V> WindowTree<State, Logic, V>
where
    State: 'static,
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    /// The window this turn belongs to.
    pub fn window(&self) -> ProjectionId {
        self.window
    }

    /// Open a window whose view is `logic`, a lens over the shared state, with
    /// its own options. It joins the document at once; the event source gives
    /// it a native window when this turn ends.
    pub fn open(&mut self, logic: Logic, options: HostOptions) -> ProjectionId {
        let dom = HostTree::dom(self);
        let id = self.runner.push_forest_projection(dom, logic);
        self.requests.opened.push((id, options));
        id
    }

    /// Close window `id`, this one or another, when this turn ends.
    pub fn close(&mut self, id: ProjectionId) {
        if !self.requests.closed.contains(&id) {
            self.requests.closed.push(id);
        }
    }

    /// Ask window `id` to redraw when this turn ends: for a change it shows
    /// that no document mutation carries (a leaf's or producer's own state),
    /// which the host cannot see.
    pub fn redraw(&mut self, id: ProjectionId) {
        if !self.requests.redraws.contains(&id) {
            self.requests.redraws.push(id);
        }
    }

    /// The runner every window shares.
    pub fn multi(&self) -> &MultiRunner<State, Logic, V> {
        &self.runner
    }

    /// Update the shared state and rebuild only this window, for a change
    /// only this window shows.
    pub fn update_local(&mut self, f: impl FnOnce(&mut State)) {
        self.runner.update_local(self.window, f);
    }
}

impl<State, Logic, V> sealed::Sealed for WindowTree<State, Logic, V>
where
    State: 'static,
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
}

impl<State, Logic, V> HostTree<State> for WindowTree<State, Logic, V>
where
    State: 'static,
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    fn dom(&self) -> DomHandle {
        self.runner
            .dom(self.window)
            .expect("a lent tree's window is live")
    }

    fn root(&self) -> NodeId {
        self.runner
            .root(self.window)
            .expect("a lent tree's window is live")
    }

    fn mount(&self) -> NodeId {
        self.runner
            .window_root(self.window)
            .expect("a lent tree's window is live")
    }

    fn state(&self) -> &State {
        self.runner.state()
    }

    fn update(&mut self, f: impl FnOnce(&mut State)) {
        self.runner.update(f);
    }

    fn drain_mutations(&mut self, out: &mut Vec<DomMutation<NodeId>>) {
        self.router.drain(&self.runner, self.window, out);
    }

    fn focus(&self) -> Option<NodeId> {
        self.runner.focus(self.window)
    }

    fn set_focus(&mut self, node: Option<NodeId>) {
        self.runner.set_focus(self.window, node);
    }

    fn focusables(&self) -> Vec<NodeId> {
        self.runner.focusables(self.window)
    }

    fn default_prevented(&self) -> bool {
        self.runner.default_prevented(self.window)
    }

    fn pointer_capture(&self) -> Option<NodeId> {
        self.runner.pointer_capture(self.window)
    }

    fn pointer_target(&self, hit: NodeId) -> Option<NodeId> {
        self.runner.pointer_target(self.window, hit)
    }

    fn wheel_target(&self, hit: NodeId) -> Option<NodeId> {
        self.runner.wheel_target(self.window, hit)
    }

    fn take_file_request(&mut self) -> Option<FileRequest> {
        self.runner.take_file_request(self.window)
    }

    fn dispatch_click(&mut self, target: NodeId, event: PointerClick) {
        self.runner.dispatch_click(self.window, target, event);
    }

    fn dispatch_value(&mut self, target: NodeId, event: ValueEvent) {
        self.runner.dispatch_value(self.window, target, event);
    }

    fn dispatch_file(&mut self, target: NodeId, event: FileEvent) {
        self.runner.dispatch_file(self.window, target, event);
    }

    fn dispatch_key(&mut self, event: KeyEvent) {
        self.runner.dispatch_key(self.window, event);
    }

    fn dispatch_pointer_down(&mut self, target: NodeId, event: PointerEvent) {
        self.runner
            .dispatch_pointer_down(self.window, target, event);
    }

    fn dispatch_pointer_move(&mut self, event: PointerEvent) {
        self.runner.dispatch_pointer_move(self.window, event);
    }

    fn dispatch_pointer_up(&mut self, event: PointerEvent) {
        self.runner.dispatch_pointer_up(self.window, event);
    }

    fn hover_target(&self, hit: NodeId) -> Option<NodeId> {
        self.runner.hover_target(self.window, hit)
    }

    fn dispatch_hover(&mut self, target: NodeId, event: HoverEvent) {
        self.runner.dispatch_hover(self.window, target, event);
    }

    fn dispatch_wheel(&mut self, target: NodeId, event: WheelEvent) {
        self.runner.dispatch_wheel(self.window, target, event);
    }
}

/// One drain of the shared document, split by window.
///
/// Every window's relayout asks for its own mutations; the first to ask after
/// a change drains the document once and files each mutation under the window
/// whose subtree it touched. A mutation it cannot place (a node detached by a
/// later mutation in the same batch, a change at the document itself) is filed
/// under every window, so a window is never left stale; the cost of a wrong
/// guess is one extra rebuild.
#[derive(Default)]
pub(crate) struct MutationRouter {
    queues: HashMap<ProjectionId, Vec<DomMutation<NodeId>>>,
}

impl MutationRouter {
    fn drain<State, Logic, V>(
        &mut self,
        runner: &MultiRunner<State, Logic, V>,
        window: ProjectionId,
        out: &mut Vec<DomMutation<NodeId>>,
    ) where
        State: 'static,
        Logic: FnMut(&State) -> V,
        V: RootView<State>,
    {
        self.collect(runner);
        if let Some(queue) = self.queues.get_mut(&window) {
            out.append(queue);
        }
    }

    /// Drain the document and file what it held. Returns the windows that
    /// received something, which a host asks to redraw.
    pub(crate) fn collect<State, Logic, V>(
        &mut self,
        runner: &MultiRunner<State, Logic, V>,
    ) -> HashSet<ProjectionId>
    where
        State: 'static,
        Logic: FnMut(&State) -> V,
        V: RootView<State>,
    {
        let mut touched = HashSet::new();
        let Some(first) = runner.projection_ids().next() else {
            return touched;
        };
        let dom = runner.dom(first).expect("a live projection has a document");
        let mut drained = Vec::new();
        dom.borrow_mut().drain_mutations(&mut drained);
        if drained.is_empty() {
            return touched;
        }
        let roots: Vec<(ProjectionId, NodeId)> = runner
            .projection_ids()
            .filter_map(|id| runner.window_root(id).map(|root| (id, root)))
            .collect();
        let dom = dom.borrow();
        let window_of = |node: NodeId| -> Option<ProjectionId> {
            // A node retired since (a closed window's subtree) has no
            // ancestors to read, so it cannot be placed.
            let mut current = Some(node);
            while let Some(id) = current {
                if !dom.is_live(id) {
                    return None;
                }
                if let Some(&(window, _)) = roots.iter().find(|(_, root)| *root == id) {
                    return Some(window);
                }
                current = dom.parent(id);
            }
            None
        };
        for mutation in drained {
            let anchors: Vec<NodeId> = match &mutation {
                DomMutation::Inserted { parent, .. } => vec![*parent],
                DomMutation::Removed { former_parent, .. } => vec![*former_parent],
                DomMutation::AttributeChanged { node, .. }
                | DomMutation::CharacterDataChanged { node }
                | DomMutation::FormControlStateChanged { node }
                | DomMutation::FormControlInteractionStateChanged { node }
                | DomMutation::OptionStateChanged { node }
                | DomMutation::FormControlCustomValidityChanged { node }
                | DomMutation::SubtreeReplaced { node } => vec![*node],
                DomMutation::Moved {
                    from_parent,
                    to_parent,
                    ..
                } => vec![*from_parent, *to_parent],
            };
            let mut windows = Vec::new();
            let mut placed = true;
            for anchor in anchors {
                match window_of(anchor) {
                    Some(window) => windows.push(window),
                    None => placed = false,
                }
            }
            if !placed {
                windows = roots.iter().map(|(window, _)| *window).collect();
            }
            windows.sort_by_key(|window| window.0);
            windows.dedup();
            for window in windows {
                self.queues
                    .entry(window)
                    .or_default()
                    .push(mutation.clone());
                touched.insert(window);
            }
        }
        touched
    }

    /// The windows holding mutations they have not laid out yet.
    pub(crate) fn pending(&self) -> HashSet<ProjectionId> {
        self.queues
            .iter()
            .filter(|(_, queue)| !queue.is_empty())
            .map(|(window, _)| *window)
            .collect()
    }

    /// Drop a closed window's queue.
    pub(crate) fn forget(&mut self, window: ProjectionId) {
        self.queues.remove(&window);
    }
}

/// A multi-window host's per-window slot: a [`Host`] over a [`WindowTree`], or
/// an event source's wrapper around one (the winit host's, which adds the
/// native window).
pub trait WindowSlot<State: 'static, Logic, V>
where
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    fn host(&self) -> &Host<State, Logic, V, WindowTree<State, Logic, V>>;
    fn host_mut(&mut self) -> &mut Host<State, Logic, V, WindowTree<State, Logic, V>>;
}

impl<State, Logic, V> WindowSlot<State, Logic, V>
    for Host<State, Logic, V, WindowTree<State, Logic, V>>
where
    State: 'static,
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    fn host(&self) -> &Self {
        self
    }

    fn host_mut(&mut self) -> &mut Self {
        self
    }
}

/// The windows of one application over one forest document, taking turns.
///
/// Between turns the multi host keeps what every window shares: the runner and
/// its mutation router, the [`AppShared`] part of host state, and the
/// application's hooks. [`with_window`](Self::with_window) lends them to one
/// window's [`Host`] for the length of a closure, so that window runs the same
/// frame and input pipeline a single-window host runs, then takes them back.
pub struct MultiHost<
    State: 'static,
    Logic,
    V,
    W = Host<State, Logic, V, WindowTree<State, Logic, V>>,
> where
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    tree: Option<(MultiRunner<State, Logic, V>, MutationRouter, WindowRequests)>,
    shared: AppShared,
    hooks: HostHooks<State, Logic, V, WindowTree<State, Logic, V>>,
    dom: DomHandle,
    /// Index-aligned with the runner's projection slots, which are never
    /// reused: `windows[id.0]` is window `id`, `None` before it is attached
    /// and once it closed.
    windows: Vec<Option<W>>,
}

impl<State, Logic, V, W> MultiHost<State, Logic, V, W>
where
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
    W: WindowSlot<State, Logic, V>,
{
    /// A multi-window host over `state`, with no windows yet.
    pub fn new(
        state: State,
        shared: AppShared,
        hooks: HostHooks<State, Logic, V, WindowTree<State, Logic, V>>,
    ) -> Self {
        Self {
            tree: Some((
                MultiRunner::new(state),
                MutationRouter::default(),
                WindowRequests::default(),
            )),
            shared,
            hooks,
            dom: Rc::new(RefCell::new(ScriptedDom::new())),
            windows: Vec::new(),
        }
    }

    /// Open a window whose view is `logic` (its lens over the shared state)
    /// under a fresh window-root of the one document, with `window` as its
    /// host.
    pub fn open(&mut self, logic: Logic, window: W) -> ProjectionId {
        let (runner, ..) = self.tree.as_mut().expect("windows open between turns");
        let id = runner.push_forest_projection(self.dom.clone(), logic);
        self.attach(id, window);
        id
    }

    /// Give window `id`, opened from a hook, its host.
    pub fn attach(&mut self, id: ProjectionId, window: W) {
        if self.windows.len() <= id.0 {
            self.windows.resize_with(id.0 + 1, || None);
        }
        self.windows[id.0] = Some(window);
    }

    /// What hooks asked of the event source since the last call: windows to
    /// open (a host to attach to each), to close, and to redraw.
    pub fn take_requests(&mut self) -> WindowRequests {
        let (.., requests) = self.tree.as_mut().expect("asked between turns");
        std::mem::take(requests)
    }

    /// Close window `id`: tear its tree and its window-root down, retire the
    /// producers whose nodes went with it, and hand its host back to drop. The
    /// shared state and the other windows are untouched.
    pub fn close(&mut self, id: ProjectionId) -> Option<W> {
        let (runner, router, _) = self.tree.as_mut().expect("windows close between turns");
        runner.remove_projection(id);
        router.forget(id);
        let dom = self.dom.borrow();
        let renderer = self.shared.render_core.as_ref().map(|core| core.renderer());
        self.shared.producers.retire_orphans(
            |node| crate::window_dom::ancestor_or_self(&dom, node, dom.document()),
            renderer,
        );
        drop(dom);
        self.windows.get_mut(id.0).and_then(Option::take)
    }

    /// The windows whose layout predates the shared sheet: one window swapped
    /// it, and the others lay out afresh when they next draw.
    pub fn behind_on_sheet(&self) -> Vec<ProjectionId> {
        self.windows()
            .filter(|&id| {
                self.slot(id).is_some_and(|window| {
                    window.host().s.layout_sheet_generation != self.shared.sheet_generation
                })
            })
            .collect()
    }

    /// Window `id`'s slot between turns, for what the event source keeps
    /// beside the host (its native window). Host methods need a turn.
    pub fn slot(&self, id: ProjectionId) -> Option<&W> {
        self.windows.get(id.0).and_then(Option::as_ref)
    }

    /// The same, mutably.
    pub fn slot_mut(&mut self, id: ProjectionId) -> Option<&mut W> {
        self.windows.get_mut(id.0).and_then(Option::as_mut)
    }

    /// The open windows, in the order they were opened.
    pub fn windows(&self) -> impl Iterator<Item = ProjectionId> + '_ {
        self.windows
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| slot.as_ref().map(|_| ProjectionId(index)))
    }

    /// The application state every window renders.
    pub fn state(&self) -> &State {
        self.tree
            .as_ref()
            .expect("the state is read between turns")
            .0
            .state()
    }

    /// The document every window's subtree lives in.
    pub fn dom(&self) -> DomHandle {
        self.dom.clone()
    }

    /// Window `id`'s window-root element.
    pub fn window_root(&self, id: ProjectionId) -> Option<NodeId> {
        self.tree.as_ref()?.0.window_root(id)
    }

    /// Drain the document and report which windows a change reached since
    /// they last laid out, so the event source asks those to redraw.
    pub fn touched_windows(&mut self) -> HashSet<ProjectionId> {
        let (runner, router, _) = self.tree.as_mut().expect("asked between turns");
        router.collect(runner);
        router.pending()
    }

    /// Run `f` as window `id`'s turn: its host holds the runner, the shared
    /// part and the hooks until `f` returns. `None` for a closed window.
    pub fn with_window<R>(&mut self, id: ProjectionId, f: impl FnOnce(&mut W) -> R) -> Option<R> {
        let held_elsewhere: HashSet<u64> = self
            .windows
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != id.0)
            .filter_map(|(_, slot)| slot.as_ref())
            .flat_map(|window| window.host().s.leaf_keys.iter().copied())
            .collect();
        let window = self.windows.get_mut(id.0)?.as_mut()?;
        let (runner, router, requests) = self.tree.take().expect("turns do not nest");
        let host = window.host_mut();
        host.s.runner = Some(WindowTree {
            runner,
            router,
            requests,
            window: id,
        });
        self.shared.held_elsewhere = held_elsewhere;
        std::mem::swap(&mut host.s.shared, &mut self.shared);
        std::mem::swap(&mut host.hooks, &mut self.hooks);
        let result = f(window);
        let host = window.host_mut();
        std::mem::swap(&mut host.hooks, &mut self.hooks);
        std::mem::swap(&mut host.s.shared, &mut self.shared);
        let tree = host.s.runner.take().expect("a turn hands its tree back");
        self.tree = Some((tree.runner, tree.router, tree.requests));
        Some(result)
    }
}
