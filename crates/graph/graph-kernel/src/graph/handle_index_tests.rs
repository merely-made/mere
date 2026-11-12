// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The warm handle index answers as a rebuilt one does after every write path.
use super::node_facets::SEMANTIC_PROPERTIES;
use super::*;
use crate::types::{GraphScope, NodeProperty};
use euclid::default::Point2D;

fn property(id: &str, value: &str) -> NodeProperty {
    NodeProperty {
        statement_id: id.into(),
        predicate: "https://schema.org/position".into(),
        value: value.into(),
        datatype: None,
        lang: None,
        graph_scope: GraphScope::User,
        provenance_iri: Some("urn:author:a".into()),
        asserted_at_ms: Some(0),
    }
}

fn cold(graph: &Graph) -> Graph {
    let mut cold = graph.clone();
    cold.clear_handle_index();
    cold
}

fn found(graph: &Graph, id: &str) -> Option<RelationKey> {
    let warm = graph.find_semantic_statement(id).map(|(key, _)| key);
    assert_eq!(warm, cold(graph).find_semantic_statement(id).map(|(key, _)| key));
    warm
}

fn available(graph: &Graph, resource: Uuid, held: &NodeProperty) -> bool {
    let warm = graph.literal_handle_available(resource, held);
    assert_eq!(warm, cold(graph).literal_handle_available(resource, held));
    warm
}

fn statement_on(graph: &Graph, from: NodeKey, to: NodeKey) -> (RelationKey, String) {
    let key = graph.projected_relations_between(from, to).next().unwrap().0;
    let id = graph.get_relation(key).unwrap().assertions()[0]
        .statement_id
        .clone();
    (key, id)
}

fn three() -> (Graph, [NodeKey; 3]) {
    let mut graph = Graph::new();
    let keys = ["one", "two", "three"].map(|name| {
        let key = graph.add_node(format!("https://example.test/{name}"), Point2D::zero());
        graph.refresh_surface_resource(key);
        key
    });
    (graph, keys)
}

#[test]
fn warm_index_tracks_semantic_writes_and_mutable_handouts() {
    let (mut graph, [a, b, c]) = three();
    assert_eq!(found(&graph, "absent"), None);
    let spec = SemanticStatementSpec {
        predicate: "https://example.test/vocab#next".into(),
        ..Default::default()
    };
    graph.assert_semantic_statement(a, b, spec.clone());
    let (key, id) = statement_on(&graph, a, b);
    assert_eq!(found(&graph, &id), Some(key));

    // A payload handed out mutably is read live until the next note folds it.
    graph.get_relation_mut(key).unwrap().assertions.statements[0].statement_id = "renamed".into();
    assert_eq!(found(&graph, "renamed"), Some(key));
    assert_eq!(found(&graph, &id), None);
    graph.assert_semantic_statement(b, c, spec);
    assert_eq!(found(&graph, "renamed"), Some(key));
    let (other, other_id) = statement_on(&graph, b, c);
    assert_eq!(found(&graph, &other_id), Some(other));
    assert!(!available(&graph, Uuid::nil(), &property(&other_id, "x")));
}

#[test]
fn warm_index_tracks_literals_on_resources_and_surfaces() {
    let (mut graph, [a, b, c]) = three();
    let (ra, rb) = (graph.shown_resource_id(a).unwrap(), graph.shown_resource_id(b).unwrap());
    assert!(available(&graph, ra, &property("held", "1")));
    assert!(graph.append_node_properties(a, vec![property("held", "1")]));
    assert!(available(&graph, ra, &property("held", "1")));
    assert!(!available(&graph, ra, &property("held", "2")));
    assert!(!available(&graph, rb, &property("held", "1")));
    assert!(!graph.append_node_properties(b, vec![property("held", "1")]));

    // A retained Surface literal claims its id against every Resource.
    assert!(graph.set_node_facet(c, SEMANTIC_PROPERTIES, &vec![property("legacy", "1")]));
    assert!(!available(&graph, rb, &property("legacy", "1")));

    // Removal leaves a stale holder; the live check sees the id is free again.
    assert!(graph.retract_resource_property(ra, "held").unwrap());
    assert!(available(&graph, rb, &property("held", "1")));
    assert!(graph.append_node_properties(b, vec![property("held", "1")]));
    assert!(!available(&graph, ra, &property("held", "1")));
}

#[test]
fn wholesale_facet_replacement_clears_the_index() {
    let (mut graph, [a, _, c]) = three();
    let ra = graph.shown_resource_id(a).unwrap();
    assert!(available(&graph, ra, &property("legacy", "1")));
    let node_id = graph.get_node(c).unwrap().id;
    let mut facets = graph.facets().clone();
    facets
        .set(
            node_id,
            chartulary::FacetId::new(SEMANTIC_PROPERTIES),
            serde_json::to_value(vec![property("legacy", "1")]).unwrap(),
            &chartulary::AcceptAll,
        )
        .unwrap();
    graph.install_facets(facets).unwrap();
    assert!(!available(&graph, ra, &property("legacy", "1")));
}

/// The pre-index owner lookup: walk every Surface and Resource relation.
fn owners_by_scan(graph: &Graph, id: &str) -> Vec<RelationKey> {
    use petgraph::visit::{EdgeRef, IntoEdgeReferences};
    graph
        .inner
        .inner()
        .edge_references()
        .map(|edge| (RelationKey::Surface(edge.id()), edge.weight()))
        .chain(
            graph
                .resource_relations()
                .map(|(key, _, _, payload)| (RelationKey::Resource(key), payload)),
        )
        .filter(|(_, payload)| payload.assertions().iter().any(|s| s.statement_id == id))
        .map(|(key, _)| key)
        .collect()
}

#[test]
fn withdrawals_match_a_scan_and_a_state_rebuilt_from_scratch() {
    let (mut graph, [a, b, c]) = three();
    let mut ids = Vec::new();
    for (i, (from, to)) in [(a, b), (b, c), (a, c), (a, b)].into_iter().enumerate() {
        let spec = SemanticStatementSpec {
            predicate: format!("https://example.test/vocab#p{i}"),
            ..Default::default()
        };
        graph.assert_semantic_statement(from, to, spec);
        let key = graph
            .projected_relations_between(from, to)
            .last()
            .unwrap()
            .0;
        ids.push(
            graph.get_relation(key).unwrap().assertions().last().unwrap().statement_id.clone(),
        );
    }
    ids.push("absent".into());
    // A reference graph clears its index before every operation, so each of
    // its lookups is rebuilt from scratch; the live graph keeps its warm index.
    let mut reference = graph.clone();
    for id in &ids {
        assert_eq!(graph.semantic_statement_owner_keys(id), owners_by_scan(&graph, id));
    }
    for id in &ids {
        reference.clear_handle_index();
        let live = graph.withdraw_semantic_assertion(id, Some(7), Some("urn:w".into()));
        let rebuilt = reference.withdraw_semantic_assertion(id, Some(7), Some("urn:w".into()));
        assert_eq!(live, rebuilt);
        assert_eq!(live, id != "absent");
        // A second withdrawal of the same id applies nothing.
        assert!(!graph.withdraw_semantic_assertion(id, Some(8), None));
        for probe in &ids {
            assert_eq!(graph.semantic_statement_owner_keys(probe), owners_by_scan(&graph, probe));
        }
    }
    for id in &ids {
        assert_eq!(
            graph.find_semantic_statement(id).map(|(k, s)| (k, s.withdrawn.clone())),
            reference.find_semantic_statement(id).map(|(k, s)| (k, s.withdrawn.clone())),
        );
    }
    // Interleave a fresh assert after withdrawals: the index still tracks it.
    let spec = SemanticStatementSpec {
        predicate: "https://example.test/vocab#late".into(),
        ..Default::default()
    };
    graph.assert_semantic_statement(b, c, spec);
    let (key, id) = statement_on(&graph, b, c);
    assert_eq!(graph.semantic_statement_owner_keys(&id), owners_by_scan(&graph, &id));
    assert!(graph.semantic_statement_owner_keys(&id).contains(&key));
}
