// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::graph::apply::assert_semantic_relation_in_scope;
use crate::graph::{SemanticStatementSpec, SemanticSubKind};
use crate::rdf::apply_import;
use crate::types::{GraphScope, NodeProperty};
#[allow(unused_imports)]
use mere::formats::serialize::*;
#[allow(unused_imports)]
use mere::*;

/// A graph exercising the projection surface that file I/O must preserve:
/// curated literals, a recognized statement with reifier metadata (an RDF 1.2
/// triple term), a raw predicate, a scoped + typed property, rdf:type.
fn rich_graph() -> Graph {
    let mut graph = Graph::new();
    let a = graph.add_node("https://a.test".to_string(), Default::default());
    let b = graph.add_node("https://b.test".to_string(), Default::default());
    let c = graph.add_node("https://c.test".to_string(), Default::default());

    graph.get_node_mut(a).expect("a").title = "Article A".to_string();
    assert!(graph.insert_node_tags(a, vec!["research".into()]));

    assert_semantic_relation_in_scope(
        &mut graph,
        a,
        b,
        SemanticSubKind::Cites,
        Some("cited in the intro".to_string()),
        GraphScope::Source,
    );
    // Attach reifier metadata so a triple term must survive file I/O.
    let edge = graph
        .projected_relations_between(a, b)
        .next()
        .expect("edge")
        .0;
    let statement = graph
        .get_relation_mut(edge)
        .and_then(|p| p.assertions.statements.iter_mut().next())
        .expect("statement");
    statement.provenance_iri = Some("https://persona.test/mark".to_string());
    statement.asserted_at_ms = Some(1_720_000_000_000);

    graph.assert_semantic_statement(
        a,
        c,
        SemanticStatementSpec {
            predicate: "https://example.test/vocab#inspiredBy".to_string(),
            ..Default::default()
        },
    );

    let mut published = NodeProperty::new(
        "https://schema.org/datePublished".to_string(),
        "2026-07-04".to_string(),
    )
    .with_graph_scope(GraphScope::User);
    published.datatype = Some("http://www.w3.org/2001/XMLSchema#date".to_string());
    crate::graph::apply::apply_graph_delta(
        &mut graph,
        crate::graph::apply::GraphDelta::AppendNodeProperty {
            key: a,
            property: published,
        },
    );

    graph
}

fn sorted_dataset(graph: &Graph) -> Vec<String> {
    let mut lines: Vec<String> = mere::dataset_quads(graph)
        .iter()
        .map(|quad| format!("{quad} ."))
        .collect();
    lines.sort();
    lines
}

#[test]
fn nquads_round_trip_is_lossless_and_drops_vocabulary() {
    let graph = rich_graph();
    let text = to_nquads(&graph);

    // Self-describing: the file carries the vocabulary alignment.
    assert!(
        text.contains("http://purl.org/spar/cito/cites"),
        "N-Quads file publishes the vocabulary alignment"
    );

    let contribution = from_nquads_envelope(&text, "gate").expect("parse N-Quads");
    let mut reimported = Graph::new();
    let outcome = apply_import(&mut reimported, &contribution);
    assert_eq!(outcome.edges_skipped, 0, "self-contained contribution");

    assert_eq!(
        sorted_dataset(&graph),
        sorted_dataset(&reimported),
        "graph -> N-Quads -> contribution -> graph is lossless under the profile"
    );
}

#[test]
fn trig_round_trip_matches_nquads() {
    let graph = rich_graph();
    let text = to_trig(&graph).expect("serialize TriG");

    let contribution = from_trig_envelope(&text, "gate").expect("parse TriG");
    let mut reimported = Graph::new();
    apply_import(&mut reimported, &contribution);

    assert_eq!(
        sorted_dataset(&graph),
        sorted_dataset(&reimported),
        "TriG carries the same dataset as N-Quads"
    );
}
