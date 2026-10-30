// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use layout_dom_api::{LayoutDom, LayoutDomMut};
use meristem::{MessageCtx, MessageResult, Mut, View, ViewId, ViewMarker, ViewPathTracker};

use super::placement::{placements, take_slot};
use super::{
    ComponentRoute, Entry, LifecycleEvent, LifecycleRefusal, ROOT_CONTAINER, RetainedComposition,
    RetainedState,
};
use crate::{GenetCtx, GenetElement, attr_qual, html_qual};

impl<State: 'static, Action: 'static> ViewMarker for RetainedComposition<State, Action> {}

impl<State: 'static, Action: 'static> View<State, Action, GenetCtx>
    for RetainedComposition<State, Action>
{
    type Element = GenetElement;
    type ViewState = RetainedState<State, Action>;

    fn build(&self, ctx: &mut GenetCtx, app_state: &mut State) -> (GenetElement, Self::ViewState) {
        let dom = ctx.dom();
        let root = dom.borrow_mut().create_element(html_qual("div"));
        dom.borrow_mut()
            .set_attribute(root, attr_qual("class"), "cambium-composition");
        let parking = dom.borrow_mut().create_element(html_qual("div"));
        dom.borrow_mut()
            .set_attribute(parking, attr_qual("class"), "cambium-composition-parking");
        dom.borrow_mut()
            .set_attribute(parking, attr_qual("style"), "display:none;");
        dom.borrow_mut()
            .set_attribute(parking, attr_qual("inert"), "");
        dom.borrow_mut()
            .set_attribute(parking, attr_qual("aria-hidden"), "true");
        dom.borrow_mut()
            .set_attribute(parking, attr_qual("data-cambium-parked"), "true");
        dom.borrow_mut().insert_before(root, parking, None);

        let mut state = RetainedState {
            dom: dom.clone(),
            parking,
            entries: BTreeMap::new(),
            containers: BTreeMap::from([(ROOT_CONTAINER.clone(), root)]),
            next_slot: 0,
        };
        self.rebuild_containers(root, &mut state);
        let placements = placements(&self.model, &self.collapsed);
        for descriptor in &self.model.components {
            let Some(slot) = take_slot(&mut state) else {
                continue;
            };
            let place = placements
                .get(&descriptor.key)
                .cloned()
                .unwrap_or_else(|| super::placement::empty_placement());
            let route = ComponentRoute {
                slot,
                route_generation: 0,
                placement_generation: 0,
            };
            let child = self.build_child(ctx, app_state, descriptor, route, state.parking);
            let mut entry = Entry {
                slot,
                route_generation: 0,
                placement_generation: 0,
                descriptor: descriptor.clone(),
                child: Some(child),
                container: place.container,
                location: place.location,
                location_signature: place.signature,
                hidden: place.hidden,
            };
            if let Some(child) = &mut entry.child {
                let target = if entry.hidden {
                    state.parking
                } else {
                    *state
                        .containers
                        .get(&entry.container)
                        .unwrap_or(&state.parking)
                };
                dom.borrow_mut()
                    .insert_before(target, child.element.node, None);
            }
            self.lifecycle.push(LifecycleEvent::Created {
                key: descriptor.key.clone(),
                route: entry.route(),
            });
            self.lifecycle
                .publish_route(&descriptor.key, entry.route(), !entry.hidden);
            if entry.hidden {
                self.lifecycle.push(LifecycleEvent::Hidden {
                    key: descriptor.key.clone(),
                    route: entry.route(),
                });
            }
            state.entries.insert(descriptor.key.clone(), entry);
        }
        self.reorder_presentation(root, &self.model.placement, &self.collapsed, &state, false);
        (GenetElement::new(root, dom), state)
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        element: Mut<'_, GenetElement>,
        app_state: &mut State,
    ) {
        if !Rc::ptr_eq(&element.dom, &state.dom) || !Rc::ptr_eq(&ctx.dom(), &state.dom) {
            self.lifecycle.refuse(LifecycleRefusal::ForeignDocument);
            return;
        }
        let root = *element.node;
        let next_places = placements(&self.model, &self.collapsed);

        // Preflight generation increments before the first DOM mutation.
        for (key, entry) in &state.entries {
            let next = self.model.components.iter().find(|item| &item.key == key);
            let place = next_places
                .get(key)
                .cloned()
                .unwrap_or_else(|| super::placement::empty_placement());
            let replaced = next.is_none_or(|next| {
                next.kind != entry.descriptor.kind || next.binding != entry.descriptor.binding
            });
            let moved = place.signature != entry.location_signature || place.hidden != entry.hidden;
            if replaced && entry.route_generation.checked_add(1).is_none() {
                self.lifecycle
                    .refuse(LifecycleRefusal::RouteGenerationExhausted);
                return;
            }
            if moved && entry.placement_generation.checked_add(1).is_none() {
                self.lifecycle
                    .refuse(LifecycleRefusal::PlacementGenerationExhausted);
                return;
            }
            if (replaced || moved || next.is_none())
                && entry.child.as_ref().is_some_and(|child| {
                    ctx.subtree_move_has_active_interaction(child.element.node)
                })
            {
                self.lifecycle.refuse(LifecycleRefusal::ActiveInteraction);
                return;
            }
            let retires_focused_entry =
                next.is_none() || replaced || (place.hidden && !entry.hidden);
            if retires_focused_entry
                && entry
                    .child
                    .as_ref()
                    .is_some_and(|child| ctx.focused_node_in_subtree(child.element.node).is_some())
            {
                self.lifecycle.refuse(LifecycleRefusal::FocusedRetirement);
                return;
            }
        }
        let new_entries = self
            .model
            .components
            .iter()
            .filter(|descriptor| !state.entries.contains_key(&descriptor.key))
            .count() as u64;
        if state.next_slot.checked_add(new_entries).is_none() {
            self.lifecycle.refuse(LifecycleRefusal::SlotExhausted);
            return;
        }

        let desired_containers = self.rebuild_containers(root, state);

        let accepted: BTreeSet<_> = self
            .model
            .components
            .iter()
            .map(|item| item.key.clone())
            .collect();
        let removed: Vec<_> = state
            .entries
            .keys()
            .filter(|key| !accepted.contains(*key))
            .cloned()
            .collect();
        for key in removed {
            if let Some(mut entry) = state.entries.remove(&key) {
                if let Some(mut child) = entry.child.take() {
                    ctx.with_id(ViewId::new(entry.slot), |ctx| {
                        ctx.with_id(ViewId::new(entry.route_generation), |ctx| {
                            ctx.with_id(ViewId::new(entry.placement_generation), |ctx| {
                                child.view.as_ref().dyn_teardown(
                                    &mut child.view_state,
                                    ctx,
                                    child.element.as_mut(),
                                );
                            });
                        });
                    });
                    state.dom.borrow_mut().remove(child.element.node);
                }
                self.lifecycle.push(LifecycleEvent::Disposed {
                    key,
                    route: entry.route(),
                });
                self.lifecycle.forget_route(&entry.descriptor.key);
            }
        }

        for descriptor in &self.model.components {
            let place = next_places
                .get(&descriptor.key)
                .cloned()
                .unwrap_or_else(|| super::placement::empty_placement());
            let Some(entry) = state.entries.get_mut(&descriptor.key) else {
                let Some(slot) = take_slot(state) else {
                    continue;
                };
                let route = ComponentRoute {
                    slot,
                    route_generation: 0,
                    placement_generation: 0,
                };
                let child = self.build_child(ctx, app_state, descriptor, route, state.parking);
                let target = if place.hidden {
                    state.parking
                } else {
                    *state
                        .containers
                        .get(&place.container)
                        .unwrap_or(&state.parking)
                };
                state
                    .dom
                    .borrow_mut()
                    .insert_before(target, child.element.node, None);
                let entry = Entry {
                    slot,
                    route_generation: 0,
                    placement_generation: 0,
                    descriptor: descriptor.clone(),
                    child: Some(child),
                    container: place.container,
                    location: place.location,
                    location_signature: place.signature,
                    hidden: place.hidden,
                };
                self.lifecycle.push(LifecycleEvent::Created {
                    key: descriptor.key.clone(),
                    route: entry.route(),
                });
                self.lifecycle
                    .publish_route(&descriptor.key, entry.route(), !entry.hidden);
                if entry.hidden {
                    self.lifecycle.push(LifecycleEvent::Hidden {
                        key: descriptor.key.clone(),
                        route: entry.route(),
                    });
                }
                state.entries.insert(descriptor.key.clone(), entry);
                continue;
            };

            let old_route = entry.route();
            let was_hidden = entry.hidden;
            let moved = place.signature != entry.location_signature || place.hidden != entry.hidden;
            let replaced = descriptor.kind != entry.descriptor.kind
                || descriptor.binding != entry.descriptor.binding;
            if replaced {
                entry.route_generation += 1;
            }
            if moved {
                entry.placement_generation += 1;
            }
            entry.container = place.container.clone();
            entry.location = place.location.clone();
            entry.location_signature = place.signature.clone();
            entry.hidden = place.hidden;
            entry.descriptor = descriptor.clone();
            self.lifecycle
                .publish_route(&descriptor.key, entry.route(), !entry.hidden);

            // Kind/binding replacement is a new retained incarnation. Tear
            // down the old view under its old route before allocating state.
            if replaced {
                if let Some(mut old) = entry.child.take() {
                    ctx.with_id(ViewId::new(old_route.slot), |ctx| {
                        ctx.with_id(ViewId::new(old_route.route_generation), |ctx| {
                            ctx.with_id(ViewId::new(old_route.placement_generation), |ctx| {
                                old.view.as_ref().dyn_teardown(
                                    &mut old.view_state,
                                    ctx,
                                    old.element.as_mut(),
                                );
                            });
                        });
                    });
                    state.dom.borrow_mut().remove(old.element.node);
                }
                self.lifecycle.push(LifecycleEvent::Replaced {
                    key: descriptor.key.clone(),
                    route: entry.route(),
                });
            }

            if entry.hidden {
                if moved && !was_hidden {
                    self.lifecycle.push(LifecycleEvent::Hidden {
                        key: descriptor.key.clone(),
                        route: entry.route(),
                    });
                }
                if let Some(child) = &entry.child {
                    if state.dom.borrow().parent(child.element.node) != Some(state.parking) {
                        state.dom.borrow_mut().insert_before(
                            state.parking,
                            child.element.node,
                            None,
                        );
                    }
                }
                continue;
            }

            let target = *state
                .containers
                .get(&entry.container)
                .unwrap_or(&state.parking);
            if entry.child.is_none() {
                let child =
                    self.build_child(ctx, app_state, descriptor, entry.route(), state.parking);
                state
                    .dom
                    .borrow_mut()
                    .insert_before(target, child.element.node, None);
                entry.child = Some(child);
            } else if let Some(child) = &mut entry.child {
                let next_view = (self.factory)(descriptor);
                if moved {
                    let old_view = &child.view;
                    ctx.with_route_refresh(|ctx| {
                        ctx.with_id(ViewId::new(entry.slot), |ctx| {
                            ctx.with_id(ViewId::new(entry.route_generation), |ctx| {
                                ctx.with_id(ViewId::new(entry.placement_generation), |ctx| {
                                    let current_parent = state
                                        .dom
                                        .borrow()
                                        .parent(child.element.node)
                                        .or(Some(state.parking));
                                    let mut element_mut = child.element.as_mut();
                                    element_mut.parent = current_parent;
                                    old_view.as_ref().dyn_rebuild(
                                        &mut child.view_state,
                                        ctx,
                                        &**old_view,
                                        element_mut,
                                        app_state,
                                    );
                                });
                            });
                        });
                    });
                }
                ctx.with_id(ViewId::new(entry.slot), |ctx| {
                    ctx.with_id(ViewId::new(entry.route_generation), |ctx| {
                        ctx.with_id(ViewId::new(entry.placement_generation), |ctx| {
                            let current_parent = state
                                .dom
                                .borrow()
                                .parent(child.element.node)
                                .or(Some(state.parking));
                            let mut element_mut = child.element.as_mut();
                            element_mut.parent = current_parent;
                            next_view.as_ref().dyn_rebuild(
                                &mut child.view_state,
                                ctx,
                                &*child.view,
                                element_mut,
                                app_state,
                            );
                        });
                    });
                });
                child.view = next_view;
                if state.dom.borrow().parent(child.element.node) != Some(target) {
                    state
                        .dom
                        .borrow_mut()
                        .insert_before(target, child.element.node, None);
                }
            }

            if moved {
                self.lifecycle.push(LifecycleEvent::Moved {
                    key: descriptor.key.clone(),
                    route: entry.route(),
                });
            }
            if was_hidden {
                self.lifecycle.push(LifecycleEvent::Revealed {
                    key: descriptor.key.clone(),
                    route: entry.route(),
                });
            }
        }

        self.reorder_presentation(root, &self.model.placement, &self.collapsed, state, false);
        self.remove_obsolete_wrappers(state, &desired_containers, root);
        self.lifecycle.clear_refusal();

        // Suppress unused `prev` without making its content part of admission.
        let _ = prev;
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        element: Mut<'_, GenetElement>,
    ) {
        for (key, entry) in &mut state.entries {
            if let Some(mut child) = entry.child.take() {
                ctx.with_id(ViewId::new(entry.slot), |ctx| {
                    ctx.with_id(ViewId::new(entry.route_generation), |ctx| {
                        ctx.with_id(ViewId::new(entry.placement_generation), |ctx| {
                            child.view.as_ref().dyn_teardown(
                                &mut child.view_state,
                                ctx,
                                child.element.as_mut(),
                            );
                        });
                    });
                });
                state.dom.borrow_mut().remove(child.element.node);
            }
            self.lifecycle.push(LifecycleEvent::Disposed {
                key: key.clone(),
                route: entry.route(),
            });
            self.lifecycle.forget_route(key);
        }
        let node = *element.node;
        self.remove_obsolete_wrappers(state, &BTreeSet::new(), node);
        state.dom.borrow_mut().remove(state.parking);
    }

    fn message(
        &self,
        state: &mut Self::ViewState,
        message: &mut MessageCtx,
        _element: Mut<'_, GenetElement>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        let Some(slot) = message.take_first().map(ViewId::routing_id) else {
            return MessageResult::Stale;
        };
        let Some(route_generation) = message.take_first().map(ViewId::routing_id) else {
            return MessageResult::Stale;
        };
        let Some(placement_generation) = message.take_first().map(ViewId::routing_id) else {
            return MessageResult::Stale;
        };
        let Some((_, entry)) = state.entries.iter_mut().find(|(_, entry)| {
            entry.slot == slot
                && entry.route_generation == route_generation
                && entry.placement_generation == placement_generation
                && !entry.hidden
        }) else {
            return MessageResult::Stale;
        };
        let Some(child) = &mut entry.child else {
            return MessageResult::Stale;
        };
        child.view.as_ref().dyn_message(
            &mut child.view_state,
            message,
            child.element.as_mut(),
            app_state,
        )
    }
}
