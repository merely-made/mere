// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::graph::{SemanticStatementSpec, apply};

fn source(prefix: &str) -> (Graph, uuid::Uuid, uuid::Uuid, String) {
    let mut graph = Graph::new();
    let a = apply::add_node(
        &mut graph,
        None,
        format!("https://{prefix}.test/a"),
        Default::default(),
    );
    let b = apply::add_node(
        &mut graph,
        None,
        format!("https://{prefix}.test/b"),
        Default::default(),
    );
    let a = graph.shown_resource_id(a).unwrap();
    let b = graph.shown_resource_id(b).unwrap();
    let (_, claim) = graph
        .try_assert_semantic_statement_by_resource_ids(
            a,
            b,
            SemanticStatementSpec {
                predicate: "urn:test:claim".into(),
                provenance_iri: Some("urn:test:author".into()),
                ..Default::default()
            },
        )
        .unwrap();
    (graph, a, b, claim.statement_id)
}
fn spec() -> SubgraphSpec {
    SubgraphSpec { kind: SubgraphKind::Sparql {
        query:"SELECT ?member WHERE { { ?member <urn:test:claim> ?to } UNION { ?from <urn:test:claim> ?member } }".into(), member_variable:"member".into()
    }, anchors:vec![],primary_anchor:None,selectors:vec![] }
}

#[test]
fn saved_query_reconciles_at_revision_and_shared_spec_uses_receiver_graph() {
    let (mut graph, a, b, claim) = source("one");
    let mut index = SessionSubgraphs::new();
    let id = index.record_linked(&graph, spec());
    let mut expected = vec![a, b];
    expected.sort();
    assert_eq!(index.get(id).unwrap().anchors, expected);
    let calls = index.derive_calls.get();
    assert!(index.reconcile(&graph, id).is_none());
    assert_eq!(
        index.derive_calls.get(),
        calls,
        "unchanged revision skips evaluator"
    );
    // The shared payload is the spec alone; a second graph derives its own resources.
    let wire = serde_json::to_string(&spec()).unwrap();
    assert!(!wire.contains(&a.to_string()));
    let (receiver, c, d, _) = source("two");
    let shared: SubgraphSpec = serde_json::from_str(&wire).unwrap();
    let mut received = SessionSubgraphs::new();
    let other = received.record_linked(&receiver, shared);
    let mut expected = vec![c, d];
    expected.sort();
    assert_eq!(received.get(other).unwrap().anchors, expected);
    assert!(graph.retract_assertion(&claim));
    let delta = index.reconcile(&graph, id).unwrap();
    assert_eq!(delta.removed.len(), 2);
    assert!(index.get(id).unwrap().anchors.is_empty());
    assert_eq!(index.derive_calls.get(), calls + 1);
}

#[test]
fn frozen_query_stays_exact_after_retraction_while_live_roster_changes() {
    let (mut graph, a, b, claim) = source("frozen");
    let mut index = SessionSubgraphs::new();
    let id = index.try_record_linked(&graph, spec()).unwrap();
    let revision = graph.revision();
    let owner = index.freeze_linked(&mut graph, id).unwrap();
    let frozen = graph
        .resource(owner)
        .unwrap()
        .nested_selection()
        .unwrap()
        .clone();
    assert_eq!(frozen.revision(), revision);
    assert_eq!(frozen.spec(), &serde_json::to_value(spec()).unwrap());
    assert!(frozen.members().contains(&a));
    assert!(frozen.members().contains(&b));
    assert_eq!(
        frozen.statements()[0].semantic.as_ref().unwrap().statements[0].statement_id,
        claim
    );
    assert!(graph.retract_assertion(&claim));
    assert_eq!(index.reconcile(&graph, id).unwrap().removed.len(), 2);
    assert_eq!(
        graph.resource(owner).unwrap().nested_selection(),
        Some(&frozen)
    );
    // A stale cached roster cannot leak into a later freeze.
    let again = index.freeze_linked(&mut graph, id).unwrap();
    assert!(
        graph
            .resource(again)
            .unwrap()
            .nested_selection()
            .unwrap()
            .members()
            .is_empty()
    );
}

#[test]
fn invalid_query_preserves_roster_and_never_caches_refusal_then_spec_edit_recovers() {
    let (graph, _, _, _) = source("refusal");
    let mut index = SessionSubgraphs::new();
    let id = index.try_record_linked(&graph, spec()).unwrap();
    let before = index.get(id).unwrap().anchors.clone();
    let mut invalid = spec();
    invalid.kind = SubgraphKind::Sparql {
        query: "SELECT ?wrong WHERE { ?wrong ?p ?o }".into(),
        member_variable: "member".into(),
    };
    assert!(index.set_linked_spec(id, invalid));
    assert!(index.try_reconcile(&graph, id).is_err());
    assert_eq!(index.get(id).unwrap().anchors, before);
    assert!(
        index
            .derivation_error(id)
            .unwrap()
            .contains("member variable")
    );
    assert_eq!(index.reconciled_revision(id), None);
    let calls = index.derive_calls.get();
    assert!(index.reconcile(&graph, id).is_none());
    assert_eq!(index.derive_calls.get(), calls + 1);
    assert!(index.set_linked_spec(id, spec()));
    assert!(index.try_reconcile(&graph, id).unwrap().is_none());
    assert!(index.derivation_error(id).is_none());
    assert_eq!(index.reconciled_revision(id), Some(graph.revision()));
    let shared = index.shared_spec(id).unwrap();
    let wire = serde_json::to_string(&index).unwrap();
    let mut loaded: SessionSubgraphs = serde_json::from_str(&wire).unwrap();
    assert_eq!(loaded.reconciled_revision(id), None);
    assert_eq!(loaded.shared_spec(id), Some(shared));
    assert!(loaded.try_reconcile(&graph, id).unwrap().is_none());
    assert_eq!(loaded.reconciled_revision(id), Some(graph.revision()));
}

#[test]
fn coverage_refreshes_without_re_evaluation_and_unavailable_freeze_refuses() {
    use kernel::graph::{CoverageLayer, CoverageLimit, CoverageNote, ResourceNode};
    let (mut graph, _, _, _) = source("coverage");
    let missing = ResourceNode::for_term("urn:missing:member").id();
    let mut query = spec();
    query.kind = SubgraphKind::Sparql {
        query: "SELECT ?member WHERE { VALUES ?member { <urn:missing:member> } }".into(),
        member_variable: "member".into(),
    };
    let mut index = SessionSubgraphs::new();
    let id = index.try_record_linked(&graph, query).unwrap();
    assert!(
        index
            .coverage(&graph, id)
            .unwrap()
            .limits
            .iter()
            .any(|l| l.layer == CoverageLayer::Possession)
    );
    let calls = index.derive_calls.get();
    let revision = graph.revision();
    let mut limit = CoverageLimit::new(CoverageLayer::Residency, "not loaded");
    limit.resources = vec![missing];
    graph.set_known_coverage(CoverageNote {
        limits: vec![limit],
    });
    assert_eq!(graph.revision(), revision);
    assert!(index.reconcile(&graph, id).is_none());
    assert_eq!(index.derive_calls.get(), calls);
    let coverage = index.coverage(&graph, id).unwrap();
    assert!(
        coverage
            .limits
            .iter()
            .any(|l| l.layer == CoverageLayer::Residency)
    );
    assert!(
        !coverage
            .limits
            .iter()
            .any(|l| l.layer == CoverageLayer::Possession)
    );
    assert!(index.freeze_linked(&mut graph, id).is_err());
    assert_eq!(graph.revision(), revision);
}

#[test]
fn missing_shape_seed_freeze_refuses_without_confusing_a_valid_empty_query() {
    let (mut graph, a, _, _) = source("shape-freeze");
    let mut index = SessionSubgraphs::new();
    let missing = uuid::Uuid::from_u128(999);
    for (kind, missing) in [
        (SubgraphKind::Component, missing),
        (SubgraphKind::Ego { radius: 1 }, missing),
        (SubgraphKind::Component, a),
    ] {
        let definition = SubgraphSpec {
            kind,
            anchors: vec![missing.to_string()],
            primary_anchor: Some(missing.to_string()),
            selectors: vec![],
        };
        let id = index.try_record_linked(&graph, definition).unwrap();
        let revision = graph.revision();
        let count = graph.resource_nodes().count();
        assert!(
            index.freeze_linked(&mut graph, id).is_err(),
            "explicit missing seed is not an empty selection"
        );
        assert_eq!(graph.revision(), revision);
        assert_eq!(graph.resource_nodes().count(), count);
    }
    let held_surface = graph.surface_ids_showing_resource(a)[0];
    let shape = index
        .try_record_linked(
            &graph,
            SubgraphSpec {
                kind: SubgraphKind::Ego { radius: 0 },
                anchors: vec![held_surface.to_string()],
                primary_anchor: Some(held_surface.to_string()),
                selectors: vec![],
            },
        )
        .unwrap();
    let owner = index.freeze_linked(&mut graph, shape).unwrap();
    assert_eq!(
        graph
            .resource(owner)
            .unwrap()
            .nested_selection()
            .unwrap()
            .members(),
        &[a]
    );
    let mut empty = spec();
    empty.kind = SubgraphKind::Sparql {
        query: "SELECT ?member WHERE { VALUES ?member { UNDEF } }".into(),
        member_variable: "member".into(),
    };
    let empty = index.try_record_linked(&graph, empty).unwrap();
    let owner = index.freeze_linked(&mut graph, empty).unwrap();
    assert!(
        graph
            .resource(owner)
            .unwrap()
            .nested_selection()
            .unwrap()
            .members()
            .is_empty()
    );
}

#[test]
fn shape_and_query_freeze_resolve_members_in_their_own_strata_when_uuid_values_overlap() {
    let (mut graph, a, b, _) = source("overlap");
    let url = graph.resource(b).unwrap().canonical_iri().to_string();
    let surface = apply::add_node(&mut graph, Some(a), url, Default::default());
    assert_eq!(graph.get_node(surface).unwrap().id, a);
    assert_eq!(graph.shown_resource_id(surface), Some(b));
    let mut index = SessionSubgraphs::new();
    let shape = index
        .try_record_linked(
            &graph,
            SubgraphSpec {
                kind: SubgraphKind::Session,
                anchors: vec![a.to_string()],
                primary_anchor: Some(a.to_string()),
                selectors: vec![],
            },
        )
        .unwrap();
    let owner = index.freeze_linked(&mut graph, shape).unwrap();
    assert_eq!(
        graph
            .resource(owner)
            .unwrap()
            .nested_selection()
            .unwrap()
            .members(),
        &[b],
        "shape ids refer to Surfaces and resolve their shown Resource"
    );
    let mut query = spec();
    query.kind = SubgraphKind::Sparql {
        query: "SELECT ?member WHERE { VALUES ?member { <https://overlap.test/a> } }".into(),
        member_variable: "member".into(),
    };
    let query = index.try_record_linked(&graph, query).unwrap();
    let owner = index.freeze_linked(&mut graph, query).unwrap();
    assert_eq!(
        graph
            .resource(owner)
            .unwrap()
            .nested_selection()
            .unwrap()
            .members(),
        &[a],
        "query ids already refer to Resources"
    );
}
