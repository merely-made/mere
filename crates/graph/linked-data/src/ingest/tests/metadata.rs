// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn single_valued_lifecycle_metadata_rejects_conflicts_independent_of_quad_order() {
    use oxrdf::{Literal, NamedNode, Triple};

    let named = |iri: &str| Term::NamedNode(NamedNode::new(iri).unwrap());
    let literal = |value: &str| Term::Literal(Literal::new_simple_literal(value));
    let datetime = |value: &str| {
        Term::Literal(Literal::new_typed_literal(
            value,
            NamedNode::new_unchecked("http://www.w3.org/2001/XMLSchema#dateTime"),
        ))
    };
    let conflicts = [
        (
            PROV_GENERATED_AT_TIME,
            datetime("1970-01-01T00:00:00Z"),
            datetime("1970-01-01T00:00:01Z"),
            "first assertion time",
        ),
        (
            MERE_LAST_ASSERTED_AT,
            datetime("1970-01-01T00:00:00Z"),
            datetime("1970-01-01T00:00:01Z"),
            "last assertion time",
        ),
        (
            PROV_INVALIDATED_AT_TIME,
            datetime("1970-01-01T00:00:00Z"),
            datetime("1970-01-01T00:00:01Z"),
            "withdrawal time",
        ),
        (
            PROV_WAS_GENERATED_BY,
            named("https://method.test/one"),
            named("https://method.test/two"),
            "method IRI",
        ),
        (
            PROV_WAS_ATTRIBUTED_TO,
            named("urn:actor:asserted-one"),
            named("urn:actor:asserted-two"),
            "asserted provenance attribution",
        ),
        (
            MERE_METHOD_VERSION,
            literal("1"),
            literal("2"),
            "method version",
        ),
        (
            PROV_WAS_INVALIDATED_BY,
            named("urn:mere:withdrawal-activity:v1:one"),
            named("urn:mere:withdrawal-activity:v1:two"),
            "withdrawal activity",
        ),
        (
            MERE_ASSERTED_GRAPH_KIND,
            literal("user"),
            literal("source"),
            "original assertion graph kind",
        ),
        (
            MERE_ASSERTED_GRAPH_NAME,
            literal("https://graph.test/one"),
            literal("https://graph.test/two"),
            "original assertion graph name",
        ),
    ];

    for (predicate, first, second, field) in conflicts {
        let reifier = NamedNode::new(crate::reifier::statement_reifier_id("conflict")).unwrap();
        let reifies = NamedNode::new(RDF_REIFIES).unwrap();
        let base = Triple::new(
            NamedNode::new("https://subject.test/").unwrap(),
            NamedNode::new("https://predicate.test/rel").unwrap(),
            NamedNode::new("https://object.test/").unwrap(),
        );
        let first_metadata = Quad::new(
            reifier.clone(),
            NamedNode::new(predicate).unwrap(),
            first,
            GraphName::DefaultGraph,
        );
        let second_metadata = Quad::new(
            reifier.clone(),
            NamedNode::new(predicate).unwrap(),
            second,
            GraphName::DefaultGraph,
        );
        let reifies_quad = Quad::new(reifier, reifies, Term::from(base), GraphName::DefaultGraph);

        let error_for_order = |reverse: bool| {
            let mut quads = vec![reifies_quad.clone()];
            if reverse {
                quads.extend([second_metadata.clone(), first_metadata.clone()]);
            } else {
                quads.extend([first_metadata.clone(), second_metadata.clone()]);
            }
            from_quads(quads, "lifecycle-conflict")
                .expect_err("conflicting single-valued metadata must be rejected")
                .to_string()
        };
        let forward = error_for_order(false);
        let reversed = error_for_order(true);
        assert_eq!(
            forward, reversed,
            "{predicate} must not depend on quad order"
        );
        assert!(
            forward.contains(&format!("conflicting {field} metadata")),
            "unexpected error for {predicate}: {forward}"
        );
    }
}

#[test]
fn identical_single_valued_lifecycle_metadata_duplicates_are_accepted() {
    use oxrdf::{Literal, NamedNode, Triple};

    let reifier = NamedNode::new(crate::reifier::statement_reifier_id("duplicate")).unwrap();
    let reifies = NamedNode::new(RDF_REIFIES).unwrap();
    let predicate = NamedNode::new(MERE_LAST_ASSERTED_AT).unwrap();
    let date = Literal::new_typed_literal(
        "1970-01-01T00:00:42Z",
        NamedNode::new_unchecked("http://www.w3.org/2001/XMLSchema#dateTime"),
    );
    let base = Triple::new(
        NamedNode::new("https://subject.test/").unwrap(),
        NamedNode::new("https://predicate.test/rel").unwrap(),
        NamedNode::new("https://object.test/").unwrap(),
    );
    let quads = [
        Quad::new(
            reifier.clone(),
            reifies,
            Term::from(base),
            GraphName::DefaultGraph,
        ),
        Quad::new(
            reifier.clone(),
            predicate.clone(),
            date.clone(),
            GraphName::DefaultGraph,
        ),
        Quad::new(reifier, predicate, date, GraphName::DefaultGraph),
    ];

    let contribution = from_quads(quads, "lifecycle-duplicates").unwrap();
    assert_eq!(contribution.edges.len(), 1);
    assert_eq!(contribution.edges[0].asserted_at_ms, Some(42_000));
}

#[test]
fn conflicting_reifies_targets_are_rejected_independent_of_quad_order() {
    use oxrdf::{NamedNode, Triple};

    let reifier_iri = crate::reifier::statement_reifier_id("target-conflict");
    let reifier = NamedNode::new(&reifier_iri).unwrap();
    let reifies = NamedNode::new(RDF_REIFIES).unwrap();
    let predicate = NamedNode::new("https://predicate.test/rel").unwrap();
    let target_quad = |object: &str| {
        Quad::new(
            reifier.clone(),
            reifies.clone(),
            Term::from(Triple::new(
                NamedNode::new("https://subject.test/").unwrap(),
                predicate.clone(),
                NamedNode::new(object).unwrap(),
            )),
            GraphName::DefaultGraph,
        )
    };
    let first = target_quad("https://object.test/one");
    let second = target_quad("https://object.test/two");
    let error_for_order = |reverse: bool| {
        let quads = if reverse {
            vec![second.clone(), first.clone()]
        } else {
            vec![first.clone(), second.clone()]
        };
        from_quads(quads, "reifier-target-conflict")
            .expect_err("one reifier cannot name two different RDF statements")
            .to_string()
    };

    let forward = error_for_order(false);
    let reversed = error_for_order(true);
    assert_eq!(forward, reversed);
    assert!(forward.contains("identifies conflicting RDF statements"));
}

#[test]
fn identical_reifies_target_duplicates_are_accepted() {
    use oxrdf::{NamedNode, Triple};

    let reifier_iri = crate::reifier::statement_reifier_id("target-duplicate");
    let reifier = NamedNode::new(&reifier_iri).unwrap();
    let target = Quad::new(
        reifier.clone(),
        NamedNode::new(RDF_REIFIES).unwrap(),
        Term::from(Triple::new(
            NamedNode::new("https://subject.test/").unwrap(),
            NamedNode::new("https://predicate.test/rel").unwrap(),
            NamedNode::new("https://object.test/").unwrap(),
        )),
        GraphName::DefaultGraph,
    );

    let contribution = from_quads([target.clone(), target], "reifier-target-duplicate").unwrap();
    assert_eq!(contribution.edges.len(), 1);
    assert_eq!(
        contribution.edges[0].statement_id.as_deref(),
        Some("target-duplicate")
    );
}

#[test]
fn partial_withdrawal_time_without_activity_is_preserved_as_tombstone() {
    use oxrdf::{Literal, NamedNode, Triple};

    let reifier =
        NamedNode::new(crate::reifier::statement_reifier_id("partial-withdrawal")).unwrap();
    let base = Quad::new(
        reifier.clone(),
        NamedNode::new(RDF_REIFIES).unwrap(),
        Term::from(Triple::new(
            NamedNode::new("https://subject.test/").unwrap(),
            NamedNode::new("https://predicate.test/rel").unwrap(),
            NamedNode::new("https://object.test/").unwrap(),
        )),
        GraphName::DefaultGraph,
    );
    let invalidated_at = Quad::new(
        reifier,
        NamedNode::new(PROV_INVALIDATED_AT_TIME).unwrap(),
        Literal::new_typed_literal(
            "1970-01-01T00:00:30Z",
            NamedNode::new_unchecked("http://www.w3.org/2001/XMLSchema#dateTime"),
        ),
        GraphName::DefaultGraph,
    );

    let contribution = from_quads([base, invalidated_at], "partial-withdrawal").unwrap();
    let withdrawal = contribution.edges[0]
        .withdrawn
        .as_ref()
        .expect("the withdrawal timestamp is itself lifecycle evidence");
    assert_eq!(withdrawal.at_ms, Some(30_000));
    assert_eq!(withdrawal.by, None);
}

#[test]
fn supplied_last_asserted_time_survives_without_lifecycle_marker() {
    use oxrdf::{Literal, NamedNode, Triple};

    let reifier = NamedNode::new(crate::reifier::statement_reifier_id("unversioned-last")).unwrap();
    let base = Quad::new(
        reifier.clone(),
        NamedNode::new(RDF_REIFIES).unwrap(),
        Term::from(Triple::new(
            NamedNode::new("https://subject.test/").unwrap(),
            NamedNode::new("https://predicate.test/rel").unwrap(),
            NamedNode::new("https://object.test/").unwrap(),
        )),
        GraphName::DefaultGraph,
    );
    let timestamp = |predicate: &str, second: &str| {
        Quad::new(
            reifier.clone(),
            NamedNode::new(predicate).unwrap(),
            Literal::new_typed_literal(
                format!("1970-01-01T00:00:{second}Z"),
                NamedNode::new_unchecked("http://www.w3.org/2001/XMLSchema#dateTime"),
            ),
            GraphName::DefaultGraph,
        )
    };
    let contribution = from_quads(
        [
            base,
            timestamp(PROV_GENERATED_AT_TIME, "10"),
            timestamp(MERE_LAST_ASSERTED_AT, "20"),
        ],
        "unversioned-last",
    )
    .unwrap();

    assert_eq!(contribution.edges[0].first_asserted_ms, Some(10_000));
    assert_eq!(contribution.edges[0].asserted_at_ms, Some(20_000));
}

#[test]
fn method_version_without_method_iri_is_rejected() {
    use oxrdf::{Literal, NamedNode, Triple};

    let reifier = NamedNode::new(crate::reifier::statement_reifier_id(
        "orphan-method-version",
    ))
    .unwrap();
    let base = Quad::new(
        reifier.clone(),
        NamedNode::new(RDF_REIFIES).unwrap(),
        Term::from(Triple::new(
            NamedNode::new("https://subject.test/").unwrap(),
            NamedNode::new("https://predicate.test/rel").unwrap(),
            NamedNode::new("https://object.test/").unwrap(),
        )),
        GraphName::DefaultGraph,
    );
    let method_version = Quad::new(
        reifier,
        NamedNode::new(MERE_METHOD_VERSION).unwrap(),
        Literal::new_simple_literal("1"),
        GraphName::DefaultGraph,
    );

    let error = from_quads([base, method_version], "orphan-method-version")
        .expect_err("method version needs its corresponding method IRI")
        .to_string();
    assert!(error.contains("method version metadata requires a method IRI"));
}
