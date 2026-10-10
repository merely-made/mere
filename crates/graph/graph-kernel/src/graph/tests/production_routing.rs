// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::super::*;
use crate::graph::apply::{self, GraphDelta, GraphDeltaResult, apply_graph_delta};
use crate::graph::capture::{CapturedDelta, replay_captured_deltas_onto};
use crate::graph::revert::revert_change;
use std::sync::{Arc, Mutex};

fn bound_graph() -> (Graph, NodeKey, NodeKey) {
    let mut graph = Graph::new();
    let a = graph.add_node("https://a.test/".into(), Default::default());
    let b = graph.add_node("https://b.test/".into(), Default::default());
    graph.ensure_surface_resource(a).unwrap();
    graph.ensure_surface_resource(b).unwrap();
    (graph, a, b)
}

fn record(graph: &mut Graph) -> Arc<Mutex<Vec<CapturedDelta>>> {
    let captured = Arc::new(Mutex::new(Vec::new()));
    let sink = captured.clone();
    graph.set_recorder(Some(Arc::new(move |delta| {
        sink.lock().unwrap().push(delta.clone());
    })));
    captured
}

fn truth(graph: &Graph) -> serde_json::Value {
    let mut snapshot = serde_json::to_value(graph.to_snapshot()).unwrap();
    snapshot.as_object_mut().unwrap().remove("timestamp_secs");
    serde_json::json!({"snapshot": snapshot, "facets": graph.facets()})
}

fn spec(predicate: &str, source: &str) -> SemanticStatementSpec {
    SemanticStatementSpec {
        predicate: predicate.into(),
        recognized_sub_kind: sub_kind_from_iri(predicate),
        provenance_iri: Some(source.into()),
        asserted_at_ms: Some(10),
        ..Default::default()
    }
}

fn semantic(kind: SemanticSubKind) -> EdgeAssertion {
    EdgeAssertion::Semantic {
        sub_kind: kind,
        label: None,
        decay_progress: None,
    }
}

#[test]
fn production_routing_all_fixed_families_have_exact_captures_and_controls() {
    let mut cases = Vec::new();
    for kind in [
        SemanticSubKind::Hyperlink,
        SemanticSubKind::AgentDerived,
        SemanticSubKind::Cites,
        SemanticSubKind::Quotes,
        SemanticSubKind::Summarizes,
        SemanticSubKind::Elaborates,
        SemanticSubKind::ExampleOf,
        SemanticSubKind::Supports,
        SemanticSubKind::Contradicts,
        SemanticSubKind::Questions,
        SemanticSubKind::SameEntityAs,
        SemanticSubKind::DuplicateOf,
        SemanticSubKind::CanonicalMirrorOf,
        SemanticSubKind::DependsOn,
        SemanticSubKind::Blocks,
        SemanticSubKind::NextStep,
    ] {
        cases.push((semantic(kind), RelationSelector::Semantic(kind), true));
    }
    cases.push((
        semantic(SemanticSubKind::UserGrouped),
        RelationSelector::Semantic(SemanticSubKind::UserGrouped),
        false,
    ));
    for (kind, resource) in [
        (ContainmentSubKind::UrlPath, true),
        (ContainmentSubKind::Domain, true),
        (ContainmentSubKind::FileSystem, true),
        (ContainmentSubKind::ClipSource, true),
        (ContainmentSubKind::UserFolder, false),
        (ContainmentSubKind::NotebookSection, false),
        (ContainmentSubKind::CollectionMember, false),
    ] {
        cases.push((
            EdgeAssertion::Containment { sub_kind: kind },
            RelationSelector::Containment(kind),
            resource,
        ));
    }
    for (kind, resource) in [
        (ImportedSubKind::RssMembership, true),
        (ImportedSubKind::FileSystemImport, true),
        (ImportedSubKind::ArchiveMembership, true),
        (ImportedSubKind::SharedCollection, true),
        (ImportedSubKind::BookmarkFolder, false),
        (ImportedSubKind::HistoryImport, false),
        (ImportedSubKind::SessionImport, false),
    ] {
        cases.push((
            EdgeAssertion::Imported { sub_kind: kind },
            RelationSelector::Imported(kind),
            resource,
        ));
    }
    for (kind, resource) in [
        (ProvenanceSubKind::ClippedFrom, true),
        (ProvenanceSubKind::ExcerptedFrom, true),
        (ProvenanceSubKind::SummarizedFrom, true),
        (ProvenanceSubKind::TranslatedFrom, true),
        (ProvenanceSubKind::RewrittenFrom, true),
        (ProvenanceSubKind::GeneratedFrom, true),
        (ProvenanceSubKind::ExtractedFrom, true),
        (ProvenanceSubKind::ImportedFromSource, true),
        (ProvenanceSubKind::CopiedFrom, false),
    ] {
        cases.push((
            EdgeAssertion::Provenance { sub_kind: kind },
            RelationSelector::Provenance(kind),
            resource,
        ));
    }
    for kind in [
        ArrangementSubKind::FrameMember,
        ArrangementSubKind::TileGroup,
        ArrangementSubKind::SplitPair,
    ] {
        cases.push((
            EdgeAssertion::Arrangement { sub_kind: kind },
            RelationSelector::Arrangement(kind),
            false,
        ));
    }
    assert_eq!(cases.len(), 43);
    for (assertion, selector, resource) in cases {
        let (mut graph, a, b) = bound_graph();
        let mut replayed = graph.clone();
        let captured = record(&mut graph);
        let key = apply::assert_relation(&mut graph, a, b, assertion.clone()).unwrap();
        assert_eq!(
            matches!(key, RelationKey::Resource(_)),
            resource,
            "{selector:?}"
        );
        assert!(graph.get_relation(key).unwrap().has_relation(selector));
        assert_eq!(graph.find_edge_key(a, b).is_none(), resource);
        assert_eq!(graph.resource_relations().count(), usize::from(resource));
        let revision = graph.revision();
        assert!(apply::assert_relation(&mut graph, a, b, assertion).is_none());
        assert_eq!(graph.revision(), revision, "same assertion is a no-op");
        let edits = captured.lock().unwrap().clone();
        assert_eq!(edits.len(), 1);
        assert_eq!(
            matches!(edits[0], CapturedDelta::ReplaySetResourceEdgesByIds { .. }),
            resource
        );
        replay_captured_deltas_onto(&mut replayed, edits);
        assert!(
            replayed
                .projected_relations_between(a, b)
                .any(|(_, payload)| payload.has_relation(selector)),
            "exact replay retains {selector:?}, including session arrangements"
        );
        assert_eq!(truth(&replayed), truth(&graph));
    }
}

#[test]
fn production_routing_custom_nature_preserves_handles_and_refuses_conflicts() {
    let (mut graph, a, b) = bound_graph();
    let predicate = "urn:test:production-routing";
    let (resource_key, held) = graph
        .assert_semantic_statement(a, b, spec(predicate, "urn:alice"))
        .unwrap();
    assert!(matches!(resource_key, RelationKey::Resource(_)));
    graph
        .declare_predicate(predicate, GraphStratum::Surface)
        .unwrap();
    let (same_key, same) = graph
        .assert_semantic_statement(a, b, spec(predicate, "urn:alice"))
        .unwrap();
    assert_eq!(
        (same_key, same.statement_id),
        (resource_key, held.statement_id.clone())
    );
    let (surface_key, surface) = graph
        .assert_semantic_statement(a, b, spec(predicate, "urn:bob"))
        .unwrap();
    assert!(matches!(surface_key, RelationKey::Surface(_)));
    let mut opposing = graph.clone();
    opposing
        .declare_predicate(predicate, GraphStratum::Resource)
        .unwrap();
    let left = graph
        .resource_record(ResourceNode::for_term(predicate).id())
        .unwrap();
    let right = opposing
        .resource_record(ResourceNode::for_term(predicate).id())
        .unwrap();
    let facet = super::super::predicate_declarations::PREDICATE_DECLARATIONS_FACET;
    let left = left.facets.iter().find(|f| f.facet == facet).unwrap();
    let right = right.facets.iter().find(|f| f.facet == facet).unwrap();
    let merged = super::super::predicate_declarations::merge_predicate_declarations(
        predicate,
        &serde_json::from_str(&left.value_json).unwrap(),
        &serde_json::from_str(&right.value_json).unwrap(),
    )
    .unwrap();
    let mut record = graph
        .resource_record(ResourceNode::for_term(predicate).id())
        .unwrap();
    record
        .facets
        .iter_mut()
        .find(|f| f.facet == facet)
        .unwrap()
        .value_json = serde_json::to_string(&merged).unwrap();
    graph.set_resource_record(ResourceNode::for_term(predicate).id(), Some(record));
    let before = truth(&graph);
    assert!(
        graph
            .assert_semantic_statement(a, b, spec(predicate, "urn:new"))
            .is_none()
    );
    assert_eq!(truth(&graph), before);
    assert!(
        graph
            .assert_semantic_statement(a, b, spec(predicate, "urn:alice"))
            .is_some()
    );
    apply_graph_delta(
        &mut graph,
        GraphDelta::NavigateNode {
            key: a,
            url: "https://later.test/".into(),
        },
    );
    assert!(graph.retract_semantic_statement(a, b, &held.statement_id));
    assert!(!graph.retract_assertion(&held.statement_id));
    assert!(
        graph
            .find_semantic_statement(&surface.statement_id)
            .is_some()
    );
}

#[test]
fn production_routing_selector_retracts_both_held_stores_but_raw_replay_is_surface_only() {
    let (mut graph, a, b) = bound_graph();
    let raw = CapturedDelta::ReplayAssertRelationByIds {
        from_id: graph.get_node(a).unwrap().id.to_string(),
        to_id: graph.get_node(b).unwrap().id.to_string(),
        assertion: semantic(SemanticSubKind::Cites),
    };
    replay_captured_deltas_onto(&mut graph, [raw]);
    let surface = graph.find_edge_key(a, b).unwrap();
    assert!(
        graph
            .get_edge(surface)
            .unwrap()
            .has_relation(RelationSelector::Semantic(SemanticSubKind::Cites))
    );
    assert_eq!(graph.resource_relations().count(), 0);
    graph
        .assert_semantic_statement(
            a,
            b,
            spec(predicate_iri(SemanticSubKind::Cites), "urn:other"),
        )
        .unwrap();
    apply::assert_relation(
        &mut graph,
        a,
        b,
        EdgeAssertion::Arrangement {
            sub_kind: ArrangementSubKind::FrameMember,
        },
    )
    .unwrap();
    let mut replayed = graph.clone();
    let captured = record(&mut graph);
    let result = apply_graph_delta(
        &mut graph,
        GraphDelta::RetractRelations {
            from: a,
            to: b,
            selector: RelationSelector::Semantic(SemanticSubKind::Cites),
        },
    );
    assert!(matches!(result, GraphDeltaResult::EdgesRemoved(2)));
    assert!(
        graph
            .get_edge(surface)
            .unwrap()
            .has_relation(RelationSelector::Arrangement(
                ArrangementSubKind::FrameMember
            ))
    );
    assert_eq!(graph.resource_relations().count(), 0);
    replay_captured_deltas_onto(&mut replayed, captured.lock().unwrap().clone());
    assert_eq!(truth(&replayed), truth(&graph));

    let (mut control, a, b) = bound_graph();
    graph = control.clone();
    graph
        .assert_semantic_statement(
            a,
            b,
            spec(predicate_iri(SemanticSubKind::Cites), "urn:held"),
        )
        .unwrap();
    let retained = graph.to_snapshot().resource_edges;
    let a_id = graph.get_node(a).unwrap().id;
    let b_id = graph.get_node(b).unwrap().id;
    assert_eq!(
        graph.replay_retract_relations_by_ids(
            a_id,
            b_id,
            RelationSelector::Semantic(SemanticSubKind::Cites)
        ),
        0
    );
    assert_eq!(graph.to_snapshot().resource_edges, retained);
    control
        .replay_assert_relation_by_ids(a_id, b_id, semantic(SemanticSubKind::Cites))
        .unwrap();
    assert!(control.find_edge_key(a, b).is_some());
    assert_eq!(control.resource_relations().count(), 0);
}

#[test]
fn production_routing_undo_preserves_later_navigation_and_surface_controls() {
    let (mut graph, a, b) = bound_graph();
    apply::assert_relation(&mut graph, a, b, semantic(SemanticSubKind::UserGrouped)).unwrap();
    let before = graph.clone();
    let captured = record(&mut graph);
    let key = apply::assert_relation(&mut graph, a, b, semantic(SemanticSubKind::Cites)).unwrap();
    let handle = graph.get_relation(key).unwrap().semantic_statements()[0]
        .statement_id
        .clone();
    let change = captured.lock().unwrap().clone();
    let after = graph.clone();
    apply_graph_delta(
        &mut graph,
        GraphDelta::NavigateNode {
            key: a,
            url: "https://later.test/".into(),
        },
    );
    let binding = graph.shown_resource_id(a);
    let undo = revert_change(&change, &before, &after, &graph);
    replay_captured_deltas_onto(&mut graph, undo.edits);
    assert!(graph.find_semantic_statement(&handle).is_none());
    assert_eq!(graph.shown_resource_id(a), binding);
    assert_eq!(graph.get_node(a).unwrap().url(), "https://later.test/");
    assert!(
        graph
            .get_edge(graph.find_edge_key(a, b).unwrap())
            .unwrap()
            .has_relation(RelationSelector::Semantic(SemanticSubKind::UserGrouped))
    );
}

#[test]
fn production_routing_live_containment_and_legacy_snapshot_controls_are_separate() {
    let mut graph = Graph::new();
    let parent = graph.add_node("https://same.test/a/".into(), Default::default());
    let child = graph.add_node("https://same.test/a/b".into(), Default::default());
    let baseline = graph.to_snapshot();
    graph.derive_containment_for(child);
    assert!(graph.find_edge_key(child, parent).is_none());
    assert!(graph.resource_relations().any(|(_, _, _, p)| {
        p.has_relation(RelationSelector::Containment(ContainmentSubKind::UrlPath))
    }));
    let legacy = Graph::try_from_snapshot(&baseline).unwrap();
    assert!(
        legacy
            .get_edge(legacy.find_edge_key(child, parent).unwrap())
            .unwrap()
            .has_relation(RelationSelector::Containment(ContainmentSubKind::UrlPath))
    );
    assert_eq!(legacy.resource_relations().count(), 0);
}

#[test]
fn production_routing_semantic_and_literal_handles_cannot_cross_claim_types() {
    use crate::types::NodeProperty;
    for legacy in [false, true] {
        let (mut graph, a, b) = bound_graph();
        let mut property = NodeProperty::new("urn:property:name".into(), "held literal".into());
        property.statement_id = "held-literal".into();
        if legacy {
            let node_id = graph.get_node(a).unwrap().id;
            apply_graph_delta(
                &mut graph,
                GraphDelta::ReplayAppendNodePropertyById {
                    node_id,
                    property: property.clone(),
                },
            );
        } else {
            graph
                .append_resource_properties(
                    graph.shown_resource_id(a).unwrap(),
                    vec![property.clone()],
                )
                .unwrap();
        }
        let before = truth(&graph);
        let mut statement = SemanticStatement::new(
            "urn:semantic:claim".into(),
            None,
            None,
            GraphScope::Default,
            Some("urn:author".into()),
            Some(10),
        );
        statement.statement_id = property.statement_id;
        assert!(matches!(
            graph.try_assert_persisted_semantic_statement(a, b, statement.clone()),
            Err(StatementWriteError::HandleCollision(_))
        ));
        assert_eq!(truth(&graph), before, "collision refuses without mutations");
        statement.statement_id = "distinct-semantic".into();
        graph
            .try_assert_persisted_semantic_statement(a, b, statement)
            .unwrap();
        let held = graph
            .find_semantic_statement("distinct-semantic")
            .unwrap()
            .1
            .clone();
        let mut colliding_literal = NodeProperty::new("urn:property:other".into(), "other".into());
        colliding_literal.statement_id = held.statement_id.clone();
        let before = truth(&graph);
        assert!(
            graph
                .append_resource_properties(
                    graph.shown_resource_id(b).unwrap(),
                    vec![colliding_literal.clone()]
                )
                .is_err()
        );
        assert_eq!(truth(&graph), before);
        colliding_literal.statement_id = "distinct-literal".into();
        assert!(
            graph
                .append_resource_properties(
                    graph.shown_resource_id(b).unwrap(),
                    vec![colliding_literal]
                )
                .unwrap()
        );
        assert_eq!(
            graph.find_semantic_statement(&held.statement_id).unwrap().1,
            &held
        );
    }
}
