// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::{ComponentRoute, LiveChild, ROOT_CONTAINER, RetainedComposition, RetainedState};
use crate::composition::{ComponentDescriptor, ContainerKey, ContainerKind, PlacementNode};
use crate::{GenetCtx, attr_qual, html_qual};
use genet_scripted_dom::NodeId;
use layout_dom_api::{LayoutDom, LayoutDomMut, LocalName, Namespace};
use meristem::{ViewId, ViewPathTracker};
use std::collections::BTreeSet;

impl<State: 'static, Action: 'static> RetainedComposition<State, Action> {
    pub(super) fn build_child(
        &self,
        ctx: &mut GenetCtx,
        app_state: &mut State,
        descriptor: &ComponentDescriptor,
        route: ComponentRoute,
        parent: NodeId,
    ) -> LiveChild<State, Action> {
        let view = (self.factory)(descriptor);
        let (element, view_state) = ctx.with_id(ViewId::new(route.slot), |ctx| {
            ctx.with_id(ViewId::new(route.route_generation), |ctx| {
                ctx.with_id(ViewId::new(route.placement_generation), |ctx| {
                    view.dyn_build(ctx, app_state)
                })
            })
        });
        // A new element is detached; the final parent is assigned during the
        // attach transaction. The retained move itself preserves its NodeId.
        let _ = parent;
        LiveChild {
            view,
            view_state,
            element,
        }
    }

    pub(super) fn rebuild_containers(
        &self,
        root: NodeId,
        state: &mut RetainedState<State, Action>,
    ) -> BTreeSet<ContainerKey> {
        fn descend<State, Action>(
            view: &RetainedComposition<State, Action>,
            nodes: &[PlacementNode],
            parent: NodeId,
            inherited_hidden: bool,
            state: &mut RetainedState<State, Action>,
            desired: &mut BTreeSet<ContainerKey>,
        ) {
            for node in nodes {
                let (key, children, is_group) = match node {
                    PlacementNode::Component { .. } => continue,
                    PlacementNode::Group { key, children } => (
                        ContainerKey {
                            kind: ContainerKind::Group,
                            key: key.0.clone(),
                        },
                        children,
                        true,
                    ),
                    PlacementNode::Drawer { key, children } => (
                        ContainerKey {
                            kind: ContainerKind::Drawer,
                            key: key.0.clone(),
                        },
                        children,
                        false,
                    ),
                };
                let wrapper = if let Some(wrapper) = state.containers.get(&key).copied() {
                    wrapper
                } else {
                    let wrapper = state.dom.borrow_mut().create_element(html_qual("div"));
                    state.dom.borrow_mut().set_attribute(
                        wrapper,
                        attr_qual("class"),
                        if is_group {
                            "cambium-composition-group"
                        } else {
                            "cambium-composition-drawer"
                        },
                    );
                    state.dom.borrow_mut().set_attribute(
                        wrapper,
                        attr_qual("data-composition-key"),
                        &key.key,
                    );
                    state.containers.insert(key.clone(), wrapper);
                    wrapper
                };
                if state.dom.borrow().parent(wrapper) != Some(parent) {
                    state.dom.borrow_mut().insert_before(parent, wrapper, None);
                }
                desired.insert(key.clone());
                let hidden = inherited_hidden || view.collapsed.contains(&key);
                {
                    let mut dom = state.dom.borrow_mut();
                    if hidden {
                        set_attribute_if_changed(&mut dom, wrapper, "style", "display:none;");
                        set_attribute_if_changed(&mut dom, wrapper, "inert", "");
                        set_attribute_if_changed(&mut dom, wrapper, "aria-hidden", "true");
                    } else {
                        remove_attribute_if_present(&mut dom, wrapper, "style");
                        remove_attribute_if_present(&mut dom, wrapper, "inert");
                        remove_attribute_if_present(&mut dom, wrapper, "aria-hidden");
                    }
                }
                descend(view, children, wrapper, hidden, state, desired);
            }
        }
        let mut desired = BTreeSet::new();
        descend(
            self,
            &self.model.placement,
            root,
            false,
            state,
            &mut desired,
        );
        desired
    }

    pub(super) fn reorder_presentation(
        &self,
        parent: NodeId,
        nodes: &[PlacementNode],
        collapsed: &BTreeSet<ContainerKey>,
        state: &RetainedState<State, Action>,
        inherited_hidden: bool,
    ) {
        let mut expected = Vec::new();
        for node in nodes {
            match node {
                PlacementNode::Component { key } => {
                    let Some(entry) = state.entries.get(key) else {
                        continue;
                    };
                    if !entry.hidden {
                        if let Some(child) = &entry.child {
                            expected.push(child.element.node);
                        }
                    }
                },
                PlacementNode::Group { key, children } => {
                    reorder_container(
                        self,
                        ContainerKey {
                            kind: ContainerKind::Group,
                            key: key.0.clone(),
                        },
                        children,
                        parent,
                        collapsed,
                        state,
                        inherited_hidden,
                        &mut expected,
                    );
                },
                PlacementNode::Drawer { key, children } => {
                    reorder_container(
                        self,
                        ContainerKey {
                            kind: ContainerKind::Drawer,
                            key: key.0.clone(),
                        },
                        children,
                        parent,
                        collapsed,
                        state,
                        inherited_hidden,
                        &mut expected,
                    );
                },
            }
        }
        if parent == state.containers[&ROOT_CONTAINER] {
            expected.push(state.parking);
        }
        let current: Vec<_> = state.dom.borrow().dom_children(parent).collect();
        if current != expected {
            let mut dom = state.dom.borrow_mut();
            for node in expected {
                dom.insert_before(parent, node, None);
            }
        }
        let _ = inherited_hidden;
    }

    pub(super) fn remove_obsolete_wrappers(
        &self,
        state: &mut RetainedState<State, Action>,
        desired: &BTreeSet<ContainerKey>,
        root: NodeId,
    ) {
        let obsolete: Vec<_> = state
            .containers
            .iter()
            .filter_map(|(key, node)| {
                (key.kind != ContainerKind::Root && !desired.contains(key)).then_some(*node)
            })
            .collect();
        for node in obsolete {
            if state.dom.borrow().is_live(node) {
                state.dom.borrow_mut().remove(node);
            }
        }
        state.containers.retain(|key, node| {
            key.kind == ContainerKind::Root || desired.contains(key) || *node == root
        });
    }
}

fn set_attribute_if_changed(
    dom: &mut genet_scripted_dom::ScriptedDom,
    node: NodeId,
    name: &str,
    value: &str,
) {
    let ns = Namespace::from("");
    let local = LocalName::from(name);
    if dom.attribute(node, &ns, &local) != Some(value) {
        dom.set_attribute(node, attr_qual(name), value);
    }
}

fn remove_attribute_if_present(
    dom: &mut genet_scripted_dom::ScriptedDom,
    node: NodeId,
    name: &str,
) {
    let ns = Namespace::from("");
    let local = LocalName::from(name);
    if dom.attribute(node, &ns, &local).is_some() {
        dom.remove_attribute(node, attr_qual(name));
    }
}

fn reorder_container<State: 'static, Action: 'static>(
    view: &RetainedComposition<State, Action>,
    container: ContainerKey,
    children: &[PlacementNode],
    _parent: NodeId,
    collapsed: &BTreeSet<ContainerKey>,
    state: &RetainedState<State, Action>,
    inherited_hidden: bool,
    expected: &mut Vec<NodeId>,
) {
    let Some(wrapper) = state.containers.get(&container).copied() else {
        return;
    };
    expected.push(wrapper);
    view.reorder_presentation(
        wrapper,
        children,
        collapsed,
        state,
        inherited_hidden || collapsed.contains(&container),
    );
}
