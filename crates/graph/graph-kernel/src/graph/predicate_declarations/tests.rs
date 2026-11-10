// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::sync::{Arc, Mutex};

use super::*;
use crate::graph::revert::{Part, revert_change};
use crate::graph::{
    EdgePayload, SemanticStatement, SemanticSubKind, predicate_iri, replay_captured_deltas_onto,
};
use crate::types::GraphScope;
use euclid::default::Point2D;

const PREDICATE: &str = "https://Vocabulary.test/terms?edition=1#custom";

fn value(id: &str, stratum: GraphStratum, author: Author) -> Value {
    serde_json::to_value(PredicateDeclarations {
        variants: vec![PredicateDeclaration {
            declaration_id: id.into(),
            stratum,
            author,
            declared_at_ms: Some(17),
        }],
        selected: Some(id.into()),
    })
    .unwrap()
}

fn record(predicate: &str, value: &Value) -> PersistedResourceRecord {
    PersistedResourceRecord {
        canonical_iri: predicate.into(),
        facets: vec![PersistedResourceFacet {
            facet: PREDICATE_DECLARATIONS_FACET.into(),
            value_json: value.to_string(),
        }],
    }
}

fn capture(graph: &mut Graph) -> Arc<Mutex<Vec<CapturedDelta>>> {
    let deltas = Arc::new(Mutex::new(vec![]));
    let recorded = Arc::clone(&deltas);
    graph.set_recorder(Some(Arc::new(move |delta| {
        recorded.lock().unwrap().push(delta.clone())
    })));
    deltas
}

#[test]
fn predicate_declarations_persist_exact_origins_identity_and_replay() {
    let mut graph = Graph::new();
    let deltas = capture(&mut graph);
    let mut author = Author::engine("extractor", "v2");
    author.via = Some("origin-host".into());
    let id = graph.write_as(author.clone(), |graph| {
        graph
            .declare_predicate(PREDICATE, GraphStratum::Surface)
            .unwrap()
    });
    let declarations = graph.predicate_declarations(PREDICATE).unwrap().unwrap();
    assert_eq!(declarations.selected.as_deref(), Some(id.as_str()));
    assert_eq!(declarations.variants[0].author, author);
    assert!(declarations.variants[0].declared_at_ms.is_some());
    assert_eq!(
        graph
            .resource(ResourceNode::for_term(PREDICATE).id())
            .unwrap()
            .canonical_iri(),
        PREDICATE
    );
    assert_eq!(
        graph.node_count(),
        0,
        "a declaration needs no invented surface"
    );
    assert_eq!(
        graph.clone().predicate_declarations(PREDICATE).unwrap(),
        Some(declarations.clone())
    );
    let snapshot = graph.to_snapshot();
    let bytes = serde_json::to_vec(&snapshot).unwrap();
    let saved = serde_json::from_slice(&bytes).unwrap();
    let restored = Graph::try_from_snapshot(&saved).unwrap();
    assert_eq!(
        restored.predicate_declarations(PREDICATE).unwrap(),
        Some(declarations.clone())
    );
    let captured = deltas.lock().unwrap().clone();
    assert_eq!(captured.len(), 1);
    assert!(matches!(
        &captured[0],
        CapturedDelta::ReplaySetResourceRecordById { .. }
    ));
    let captured: Vec<CapturedDelta> =
        postcard::from_bytes(&postcard::to_allocvec(&captured).unwrap()).unwrap();
    let mut replayed = Graph::new();
    replay_captured_deltas_onto(&mut replayed, captured.iter().cloned());
    assert_eq!(
        replayed.predicate_declarations(PREDICATE).unwrap(),
        Some(declarations)
    );
    assert_eq!(
        replayed.effective_predicate_stratum(PREDICATE),
        Ok(GraphStratum::Surface)
    );
    assert_eq!(
        replayed.effective_predicate_stratum("https://unrelated.test/predicate"),
        Ok(GraphStratum::Resource)
    );
    let revision = graph.revision();
    let same_id = graph.write_as(author, |graph| {
        graph
            .declare_predicate(PREDICATE, GraphStratum::Surface)
            .unwrap()
    });
    assert_eq!(id, same_id);
    assert_eq!(graph.revision(), revision);
    assert_eq!(
        deltas.lock().unwrap().len(),
        1,
        "no-op declarations emit no capture"
    );
}

#[test]
fn predicate_declarations_conflicts_require_exact_choice_with_unambiguous_controls() {
    let left = value("alice", GraphStratum::Resource, Author::person("alice"));
    let right = value("bob", GraphStratum::Surface, Author::person("bob"));
    let merged = merge_predicate_declarations(PREDICATE, &left, &right).unwrap();
    assert_eq!(
        merge_predicate_declarations(PREDICATE, &right, &left).unwrap(),
        merged
    );
    let declarations = parse_predicate_declarations(PREDICATE, &merged).unwrap();
    assert_eq!(declarations.variants.len(), 2);
    assert_eq!(declarations.selected, None);
    assert!(matches!(
        declarations.effective_stratum(PREDICATE),
        Err(PredicateDeclarationError::Conflict { .. })
    ));
    assert_eq!(
        merge_predicate_declarations(PREDICATE, &left, &left).unwrap(),
        left
    );
    let agreement = value("carol", GraphStratum::Resource, Author::person("carol"));
    let agreed = merge_predicate_declarations(PREDICATE, &left, &agreement).unwrap();
    assert_eq!(
        parse_predicate_declarations(PREDICATE, &agreed)
            .unwrap()
            .effective_stratum(PREDICATE),
        Ok(GraphStratum::Resource)
    );
    let mut graph = Graph::new();
    let resource_id = ResourceNode::for_term(PREDICATE).id();
    assert!(graph.set_resource_record(resource_id, Some(record(PREDICATE, &merged))));
    assert!(matches!(
        graph.effective_predicate_stratum(PREDICATE),
        Err(PredicateDeclarationError::Conflict { .. })
    ));
    assert_eq!(
        graph.effective_predicate_stratum("https://unrelated.test/predicate"),
        Ok(GraphStratum::Resource)
    );
    let deltas = capture(&mut graph);
    let before = graph.predicate_declarations(PREDICATE).unwrap().unwrap();
    assert!(
        graph
            .select_predicate_declaration(PREDICATE, "absent")
            .is_err()
    );
    assert_eq!(
        graph.predicate_declarations(PREDICATE).unwrap().unwrap(),
        before
    );
    assert!(deltas.lock().unwrap().is_empty());
    assert!(
        graph
            .select_predicate_declaration(PREDICATE, "bob")
            .unwrap()
    );
    assert_eq!(
        graph.effective_predicate_stratum(PREDICATE),
        Ok(GraphStratum::Surface)
    );
    assert_eq!(
        graph
            .predicate_declarations(PREDICATE)
            .unwrap()
            .unwrap()
            .variants,
        before.variants
    );
    assert!(
        !graph
            .select_predicate_declaration(PREDICATE, "bob")
            .unwrap()
    );
    assert_eq!(deltas.lock().unwrap().len(), 1);
    let mut replayed = Graph::new();
    assert!(replayed.set_resource_record(resource_id, Some(record(PREDICATE, &merged))));
    replay_captured_deltas_onto(&mut replayed, deltas.lock().unwrap().iter().cloned());
    assert_eq!(
        replayed.effective_predicate_stratum(PREDICATE),
        Ok(GraphStratum::Surface)
    );
}

#[test]
fn predicate_declarations_reused_origin_and_builtin_edits_are_atomic_errors() {
    let left = value("same-id", GraphStratum::Resource, Author::person("alice"));
    let different_origin = value("same-id", GraphStratum::Resource, Author::person("bob"));
    assert!(merge_predicate_declarations(PREDICATE, &left, &different_origin).is_err());
    let distinct = value(
        "different-id",
        GraphStratum::Resource,
        Author::person("bob"),
    );
    assert!(merge_predicate_declarations(PREDICATE, &left, &distinct).is_ok());
    assert_eq!(
        left,
        value("same-id", GraphStratum::Resource, Author::person("alice"))
    );
    let mut graph = Graph::new();
    let deltas = capture(&mut graph);
    for kind in [SemanticSubKind::UserGrouped, SemanticSubKind::Cites] {
        let iri = predicate_iri(kind);
        let before = graph.effective_predicate_stratum(iri).unwrap();
        assert!(matches!(
            graph.declare_predicate(iri, GraphStratum::Surface),
            Err(PredicateDeclarationError::BuiltIn { .. })
        ));
        assert!(graph.select_predicate_declaration(iri, "any").is_err());
        assert_eq!(graph.effective_predicate_stratum(iri), Ok(before));
    }
    assert_eq!(graph.resource_nodes().count(), 0);
    assert_eq!(graph.revision(), 0);
    assert!(deltas.lock().unwrap().is_empty());
    assert!(
        graph
            .declare_predicate(PREDICATE, GraphStratum::Surface)
            .is_ok()
    );
    assert_eq!(graph.resource_nodes().count(), 1);
    assert_eq!(deltas.lock().unwrap().len(), 1);
}

#[test]
fn predicate_declarations_checked_load_and_raw_record_reject_invalid_typed_data() {
    let mut graph = Graph::new();
    let resource_id = ResourceNode::for_term(PREDICATE).id();
    let valid = value("valid", GraphStratum::Surface, Author::person("alice"));
    assert!(graph.set_resource_record(resource_id, Some(record(PREDICATE, &valid))));
    let snapshot = graph.to_snapshot();
    let mut bad = valid.clone();
    bad["selected"] = Value::String("absent".into());
    assert!(!graph.set_resource_record(resource_id, Some(record(PREDICATE, &bad))));
    assert_eq!(graph.to_snapshot().resources, snapshot.resources);
    let mut invalid = snapshot.clone();
    invalid.resources[0] = record(PREDICATE, &bad);
    let bytes = serde_json::to_vec(&invalid).unwrap();
    let error = Graph::try_from_snapshot(&invalid)
        .err()
        .expect("invalid typed declaration")
        .to_string();
    assert!(
        error.contains(PREDICATE_DECLARATIONS_FACET) && error.contains("absent"),
        "{error}"
    );
    assert_eq!(
        serde_json::to_vec(&invalid).unwrap(),
        bytes,
        "failed load retains source bytes"
    );
    assert!(Graph::try_from_snapshot(&snapshot).is_ok());
    let mut opaque = record(PREDICATE, &bad);
    opaque.facets[0].facet = "foreign.uninterpreted".into();
    invalid.resources[0] = opaque.clone();
    let restored = Graph::try_from_snapshot(&invalid).unwrap();
    assert_eq!(restored.resource_record(resource_id).unwrap(), opaque);
    assert_eq!(
        restored.effective_predicate_stratum(PREDICATE),
        Ok(GraphStratum::Resource)
    );
    let legacy = Graph::new().to_snapshot();
    assert_eq!(
        Graph::try_from_snapshot(&legacy)
            .unwrap()
            .effective_predicate_stratum(PREDICATE),
        Ok(GraphStratum::Resource)
    );
}

#[test]
fn predicate_declarations_conditional_undo_keeps_later_facets_and_declarations() {
    let mut graph = Graph::new();
    let resource_id = ResourceNode::for_term(PREDICATE).id();
    assert!(graph.set_resource_record(
        resource_id,
        Some(PersistedResourceRecord {
            canonical_iri: PREDICATE.into(),
            facets: vec![PersistedResourceFacet {
                facet: "foreign.before".into(),
                value_json: "1".into()
            }]
        })
    ));
    let before = graph.clone();
    let deltas = capture(&mut graph);
    graph
        .declare_predicate(PREDICATE, GraphStratum::Surface)
        .unwrap();
    let change = deltas.lock().unwrap().clone();
    let after = graph.clone();
    let mut updated = graph.resource_record(resource_id).unwrap();
    updated.facets.push(PersistedResourceFacet {
        facet: "foreign.later".into(),
        value_json: "{\"nested\":[2]}".into(),
    });
    assert!(graph.set_resource_record(resource_id, Some(updated)));
    let undo = revert_change(&change, &before, &after, &graph);
    assert!(undo.kept.is_empty());
    replay_captured_deltas_onto(&mut graph, undo.edits.iter().cloned());
    assert_eq!(graph.predicate_declarations(PREDICATE).unwrap(), None);
    assert_eq!(graph.resource_record(resource_id).unwrap().facets.len(), 2);
    assert_eq!(
        graph.effective_predicate_stratum(PREDICATE),
        Ok(GraphStratum::Resource)
    );
    let mut later = after.clone();
    later.write_as(Author::person("later-author"), |graph| {
        graph
            .declare_predicate(PREDICATE, GraphStratum::Resource)
            .unwrap()
    });
    let current = later.predicate_declarations(PREDICATE).unwrap();
    let kept = revert_change(&change, &before, &after, &later);
    assert!(kept.kept.contains(&Part::ResourceFacet(
        resource_id,
        PREDICATE_DECLARATIONS_FACET.into()
    )));
    replay_captured_deltas_onto(&mut later, kept.edits.iter().cloned());
    assert_eq!(later.predicate_declarations(PREDICATE).unwrap(), current);
    assert_eq!(
        later.effective_predicate_stratum(PREDICATE),
        Ok(GraphStratum::Resource)
    );
}

#[test]
fn predicate_declarations_nature_changes_leave_held_claims_in_both_strata_exact() {
    let mut graph = Graph::new();
    let a = graph.add_node("https://surface.test/a".into(), Point2D::zero());
    let b = graph.add_node("https://surface.test/b".into(), Point2D::zero());
    let held = |id: &str| {
        let mut payload = EdgePayload::new();
        payload.push_persisted_semantic_statement(SemanticStatement {
            statement_id: id.into(),
            predicate: PREDICATE.into(),
            recognized_sub_kind: None,
            label: Some("held claim".into()),
            graph_scope: GraphScope::User,
            provenance_iri: Some("https://original-author.test/".into()),
            asserted_at_ms: Some(19),
            first_asserted_ms: Some(19),
            withdrawn: None,
            basis: Default::default(),
        });
        payload
    };
    graph.connect_surface(a, b, held("surface-claim"));
    let ra = ResourceNode::for_term("https://resource.test/a");
    let rb = ResourceNode::for_term("https://resource.test/b");
    for resource in [&ra, &rb] {
        assert!(graph.set_resource_record(
            resource.id(),
            Some(PersistedResourceRecord {
                canonical_iri: resource.canonical_iri().into(),
                facets: vec![]
            })
        ));
    }
    let edge =
        super::super::snapshot::persisted_edge_for_ids(ra.id(), rb.id(), &held("resource-claim"));
    assert!(graph.set_resource_edges_between(ra.id(), rb.id(), &[edge]));
    let before = graph.to_snapshot();
    let deltas = capture(&mut graph);
    let first = graph
        .declare_predicate(PREDICATE, GraphStratum::Surface)
        .unwrap();
    let second = graph
        .declare_predicate(PREDICATE, GraphStratum::Resource)
        .unwrap();
    assert_ne!(first, second);
    assert_eq!(
        graph.effective_predicate_stratum(PREDICATE),
        Ok(GraphStratum::Resource)
    );
    assert_eq!(
        graph
            .predicate_declarations(PREDICATE)
            .unwrap()
            .unwrap()
            .variants
            .len(),
        2
    );
    assert_eq!(graph.to_snapshot().edges, before.edges);
    assert_eq!(graph.to_snapshot().resource_edges, before.resource_edges);
    assert_eq!(deltas.lock().unwrap().len(), 2);
    assert!(
        deltas
            .lock()
            .unwrap()
            .iter()
            .all(|delta| matches!(delta, CapturedDelta::ReplaySetResourceRecordById { .. }))
    );
    let mut replayed = Graph::try_from_snapshot(&before).unwrap();
    let restored_before = replayed.to_snapshot();
    assert!(restored_before.edges.iter().any(|edge| {
        edge.retained_assertions()
            .iter()
            .any(|statement| statement.statement_id == "surface-claim")
    }));
    assert!(
        restored_before
            .edges
            .iter()
            .any(|edge| edge.retained_assertions().iter().any(|record| {
                super::super::RelationKind::from_predicate_iri(&record.predicate)
                    .is_some_and(|kind| matches!(kind, super::super::RelationKind::Containment(_)))
            })),
        "restored aggregate control is present"
    );
    replay_captured_deltas_onto(&mut replayed, deltas.lock().unwrap().iter().cloned());
    assert_eq!(replayed.to_snapshot().edges, restored_before.edges);
    assert_eq!(
        replayed.to_snapshot().resource_edges,
        restored_before.resource_edges
    );
    assert_eq!(
        replayed.predicate_declarations(PREDICATE).unwrap(),
        graph.predicate_declarations(PREDICATE).unwrap()
    );
}
