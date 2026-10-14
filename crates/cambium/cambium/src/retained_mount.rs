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
}

impl RetainedMount {
    /// Create a detached owner container. Construct its runner with
    /// `GenetAppRunner::new_at(dom, mount.node(), ...)`.
    pub fn new(dom: DomHandle) -> Self {
        let node = dom.borrow_mut().create_element(html_qual("div"));
        dom.borrow_mut()
            .set_attribute(node, attr_qual("class"), "cambium-retained-owner");
        dom.borrow_mut().set_attribute(
            node,
            attr_qual("style"),
            "position:relative; width:100%; height:100%; overflow:hidden;",
        );
        Self(Rc::new(Owner { dom, node }))
    }
    pub fn node(&self) -> NodeId {
        self.0.node
    }
    pub fn dom(&self) -> DomHandle {
        self.0.dom.clone()
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
        self.dom.borrow_mut().remove(self.node);
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
                dom.remove(prev.owner.node());
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
        if element.dom.borrow().parent(self.owner.node()) == Some(*element.node) {
            element.dom.borrow_mut().remove(self.owner.node());
        }
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
mod tests {
    use super::*;
    use crate::{GenetAppRunner, PointerClick, Slot, WorkspaceModel, el, on_click};
    use genet_scripted_dom::ScriptedDom;
    use std::cell::RefCell;
    use workbench::{TileId, TileTree, Workspace};

    #[test]
    fn actual_owner_keeps_dom_and_state_after_destination_first_mount() {
        let dom = Rc::new(RefCell::new(ScriptedDom::new()));
        let mount = RetainedMount::new(dom.clone());
        let mut owner = GenetAppRunner::new_at(
            dom.clone(),
            mount.node(),
            |count: &usize| {
                on_click(
                    el::<_, usize, ()>("button", count.to_string()),
                    |count: &mut usize, _: PointerClick| {
                        *count += 1;
                    },
                )
            },
            0usize,
        );
        let button = owner.root();
        let tile = workbench::Tile {
            id: TileId(7),
            title: "Owner".into(),
            content: workbench::ContentSource::Open {
                kind: "test".into(),
                id: "7".into(),
            },
            accent: None,
        };
        let workspace = Workspace::new(TileTree::single(tile));
        let source_mount = mount.clone();
        let mut source = GenetAppRunner::new(
            dom.clone(),
            move |enabled: &bool| -> Box<dyn crate::AnyView<bool, (), GenetCtx, GenetElement>> {
                let fill = source_mount.clone();
                let enabled = *enabled;
                Box::new(crate::workspace_view(
                    &WorkspaceModel {
                        workspace: &workspace,
                        current: Some(TileId(7)),
                        float_layer_visible: true,
                    },
                    |_: &mut bool, _| {},
                    move |_| {
                        if enabled {
                            Slot::View(Box::new(fill.view::<bool, ()>()))
                        } else {
                            Slot::Surface
                        }
                    },
                ))
            },
            true,
        );
        let old_parent = dom.borrow().parent(mount.node()).unwrap();
        let destination_mount = mount.clone();
        let mut destination = GenetAppRunner::new(
            dom.clone(),
            move |_: &()| destination_mount.view::<(), ()>(),
            (),
        );
        assert_ne!(dom.borrow().parent(mount.node()), Some(old_parent));
        // Explicit source teardown cannot withdraw a destination-first body.
        source.update(|enabled| *enabled = false);
        destination.update(|_| {});
        assert_eq!(dom.borrow().parent(button), Some(mount.node()));
        assert_eq!(owner.root(), button);
        owner.dispatch_click(button, PointerClick::at((1.0, 1.0)));
        assert_eq!(*owner.state(), 1);
        assert_eq!(owner.root(), button);
        assert_eq!(dom.borrow().parent(mount.node()), Some(destination.root()));
    }

    #[test]
    #[should_panic(expected = "retained mount must share the projection document")]
    fn unrelated_document_cannot_mount_a_foreign_owner_identity() {
        let owner = RetainedMount::new(Rc::new(RefCell::new(ScriptedDom::new())));
        let other = Rc::new(RefCell::new(ScriptedDom::new()));
        let _ = GenetAppRunner::new(other, move |_: &()| owner.view::<(), ()>(), ());
    }
}
