// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

fn truth(graph: &Graph) -> serde_json::Value {
    let mut snapshot = graph.to_snapshot();
    snapshot.timestamp_secs = 0;
    serde_json::to_value(snapshot).unwrap()
}

fn statement(url: &str, rel: &str) -> LinkStatement {
    LinkStatement {
        target_url: url.into(),
        rel: rel.into(),
    }
}

#[test]
fn a_live_target_arrival_rederives_the_pending_source_claim() {
    let mut graph = Graph::new();
    let source_url = "https://example.org/source";
    let target_url = "https://example.org/target";
    let source =
        kernel::graph::apply::add_node(&mut graph, None, source_url.into(), Default::default());
    let source_id = graph.shown_resource_id(source).unwrap();
    let before = truth(&graph);
    let outcome = apply_link_statements(&mut graph, source, &[statement(target_url, "cites")]);
    assert_eq!(outcome.pending_targets, vec![target_url]);
    assert_eq!(truth(&graph), before);

    let target =
        kernel::graph::apply::add_node(&mut graph, None, target_url.into(), Default::default());
    let target_id = graph.shown_resource_id(target).unwrap();
    let claims: Vec<_> = graph
        .resource_relations()
        .filter(|(_, from, to, _)| *from == source_id && *to == target_id)
        .flat_map(|(_, _, _, payload)| payload.semantic_statements())
        .filter(|claim| claim.recognized_sub_kind == Some(SemanticSubKind::Cites))
        .collect();
    assert_eq!(
        claims.len(),
        1,
        "arrival must derive the retained page claim"
    );
    assert_eq!(claims[0].graph_scope, kernel::types::GraphScope::Source);
    assert_eq!(claims[0].provenance_iri.as_deref(), Some(source_url));
    assert!(graph.find_edge_key(source, target).is_none());
}

#[test]
fn replay_preserves_the_pending_index_and_exact_recorded_claim() {
    use std::sync::{Arc, Mutex};
    let mut graph = Graph::new();
    let source = kernel::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.org/source".into(),
        Default::default(),
    );
    apply_link_statements(
        &mut graph,
        source,
        &[statement("https://example.org/target", "cites")],
    );
    let mut replay = graph.clone();
    let state = replay.pending_link_state().clone();
    let deltas = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&deltas);
    graph.set_recorder(Some(Arc::new(move |delta| {
        sink.lock().unwrap().push(delta.clone())
    })));
    kernel::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.org/target".into(),
        Default::default(),
    );
    assert!(graph.pending_links().is_empty());
    kernel::graph::capture::replay_captured_deltas_onto(
        &mut replay,
        deltas.lock().unwrap().clone(),
    );
    assert_eq!(
        replay.pending_link_state(),
        &state,
        "replay cannot consume ambient inputs"
    );
    assert_eq!(truth(&replay), truth(&graph));
    assert_eq!(replay.retry_pending_links().already_held, 1);
    assert!(replay.pending_links().is_empty());
    assert_eq!(truth(&replay), truth(&graph));
}

#[test]
fn purge_prevents_arrival_derivation_and_source_navigation_keeps_original_ownership() {
    let mut graph = Graph::new();
    let source = kernel::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.org/source".into(),
        Default::default(),
    );
    let original = graph.shown_resource_id(source).unwrap();
    apply_link_statements(
        &mut graph,
        source,
        &[statement("https://example.org/target", "cites")],
    );
    let mut purged = graph.clone();
    let before = truth(&purged);
    assert_eq!(purged.purge_pending_links(), 1);
    assert_eq!(truth(&purged), before);
    kernel::graph::apply::add_node(
        &mut purged,
        None,
        "https://example.org/target".into(),
        Default::default(),
    );
    assert_eq!(purged.resource_relations().count(), 0);

    kernel::graph::apply::apply_graph_delta(
        &mut graph,
        kernel::graph::apply::GraphDelta::NavigateNode {
            key: source,
            url: "https://example.org/later".into(),
        },
    );
    let current = graph.shown_resource_id(source).unwrap();
    assert_ne!(current, original);
    kernel::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.org/target".into(),
        Default::default(),
    );
    let claims: Vec<_> = graph.resource_relations().collect();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].1, original);
    assert_ne!(claims[0].1, current);
    assert!(graph.pending_links().is_empty());
    let before_truth = truth(&graph);
    assert_eq!(graph.purge_pending_links(), 0);
    assert_eq!(truth(&graph), before_truth);
}

#[test]
fn a_resource_only_target_arrival_needs_no_appearance_and_duplicates_do_not_multiply() {
    let mut graph = Graph::new();
    let source = kernel::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.org/source".into(),
        Default::default(),
    );
    let links = [statement("https://example.org/target", "cites")];
    apply_link_statements(&mut graph, source, &links);
    apply_link_statements(&mut graph, source, &links);
    assert_eq!(graph.pending_links().len(), 1);
    let target = kernel::graph::ResourceNode::new(&links[0].target_url);
    kernel::graph::apply::apply_graph_delta(
        &mut graph,
        kernel::graph::apply::GraphDelta::ReplaySetResourceRecordById {
            resource_id: target.id(),
            record: Some(kernel::persistence::PersistedResourceRecord {
                canonical_iri: target.canonical_iri().into(),
                facets: vec![],
            }),
        },
    );
    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.resource_relations().count(), 1);
    assert!(graph.pending_links().is_empty());
    let before_truth = truth(&graph);
    assert_eq!(graph.retry_pending_links().asserted, 0);
    assert_eq!(truth(&graph), before_truth);
    assert_eq!(
        apply_link_statements(&mut graph, source, &links).edges_asserted,
        0
    );
    assert!(
        graph.pending_links().is_empty(),
        "a held Resource needs no Surface lookup"
    );
    assert_eq!(truth(&graph), before_truth);
}

#[test]
fn carried_import_preserves_its_handle_before_consuming_pending_extraction() {
    let source_url = "https://example.org/source";
    let target_url = "https://example.org/target";
    let mut carried = Graph::new();
    let from =
        kernel::graph::apply::add_node(&mut carried, None, source_url.into(), Default::default());
    let to =
        kernel::graph::apply::add_node(&mut carried, None, target_url.into(), Default::default());
    let handle = "carried opaque handle";
    carried
        .try_assert_persisted_semantic_statement(
            from,
            to,
            kernel::graph::SemanticStatement {
                statement_id: handle.into(),
                predicate: predicate_iri(SemanticSubKind::Cites).into(),
                recognized_sub_kind: Some(SemanticSubKind::Cites),
                graph_scope: kernel::types::GraphScope::Source,
                provenance_iri: Some(source_url.into()),
                label: Some("carried label".into()),
                asserted_at_ms: Some(42),
            },
        )
        .unwrap();
    let envelope =
        crate::from_jsonld_envelope(&serde_json::to_vec(&crate::to_jsonld(&carried)).unwrap())
            .unwrap();
    let mut graph = Graph::new();
    let source =
        kernel::graph::apply::add_node(&mut graph, None, source_url.into(), Default::default());
    apply_link_statements(&mut graph, source, &[statement(target_url, "cites")]);
    crate::apply_import_with_identity(&mut graph, &envelope, |_| crate::SubjectIdentity::Page);
    let claims: Vec<_> = graph
        .resource_relations()
        .flat_map(|(_, _, _, payload)| payload.semantic_statements())
        .collect();
    assert_eq!(
        claims.len(),
        1,
        "carried truth must not compete with ambient mintage"
    );
    assert_eq!(claims[0].statement_id, handle);
    assert_eq!(claims[0].label.as_deref(), Some("carried label"));
    assert_eq!(claims[0].asserted_at_ms, Some(42));
    assert!(graph.pending_links().is_empty());
}

#[test]
fn a_surface_owned_pending_link_requires_its_original_appearance_context() {
    let mut graph = Graph::new();
    let source = kernel::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.org/source".into(),
        Default::default(),
    );
    let target_url = "https://example.org/target";
    let predicate = predicate_iri(SemanticSubKind::UserGrouped);
    apply_link_statements(&mut graph, source, &[statement(target_url, predicate)]);
    let target =
        kernel::graph::apply::add_node(&mut graph, None, target_url.into(), Default::default());
    let edge = graph
        .find_edge_key(source, target)
        .expect("a Surface-owned claim");
    assert_eq!(graph.get_edge(edge).unwrap().semantic_statements().len(), 1);
    assert_eq!(graph.resource_relations().count(), 0);
    assert!(graph.pending_links().is_empty());

    graph.queue_pending_link(kernel::graph::PendingLink {
        source_resource: graph.shown_resource_id(source).unwrap(),
        source_surface: None,
        target_iri: target_url.into(),
        statement: SemanticStatementSpec {
            predicate: predicate.into(),
            recognized_sub_kind: Some(SemanticSubKind::UserGrouped),
            provenance_iri: Some("urn:another-source".into()),
            graph_scope: kernel::types::GraphScope::Source,
            ..Default::default()
        },
    });
    let before = truth(&graph);
    assert_eq!(graph.retry_pending_links().remaining, 1);
    assert_eq!(truth(&graph), before);
    assert_eq!(graph.resource_relations().count(), 0);
}

#[test]
fn suppression_restores_after_panic_and_does_not_escape_into_a_clone() {
    let mut graph = Graph::new();
    let source = kernel::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.org/source".into(),
        Default::default(),
    );
    apply_link_statements(
        &mut graph,
        source,
        &[statement("https://example.org/target", "cites")],
    );
    let mut escaped = graph.without_pending_derivation(|graph| graph.clone());
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        graph.without_pending_derivation(|_| panic!("scope control"));
    }));
    assert!(result.is_err());
    for graph in [&mut graph, &mut escaped] {
        kernel::graph::apply::add_node(
            graph,
            None,
            "https://example.org/target".into(),
            Default::default(),
        );
        assert_eq!(graph.resource_relations().count(), 1);
        assert!(graph.pending_links().is_empty());
    }
}

#[test]
fn rebuilding_from_a_real_document_preserves_truth_and_sources() {
    let source_url = "https://example.org/source";
    let target_url = "https://example.org/target";
    let doc = inker::EngineDocument {
        address: source_url.into(),
        title: Some("Source".into()),
        content_type: "text/knot".into(),
        lang: None,
        provenance: Default::default(),
        trust: Default::default(),
        diagnostics: vec![],
        navigation: Default::default(),
        blocks: vec![inker::Block::Paragraph {
            spans: vec![inker::InlineSpan::Link {
                url: target_url.into(),
                title: None,
                spans: vec![inker::InlineSpan::Text("Target".into())],
                predicate: Some("cites".into()),
            }],
        }],
    };
    let documents = vec![doc];
    let original_docs = documents.clone();
    let mut graph = Graph::new();
    let source =
        kernel::graph::apply::add_node(&mut graph, None, source_url.into(), Default::default());
    apply_link_statements(&mut graph, source, &inker::link_statements(&documents[0]));
    let index = graph.pending_link_state().clone();
    let before = truth(&graph);
    graph.purge_pending_links();
    let result = rebuild_pending_links(&mut graph, &documents);
    assert_eq!(result.queued, 1);
    assert!(result.missing_sources.is_empty());
    assert_eq!(graph.pending_link_state(), &index);
    assert_eq!(truth(&graph), before);
    assert_eq!(documents, original_docs);
    kernel::graph::apply::add_node(&mut graph, None, target_url.into(), Default::default());
    assert_eq!(graph.resource_relations().count(), 1);
    let asserted = truth(&graph);
    assert_eq!(graph.purge_pending_links(), 0);
    assert_eq!(truth(&graph), asserted);
    assert_eq!(documents, original_docs);
    let mut absent = Graph::new();
    let empty = truth(&absent);
    assert_eq!(
        rebuild_pending_links(&mut absent, &documents)
            .missing_sources
            .len(),
        1
    );
    assert_eq!(truth(&absent), empty);
    assert!(absent.pending_links().is_empty());
}

#[test]
fn deleted_sources_and_ambiguous_appearances_do_not_redirect_pending_claims() {
    use kernel::graph::apply::{GraphDelta, apply_graph_delta};
    let mut graph = Graph::new();
    let source = kernel::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.org/source".into(),
        Default::default(),
    );
    apply_link_statements(
        &mut graph,
        source,
        &[statement("https://example.org/target", "cites")],
    );
    let source_id = graph.shown_resource_id(source).unwrap();
    apply_graph_delta(&mut graph, GraphDelta::RemoveNode { key: source });
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetResourceRecordById {
            resource_id: source_id,
            record: None,
        },
    );
    kernel::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.org/target".into(),
        Default::default(),
    );
    assert_eq!(graph.pending_links().len(), 1);
    assert_eq!(graph.resource_relations().count(), 0);
    assert!(graph.resource(source_id).is_none());

    let mut graph = Graph::new();
    let source = kernel::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.org/source".into(),
        Default::default(),
    );
    apply_link_statements(
        &mut graph,
        source,
        &[statement("https://example.org/target", "user-grouped")],
    );
    graph.without_pending_derivation(|graph| {
        for _ in 0..2 {
            kernel::graph::apply::add_node(
                graph,
                None,
                "https://example.org/target".into(),
                Default::default(),
            );
        }
    });
    let before = truth(&graph);
    assert_eq!(graph.retry_pending_links().remaining, 1);
    assert_eq!(truth(&graph), before);
    // Removal is not an arrival, even if it makes appearance choice unambiguous.
    let target = graph
        .get_node_by_url("https://example.org/target")
        .unwrap()
        .0;
    apply_graph_delta(&mut graph, GraphDelta::RemoveNode { key: target });
    assert_eq!(graph.pending_links().len(), 1);
    assert_eq!(graph.retry_pending_links().asserted, 1);
    assert!(graph.pending_links().is_empty());
}

#[test]
fn refused_import_without_admission_does_not_derive_ambient_claims() {
    let mut graph = Graph::new();
    let source = kernel::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.org/source".into(),
        Default::default(),
    );
    apply_link_statements(
        &mut graph,
        source,
        &[statement("https://example.org/target", "cites")],
    );
    let target = graph.without_pending_derivation(|graph| {
        kernel::graph::apply::add_node(
            graph,
            None,
            "https://example.org/target".into(),
            Default::default(),
        )
    });
    let (_, held) = graph
        .try_assert_semantic_statement(
            source,
            target,
            SemanticStatementSpec {
                predicate: "urn:held".into(),
                ..Default::default()
            },
        )
        .unwrap();
    let nodes = ["https://example.org/source", "https://example.org/target"]
        .into_iter()
        .map(|id| crate::NodeContribution {
            id: id.into(),
            types: vec![],
            title: None,
            tags: vec![],
            properties: vec![],
        })
        .collect();
    let contribution = crate::GraphContribution {
        nodes,
        edges: vec![crate::EdgeContribution {
            subject: "https://example.org/source".into(),
            object: "https://example.org/target".into(),
            predicate: "urn:collision".into(),
            statement_id: Some(held.statement_id),
            label: None,
            provenance_iri: None,
            asserted_at_ms: None,
            graph_scope: Default::default(),
        }],
    };
    let before = truth(&graph);
    let outcome = crate::apply_contribution_with_identity(&mut graph, &contribution, |_| {
        crate::SubjectIdentity::Page
    });
    assert_eq!(outcome.edges_asserted, 0);
    assert_eq!(outcome.nodes_created, 0);
    assert_eq!(truth(&graph), before);
    assert_eq!(graph.pending_links().len(), 1);
    assert_eq!(graph.retry_pending_links().asserted, 1);
}

#[test]
fn document_rebuild_accepts_a_held_prepared_source_identity() {
    use kernel::graph::apply::{GraphDelta, apply_graph_delta};
    let doc = inker::EngineDocument {
        address: "https://EXAMPLE.org/source#fragment".into(),
        title: None,
        content_type: "text/knot".into(),
        lang: None,
        provenance: Default::default(),
        trust: Default::default(),
        diagnostics: vec![],
        navigation: Default::default(),
        blocks: vec![inker::Block::Paragraph {
            spans: vec![inker::InlineSpan::Link {
                url: "https://example.org/target".into(),
                title: None,
                spans: vec![],
                predicate: Some("cites".into()),
            }],
        }],
    };
    let original = doc.clone();
    let prepared = kernel::graph::ResourceNode::for_term(&doc.address);
    let mut graph = Graph::new();
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetResourceRecordById {
            resource_id: prepared.id(),
            record: Some(kernel::persistence::PersistedResourceRecord {
                canonical_iri: prepared.canonical_iri().into(),
                facets: vec![],
            }),
        },
    );
    let before = truth(&graph);
    let report = rebuild_pending_links_from_sources(&mut graph, [(prepared.id(), &doc)]);
    assert_eq!(report.queued, 1);
    assert!(report.missing_sources.is_empty());
    assert_eq!(truth(&graph), before);
    assert_eq!(graph.pending_links()[0].source_resource, prepared.id());
    kernel::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.org/target".into(),
        Default::default(),
    );
    let (_, source, _, _) = graph.resource_relations().next().unwrap();
    assert_eq!(source, prepared.id());
    assert_eq!(doc, original);
}

#[test]
fn navigation_to_an_already_held_target_admits_a_surface_owned_claim() {
    use kernel::graph::apply::{GraphDelta, apply_graph_delta};
    let mut graph = Graph::new();
    let source = kernel::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.org/source".into(),
        Default::default(),
    );
    let target = kernel::graph::apply::add_node(
        &mut graph,
        None,
        "https://example.org/other".into(),
        Default::default(),
    );
    apply_link_statements(
        &mut graph,
        source,
        &[statement("https://example.org/target", "user-grouped")],
    );
    let resource = kernel::graph::ResourceNode::new("https://example.org/target");
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetResourceRecordById {
            resource_id: resource.id(),
            record: Some(kernel::persistence::PersistedResourceRecord {
                canonical_iri: resource.canonical_iri().into(),
                facets: vec![],
            }),
        },
    );
    assert_eq!(graph.pending_links().len(), 1);
    let baseline = graph.clone();
    let (resources, surfaces) = (graph.resource_nodes().count(), graph.node_count());
    use std::sync::{Arc, Mutex};
    let deltas = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&deltas);
    graph.set_recorder(Some(Arc::new(move |delta| {
        sink.lock().unwrap().push(delta.clone())
    })));
    apply_graph_delta(
        &mut graph,
        GraphDelta::NavigateNode {
            key: target,
            url: "https://example.org/target".into(),
        },
    );
    assert_eq!(
        (graph.resource_nodes().count(), graph.node_count()),
        (resources, surfaces)
    );
    assert!(graph.pending_links().is_empty());
    assert!(graph.find_edge_key(source, target).is_some());
    let mut replay = baseline;
    kernel::graph::replay_captured_deltas_onto(&mut replay, deltas.lock().unwrap().clone());
    assert_eq!(truth(&graph), truth(&replay));
    assert_eq!(replay.pending_links().len(), 1);
}
