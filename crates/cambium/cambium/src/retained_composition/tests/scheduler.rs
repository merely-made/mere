// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use crate::composition::{ComponentKey, ContainerKey, ContainerKind};
use crate::retained_composition::{
    CompositionCauseRef, CompositionInputQueue, CompositionInputs, DependencyStamp, InputVector,
    LocalInputStamp,
};
use crate::{
    ComponentView, GenetAppRunner, GenetCtx, GenetElement, PointerClick, component, el, on_click,
};
use genet_scripted_dom::ScriptedDom;
use meristem::View;

use super::{AllowBindings, descriptor, find_row, group, model, row, transaction};

#[derive(Clone, Copy)]
struct RowClicked;
impl crate::Action for RowClicked {}

#[derive(Clone, Debug, PartialEq)]
struct RowProps {
    key: String,
    revisions: Vec<u64>,
}

struct SchedulerState {
    transaction: crate::composition::CompositionTransaction,
    queue: CompositionInputQueue,
    revisions: Rc<Cell<(u64, u64, u64)>>,
    factory_counts: Rc<RefCell<BTreeMap<String, usize>>>,
    body_counts: Rc<RefCell<BTreeMap<String, usize>>>,
    collapsed: BTreeSet<ContainerKey>,
    actions: usize,
    factory: crate::RetainedComponentFactory<SchedulerState, ()>,
}

fn increment(counts: &Rc<RefCell<BTreeMap<String, usize>>>, key: &str) {
    *counts.borrow_mut().entry(key.to_owned()).or_default() += 1;
}

fn scheduler_state(transaction: crate::composition::CompositionTransaction) -> SchedulerState {
    let queue = CompositionInputQueue::new().unwrap();
    let revisions = Rc::new(Cell::new((1, 1, 1)));
    let factory_counts = Rc::new(RefCell::new(BTreeMap::new()));
    let body_counts = Rc::new(RefCell::new(BTreeMap::new()));
    let factory_revisions = revisions.clone();
    let factory_counts_inner = factory_counts.clone();
    let body_counts_inner = body_counts.clone();
    let factory = Rc::new(
        move |descriptor: &crate::composition::ComponentDescriptor| {
            let key = descriptor.key.0.clone();
            increment(&factory_counts_inner, &key);
            let (shared, left, right) = factory_revisions.get();
            let revisions = if key == "a" {
                vec![shared, left]
            } else {
                vec![shared, right]
            };
            let body_counts = body_counts_inner.clone();
            let body_key = key.clone();
            let click_key = key.clone();
            Box::new(
                component(
                    RowProps {
                        key: key.clone(),
                        revisions,
                    },
                    |_| 0usize,
                    |_, _, _| {},
                    move |props: &RowProps, local: &usize| {
                        increment(&body_counts, &body_key);
                        let label = format!("{}:{:?}:{local}", props.key, props.revisions);
                        Box::new(on_click(
                            el::<_, usize, RowClicked>("button", label)
                                .attr("data-row", props.key.clone()),
                            |local: &mut usize, _| {
                                *local += 1;
                                RowClicked
                            },
                        )) as ComponentView<usize, RowClicked>
                    },
                    move |state: &mut SchedulerState, RowClicked| {
                        let _ = &click_key;
                        state.actions += 1;
                    },
                )
                .memo(),
            ) as crate::ErasedComponentView<SchedulerState, ()>
        },
    );

    let state = SchedulerState {
        transaction,
        queue,
        revisions,
        factory_counts,
        body_counts,
        collapsed: BTreeSet::new(),
        actions: 0,
        factory,
    };
    publish_inputs(&state, (1, 1, 1));
    state
}

fn input_vector(key: &str, revisions: (u64, u64, u64)) -> InputVector {
    let (shared, left, right) = revisions;
    let mut dependencies = vec![DependencyStamp::new(1, 9, "shared", shared).unwrap()];
    dependencies.push(if key == "a" {
        DependencyStamp::new(1, 9, "left", left).unwrap()
    } else {
        DependencyStamp::new(1, 9, "right", right).unwrap()
    });
    InputVector::new(
        dependencies,
        vec![LocalInputStamp::new("editor", 0).unwrap()],
    )
    .unwrap()
}

fn inputs(revisions: (u64, u64, u64)) -> CompositionInputs {
    CompositionInputs::new(BTreeMap::from([
        (ComponentKey::from("a"), input_vector("a", revisions)),
        (ComponentKey::from("b"), input_vector("b", revisions)),
    ]))
    .unwrap()
}

fn inputs_only_a(revisions: (u64, u64, u64)) -> CompositionInputs {
    CompositionInputs::new(BTreeMap::from([(
        ComponentKey::from("a"),
        input_vector("a", revisions),
    )]))
    .unwrap()
}

fn publish_inputs(state: &SchedulerState, revisions: (u64, u64, u64)) {
    state.revisions.set(revisions);
    state.queue.publish(inputs(revisions), None).unwrap();
}

fn scheduler_logic(state: &SchedulerState) -> crate::RetainedComposition<SchedulerState, ()> {
    crate::RetainedComposition::new(&state.transaction, state.factory.clone())
        .with_collapsed(state.collapsed.clone())
        .with_inputs(state.queue.capture())
}

fn count(counts: &Rc<RefCell<BTreeMap<String, usize>>>, key: &str) -> usize {
    counts.borrow().get(key).copied().unwrap_or_default()
}

fn paired_state(grouped: bool) -> SchedulerState {
    let placement = if grouped {
        vec![group("panels", vec![row("a"), row("b")])]
    } else {
        vec![row("a"), row("b")]
    };
    scheduler_state(transaction(model(
        placement,
        vec![descriptor("a", "row"), descriptor("b", "row")],
    )))
}

#[test]
fn named_dependencies_skip_unrelated_factory_and_body_and_apply_shared_changes_together() {
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(dom, scheduler_logic, paired_state(false));
    assert_eq!(count(&runner.state().factory_counts, "a"), 1);
    assert_eq!(count(&runner.state().factory_counts, "b"), 1);
    assert_eq!(count(&runner.state().body_counts, "a"), 1);
    assert_eq!(count(&runner.state().body_counts, "b"), 1);

    runner.update(|state| publish_inputs(state, (2, 1, 1)));
    assert_eq!(count(&runner.state().factory_counts, "a"), 2);
    assert_eq!(count(&runner.state().factory_counts, "b"), 2);
    assert_eq!(count(&runner.state().body_counts, "a"), 2);
    assert_eq!(count(&runner.state().body_counts, "b"), 2);

    runner.update(|state| publish_inputs(state, (2, 2, 1)));
    assert_eq!(count(&runner.state().factory_counts, "a"), 3);
    assert_eq!(count(&runner.state().factory_counts, "b"), 2);
    assert_eq!(count(&runner.state().body_counts, "a"), 3);
    assert_eq!(count(&runner.state().body_counts, "b"), 2);
    assert_eq!(runner.state().queue.pending_components(), 0);
}

#[test]
fn unchanged_reorder_refreshes_routes_without_rebuilding_factory_or_body_and_commands_run_once() {
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(dom.clone(), scheduler_logic, paired_state(false));
    let before = find_row(&dom.borrow(), runner.root(), "a").unwrap();
    runner.update(|state| {
        state.transaction = transaction(model(
            vec![row("b"), row("a")],
            vec![descriptor("a", "row"), descriptor("b", "row")],
        ));
    });
    assert_eq!(find_row(&dom.borrow(), runner.root(), "a"), Some(before));
    assert_eq!(count(&runner.state().factory_counts, "a"), 1);
    assert_eq!(count(&runner.state().body_counts, "a"), 1);

    runner.dispatch_click(before, PointerClick::at((1.0, 1.0)));
    assert_eq!(runner.state().actions, 1);
    assert_eq!(count(&runner.state().factory_counts, "a"), 2);
}

#[test]
fn hidden_input_invalidations_defer_then_reveal_catches_up_once() {
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(dom, scheduler_logic, paired_state(true));
    let group_key = ContainerKey {
        kind: ContainerKind::Group,
        key: "panels".into(),
    };
    runner.update(|state| {
        state.collapsed.insert(group_key.clone());
    });
    let builds_a = count(&runner.state().factory_counts, "a");
    let builds_b = count(&runner.state().factory_counts, "b");

    runner.update(|state| publish_inputs(state, (2, 1, 1)));
    assert_eq!(count(&runner.state().factory_counts, "a"), builds_a);
    assert_eq!(count(&runner.state().factory_counts, "b"), builds_b);
    assert_eq!(runner.state().queue.pending_components(), 2);

    runner.update(|state| {
        state.collapsed.clear();
    });
    assert_eq!(count(&runner.state().factory_counts, "a"), builds_a + 1);
    assert_eq!(count(&runner.state().factory_counts, "b"), builds_b + 1);
    assert_eq!(runner.state().queue.pending_components(), 0);
}

#[test]
fn hidden_same_vector_invalidation_is_still_evaluated_on_reveal() {
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(dom, scheduler_logic, paired_state(true));
    let group_key = ContainerKey {
        kind: ContainerKind::Group,
        key: "panels".into(),
    };
    runner.update(|state| {
        state.collapsed.insert(group_key.clone());
    });
    let builds_a = count(&runner.state().factory_counts, "a");
    let builds_b = count(&runner.state().factory_counts, "b");
    let same_inputs = inputs((1, 1, 1));
    runner.update(|state| {
        state
            .queue
            .publish(
                same_inputs,
                Some(CompositionCauseRef {
                    producer_id: 90,
                    sequence: 1,
                }),
            )
            .unwrap();
    });
    assert_eq!(count(&runner.state().factory_counts, "a"), builds_a);
    assert_eq!(count(&runner.state().factory_counts, "b"), builds_b);
    assert_eq!(runner.state().queue.pending_components(), 2);

    runner.update(|state| {
        state.collapsed.clear();
    });
    assert_eq!(count(&runner.state().factory_counts, "a"), builds_a + 1);
    assert_eq!(count(&runner.state().factory_counts, "b"), builds_b + 1);
    assert_eq!(runner.state().queue.pending_components(), 0);
}

#[test]
fn visible_same_vector_invalidation_runs_the_component_evaluation() {
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(dom, scheduler_logic, paired_state(false));
    runner.update(|state| {
        state
            .queue
            .publish(
                inputs((1, 1, 1)),
                Some(CompositionCauseRef {
                    producer_id: 90,
                    sequence: 2,
                }),
            )
            .unwrap();
    });

    assert_eq!(count(&runner.state().factory_counts, "a"), 2);
    assert_eq!(count(&runner.state().factory_counts, "b"), 2);
    assert_eq!(runner.state().queue.pending_components(), 0);
}

#[test]
fn missing_component_input_vector_is_not_treated_as_an_empty_clean_vector() {
    let mut state = paired_state(false);
    state.queue.publish(inputs_only_a((1, 1, 1)), None).unwrap();
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(dom, scheduler_logic, state);
    assert_eq!(count(&runner.state().factory_counts, "a"), 1);
    assert_eq!(count(&runner.state().factory_counts, "b"), 1);

    runner.update(|_| {});
    assert_eq!(count(&runner.state().factory_counts, "a"), 1);
    assert_eq!(count(&runner.state().factory_counts, "b"), 2);
}

#[test]
fn latest_nonempty_mutation_batch_is_shared_and_visibility_tracks_parking() {
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(dom, scheduler_logic, paired_state(true));
    let initial = runner.state().queue.latest_mutations();
    assert_eq!(initial.len(), 2);
    assert!(initial.iter().all(|record| {
        record.mutation_cursor_end > record.mutation_cursor_start && record.visible
    }));
    assert_eq!(initial[0].reference, initial[1].reference);
    let first_reference = initial[0].reference;

    runner.update(|_| {});
    let steady = runner.state().queue.latest_mutations();
    assert_eq!(steady[0].reference, first_reference);
    assert_eq!(steady[1].reference, first_reference);

    runner.update(|state| {
        state.collapsed.insert(ContainerKey {
            kind: ContainerKind::Group,
            key: "panels".into(),
        });
    });
    let parked = runner.state().queue.latest_mutations();
    assert_eq!(parked.len(), 2);
    assert!(parked.iter().all(|record| !record.visible));
    assert!(
        parked
            .iter()
            .all(|record| record.reference != first_reference)
    );
}
