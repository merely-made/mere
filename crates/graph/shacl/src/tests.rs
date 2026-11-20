// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use oxrdf::{GraphName, Literal, Quad, Triple};

use super::*;

fn iri(local: &str) -> NamedNode {
    NamedNode::new_unchecked(format!("https://example.test/{local}"))
}

fn shapes(turtle: &str) -> Graph {
    mere::parse_turtle(turtle, None)
        .unwrap()
        .quads(View::Live)
        .map(|quad| Triple::new(quad.subject, quad.predicate, quad.object))
        .collect()
}

#[test]
fn named_live_view_cannot_borrow_required_property_from_other_graphs_or_history() {
    let kept = iri("kept");
    let missing = iri("required");
    let subject = iri("river-map");
    let dataset = Arc::new(IndexedDataset::from_quads([
        (
            Quad::new(
                subject.clone(),
                iri("seen"),
                Literal::from("yes"),
                kept.clone(),
            ),
            true,
        ),
        (
            Quad::new(
                subject.clone(),
                missing.clone(),
                Literal::from("yes"),
                iri("other"),
            ),
            true,
        ),
        (
            Quad::new(
                subject.clone(),
                missing.clone(),
                Literal::from("yes"),
                GraphName::DefaultGraph,
            ),
            true,
        ),
        (
            Quad::new(
                subject.clone(),
                missing.clone(),
                Literal::from("yes"),
                kept.clone(),
            ),
            false,
        ),
    ]));
    let shapes = shapes(
        r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <https://example.test/> .
        ex:shape a sh:NodeShape ; sh:targetNode ex:river-map ;
            sh:property [ sh:path ex:required ; sh:minCount 1 ; sh:message "Required property" ] .
    "#,
    );
    let selected = GraphSelector::Named(kept.into());
    let live = validate(dataset.clone(), View::Live, selected.clone(), &shapes).unwrap();
    assert!(!live.conforms);
    let [finding] = live.findings.as_slice() else {
        panic!("Expected one finding")
    };
    assert_eq!(finding.focus_node, Term::from(subject));
    assert_eq!(
        finding.path.as_ref().and_then(PropertyPath::predicate),
        Some(&missing)
    );
    assert_eq!(finding.message.as_deref(), Some("Required property"));
    assert!(live.dataset.iter().any(|quad| quad.predicate.as_str()
        == "http://www.w3.org/ns/shacl#conforms"
        && quad.object == Term::from(Literal::from(false)).as_ref()));
    assert!(
        validate(dataset.clone(), View::History, selected, &shapes)
            .unwrap()
            .conforms
    );
    assert!(
        validate(dataset.clone(), View::Live, GraphSelector::Default, &shapes)
            .unwrap()
            .conforms
    );
    assert!(
        validate(dataset, View::Live, GraphSelector::Union, &shapes)
            .unwrap()
            .conforms
    );
}

#[test]
fn sparql_constraints_use_the_same_named_graph() {
    let kept = iri("kept");
    let subject = iri("river-map");
    let dataset = IndexedDataset::from_quads([
        (
            Quad::new(
                subject.clone(),
                iri("seen"),
                Literal::from("yes"),
                kept.clone(),
            ),
            true,
        ),
        (
            Quad::new(
                subject.clone(),
                iri("required"),
                Literal::from("yes"),
                iri("other"),
            ),
            true,
        ),
    ]);
    let shapes = shapes(
        r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <https://example.test/> .
        ex:shape a sh:NodeShape ; sh:targetNode ex:river-map ;
            sh:sparql [ sh:select "SELECT $this WHERE { FILTER NOT EXISTS { $this <https://example.test/required> ?value } }" ] .
    "#,
    );
    let report = validate(
        dataset,
        View::Live,
        GraphSelector::Named(kept.into()),
        &shapes,
    )
    .unwrap();
    assert!(!report.conforms);
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].focus_node, Term::from(subject));
}
