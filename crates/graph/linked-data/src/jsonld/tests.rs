// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::ingest::{IngestError, from_quads};
use crate::reifier::statement_reifier_id;
use kernel::graph::fixtures::GraphFixtures;
use kernel::graph::resource::ResourceNode;
use kernel::graph::{Graph, SemanticStatement};
use kernel::persistence::{PersistedEdge, PersistedEdgeFamily, PersistedResourceRecord};
use kernel::persistence::{PersistedSemanticEdgeData, PersistedSemanticStatement};
use kernel::types::{GraphScope, NodeProperty};

fn quad(subject: &str, predicate: &str, object: Term, graph: GraphName) -> Quad {
    Quad::new(
        NamedNode::new(subject).unwrap(),
        NamedNode::new(predicate).unwrap(),
        object,
        graph,
    )
}

fn classic_record(
    handle: &str,
    base_subject: &str,
    predicate: &str,
    object: Term,
    graph: GraphName,
) -> Vec<Quad> {
    let wrapper = statement_reifier_id(handle);
    [
        (RDF_TYPE, NamedNode::new_unchecked(RDF_STATEMENT).into()),
        (RDF_SUBJECT, NamedNode::new(base_subject).unwrap().into()),
        (RDF_PREDICATE, NamedNode::new(predicate).unwrap().into()),
        (RDF_OBJECT, object),
    ]
    .into_iter()
    .map(|(predicate, object)| quad(&wrapper, predicate, object, graph.clone()))
    .collect()
}

fn normalized(graph: &Graph) -> Vec<String> {
    let mut quads: Vec<_> = crate::dataset_quads(graph)
        .into_iter()
        .map(|quad| quad.to_string())
        .collect();
    quads.sort();
    quads.dedup();
    quads
}

fn profile_graph() -> Graph {
    let mut graph = Graph::new();
    let source = graph.add_node("https://source.test/".into(), Default::default());
    let target = graph.add_node("https://target.test/".into(), Default::default());
    for (index, scope) in [
        GraphScope::Default,
        GraphScope::Source,
        GraphScope::User,
        GraphScope::User,
        GraphScope::Agent,
        GraphScope::Moot,
        GraphScope::Custom("https://scope.test/exact#Case".into()),
    ]
    .into_iter()
    .enumerate()
    {
        assert!(
            graph
                .assert_persisted_semantic_statement(
                    source,
                    target,
                    SemanticStatement {
                        statement_id: [
                            "",
                            "plain",
                            "space id",
                            "line\nID",
                            "nul\0ID",
                            "日本語",
                            "custom"
                        ][index]
                            .into(),
                        predicate: "https://example.test/claims".into(),
                        recognized_sub_kind: None,
                        label: Some(format!("label {index}")),
                        graph_scope: scope,
                        provenance_iri: Some(format!("https://author.test/{index}")),
                        asserted_at_ms: Some(index as u64),
                    },
                )
                .is_some()
        );
    }
    let mut properties = Vec::new();
    for (handle, author, time) in [
        ("literal\nA", "https://author.test/A", 0),
        ("literal B", "https://author.test/B", 9),
    ] {
        let mut property =
            NodeProperty::new("https://example.test/greeting".into(), "bonjour".into());
        property.statement_id = handle.into();
        property.lang = Some("fr".into());
        property.provenance_iri = Some(author.into());
        property.asserted_at_ms = Some(time);
        properties.push(property);
    }
    let mut date = NodeProperty::new("https://example.test/date".into(), "2026-10-07".into())
        .with_graph_scope(GraphScope::User);
    date.statement_id = "typed\0literal".into();
    date.datatype = Some("http://www.w3.org/2001/XMLSchema#date".into());
    date.provenance_iri = Some("https://author.test/A".into());
    date.asserted_at_ms = Some(42);
    properties.push(date);

    // Explicit resources need no browsing surfaces to participate in export.
    let a = ResourceNode::for_term("https://terms.test/vocab#A");
    let b = ResourceNode::for_term("https://terms.test/vocab#B");
    let mut snapshot = graph.to_snapshot();
    snapshot
        .resources
        .extend([&a, &b].map(|resource| PersistedResourceRecord {
            canonical_iri: resource.canonical_iri().into(),
            facets: Vec::new(),
        }));
    snapshot.resource_edges.push(PersistedEdge {
        from_node_id: a.id().to_string(),
        to_node_id: b.id().to_string(),
        families: vec![PersistedEdgeFamily::Semantic],
        semantic: Some(PersistedSemanticEdgeData {
            statements: ["resource one", "resource\n二"]
                .into_iter()
                .enumerate()
                .map(|(index, id)| PersistedSemanticStatement {
                    statement_id: id.into(),
                    predicate: kernel::graph::resource::TAGGED_WITH_IRI.into(),
                    recognized_sub_kind: None,
                    label: Some(format!("resource {index}")),
                    graph_scope: GraphScope::User,
                    provenance_iri: Some(format!("https://tagger.test/{index}")),
                    asserted_at_ms: Some(100 + index as u64),
                })
                .collect(),
            ..Default::default()
        }),
        traversal: None,
        containment: None,
        arrangement: None,
        imported: None,
        provenance: None,
    });
    let mut graph = Graph::try_from_snapshot(&snapshot).unwrap_or_else(|error| panic!("{error}"));
    let source = graph
        .nodes()
        .find(|(_, node)| node.url() == "https://source.test/")
        .map(|(key, _)| key)
        .unwrap();
    // Surface facets live in their own sidecar, rather than GraphSnapshot.
    assert!(graph.append_node_properties(source, properties));
    assert_eq!(graph.node_properties(source).unwrap().len(), 3);
    graph
}

mod descriptions;
mod profile;
