// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::graph::{CoverageLayer, CoverageLimit, CoverageNote, ResourceNode, apply};

fn held_graph() -> (Graph, uuid::Uuid) {
    let mut graph = Graph::new();
    let surface = apply::add_node(
        &mut graph,
        None,
        "https://example.org/member".into(),
        Default::default(),
    );
    let member = graph.shown_resource_id(surface).unwrap();
    (graph, member)
}

#[test]
fn select_resolves_named_resource_cells_and_reports_missing_without_inventing_nodes() {
    let (mut graph, member) = held_graph();
    let missing = ResourceNode::for_term("urn:missing:Member").id();
    let query = "SELECT ?member WHERE { VALUES ?member { <https://example.org/member> <https://example.org/member> <urn:missing:Member> UNDEF } }";
    let count = graph.resource_nodes().count();
    let result = sparql_resource_members(&graph, query, "member").unwrap();
    assert_eq!(result.members, vec![member]);
    assert_eq!(graph.resource_nodes().count(), count);
    assert!(
        result
            .coverage
            .limits
            .iter()
            .any(|l| l.layer == CoverageLayer::Possession && l.resources == vec![missing])
    );
    let mut limit = CoverageLimit::new(CoverageLayer::Residency, "known but not loaded");
    limit.resources = vec![missing];
    graph.set_known_coverage(CoverageNote {
        limits: vec![limit],
    });
    let result = sparql_resource_members(&graph, query, "member").unwrap();
    assert!(
        result
            .coverage
            .limits
            .iter()
            .any(|l| l.layer == CoverageLayer::Residency)
    );
    assert!(
        !result
            .coverage
            .limits
            .iter()
            .any(|l| l.layer == CoverageLayer::Possession)
    );
}

#[test]
fn saved_query_refuses_boolean_missing_column_and_literal_iri_alias() {
    let (graph, _) = held_graph();
    for query in [
        "ASK { ?s ?p ?o }",
        "SELECT ?other WHERE { VALUES ?other { <https://example.org/member> } }",
        "SELECT ?member WHERE { VALUES ?member { \"https://example.org/member\" } }",
        "SELECT (BNODE() AS ?member) WHERE {}",
    ] {
        assert!(
            sparql_resource_members(&graph, query, "member").is_err(),
            "{query}"
        );
    }
}
