// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::{sparql, sparql_with_history};
use kernel::graph::fixtures::GraphFixtures;
use kernel::graph::{Graph, SemanticStatement, SemanticSubKind, predicate_iri};
use kernel::types::{AssertionWithdrawal, GraphScope};

#[test]
fn history_requires_an_explicit_projection_and_named_graph_query() {
    let mut graph = Graph::new();
    let from = graph.add_node("https://history-query.test/from".into(), Default::default());
    let to = graph.add_node("https://history-query.test/to".into(), Default::default());
    let id = "history-query-assertion";
    graph
        .try_assert_persisted_semantic_statement(
            from,
            to,
            SemanticStatement {
                statement_id: id.into(),
                predicate: predicate_iri(SemanticSubKind::Cites).into(),
                recognized_sub_kind: Some(SemanticSubKind::Cites),
                label: None,
                basis: Default::default(),
                graph_scope: GraphScope::Default,
                provenance_iri: None,
                first_asserted_ms: None,
                asserted_at_ms: None,
                withdrawn: Some(AssertionWithdrawal {
                    at_ms: None,
                    by: None,
                }),
            },
        )
        .unwrap();

    let history_query = format!(
        "ASK {{ GRAPH <{}> {{ ?reifier <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> ?fact }} }}",
        crate::ASSERTION_HISTORY_GRAPH_IRI
    );
    assert_eq!(
        sparql(&graph, &history_query).unwrap().rows,
        vec![vec![Some("false".into())]],
        "default SPARQL sees the held projection only"
    );
    assert_eq!(
        sparql_with_history(&graph, &history_query).unwrap().rows,
        vec![vec![Some("true".into())]],
        "history is visible only by selecting the explicit projection and graph"
    );
}
