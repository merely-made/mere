// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A host-owned retained runner mounted inline in a shared forest document.
//!
//! Unlike an erased child reconstructed from props, the runner remains outside
//! the projection. A slot owns only its presentation wrapper. Changing station
//! reparents the actual owner container, preserving its DOM and widget tree.
//! The owner alone dispatches input into its runner. Hosts must use window-
//! scoped layout and validate current station custody before dispatch.

use crate::{DomHandle, GenetCtx, GenetElement, attr_qual, html_qual};
use genet_scripted_dom::NodeId;
use layout_dom_api::{LayoutDom, LayoutDomMut};
use meristem::{MessageCtx, MessageResult, Mut, View, ViewMarker};
use std::{marker::PhantomData, rc::Rc};

#[derive(Clone)]
pub struct RetainedMount(Rc<Owner>);
struct Owner {
    dom: DomHandle,
    node: NodeId,
    parking: NodeId,
}

impl RetainedMount {
    /// Create a parked owner container. Construct its runner with
    /// `GenetAppRunner::new_at(dom, mount.node(), ...)`.
    pub fn new(dom: DomHandle) -> Self {
        // Genet remove releases a subtree. Withdrawal must move a live
        // runner somewhere owned, rather than remove its root. This hidden
        // parking container stays outside each window's scoped layout.
        let mut document = dom.borrow_mut();
        let parking = document.create_element(html_qual("div"));
        document.set_attribute(parking, attr_qual("class"), "cambium-retained-parking");
        document.set_attribute(parking, attr_qual("style"), "display:none;");
        let root = document.document();
        document.append_child(root, parking);
        let node = document.create_element(html_qual("div"));
        document.append_child(parking, node);
        drop(document);
        dom.borrow_mut()
            .set_attribute(node, attr_qual("class"), "cambium-retained-owner");
        dom.borrow_mut().set_attribute(
            node,
            attr_qual("style"),
            "position:relative; width:100%; height:100%; overflow:hidden;",
        );
        Self(Rc::new(Owner { dom, node, parking }))
    }
    pub fn node(&self) -> NodeId {
        self.0.node
    }
    pub fn dom(&self) -> DomHandle {
        self.0.dom.clone()
    }
    /// Withdraw only a presentation still under this source container. A
    /// destination-first move cannot be pulled out by an old window teardown.
    /// The runner's actual subtree remains parked in this shared forest.
    pub fn withdraw_from(&self, source: NodeId) -> bool {
        let mut dom = self.0.dom.borrow_mut();
        if !dom.is_live(self.node()) || !dom.is_live(source) {
            return false;
        }
        let mut parent = dom.parent(self.node());
        while let Some(node) = parent {
            if node == source {
                dom.insert_before(self.0.parking, self.node(), None);
                return true;
            }
            parent = dom.parent(node);
        }
        false
    }
    /// Mount in `Slot::View`. The owning runner remains the sole message owner.
    pub fn view<State: 'static, Action: 'static>(&self) -> RetainedMountView<State, Action> {
        RetainedMountView {
            owner: self.clone(),
            marker: PhantomData,
        }
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        let mut dom = self.dom.borrow_mut();
        if dom.is_live(self.node) {
            dom.remove(self.node);
        }
        if dom.is_live(self.parking) {
            dom.remove(self.parking);
        }
    }
}

pub struct RetainedMountView<State, Action> {
    owner: RetainedMount,
    marker: PhantomData<fn(State) -> Action>,
}
impl<State, Action> Clone for RetainedMountView<State, Action> {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
            marker: PhantomData,
        }
    }
}
impl<State, Action> ViewMarker for RetainedMountView<State, Action> {}
impl<State: 'static, Action: 'static> View<State, Action, GenetCtx>
    for RetainedMountView<State, Action>
{
    type Element = GenetElement;
    type ViewState = ();
    fn build(&self, ctx: &mut GenetCtx, _: &mut State) -> (GenetElement, ()) {
        let dom = ctx.dom();
        assert!(
            Rc::ptr_eq(&dom, &self.owner.0.dom),
            "retained mount must share the projection document"
        );
        let wrapper = dom.borrow_mut().create_element(html_qual("div"));
        dom.borrow_mut()
            .set_attribute(wrapper, attr_qual("class"), "cambium-retained-slot");
        dom.borrow_mut().set_attribute(
            wrapper,
            attr_qual("style"),
            "width:100%; height:100%; overflow:hidden;",
        );
        dom.borrow_mut()
            .insert_before(wrapper, self.owner.node(), None);
        (GenetElement::new(wrapper, dom), ())
    }
    fn rebuild(
        &self,
        prev: &Self,
        _: &mut (),
        _: &mut GenetCtx,
        element: Mut<'_, GenetElement>,
        _: &mut State,
    ) {
        assert!(
            Rc::ptr_eq(&element.dom, &self.owner.0.dom),
            "retained mount must share the projection document"
        );
        if !Rc::ptr_eq(&self.owner.0, &prev.owner.0) {
            let mut dom = element.dom.borrow_mut();
            if dom.parent(prev.owner.node()) == Some(*element.node) {
                dom.insert_before(prev.owner.0.parking, prev.owner.node(), None);
            }
            dom.insert_before(*element.node, self.owner.node(), None);
        } else if element.dom.borrow().parent(self.owner.node()) != Some(*element.node) {
            element
                .dom
                .borrow_mut()
                .insert_before(*element.node, self.owner.node(), None);
        }
    }
    fn teardown(&self, _: &mut (), _: &mut GenetCtx, element: Mut<'_, GenetElement>) {
        // A destination can mount before the old projection tears down. Only
        // detach a body still owned by this wrapper; old teardown cannot pull
        // the body out of its current destination.
        self.owner.withdraw_from(*element.node);
    }
    fn message(
        &self,
        _: &mut (),
        _: &mut MessageCtx,
        _: Mut<'_, GenetElement>,
        _: &mut State,
    ) -> MessageResult<Action> {
        MessageResult::Stale
    }
}

#[cfg(test)]
#[path = "retained_mount/tests.rs"]
mod tests;
