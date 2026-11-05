// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn compatibility_summaries_do_not_change_the_default_layout() {
    let assertion = |id: &str| {
        serde_json::from_value(serde_json::json!({
            "id": id, "predicate": "urn:test:related", "recognized_kind": null,
            "label": null, "scope": {"kind": "default"}, "asserter": null,
            "first_asserted_ms": 1, "asserted_at_ms": 1
        }))
        .unwrap()
    };
    let mut reading = LinkPairReading {
        source: LinkSource::Unavailable {
            reason: "fixture".into(),
        },
        links: vec![LinkReading {
            key: LinkKey::new(LinkStratum::Resource, "a", "b"),
            buckets: vec![LinkBucket {
                displayed_from: "a".into(),
                displayed_to: "b".into(),
                source_from: LinkEndpoint {
                    id: "a".into(),
                    address: None,
                },
                source_to: LinkEndpoint {
                    id: "b".into(),
                    address: None,
                },
                assertions: vec![assertion("one"), assertion("two")],
                legacy_members: vec![LinkLegacyMember {
                    family: "Traversal".into(),
                    sub_kind: None,
                }],
                legacy_summaries: vec![],
            }],
        }],
        coverage: LinkCoverage::default(),
    };
    let before = OpenedLinkLayout::default_for(&reading);
    assert!(matches!(before, OpenedLinkLayout::Timeline { .. }));
    reading.links[0].buckets[0]
        .legacy_summaries
        .push(LinkLegacyFamily {
            family: "Traversal".into(),
            data: serde_json::json!({}),
        });
    assert_eq!(OpenedLinkLayout::default_for(&reading), before);
    // Real undated members still affect the majority; a tie uses List.
    reading.links[0].buckets[0]
        .legacy_members
        .push(LinkLegacyMember {
            family: "Traversal".into(),
            sub_kind: None,
        });
    assert!(matches!(
        OpenedLinkLayout::default_for(&reading),
        OpenedLinkLayout::List { .. }
    ));
    // Summary-only readings retain their original undated contribution.
    reading.links[0].buckets[0].legacy_members.clear();
    reading.links[0].buckets[0]
        .legacy_summaries
        .push(LinkLegacyFamily {
            family: "other-family".into(),
            data: serde_json::json!({}),
        });
    assert!(matches!(
        OpenedLinkLayout::default_for(&reading),
        OpenedLinkLayout::List { .. }
    ));
}

#[test]
fn keys_are_unordered_within_their_stratum() {
    assert_eq!(
        LinkKey::new(LinkStratum::Resource, "urn:z", "urn:a"),
        LinkKey::new(LinkStratum::Resource, "urn:a", "urn:z")
    );
    assert_ne!(
        LinkKey::new(LinkStratum::Resource, "urn:a", "urn:z"),
        LinkKey::new(LinkStratum::Surface, "urn:a", "urn:z")
    );
}

#[test]
fn source_and_revision_availability_round_trip_explicitly() {
    let states = [
        LinkSource::Unavailable {
            reason: "no owning authority".into(),
        },
        LinkSource::Available {
            binding: SourceBinding {
                authority: "mere:test".into(),
                domain: "graph".into(),
                resource: "g1".into(),
            },
            revision: LinkSourceRevision::Unavailable {
                reason: "no public revision".into(),
            },
        },
        LinkSource::Available {
            binding: SourceBinding {
                authority: "mere:test".into(),
                domain: "graph".into(),
                resource: "g1".into(),
            },
            revision: LinkSourceRevision::Available {
                revision: PublicSourceRevision::new("r7"),
            },
        },
    ];
    for state in states {
        let encoded = serde_json::to_vec(&state).unwrap();
        assert_eq!(
            serde_json::from_slice::<LinkSource>(&encoded).unwrap(),
            state
        );
    }
}

#[test]
fn prior_p1_assertion_json_defaults_new_lifecycle_and_basis_fields() {
    let old = serde_json::json!({
        "id": "urn:assertion:old",
        "predicate": "https://example.test/cites",
        "recognized_kind": "cites",
        "label": "an existing claim",
        "scope": {"kind": "default"},
        "asserter": "urn:agent:alice",
        "asserted_at_ms": 42
    });

    let decoded: LinkAssertion = serde_json::from_value(old).unwrap();
    assert_eq!(decoded.asserted_at_ms, Some(42));
    assert_eq!(decoded.first_asserted_ms, None);
    assert_eq!(decoded.withdrawn, None);
    assert_eq!(decoded.basis, LinkAssertionBasis::default());
}

#[test]
fn custom_assertion_scope_round_trips_without_normalization() {
    let scope = LinkGraphScope::Custom("urn:scope:review".into());
    let json = serde_json::to_string(&scope).expect("writes");
    assert_eq!(
        serde_json::from_str::<LinkGraphScope>(&json).expect("reads"),
        scope
    );
}
#[test]
fn rest_summary_preserves_independent_reasons_and_excludes_withdrawal() {
    let assertion = |id: &str, predicate: &str, withdrawn: bool| -> LinkAssertion {
        serde_json::from_value(serde_json::json!({
            "id":id,"predicate":predicate,"recognized_kind":null,"label":null,
            "scope":{"kind":"default"},"asserter":null,"asserted_at_ms":null,
            "withdrawn":if withdrawn {serde_json::json!({"at_ms":7,"by":null})} else {serde_json::Value::Null}
        })).unwrap()
    };
    let bucket = |from: &str, to: &str, assertions| LinkBucket {
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
        assertions,
        legacy_members: vec![],
        legacy_summaries: vec![],
    };
    let mut reading = LinkReading {
        key: LinkKey::new(LinkStratum::Resource, "urn:a", "urn:b"),
        buckets: vec![
            bucket(
                "surface:a",
                "surface:b",
                vec![
                    assertion("one", "https://mere.computer/ns/rel#supports", false),
                    assertion("two", "https://mere.computer/ns/rel#supports", false),
                ],
            ),
            bucket(
                "surface:b",
                "surface:a",
                vec![assertion(
                    "ended",
                    "https://mere.computer/ns/rel#contradicts",
                    true,
                )],
            ),
        ],
    };
    let before = reading.clone();
    let summary = reading.rest_summary("surface:a", "surface:b");
    assert_eq!(summary.held_assertions, 2);
    assert_eq!(summary.strength, 2.0);
    assert_eq!(
        summary.marks,
        LinkMarks {
            forward: true,
            reverse: false,
            opposed: false
        }
    );
    assert_eq!(
        reading, before,
        "rest drawing cannot change assertion history"
    );
    reading.buckets[1].assertions.push(assertion(
        "three",
        "https://mere.computer/ns/rel#contradicts",
        false,
    ));
    let summary = reading.rest_summary("surface:a", "surface:b");
    assert_eq!(summary.strength, 3.0);
    assert_eq!(
        summary.marks,
        LinkMarks {
            forward: true,
            reverse: true,
            opposed: true
        }
    );
    assert_eq!(
        reading.rest_summary("unrelated:a", "unrelated:b"),
        LinkRestSummary::default()
    );
    assert_eq!(
        reading.rest_summary("surface:b", "surface:a").marks,
        summary.marks.reversed()
    );
    reading.buckets[0].legacy_members.push(LinkLegacyMember {
        family: "Traversal".into(),
        sub_kind: None,
    });
    reading.buckets[0].legacy_summaries.push(LinkLegacyFamily {
        family: "Traversal".into(),
        data: serde_json::json!({}),
    });
    let summary = reading.rest_summary("surface:a", "surface:b");
    assert_eq!(
        (
            summary.held_assertions,
            summary.legacy_members,
            summary.strength
        ),
        (3, 1, 4.0)
    );
}
