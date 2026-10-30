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
    ComponentRoute, Entry, LifecycleEvent, LifecycleRefusal, MutationRecord, ROOT_CONTAINER,
    RetainedComposition, RetainedState,
};
use crate::composition::ComponentKey;
use crate::reactive_diagnostics::{
    CauseRelation, DiagnosticDetails, DiagnosticKind, DiagnosticReason, Producer, ProducerRef,
    Subject, diagnostic_span,
};
use crate::{DomHandle, GenetCtx, GenetElement, attr_qual, html_qual};

impl<State: 'static, Action: 'static> ViewMarker for RetainedComposition<State, Action> {}

impl<State: 'static, Action: 'static> View<State, Action, GenetCtx>
    for RetainedComposition<State, Action>
{
    type Element = GenetElement;
    type ViewState = RetainedState<State, Action>;

    fn build(&self, ctx: &mut GenetCtx, app_state: &mut State) -> (GenetElement, Self::ViewState) {
        let flush_inputs = self.inputs.clone();
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
        let mutation_cursor_start = mutation_cursor(&state.dom);
        let placements = placements(&self.model, &self.collapsed);
        let mut built = BTreeSet::new();
        let mut causes = Vec::new();
        let mut mutation_keys = BTreeMap::new();
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
            let component_cursor_start = mutation_cursor(&state.dom);
            let build_span = (!place.hidden)
                .then(|| component_operation_span(flush_inputs.as_ref(), &descriptor.key, route))
                .flatten();
            let _build_span_guard = build_span.as_ref().map(|operation| operation.span.enter());
            let child = if place.hidden && flush_inputs.is_some() {
                None
            } else {
                Some(self.build_child(ctx, app_state, descriptor, route, state.parking))
            };
            let entry = Entry {
                slot,
                route_generation: 0,
                placement_generation: 0,
                descriptor: descriptor.clone(),
                child,
                container: place.container,
                location: place.location,
                location_signature: place.signature,
                hidden: place.hidden,
                applied_inputs: if place.hidden && flush_inputs.is_some() {
                    None
                } else {
                    flush_inputs
                        .as_ref()
                        .map(|inputs| inputs.vector_or_empty(&descriptor.key))
                },
                message_dirty: false,
                deferred_inputs: place.hidden && flush_inputs.is_some(),
                last_deferred_generation: if place.hidden {
                    flush_inputs
                        .as_ref()
                        .and_then(|inputs| inputs.captured_generation(&descriptor.key))
                } else {
                    None
                },
            };
            if let Some(child) = &entry.child {
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
            if !entry.hidden {
                built.insert(descriptor.key.clone());
                if let Some(inputs) = &flush_inputs {
                    if let Some(subject) = input_subject(inputs, entry.route()) {
                        let component_cursor_end = mutation_cursor(&state.dom);
                        if let Some(reference) = inputs.emit_component(
                            DiagnosticKind::ComponentInputEvaluated,
                            subject,
                            diagnostic_details_with_mutations(
                                inputs.captured_generation(&descriptor.key).unwrap_or(0),
                                1,
                                DiagnosticReason::DependencyChanged,
                                component_cursor_start,
                                component_cursor_end,
                            ),
                            &descriptor.key,
                        ) {
                            causes.push(reference);
                            record_operation(build_span.as_ref(), reference);
                            if component_cursor_end > component_cursor_start {
                                mutation_keys.insert(
                                    descriptor.key.clone(),
                                    (
                                        subject,
                                        reference,
                                        component_cursor_start,
                                        component_cursor_end,
                                        true,
                                    ),
                                );
                            }
                        }
                    }
                }
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
                if let Some(inputs) = &flush_inputs {
                    if let (Some(generation), Some(subject)) = (
                        inputs.captured_generation(&descriptor.key),
                        input_subject(inputs, entry.route()),
                    ) {
                        if let Some(reference) = inputs.emit_component(
                            DiagnosticKind::ComponentInputDeferred,
                            subject,
                            diagnostic_details(
                                generation,
                                inputs.pending_notices(&descriptor.key).unwrap_or(1),
                                DiagnosticReason::Hidden,
                            ),
                            &descriptor.key,
                        ) {
                            causes.push(reference);
                        }
                    }
                }
            }
            state.entries.insert(descriptor.key.clone(), entry);
        }
        self.reorder_presentation(root, &self.model.placement, &self.collapsed, &state, false);
        let mutation_cursor_end = mutation_cursor(&state.dom);
        if let Some(inputs) = &flush_inputs {
            let flush_ref = inputs.emit_flush_completed(
                inputs
                    .capture_ref()
                    .map_or(0, |reference| reference.sequence),
                built.len(),
                mutation_cursor_start,
                mutation_cursor_end,
                &causes,
            );
            let batch_ref = flush_ref.and_then(|flush_ref| {
                inputs.emit_mutation_batch(flush_ref, mutation_cursor_start, mutation_cursor_end)
            });
            if let Some(reference) = batch_ref {
                for (key, (subject, _, _, _, visible)) in mutation_keys {
                    inputs.record_mutation(
                        key,
                        MutationRecord {
                            reference,
                            subject,
                            mutation_cursor_start,
                            mutation_cursor_end,
                            visible,
                        },
                    );
                }
            }
            inputs.acknowledge(&built);
            for (key, entry) in &state.entries {
                inputs.set_mutation_visibility(key, !entry.hidden);
            }
        }
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
        // Freeze the complete input table before the first factory can run.
        let flush_inputs = self.inputs.clone();
        let root = *element.node;
        let mutation_cursor_start = mutation_cursor(&state.dom);
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
        let mut evaluated = BTreeSet::new();
        let mut causes = Vec::new();
        let mut mutation_keys = BTreeMap::new();
        let mut route_moves = BTreeMap::new();
        for key in removed {
            if let Some(mut entry) = state.entries.remove(&key) {
                let component_cursor_start = mutation_cursor(&state.dom);
                let old_route = entry.route();
                let teardown_span = entry
                    .child
                    .as_ref()
                    .and_then(|_| component_operation_span(flush_inputs.as_ref(), &key, old_route));
                let _teardown_span_guard = teardown_span
                    .as_ref()
                    .map(|operation| operation.span.enter());
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
                if let Some(inputs) = &flush_inputs {
                    if let Some(subject) = input_subject(inputs, old_route) {
                        let component_cursor_end = mutation_cursor(&state.dom);
                        if let Some(reference) = inputs.emit_component(
                            DiagnosticKind::ComponentTornDown,
                            subject,
                            diagnostic_details_with_mutations(
                                inputs.captured_generation(&key).unwrap_or(0),
                                1,
                                DiagnosticReason::Teardown,
                                component_cursor_start,
                                component_cursor_end,
                            ),
                            &key,
                        ) {
                            causes.push(reference);
                            record_operation(teardown_span.as_ref(), reference);
                            if component_cursor_end > component_cursor_start {
                                mutation_keys.insert(
                                    key.clone(),
                                    (
                                        subject,
                                        reference,
                                        component_cursor_start,
                                        component_cursor_end,
                                        false,
                                    ),
                                );
                            }
                        }
                    }
                }
                self.lifecycle.push(LifecycleEvent::Disposed {
                    key,
                    route: entry.route(),
                });
                self.lifecycle.forget_route(&entry.descriptor.key);
            }
        }

        for descriptor in &self.model.components {
            let component_cursor_start = mutation_cursor(&state.dom);
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
                let build_span = (!place.hidden)
                    .then(|| {
                        component_operation_span(flush_inputs.as_ref(), &descriptor.key, route)
                    })
                    .flatten();
                let _build_span_guard = build_span.as_ref().map(|operation| operation.span.enter());
                let child = if place.hidden && flush_inputs.is_some() {
                    None
                } else {
                    Some(self.build_child(ctx, app_state, descriptor, route, state.parking))
                };
                let target = if place.hidden {
                    state.parking
                } else {
                    *state
                        .containers
                        .get(&place.container)
                        .unwrap_or(&state.parking)
                };
                if let Some(child) = &child {
                    state
                        .dom
                        .borrow_mut()
                        .insert_before(target, child.element.node, None);
                }
                let entry = Entry {
                    slot,
                    route_generation: 0,
                    placement_generation: 0,
                    descriptor: descriptor.clone(),
                    child,
                    container: place.container,
                    location: place.location,
                    location_signature: place.signature,
                    hidden: place.hidden,
                    applied_inputs: if place.hidden && flush_inputs.is_some() {
                        None
                    } else {
                        flush_inputs
                            .as_ref()
                            .map(|inputs| inputs.vector_or_empty(&descriptor.key))
                    },
                    message_dirty: false,
                    deferred_inputs: place.hidden && flush_inputs.is_some(),
                    last_deferred_generation: if place.hidden {
                        flush_inputs
                            .as_ref()
                            .and_then(|inputs| inputs.captured_generation(&descriptor.key))
                    } else {
                        None
                    },
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
                    if let Some(inputs) = &flush_inputs {
                        if let (Some(generation), Some(subject)) = (
                            inputs.captured_generation(&descriptor.key),
                            input_subject(inputs, entry.route()),
                        ) {
                            if let Some(reference) = inputs.emit_component(
                                DiagnosticKind::ComponentInputDeferred,
                                subject,
                                diagnostic_details(
                                    generation,
                                    inputs.pending_notices(&descriptor.key).unwrap_or(1),
                                    DiagnosticReason::Hidden,
                                ),
                                &descriptor.key,
                            ) {
                                causes.push(reference);
                            }
                        }
                    }
                }
                state.entries.insert(descriptor.key.clone(), entry);
                if !place.hidden {
                    evaluated.insert(descriptor.key.clone());
                    if let Some(inputs) = &flush_inputs {
                        if let Some(subject) = input_subject(inputs, route) {
                            let component_cursor_end = mutation_cursor(&state.dom);
                            if let Some(reference) = inputs.emit_component(
                                DiagnosticKind::ComponentInputEvaluated,
                                subject,
                                diagnostic_details_with_mutations(
                                    inputs.captured_generation(&descriptor.key).unwrap_or(0),
                                    1,
                                    DiagnosticReason::DependencyChanged,
                                    component_cursor_start,
                                    component_cursor_end,
                                ),
                                &descriptor.key,
                            ) {
                                causes.push(reference);
                                record_operation(build_span.as_ref(), reference);
                                if component_cursor_end > component_cursor_start {
                                    mutation_keys.insert(
                                        descriptor.key.clone(),
                                        (
                                            subject,
                                            reference,
                                            component_cursor_start,
                                            component_cursor_end,
                                            true,
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }
                continue;
            };

            let old_route = entry.route();
            let was_hidden = entry.hidden;
            let config_changed = descriptor.config != entry.descriptor.config;
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
                entry.message_dirty = false;
                let teardown_span = entry.child.as_ref().and_then(|_| {
                    component_operation_span(flush_inputs.as_ref(), &descriptor.key, old_route)
                });
                let _teardown_span_guard = teardown_span
                    .as_ref()
                    .map(|operation| operation.span.enter());
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
                    if let Some(inputs) = &flush_inputs {
                        if let Some(subject) = input_subject(inputs, old_route) {
                            if let Some(reference) = inputs.emit_component(
                                DiagnosticKind::ComponentTornDown,
                                subject,
                                diagnostic_details(
                                    inputs.captured_generation(&descriptor.key).unwrap_or(0),
                                    1,
                                    DiagnosticReason::Teardown,
                                ),
                                &descriptor.key,
                            ) {
                                record_operation(teardown_span.as_ref(), reference);
                            }
                        }
                    }
                }
                self.lifecycle.push(LifecycleEvent::Replaced {
                    key: descriptor.key.clone(),
                    route: entry.route(),
                });
            }

            if entry.hidden {
                let mut defer_generation = None;
                if let Some(inputs) = &flush_inputs {
                    let current = inputs.vector_or_empty(&descriptor.key);
                    if entry.applied_inputs.as_ref() != Some(&current)
                        || config_changed
                        || replaced
                        || entry.message_dirty
                        || inputs.inputs().vector(&descriptor.key).is_none()
                    {
                        entry.deferred_inputs = true;
                    }
                    let captured_generation = inputs.captured_generation(&descriptor.key);
                    if captured_generation.is_some() {
                        // Even a same-vector explicit invalidation must be
                        // evaluated when this entry becomes visible again.
                        entry.deferred_inputs = true;
                    }
                    if captured_generation.is_some()
                        && captured_generation != entry.last_deferred_generation
                    {
                        defer_generation = captured_generation;
                        entry.last_deferred_generation = captured_generation;
                    }
                }
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
                if let Some(inputs) = &flush_inputs {
                    if let Some(subject) = input_subject(inputs, entry.route()) {
                        let cursor_end = mutation_cursor(&state.dom);
                        let event = if let Some(generation) = defer_generation {
                            inputs.emit_component(
                                DiagnosticKind::ComponentInputDeferred,
                                subject,
                                diagnostic_details_with_mutations(
                                    generation,
                                    inputs.pending_notices(&descriptor.key).unwrap_or(1),
                                    DiagnosticReason::Hidden,
                                    component_cursor_start,
                                    cursor_end,
                                ),
                                &descriptor.key,
                            )
                        } else if moved {
                            inputs.emit_component(
                                DiagnosticKind::ComponentInputSkipped,
                                subject,
                                diagnostic_details_with_mutations(
                                    inputs.captured_generation(&descriptor.key).unwrap_or(0),
                                    0,
                                    DiagnosticReason::Hidden,
                                    component_cursor_start,
                                    cursor_end,
                                ),
                                &descriptor.key,
                            )
                        } else {
                            None
                        };
                        if let Some(reference) = event {
                            causes.push(reference);
                            if cursor_end > component_cursor_start {
                                mutation_keys.insert(
                                    descriptor.key.clone(),
                                    (
                                        subject,
                                        reference,
                                        component_cursor_start,
                                        cursor_end,
                                        false,
                                    ),
                                );
                            }
                        }
                    }
                }
                continue;
            }

            let target = *state
                .containers
                .get(&entry.container)
                .unwrap_or(&state.parking);
            let current_inputs = flush_inputs
                .as_ref()
                .map(|inputs| inputs.vector_or_empty(&descriptor.key));
            let inputs_changed = current_inputs.as_ref().is_some_and(|current| {
                entry.applied_inputs.as_ref() != Some(current) || entry.deferred_inputs
            });
            let inputs_missing = flush_inputs
                .as_ref()
                .is_some_and(|inputs| inputs.inputs().vector(&descriptor.key).is_none());
            let has_captured_invalidation = flush_inputs
                .as_ref()
                .is_some_and(|inputs| inputs.captured_generation(&descriptor.key).is_some());
            let needs_evaluation = self.inputs.is_none()
                || inputs_changed
                || inputs_missing
                || has_captured_invalidation
                || config_changed
                || entry.message_dirty
                || replaced
                || entry.child.is_none();
            let evaluation_route = entry.route();

            if moved && !replaced {
                let route = entry.route();
                let route_span =
                    component_operation_span(flush_inputs.as_ref(), &descriptor.key, route);
                if let Some(child) = &mut entry.child {
                    let old_view = &child.view;
                    let _route_span_guard =
                        route_span.as_ref().map(|operation| operation.span.enter());
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
                    if let Some(inputs) = &flush_inputs {
                        if let Some(subject) = input_subject(inputs, route) {
                            if let Some(reference) = inputs.emit_component(
                                DiagnosticKind::ComponentRouteRefreshed,
                                subject,
                                diagnostic_details(
                                    inputs.captured_generation(&descriptor.key).unwrap_or(0),
                                    1,
                                    DiagnosticReason::RouteChanged,
                                ),
                                &descriptor.key,
                            ) {
                                record_operation(route_span.as_ref(), reference);
                            }
                        }
                    }
                }
            }

            let mut did_evaluate = false;
            let had_local_dirty = entry.message_dirty;
            let evaluation_span = (entry.child.is_none() || needs_evaluation)
                .then(|| {
                    component_operation_span(
                        flush_inputs.as_ref(),
                        &descriptor.key,
                        evaluation_route,
                    )
                })
                .flatten();
            let _evaluation_span_guard = evaluation_span
                .as_ref()
                .map(|operation| operation.span.enter());
            if entry.child.is_none() {
                let child =
                    self.build_child(ctx, app_state, descriptor, evaluation_route, state.parking);
                state
                    .dom
                    .borrow_mut()
                    .insert_before(target, child.element.node, None);
                entry.child = Some(child);
                did_evaluate = true;
            } else if needs_evaluation && let Some(child) = &mut entry.child {
                let next_view = (self.factory)(descriptor);
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
                did_evaluate = true;
            }

            if let Some(child) = &entry.child {
                if state.dom.borrow().parent(child.element.node) != Some(target) {
                    state
                        .dom
                        .borrow_mut()
                        .insert_before(target, child.element.node, None);
                }
            }

            if let Some(current) = current_inputs {
                entry.applied_inputs = Some(current);
            }
            entry.deferred_inputs = false;
            entry.last_deferred_generation = None;
            let component_cursor_end = mutation_cursor(&state.dom);
            let mut component_reference = None;
            if did_evaluate {
                evaluated.insert(descriptor.key.clone());
                entry.message_dirty = false;
                if let Some(inputs) = &flush_inputs {
                    if let Some(subject) = input_subject(inputs, entry.route()) {
                        component_reference = inputs.emit_component(
                            DiagnosticKind::ComponentInputEvaluated,
                            subject,
                            diagnostic_details_with_mutations(
                                inputs.captured_generation(&descriptor.key).unwrap_or(0),
                                1,
                                if had_local_dirty {
                                    DiagnosticReason::LocalStateDirty
                                } else {
                                    DiagnosticReason::DependencyChanged
                                },
                                component_cursor_start,
                                component_cursor_end,
                            ),
                            &descriptor.key,
                        );
                        if let Some(reference) = component_reference {
                            causes.push(reference);
                            record_operation(evaluation_span.as_ref(), reference);
                            if component_cursor_end > component_cursor_start {
                                mutation_keys.insert(
                                    descriptor.key.clone(),
                                    (
                                        subject,
                                        reference,
                                        component_cursor_start,
                                        component_cursor_end,
                                        true,
                                    ),
                                );
                            }
                        }
                    }
                }
            } else {
                evaluated.insert(descriptor.key.clone());
                if let Some(inputs) = &flush_inputs {
                    if let Some(subject) = input_subject(inputs, entry.route()) {
                        component_reference = inputs.emit_component(
                            DiagnosticKind::ComponentInputSkipped,
                            subject,
                            diagnostic_details_with_mutations(
                                inputs.captured_generation(&descriptor.key).unwrap_or(0),
                                0,
                                if moved {
                                    DiagnosticReason::RouteChanged
                                } else {
                                    DiagnosticReason::EqualAndClean
                                },
                                component_cursor_start,
                                component_cursor_end,
                            ),
                            &descriptor.key,
                        );
                        if let Some(reference) = component_reference {
                            causes.push(reference);
                            if component_cursor_end > component_cursor_start {
                                mutation_keys.insert(
                                    descriptor.key.clone(),
                                    (
                                        subject,
                                        reference,
                                        component_cursor_start,
                                        component_cursor_end,
                                        true,
                                    ),
                                );
                            }
                        }
                    }
                }
            }
            if moved {
                if let (Some(inputs), Some(reference)) = (&flush_inputs, component_reference) {
                    if let Some(subject) = input_subject(inputs, entry.route()) {
                        route_moves.insert(descriptor.key.clone(), (subject, reference));
                    }
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

        let reorder_cursor_start = mutation_cursor(&state.dom);
        self.reorder_presentation(root, &self.model.placement, &self.collapsed, state, false);
        let reorder_cursor_end = mutation_cursor(&state.dom);
        if reorder_cursor_end > reorder_cursor_start {
            for (key, (subject, reference)) in route_moves {
                mutation_keys.insert(
                    key,
                    (
                        subject,
                        reference,
                        reorder_cursor_start,
                        reorder_cursor_end,
                        true,
                    ),
                );
            }
        }
        self.remove_obsolete_wrappers(state, &desired_containers, root);
        let mutation_cursor_end = mutation_cursor(&state.dom);
        self.lifecycle.clear_refusal();
        if let Some(inputs) = &flush_inputs {
            let flush_ref = inputs.emit_flush_completed(
                inputs
                    .capture_ref()
                    .map_or(0, |reference| reference.sequence),
                evaluated.len(),
                mutation_cursor_start,
                mutation_cursor_end,
                &causes,
            );
            let batch_ref = flush_ref.and_then(|flush_ref| {
                inputs.emit_mutation_batch(flush_ref, mutation_cursor_start, mutation_cursor_end)
            });
            if let Some(reference) = batch_ref {
                if mutation_cursor_end > mutation_cursor_start {
                    for (key, (subject, _, _, _, visible)) in mutation_keys {
                        inputs.record_mutation(
                            key,
                            MutationRecord {
                                reference,
                                subject,
                                mutation_cursor_start,
                                mutation_cursor_end,
                                visible,
                            },
                        );
                    }
                }
            }
            inputs.acknowledge(&evaluated);
            for (key, entry) in &state.entries {
                inputs.set_mutation_visibility(key, !entry.hidden);
            }
        }

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
            let route = entry.route();
            let teardown_span = entry
                .child
                .as_ref()
                .and_then(|_| component_operation_span(self.inputs.as_ref(), key, route));
            let _teardown_span_guard = teardown_span
                .as_ref()
                .map(|operation| operation.span.enter());
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
                if let Some(inputs) = &self.inputs {
                    if let Some(subject) = input_subject(inputs, route) {
                        if let Some(reference) = inputs.emit_component(
                            DiagnosticKind::ComponentTornDown,
                            subject,
                            diagnostic_details(
                                inputs.captured_generation(key).unwrap_or(0),
                                1,
                                DiagnosticReason::Teardown,
                            ),
                            key,
                        ) {
                            record_operation(teardown_span.as_ref(), reference);
                        }
                    }
                }
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
        let Some((key, entry)) = state.entries.iter_mut().find(|(_, entry)| {
            entry.slot == slot
                && entry.route_generation == route_generation
                && entry.placement_generation == placement_generation
                && !entry.hidden
        }) else {
            return MessageResult::Stale;
        };
        let route = entry.route();
        let message_span = component_operation_span(self.inputs.as_ref(), key, route);
        let Some(child) = &mut entry.child else {
            return MessageResult::Stale;
        };
        let _message_span_guard = message_span
            .as_ref()
            .map(|operation| operation.span.enter());
        let result = child.view.as_ref().dyn_message(
            &mut child.view_state,
            message,
            child.element.as_mut(),
            app_state,
        );
        if let Some(inputs) = &self.inputs {
            if let Some(subject) = input_subject(inputs, route) {
                if let Some(reference) = inputs.emit_component(
                    DiagnosticKind::ComponentMessageRouted,
                    subject,
                    diagnostic_details(
                        inputs.captured_generation(key).unwrap_or(0),
                        1,
                        DiagnosticReason::None,
                    ),
                    key,
                ) {
                    record_operation(message_span.as_ref(), reference);
                }
            }
        }
        entry.message_dirty = true;
        result
    }
}

fn input_subject(
    inputs: &super::CompositionInputSnapshot,
    route: ComponentRoute,
) -> Option<Subject> {
    Some(Subject {
        owner_id: inputs.producer()?.producer_id(),
        slot: route.slot,
        route_generation: route.route_generation,
        placement_generation: route.placement_generation,
    })
}

fn component_operation_span(
    inputs: Option<&super::CompositionInputSnapshot>,
    key: &ComponentKey,
    route: ComponentRoute,
) -> Option<ComponentOperationSpan> {
    let inputs = inputs?;
    let subject = input_subject(inputs, route)?;
    let captured_generation = inputs.captured_generation(key);
    let reference = inputs.emit_component(
        DiagnosticKind::ComponentOperationStarted,
        subject,
        diagnostic_details(
            captured_generation.unwrap_or_default(),
            1,
            DiagnosticReason::Accepted,
        ),
        key,
    )?;
    let producer = inputs.producer()?.clone();
    Some(ComponentOperationSpan {
        span: diagnostic_span(subject, None, Some(reference)),
        producer,
        started: reference,
    })
}

struct ComponentOperationSpan {
    span: tracing::Span,
    producer: Producer,
    started: ProducerRef,
}

fn record_operation(operation: Option<&ComponentOperationSpan>, reference: ProducerRef) {
    if let Some(operation) = operation {
        let _ = operation.producer.emit_cause_edge(
            reference,
            operation.started,
            CauseRelation::ProducedBy,
        );
    }
}

fn diagnostic_details(revision: u64, _count: u64, reason: DiagnosticReason) -> DiagnosticDetails {
    DiagnosticDetails {
        revision: (revision != 0).then_some(revision),
        mutation_cursor_start: None,
        mutation_cursor_end: None,
        reason,
        ..DiagnosticDetails::default()
    }
}

fn diagnostic_details_with_mutations(
    revision: u64,
    _count: u64,
    reason: DiagnosticReason,
    mutation_cursor_start: u64,
    mutation_cursor_end: u64,
) -> DiagnosticDetails {
    DiagnosticDetails {
        revision: (revision != 0).then_some(revision),
        mutation_cursor_start: Some(mutation_cursor_start),
        mutation_cursor_end: Some(mutation_cursor_end),
        reason,
        ..DiagnosticDetails::default()
    }
}

fn mutation_cursor(dom: &DomHandle) -> u64 {
    let dom = dom.borrow();
    let (base, pending) = dom.pending_mutations();
    base.saturating_add(pending.len() as u64)
}
