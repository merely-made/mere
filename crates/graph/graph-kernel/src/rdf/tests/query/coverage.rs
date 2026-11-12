// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::graph::{
    CoverageLayer, CoverageLimit, CoverageNote, PendingLink, ResourceNode, SemanticStatementSpec,
};
use mere::query::*;

#[test]
fn select_empty_select_and_ask_preserve_every_known_coverage_layer() {
    let mut graph = Graph::new();
    let limits = CoverageLayer::ALL
        .into_iter()
        .map(|layer| CoverageLimit::new(layer, "host boundary"))
        .collect();
    let note = CoverageNote { limits };
    let mut truth = graph.to_snapshot();
    truth.timestamp_secs = 0;
    graph.set_known_coverage(note.clone());
    for query in ["SELECT ?s WHERE { ?s ?p ?o }", "ASK { ?s ?p ?o }"] {
        let rows = sparql(&graph, query).unwrap();
        assert_eq!(rows.coverage, note);
        assert_eq!(rows, sparql_materialized(&graph, query).unwrap());
    }
    let nonempty = sparql(
        &graph,
        "SELECT ?s WHERE { VALUES ?s { <urn:held-result> } }",
    )
    .unwrap();
    assert_eq!(nonempty.rows.len(), 1);
    assert_eq!(nonempty.coverage, note);
    let mut after = graph.to_snapshot();
    after.timestamp_secs = 0;
    assert_eq!(
        serde_json::to_value(after).unwrap(),
        serde_json::to_value(truth).unwrap()
    );
    assert!(
        sparql(&Graph::new(), "ASK { ?s ?p ?o }")
            .unwrap()
            .coverage
            .limits
            .is_empty()
    );
}

#[test]
fn pending_unknown_targets_are_possession_limits_unless_known_to_be_unloaded() {
    let mut graph = Graph::new();
    let source = crate::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.org/source".into(),
        Default::default(),
    );
    let target = ResourceNode::new("https://example.org/absent");
    graph.queue_pending_link(PendingLink {
        source_resource: graph.shown_resource_id(source).unwrap(),
        source_surface: None,
        target_iri: target.canonical_iri().into(),
        statement: SemanticStatementSpec {
            predicate: "https://mere.computer/ns/rel#cites".into(),
            ..Default::default()
        },
    });
    let rows = sparql(&graph, "ASK { ?s ?p ?o }").unwrap();
    assert!(
        rows.coverage
            .limits
            .iter()
            .any(|limit| limit.layer == CoverageLayer::Possession)
    );
    for scope in [vec![target.id()], vec![]] {
        let mut limit = CoverageLimit::new(CoverageLayer::Residency, "known but not loaded");
        limit.resources = scope;
        graph.set_known_coverage(CoverageNote {
            limits: vec![limit],
        });
        let rows = sparql(&graph, "ASK { ?s ?p ?o }").unwrap();
        assert!(
            rows.coverage
                .limits
                .iter()
                .any(|limit| limit.layer == CoverageLayer::Residency)
        );
        assert!(
            !rows
                .coverage
                .limits
                .iter()
                .any(|limit| limit.layer == CoverageLayer::Possession)
        );
    }
}
