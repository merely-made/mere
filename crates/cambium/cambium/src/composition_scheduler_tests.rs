// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeMap;

use crate::composition::ComponentKey;
use crate::reactive_diagnostics::Subject;

use super::{
    CompositionCauseRef, CompositionInputError, CompositionInputQueue, CompositionInputs,
    DependencyStamp, InputVector, LocalInputStamp, MutationRecord,
};

fn stamp(domain: &str, revision: u64) -> DependencyStamp {
    DependencyStamp::new(7, 11, domain, revision).unwrap()
}

fn vector(shared: u64, local: u64) -> InputVector {
    InputVector::new(
        vec![stamp("appearance", shared)],
        vec![LocalInputStamp::new("draft", local).unwrap()],
    )
    .unwrap()
}

fn inputs(shared: u64, local: u64) -> CompositionInputs {
    CompositionInputs::new(BTreeMap::from([(
        ComponentKey::from("row-a"),
        vector(shared, local),
    )]))
    .unwrap()
}

fn two_inputs(shared: u64) -> CompositionInputs {
    CompositionInputs::new(BTreeMap::from([
        (ComponentKey::from("row-a"), vector(shared, 0)),
        (ComponentKey::from("row-b"), vector(shared, 0)),
    ]))
    .unwrap()
}

fn cause(sequence: u64) -> CompositionCauseRef {
    CompositionCauseRef {
        producer_id: 20,
        sequence,
    }
}

#[test]
fn publishes_coalesce_to_the_latest_vector_and_bounded_causes() {
    let queue = CompositionInputQueue::new().unwrap();
    let key = ComponentKey::from("row-a");
    queue.publish(inputs(1, 1), Some(cause(1))).unwrap();
    queue.publish(inputs(2, 2), Some(cause(2))).unwrap();
    queue.publish(inputs(3, 3), Some(cause(3))).unwrap();

    let snapshot = queue.capture();
    assert_eq!(snapshot.inputs().vector(&key), Some(&vector(3, 3)));
    assert_eq!(snapshot.pending_notices(&key), Some(3));
    assert_eq!(
        snapshot.pending_causes(&key),
        &[cause(1), cause(2), cause(3)]
    );
    assert_eq!(queue.pending_components(), 1);

    snapshot.acknowledge(&[key].into_iter().collect());
    assert_eq!(queue.pending_components(), 0);
}

#[test]
fn ack_only_removes_the_captured_generation_when_a_new_publish_arrives() {
    let queue = CompositionInputQueue::new().unwrap();
    let key = ComponentKey::from("row-a");
    queue.publish(inputs(1, 1), Some(cause(1))).unwrap();
    let first = queue.capture();
    queue.publish(inputs(2, 1), Some(cause(2))).unwrap();

    first.acknowledge(&[key.clone()].into_iter().collect());
    assert_eq!(queue.pending_components(), 1);
    let second = queue.capture();
    assert_eq!(second.inputs().vector(&key), Some(&vector(2, 1)));
    assert_eq!(second.pending_notices(&key), Some(2));
    second.acknowledge(&[key].into_iter().collect());
    assert_eq!(queue.pending_components(), 0);
}

#[test]
fn unchanged_publication_does_not_create_an_extra_invalidation_notice() {
    let queue = CompositionInputQueue::new().unwrap();
    let key = ComponentKey::from("row-a");
    queue.publish(inputs(1, 1), Some(cause(1))).unwrap();
    queue.publish(inputs(1, 1), None).unwrap();

    let snapshot = queue.capture();
    assert_eq!(snapshot.pending_notices(&key), Some(1));
    assert_eq!(snapshot.pending_causes(&key), &[cause(1)]);
}

#[test]
fn same_vector_with_new_cause_is_counted_as_an_explicit_invalidation() {
    let queue = CompositionInputQueue::new().unwrap();
    let key = ComponentKey::from("row-a");
    queue.publish(inputs(1, 1), Some(cause(1))).unwrap();
    queue.publish(inputs(1, 1), Some(cause(2))).unwrap();

    let snapshot = queue.capture();
    assert_eq!(snapshot.pending_notices(&key), Some(2));
    assert_eq!(snapshot.pending_causes(&key), &[cause(1), cause(2)]);
}

#[test]
fn atomic_causes_are_scoped_to_their_component_and_empty_causes_are_valid() {
    let queue = CompositionInputQueue::new().unwrap();
    let row_a = ComponentKey::from("row-a");
    let row_b = ComponentKey::from("row-b");
    queue
        .publish_with_causes(
            two_inputs(1),
            BTreeMap::from([(row_a.clone(), vec![cause(1)]), (row_b.clone(), vec![])]),
        )
        .unwrap();

    let snapshot = queue.capture();
    assert_eq!(snapshot.pending_causes(&row_a), &[cause(1)]);
    assert!(snapshot.pending_causes(&row_b).is_empty());
    assert_eq!(snapshot.pending_notices(&row_a), Some(1));
    assert_eq!(snapshot.pending_notices(&row_b), Some(1));
}

#[test]
fn invalid_atomic_cause_map_is_rejected_without_publishing_any_inputs() {
    let queue = CompositionInputQueue::new().unwrap();
    let unknown = ComponentKey::from("not-live");
    assert_eq!(
        queue.publish_with_causes(inputs(1, 1), BTreeMap::from([(unknown, vec![cause(1)])]),),
        Err(CompositionInputError::CauseForUnknownComponent)
    );
    assert_eq!(queue.pending_components(), 0);
    assert_eq!(queue.capture().inputs().components().count(), 0);
}

#[test]
fn removing_a_component_from_the_complete_table_prunes_its_pending_invalidation() {
    let queue = CompositionInputQueue::new().unwrap();
    let key = ComponentKey::from("row-a");
    queue.publish(inputs(1, 1), Some(cause(1))).unwrap();
    queue.record_mutation(
        key,
        MutationRecord {
            reference: cause(9),
            subject: Subject {
                owner_id: 1,
                slot: 2,
                route_generation: 3,
                placement_generation: 4,
            },
            mutation_cursor_start: 10,
            mutation_cursor_end: 11,
            visible: true,
        },
    );
    assert_eq!(queue.pending_components(), 1);
    assert_eq!(queue.latest_mutations().len(), 1);

    queue.publish(CompositionInputs::default(), None).unwrap();
    assert_eq!(queue.pending_components(), 0);
    assert!(queue.latest_mutations().is_empty());
    assert_eq!(queue.capture().inputs().components().count(), 0);
}

#[test]
fn generation_exhaustion_is_counted_and_refuses_without_mutating_latest() {
    let queue = CompositionInputQueue::new().unwrap();
    queue.0.borrow_mut().generation = u64::MAX;
    assert_eq!(
        queue.publish(inputs(1, 1), Some(cause(1))),
        Err(CompositionInputError::GenerationExhausted)
    );
    assert_eq!(queue.generation_exhaustions(), 1);
    assert_eq!(queue.pending_components(), 0);
    assert_eq!(queue.capture().inputs().components().count(), 0);
}

#[test]
fn queued_causes_are_bounded_and_omissions_are_counted() {
    let queue = CompositionInputQueue::new().unwrap();
    for revision in 1..=10 {
        queue
            .publish(inputs(revision, revision), Some(cause(revision)))
            .unwrap();
    }
    let snapshot = queue.capture();
    let key = ComponentKey::from("row-a");
    assert_eq!(snapshot.pending_causes(&key).len(), 8);
    assert_eq!(snapshot.omitted_causes(&key), 2);
}

#[test]
fn input_names_and_vectors_are_validated_and_sorted() {
    assert_eq!(
        DependencyStamp::new(1, 2, "", 0).unwrap_err(),
        CompositionInputError::InvalidName
    );
    assert_eq!(
        InputVector::new(vec![stamp("appearance", 1), stamp("appearance", 2)], vec![]).unwrap_err(),
        CompositionInputError::DuplicateDependency
    );
    assert_eq!(
        InputVector::new(
            vec![
                DependencyStamp::new(1, 1, "appearance", 1).unwrap(),
                DependencyStamp::new(1, 2, "layout", 1).unwrap(),
                DependencyStamp::new(1, 3, "appearance", 2).unwrap(),
            ],
            vec![],
        )
        .unwrap_err(),
        CompositionInputError::DuplicateDependency
    );
    assert_eq!(
        InputVector::new(
            vec![],
            vec![
                LocalInputStamp::new("draft", 1).unwrap(),
                LocalInputStamp::new("draft", 2).unwrap(),
            ],
        )
        .unwrap_err(),
        CompositionInputError::DuplicateLocalInput
    );
}
