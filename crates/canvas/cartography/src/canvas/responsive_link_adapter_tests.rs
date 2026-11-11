// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use scenograph::link::{LinkBucket, LinkEndpoint, LinkReading, LinkSource, LinkStratum};
use std::cell::Cell;

fn bucket(from: &str, to: &str, assertion: &str) -> LinkBucket {
    LinkBucket {
        displayed_from: from.into(),
        displayed_to: to.into(),
        source_from: LinkEndpoint {
            id: format!("source:{from}"),
            address: None,
        },
        source_to: LinkEndpoint {
            id: format!("source:{to}"),
            address: None,
        },
        assertions: vec![scenograph::link::LinkAssertion {
            id: assertion.into(),
            predicate: "urn:related".into(),
            recognized_kind: None,
            label: None,
            scope: scenograph::link::LinkGraphScope::Default,
            asserter: None,
            first_asserted_ms: None,
            asserted_at_ms: None,
            withdrawn: None,
            basis: scenograph::link::LinkAssertionBasis::default(),
        }],
        legacy_members: vec![],
        legacy_summaries: vec![],
    }
}

fn reading(key_first: &str, key_second: &str, assertion: &str) -> LinkPairReading {
    LinkPairReading {
        source: LinkSource::Unavailable {
            reason: "fixture has no public source binding".into(),
        },
        links: vec![LinkReading {
            key: LinkKey::new(LinkStratum::Resource, key_first, key_second),
            buckets: vec![
                bucket("surface-a", "surface-b", &format!("{assertion}:forward")),
                bucket("surface-b", "surface-a", &format!("{assertion}:reverse")),
            ],
        }],
        coverage: Default::default(),
    }
}

fn settings() -> ResponsivePathConfig {
    ResponsivePathConfig {
        control_points: 4,
        iterations: 1,
        field_response: false,
        compatible_attraction: true,
        ..ResponsivePathConfig::default()
    }
}

#[test]
fn cohort_keeps_each_link_appearance_and_reversible_source_identity() {
    let source = [
        reading("resource-a", "resource-b", "assertion-one"),
        reading("resource-a", "resource-b", "assertion-two"),
    ];
    let registry = FieldRegistry::default();
    let result = evaluate_link_appearances(
        &source,
        &[],
        |id| match id {
            "surface-a" => Some((0.0, 0.0)),
            "surface-b" => Some((100.0, 40.0)),
            _ => None,
        },
        &[],
        &registry,
        settings(),
    )
    .unwrap();

    assert_eq!(result.paths.len(), 2);
    assert_eq!(result.paths[0].id, 0);
    assert_eq!(result.paths[1].id, 1);
    assert_eq!(
        result.appearances[0].link_key,
        result.appearances[1].link_key
    );
    assert_ne!(
        result.appearances[0].reading_index,
        result.appearances[1].reading_index
    );
    assert_eq!(result.appearances[0].buckets.len(), 2);
    let first_ref = result.appearances[0].buckets[0];
    let second_ref = result.appearances[1].buckets[0];
    assert_eq!(
        result.readings[first_ref.reading_index].links[first_ref.link_index].buckets
            [first_ref.bucket_index]
            .assertions[0]
            .id,
        "assertion-one:forward"
    );
    assert_eq!(
        result.readings[second_ref.reading_index].links[second_ref.link_index].buckets
            [second_ref.bucket_index]
            .assertions[0]
            .id,
        "assertion-two:forward"
    );
    assert_eq!(result.endpoints.len(), 2);
    assert_eq!(result.endpoints[0].displayed_id, "surface-a");
    assert_eq!(result.endpoints[1].displayed_id, "surface-b");
}

#[test]
fn compatibility_is_explicit_and_uses_the_full_appearance_selector() {
    let source = [
        reading("resource-a", "resource-b", "assertion-one"),
        reading("resource-c", "resource-d", "assertion-two"),
    ];
    let key = &source[0].links[0].key;
    let declarations = [AppearanceCompatibility {
        reading_index: 0,
        link_key: key,
        displayed_first: "surface-a",
        displayed_second: "surface-b",
        compatibility: "urn:compatible:declared",
    }];
    let registry = FieldRegistry::default();
    let result = evaluate_link_appearances(
        &source,
        &declarations,
        |id| match id {
            "surface-a" => Some((0.0, 0.0)),
            "surface-b" => Some((100.0, 40.0)),
            _ => None,
        },
        &[],
        &registry,
        settings(),
    )
    .unwrap();
    assert_eq!(result.appearances.len(), 2);
    assert_eq!(result.appearances[0].reading_index, 0);
    assert_eq!(result.appearances[1].reading_index, 1);
    assert_eq!(
        result.appearances[0].compatibility.as_deref(),
        Some("urn:compatible:declared")
    );
    assert_eq!(result.appearances[1].compatibility.as_deref(), None);
}

#[test]
fn invalid_or_unbounded_source_cohorts_are_refused_before_evaluation() {
    let source = [reading("resource-a", "resource-b", "assertion-one")];
    let registry = FieldRegistry::default();
    assert!(matches!(
        evaluate_link_appearances(&source, &[], |_| None, &[], &registry, settings(),),
        Err(ResponsiveLinkAdapterError::MissingDisplayedPosition)
    ));

    let many =
        vec![reading("resource-a", "resource-b", "assertion"); MAX_RESPONSIVE_APPEARANCES + 1];
    assert!(matches!(
        evaluate_link_appearances(&many, &[], |_| Some((0.0, 0.0)), &[], &registry, settings(),),
        Err(ResponsiveLinkAdapterError::TooManyReadings)
    ));

    let mut one_reading = reading("resource-a", "resource-b", "assertion");
    let template = one_reading.links[0].clone();
    one_reading.links = (0..=MAX_RESPONSIVE_APPEARANCES)
        .map(|_| template.clone())
        .collect();
    assert!(matches!(
        evaluate_link_appearances(
            &[one_reading],
            &[],
            |_| Some((0.0, 0.0)),
            &[],
            &registry,
            settings(),
        ),
        Err(ResponsiveLinkAdapterError::TooManyAppearances)
    ));
}

#[test]
fn source_context_must_match_and_each_displayed_endpoint_position_is_resolved_once() {
    let source = [
        reading("resource-a", "resource-b", "assertion-one"),
        reading("resource-c", "resource-d", "assertion-two"),
    ];
    let calls = Cell::new(0);
    let registry = FieldRegistry::default();
    let result = evaluate_link_appearances(
        &source,
        &[],
        |_| {
            let call = calls.get();
            calls.set(call + 1);
            // A changing resolver would make shared endpoint handles disagree
            // if the adapter did not cache the first result.
            Some((call as f32, 10.0 + call as f32))
        },
        &[],
        &registry,
        settings(),
    )
    .unwrap();
    assert_eq!(calls.get(), 2);
    assert_eq!(
        result.appearances[0].start_endpoint_handle,
        result.appearances[1].start_endpoint_handle
    );
    assert_eq!(
        result.appearances[0].end_endpoint_handle,
        result.appearances[1].end_endpoint_handle
    );
    assert_eq!(result.endpoints[0].position, (0.0, 10.0));
    assert_eq!(result.endpoints[1].position, (1.0, 11.0));
    assert_eq!(result.paths[0].points[0], result.paths[1].points[0]);

    let mut mismatched = source.clone();
    mismatched[1].source = LinkSource::Unavailable {
        reason: "different source context".into(),
    };
    assert!(matches!(
        evaluate_link_appearances(
            &mismatched,
            &[],
            |_| Some((0.0, 0.0)),
            &[],
            &registry,
            settings(),
        ),
        Err(ResponsiveLinkAdapterError::SourceMismatch)
    ));
}

#[test]
fn empty_tombstone_bucket_cannot_become_a_responsive_appearance() {
    let mut source = [reading("resource-a", "resource-b", "assertion-one")];
    source[0].links[0].buckets[0].assertions.clear();
    source[0].links[0].buckets[0].legacy_members.clear();
    let registry = FieldRegistry::default();
    assert!(matches!(
        evaluate_link_appearances(
            &source,
            &[],
            |_| Some((0.0, 0.0)),
            &[],
            &registry,
            settings(),
        ),
        Err(ResponsiveLinkAdapterError::EmptyAppearance)
    ));
}
