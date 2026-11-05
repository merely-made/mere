// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use sceno::Stack;
use scenograph::{
    PublicSourceRevision, SourceBinding,
    link::{
        LinkAssertionBasis, LinkBucket, LinkCoverage, LinkCoverageLayer, LinkCoverageLimit,
        LinkEndpoint, LinkGraphScope, LinkKey, LinkLegacyMember, LinkReading, LinkSource,
        LinkSourceRevision, LinkStratum, LinkWithdrawal,
    },
};

fn reading() -> LinkPairReading {
    let assertion = |id: &str,
                     at: Option<u64>,
                     end: Option<Option<u64>>,
                     asserter: Option<&str>| LinkAssertion {
        id: id.into(),
        predicate: "urn:pred:related".into(),
        recognized_kind: Some("citation".into()),
        label: None,
        scope: LinkGraphScope::Default,
        asserter: asserter.map(str::to_owned),
        first_asserted_ms: at,
        asserted_at_ms: at,
        withdrawn: end.map(|at_ms| LinkWithdrawal { at_ms, by: None }),
        basis: LinkAssertionBasis::default(),
    };
    LinkPairReading {
        source: LinkSource::Available {
            binding: SourceBinding {
                authority: "mere.test".into(),
                domain: "graph".into(),
                resource: "g1".into(),
            },
            revision: LinkSourceRevision::Available {
                revision: PublicSourceRevision::new("rev-7"),
            },
        },
        links: vec![
            LinkReading {
                key: LinkKey::new(LinkStratum::Surface, "a", "b"),
                buckets: vec![
                    LinkBucket {
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
                        assertions: vec![
                            assertion("id-1", Some(10), None, Some("writer-a")),
                            assertion("id-2", Some(10), Some(Some(30)), None),
                        ],
                        legacy_members: vec![LinkLegacyMember {
                            family: "old-family".into(),
                            sub_kind: None,
                        }],
                        legacy_summaries: Vec::new(),
                    },
                    LinkBucket {
                        displayed_from: "b".into(),
                        displayed_to: "a".into(),
                        source_from: LinkEndpoint {
                            id: "b".into(),
                            address: None,
                        },
                        source_to: LinkEndpoint {
                            id: "a".into(),
                            address: None,
                        },
                        assertions: vec![assertion("id-3", None, None, None)],
                        legacy_members: Vec::new(),
                        legacy_summaries: Vec::new(),
                    },
                    LinkBucket {
                        displayed_from: "a".into(),
                        displayed_to: "b".into(),
                        source_from: LinkEndpoint {
                            id: "a-parallel".into(),
                            address: None,
                        },
                        source_to: LinkEndpoint {
                            id: "b-parallel".into(),
                            address: None,
                        },
                        assertions: vec![assertion("id-5", Some(10), None, Some("writer-a"))],
                        legacy_members: Vec::new(),
                        legacy_summaries: Vec::new(),
                    },
                ],
            },
            LinkReading {
                key: LinkKey::new(LinkStratum::Resource, "urn:r:a", "urn:r:b"),
                buckets: vec![LinkBucket {
                    displayed_from: "a".into(),
                    displayed_to: "b".into(),
                    source_from: LinkEndpoint {
                        id: "ra".into(),
                        address: Some("urn:r:a".into()),
                    },
                    source_to: LinkEndpoint {
                        id: "rb".into(),
                        address: Some("urn:r:b".into()),
                    },
                    assertions: vec![assertion("id-4", Some(20), None, Some("writer-b"))],
                    legacy_members: Vec::new(),
                    legacy_summaries: Vec::new(),
                }],
            },
        ],
        coverage: LinkCoverage {
            limits: vec![LinkCoverageLimit {
                layer: LinkCoverageLayer::Disclosure,
                reason: "partial fixture".into(),
                resources: Vec::new(),
                surfaces: vec!["a".into()],
                count: Some(1),
            }],
        },
    }
}

#[test]
fn compatibility_summaries_are_not_extra_members() {
    let mut input = reading();
    input.links[0].buckets[0]
        .legacy_summaries
        .push(scenograph::link::LinkLegacyFamily {
            family: "old-family".into(),
            data: serde_json::json!({"details": "inspector only"}),
        });
    for layout in [
        OpenedLinkLayout::List {
            group_by: scenograph::link::LinkListGroup::Family,
            stack: Stack::default(),
        },
        OpenedLinkLayout::Timeline {
            lane_by: LinkTimelineLane::Family,
            timeline: Timeline::default(),
        },
    ] {
        let opened = compile_opened_link(&input, &layout, OpenedLinkOptions::default()).unwrap();
        assert_eq!(opened.entries.len(), 6);
        assert_eq!(opened.scene.items.len(), 6);
        assert_eq!(opened.reading, input, "inspector details stay available");
        assert!(!opened.entries.iter().any(|entry| matches!(
            opened.resolve(entry),
            Some(OpenedLinkMember::LegacySummary(_))
        )));
        // Old summary-only readings still have a read-only display row.
        let mut summary_only = input.clone();
        summary_only.links[0].buckets[0].legacy_members.clear();
        let opened =
            compile_opened_link(&summary_only, &layout, OpenedLinkOptions::default()).unwrap();
        assert_eq!(opened.entries.len(), 6);
        assert!(opened.entries.iter().any(|entry| matches!(
            opened.resolve(entry),
            Some(OpenedLinkMember::LegacySummary(_))
        )));
    }
}

#[test]
fn default_layout_counts_legacy_and_undated_members_as_undated() {
    let input = reading();
    assert!(matches!(
        OpenedLinkLayout::default_for(&input),
        OpenedLinkLayout::Timeline { .. }
    ));
    let mut legacy_majority = input.clone();
    legacy_majority.links[0].buckets[0]
        .legacy_members
        .push(LinkLegacyMember {
            family: "another-legacy-family".into(),
            sub_kind: None,
        });
    legacy_majority.links[0].buckets[0]
        .legacy_members
        .push(LinkLegacyMember {
            family: "one-more-legacy-family".into(),
            sub_kind: None,
        });
    assert!(matches!(
        OpenedLinkLayout::default_for(&legacy_majority),
        OpenedLinkLayout::List { .. }
    ));
    let mut all_dated = input.clone();
    for link in &mut all_dated.links {
        for bucket in &mut link.buckets {
            bucket.legacy_members.clear();
            bucket.legacy_summaries.clear();
            for assertion in &mut bucket.assertions {
                assertion.first_asserted_ms = Some(10);
            }
        }
    }
    assert!(matches!(
        OpenedLinkLayout::default_for(&all_dated),
        OpenedLinkLayout::Timeline { .. }
    ));
}

#[test]
fn list_uses_stack_and_groups_by_source_without_merging_members() {
    let input = reading();
    let layout = OpenedLinkLayout::List {
        group_by: scenograph::link::LinkListGroup::Source,
        stack: Stack::default(),
    };
    let opened = compile_opened_link(&input, &layout, OpenedLinkOptions::default()).unwrap();
    assert_eq!(opened.entries.len(), 6);
    assert_eq!(opened.lanes.len(), 2);
    assert_eq!(opened.scene.items.len(), 6);
    assert_eq!(opened.reading.source, input.source);
    assert_eq!(opened.reading.coverage, input.coverage);
}

#[test]
fn link_scope_compiles_only_matching_stratum_and_keeps_original_provenance() {
    let input = reading();
    let selected = input.links[1].key.clone();
    let scope = Scope::Link(selected.clone());
    let layout = OpenedLinkLayout::List {
        group_by: scenograph::link::LinkListGroup::Source,
        stack: Stack::default(),
    };
    let opened =
        compile_opened_link_scope(&scope, &input, &layout, OpenedLinkOptions::default()).unwrap();

    assert_eq!(opened.reading, input, "retain the whole source disclosure");
    assert_eq!(opened.reading.links.len(), 2);
    assert_eq!(opened.reading.coverage, reading().coverage);
    assert_eq!(opened.entries.len(), 1);
    let entry = &opened.entries[0];
    assert_eq!(entry.source.link_index, 1, "keep the original source index");
    assert_eq!(opened.reading.links[entry.source.link_index].key, selected);
    assert!(matches!(
        opened.resolve(entry),
        Some(OpenedLinkMember::Assertion(assertion)) if assertion.id == "id-4"
    ));
}

#[test]
fn link_scope_rejects_wrong_scope_and_missing_key() {
    let input = reading();
    let layout = OpenedLinkLayout::List {
        group_by: scenograph::link::LinkListGroup::Family,
        stack: Stack::default(),
    };
    assert_eq!(
        compile_opened_link_scope(
            &Scope::Node("a".into()),
            &input,
            &layout,
            OpenedLinkOptions::default(),
        )
        .unwrap_err(),
        OpenedLinkCompileError::WrongScope
    );
    assert_eq!(
        compile_opened_link_scope(
            &Scope::Link(LinkKey::new(LinkStratum::Surface, "missing-a", "missing-b")),
            &input,
            &layout,
            OpenedLinkOptions::default(),
        )
        .unwrap_err(),
        OpenedLinkCompileError::MissingLink
    );
}

#[path = "tests/timeline.rs"]
mod timeline;
