// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::graph::{
    Author, PredicateDeclarations, predicate_declarations::PREDICATE_DECLARATIONS_FACET,
};
use crate::persistence::{PersistedResourceFacet, PersistedResourceRecord};

fn spec(predicate: &str, source: &str) -> SemanticStatementSpec {
    SemanticStatementSpec {
        predicate: predicate.into(),
        provenance_iri: Some(source.into()),
        ..Default::default()
    }
}

#[test]
fn nature_changes_keep_held_handles_and_route_only_new_assertions() {
    let mut graph = Graph::new();
    let a = graph.add_node_with_id(
        Uuid::from_u128(100),
        "https://example.org/a".into(),
        Default::default(),
    );
    let b = graph.add_node_with_id(
        Uuid::from_u128(101),
        "https://example.org/b".into(),
        Default::default(),
    );
    let predicate = "https://example.org/predicate#rel";
    let (resource_key, first) = graph
        .try_assert_semantic_statement(a, b, spec(predicate, "urn:author:first"))
        .unwrap();
    assert!(matches!(resource_key, RelationKey::Resource(_)));
    graph
        .declare_predicate(predicate, GraphStratum::Surface)
        .unwrap();
    let mut update = spec(predicate, "urn:author:first");
    update.label = Some("updated".into());
    let (held_key, updated) = graph.try_assert_semantic_statement(a, b, update).unwrap();
    assert_eq!(held_key, resource_key);
    assert_eq!(updated.statement_id, first.statement_id);
    let (surface_key, second) = graph
        .try_assert_semantic_statement(a, b, spec(predicate, "urn:author:second"))
        .unwrap();
    assert!(matches!(surface_key, RelationKey::Surface(_)));
    graph
        .declare_predicate(predicate, GraphStratum::Resource)
        .unwrap();
    let (legacy_key, legacy) = graph
        .try_assert_semantic_statement_by_resource_ids(
            graph.shown_resource_id(a).unwrap(),
            graph.shown_resource_id(b).unwrap(),
            spec(predicate, "urn:author:second"),
        )
        .unwrap();
    assert_eq!(legacy_key, surface_key);
    assert_eq!(legacy.statement_id, second.statement_id);

    assert_eq!(
        graph
            .find_semantic_statement(&first.statement_id)
            .unwrap()
            .0,
        resource_key
    );
    graph.update_node_url(a, "https://example.org/later".into());
    graph.refresh_surface_resource(a);
    assert!(graph.retract_assertion(&first.statement_id));
    assert!(
        graph
            .find_semantic_statement(&first.statement_id)
            .unwrap()
            .1
            .withdrawn
            .is_some()
    );
    assert!(
        graph
            .find_held_semantic_statement(&first.statement_id)
            .is_none()
    );
    assert!(
        graph
            .find_semantic_statement(&second.statement_id)
            .is_some()
    );
    assert!(!graph.retract_assertion("absent"));
    assert!(graph.retract_assertion(&second.statement_id));
}

#[test]
fn conflict_blocks_new_writes_atomically_with_existing_and_unaffected_controls() {
    let mut graph = Graph::new();
    let a = graph.add_node_with_id(
        Uuid::from_u128(102),
        "https://example.org/a".into(),
        Default::default(),
    );
    let b = graph.add_node_with_id(
        Uuid::from_u128(103),
        "https://example.org/b".into(),
        Default::default(),
    );
    let predicate = "urn:predicate:conflicted";
    let (_, held) = graph
        .try_assert_semantic_statement(a, b, spec(predicate, "urn:author:held"))
        .unwrap();
    graph
        .declare_predicate(predicate, GraphStratum::Resource)
        .unwrap();
    graph
        .write_as(Author::engine("other", "2"), |g| {
            g.declare_predicate(predicate, GraphStratum::Surface)
        })
        .unwrap();
    let mut declarations = graph.predicate_declarations(predicate).unwrap().unwrap();
    declarations.selected = None;
    let selected = declarations.variants[0].declaration_id.clone();
    let record = PersistedResourceRecord {
        canonical_iri: predicate.into(),
        facets: vec![PersistedResourceFacet {
            facet: PREDICATE_DECLARATIONS_FACET.into(),
            value_json: serde_json::to_string(&declarations).unwrap(),
        }],
    };
    assert!(graph.set_resource_record(ResourceNode::for_term(predicate).id(), Some(record)));
    let before = serde_json::to_value(graph.to_snapshot()).unwrap();
    assert!(matches!(
        graph.try_assert_semantic_statement(a, b, spec(predicate, "urn:author:new")),
        Err(StatementWriteError::Declaration(
            PredicateDeclarationError::Conflict { .. }
        ))
    ));
    assert_eq!(serde_json::to_value(graph.to_snapshot()).unwrap(), before);
    assert!(matches!(
        graph.try_assert_semantic_statement_by_resource_ids(
            graph.shown_resource_id(a).unwrap(),
            graph.shown_resource_id(b).unwrap(),
            spec(predicate, "urn:author:new")
        ),
        Err(StatementWriteError::Declaration(
            PredicateDeclarationError::Conflict { .. }
        ))
    ));
    assert_eq!(serde_json::to_value(graph.to_snapshot()).unwrap(), before);
    let (_, existing) = graph
        .try_assert_semantic_statement(a, b, spec(predicate, "urn:author:held"))
        .unwrap();
    assert_eq!(existing.statement_id, held.statement_id);
    assert!(!existing.changed);
    assert!(
        graph
            .try_assert_semantic_statement(a, b, spec("urn:predicate:unaffected", "urn:author:new"))
            .is_ok()
    );
    graph
        .select_predicate_declaration(predicate, &selected)
        .unwrap();
    assert!(
        graph
            .try_assert_semantic_statement(a, b, spec(predicate, "urn:author:new"))
            .is_ok()
    );
    assert!(graph.retract_assertion(&held.statement_id));
    // The conflict's full origin records survive every assertion operation.
    let carried: PredicateDeclarations = graph.predicate_declarations(predicate).unwrap().unwrap();
    assert_eq!(carried.variants, declarations.variants);
}

#[test]
fn caller_selected_handle_cannot_change_claim_with_distinct_handle_control() {
    let mut graph = Graph::new();
    let a = graph.add_node_with_id(
        Uuid::from_u128(104),
        "https://example.org/a".into(),
        Default::default(),
    );
    let b = graph.add_node_with_id(
        Uuid::from_u128(105),
        "https://example.org/b".into(),
        Default::default(),
    );
    let (_, held) = graph
        .try_assert_semantic_statement(a, b, spec("urn:predicate:original", "urn:author:held"))
        .unwrap();
    let mut other = SemanticStatement::new(
        "urn:predicate:other".into(),
        None,
        None,
        Default::default(),
        Some("urn:author:held".into()),
        None,
    );
    let distinct = other.statement_id.clone();
    other.statement_id = held.statement_id.clone();
    let before = serde_json::to_value(graph.to_snapshot()).unwrap();
    assert_eq!(
        graph.try_assert_persisted_semantic_statement(a, b, other.clone()),
        Err(StatementWriteError::HandleCollision(
            held.statement_id.clone()
        ))
    );
    assert_eq!(serde_json::to_value(graph.to_snapshot()).unwrap(), before);
    other.statement_id = distinct.clone();
    assert!(
        graph
            .try_assert_persisted_semantic_statement(a, b, other)
            .is_ok()
    );
    assert!(graph.find_semantic_statement(&distinct).is_some());
    assert!(graph.find_semantic_statement(&held.statement_id).is_some());
}

#[test]
fn held_pair_lookup_reads_only_the_pair_in_scan_order() {
    let mut graph = Graph::new();
    let [a, b, c] = ["a", "b", "c"]
        .map(|name| graph.add_node(format!("https://example.org/{name}"), Default::default()));
    let predicate = "https://example.org/predicate#rel";
    let (first, held) = graph
        .try_assert_semantic_statement(a, b, spec(predicate, "urn:author:first"))
        .unwrap();
    let (other, _) = graph
        .try_assert_semantic_statement(a, c, spec(predicate, "urn:author:first"))
        .unwrap();
    assert_ne!(first, other);
    // A parallel row carrying the same handle: the later edge by index wins,
    // as it did when every Resource relation was scanned in index order.
    let RelationKey::Resource(first_raw) = first else {
        panic!("resource stratum")
    };
    let payload = graph.get_resource_edge(first_raw).unwrap().clone();
    let (source, target) = graph
        .resources
        .inner()
        .edge_endpoints(first_raw.raw())
        .unwrap();
    let parallel = graph.connect_resource(source, target, payload);
    let (again, reasserted) = graph
        .try_assert_semantic_statement(a, b, spec(predicate, "urn:author:first"))
        .unwrap();
    assert_eq!(
        again,
        RelationKey::Resource(ResourceEdgeKey::from_raw(parallel))
    );
    assert_eq!(reasserted.statement_id, held.statement_id);
}

#[test]
fn surfaces_showing_a_resource_follow_reshowing_and_removal() {
    let mut graph = Graph::new();
    let [a, b] = ["a", "b"]
        .map(|name| graph.add_node(format!("https://example.org/{name}"), Default::default()));
    let (ra, rb) = (
        graph.ensure_surface_resource(a).unwrap(),
        graph.ensure_surface_resource(b).unwrap(),
    );
    let (ia, ib) = (graph.get_node(a).unwrap().id, graph.get_node(b).unwrap().id);
    assert_eq!(graph.surface_ids_showing_resource(ra), vec![ia]);
    assert!(graph.set_shown_resource(ia, Some(rb)));
    assert!(graph.surface_ids_showing_resource(ra).is_empty());
    let mut both = vec![ia, ib];
    both.sort();
    assert_eq!(graph.surface_ids_showing_resource(rb), both);
    assert!(graph.remove_node(a));
    assert_eq!(graph.surface_ids_showing_resource(rb), vec![ib]);
    assert!(graph.set_shown_resource(ib, None));
    assert!(graph.resource_surfaces.is_empty());
}
