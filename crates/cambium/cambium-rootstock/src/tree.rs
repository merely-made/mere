// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The tree behind one window: the single-window runner, or one window's
//! projection of a multi-window runner (stack seams S24, S30). The frame and
//! input pipeline asks only this of it, so `run` and the multi-window entry
//! drive the same per-window code.

use cambium::{
    DomHandle, FileEvent, FileRequest, GenetAppRunner, HoverEvent, KeyEvent, PointerClick,
    PointerEvent, ValueEvent, WheelEvent,
};
use genet_scripted_dom::NodeId;
use layout_dom_api::{DomMutation, LayoutDom as _, LayoutDomMut as _};

use crate::meristem_bounds::RootView;

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// What the host pipeline asks of the tree behind one window.
///
/// Sealed: the host implements it for [`Runner`](crate::Runner) and for a
/// multi-window host's per-window projection. Applications use those types
/// and never implement it. Dispatch results are discarded, because the host
/// drives trees whose actions are `()`.
pub trait HostTree<State>: sealed::Sealed {
    /// The document this tree builds into.
    fn dom(&self) -> DomHandle;
    /// The tree's root element.
    fn root(&self) -> NodeId;
    /// The node the tree builds under: the document for a single window, the
    /// window-root element under a forest. Layout, paint, hit testing and
    /// accessibility see the subtree below it.
    fn mount(&self) -> NodeId;
    /// Drain the document mutations that touched this tree's subtree since it
    /// last asked.
    fn drain_mutations(&mut self, out: &mut Vec<DomMutation<NodeId>>);
    /// The application state the tree renders.
    fn state(&self) -> &State;
    /// Apply a state update and rebuild what renders it.
    fn update(&mut self, f: impl FnOnce(&mut State));
    fn focus(&self) -> Option<NodeId>;
    fn set_focus(&mut self, node: Option<NodeId>);
    fn focusables(&self) -> Vec<NodeId>;
    fn default_prevented(&self) -> bool;
    fn pointer_capture(&self) -> Option<NodeId>;
    fn pointer_target(&self, hit: NodeId) -> Option<NodeId>;
    fn wheel_target(&self, hit: NodeId) -> Option<NodeId>;
    fn hover_target(&self, hit: NodeId) -> Option<NodeId>;
    fn take_file_request(&mut self) -> Option<FileRequest>;
    fn dispatch_click(&mut self, target: NodeId, event: PointerClick);
    fn dispatch_value(&mut self, target: NodeId, event: ValueEvent);
    fn dispatch_file(&mut self, target: NodeId, event: FileEvent);
    fn dispatch_key(&mut self, event: KeyEvent);
    fn dispatch_pointer_down(&mut self, target: NodeId, event: PointerEvent);
    fn dispatch_pointer_move(&mut self, event: PointerEvent);
    fn dispatch_pointer_up(&mut self, event: PointerEvent);
    fn dispatch_hover(&mut self, target: NodeId, event: HoverEvent);
    fn dispatch_wheel(&mut self, target: NodeId, event: WheelEvent);
}

impl<State, Logic, V> sealed::Sealed for GenetAppRunner<State, Logic, V, ()>
where
    State: 'static,
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
}

impl<State, Logic, V> HostTree<State> for GenetAppRunner<State, Logic, V, ()>
where
    State: 'static,
    Logic: FnMut(&State) -> V,
    V: RootView<State>,
{
    fn dom(&self) -> DomHandle {
        GenetAppRunner::dom(self)
    }

    fn root(&self) -> NodeId {
        GenetAppRunner::root(self)
    }

    fn mount(&self) -> NodeId {
        GenetAppRunner::dom(self).borrow().document()
    }

    fn drain_mutations(&mut self, out: &mut Vec<DomMutation<NodeId>>) {
        GenetAppRunner::dom(self).borrow_mut().drain_mutations(out);
    }

    fn state(&self) -> &State {
        GenetAppRunner::state(self)
    }

    fn update(&mut self, f: impl FnOnce(&mut State)) {
        GenetAppRunner::update(self, f);
    }

    fn focus(&self) -> Option<NodeId> {
        GenetAppRunner::focus(self)
    }

    fn set_focus(&mut self, node: Option<NodeId>) {
        GenetAppRunner::set_focus(self, node);
    }

    fn focusables(&self) -> Vec<NodeId> {
        GenetAppRunner::focusables(self)
    }

    fn default_prevented(&self) -> bool {
        GenetAppRunner::default_prevented(self)
    }

    fn pointer_capture(&self) -> Option<NodeId> {
        GenetAppRunner::pointer_capture(self)
    }

    fn pointer_target(&self, hit: NodeId) -> Option<NodeId> {
        GenetAppRunner::pointer_target(self, hit)
    }

    fn wheel_target(&self, hit: NodeId) -> Option<NodeId> {
        GenetAppRunner::wheel_target(self, hit)
    }

    fn take_file_request(&mut self) -> Option<FileRequest> {
        GenetAppRunner::take_file_request(self)
    }

    fn dispatch_click(&mut self, target: NodeId, event: PointerClick) {
        GenetAppRunner::dispatch_click(self, target, event);
    }

    fn dispatch_value(&mut self, target: NodeId, event: ValueEvent) {
        GenetAppRunner::dispatch_value(self, target, event);
    }

    fn dispatch_file(&mut self, target: NodeId, event: FileEvent) {
        GenetAppRunner::dispatch_file(self, target, event);
    }

    fn dispatch_key(&mut self, event: KeyEvent) {
        GenetAppRunner::dispatch_key(self, event);
    }

    fn dispatch_pointer_down(&mut self, target: NodeId, event: PointerEvent) {
        GenetAppRunner::dispatch_pointer_down(self, target, event);
    }

    fn dispatch_pointer_move(&mut self, event: PointerEvent) {
        GenetAppRunner::dispatch_pointer_move(self, event);
    }

    fn dispatch_pointer_up(&mut self, event: PointerEvent) {
        GenetAppRunner::dispatch_pointer_up(self, event);
    }

    fn hover_target(&self, hit: NodeId) -> Option<NodeId> {
        GenetAppRunner::hover_target(self, hit)
    }

    fn dispatch_hover(&mut self, target: NodeId, event: HoverEvent) {
        GenetAppRunner::dispatch_hover(self, target, event);
    }

    fn dispatch_wheel(&mut self, target: NodeId, event: WheelEvent) {
        GenetAppRunner::dispatch_wheel(self, target, event);
    }
}
