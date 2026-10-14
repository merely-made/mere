// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
use super::*;
use kernel::graph::apply::add_node;
use kernel::graph::{CoverageLayer, CoverageLimit, EdgeAssertion, GraphStratum, SemanticSubKind};

fn fixture() -> (Graph, Uuid, Uuid, Uuid) {
    let mut graph = Graph::new();
    let a = add_node(
        &mut graph,
        Some(Uuid::from_u128(1)),
        "https://source.test/a".into(),
        Default::default(),
    );
    let alias = add_node(
        &mut graph,
        Some(Uuid::from_u128(2)),
        "https://source.test/a".into(),
        Default::default(),
    );
    let b = add_node(
        &mut graph,
        Some(Uuid::from_u128(3)),
        "https://source.test/b".into(),
        Default::default(),
    );
    apply_graph_delta(
        &mut graph,
        GraphDelta::AssertRelation {
            from: a,
            to: b,
            assertion: EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::UserGrouped,
                label: None,
                decay_progress: None,
            },
            asserter_iri: "https://author.test/root".into(),
        },
    );
    let resource = graph.shown_resource_id(a).unwrap();
    apply_graph_delta(
        &mut graph,
        GraphDelta::AssertRelation {
            from: a,
            to: b,
            assertion: EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Cites,
                label: Some("source citation".into()),
                decay_progress: None,
            },
            asserter_iri: "https://author.test/root".into(),
        },
    );
    assert_eq!(graph.shown_resource_id(alias), Some(resource));
    (graph, resource, Uuid::from_u128(1), Uuid::from_u128(2))
}
fn spec() -> QueryViewSpec {
    QueryViewSpec::select(
        "Saved source view",
        "SELECT ?member WHERE { VALUES ?member { <https://source.test/a> <https://source.test/b> } }",
    )
}
#[test]
fn shared_query_resource_preserves_two_accesses_and_read_only_evaluation() {
    let (mut graph, resource, a, alias) = fixture();
    let owner = save_spec(&mut graph, None, None, Uuid::from_u128(10), &spec()).unwrap();
    let rev = graph.revision();
    let before = serde_json::to_value(graph.to_snapshot_at(0)).unwrap();
    let projection = project_view(&graph, owner).unwrap();
    assert_eq!(projection.members.len(), 2);
    let node = projection
        .nodes
        .iter()
        .find(|node| node.identity == QueryNodeId::resource(resource))
        .unwrap();
    assert_eq!(node.held_accesses, vec![a, alias]);
    assert_eq!(
        projection
            .relations
            .iter()
            .filter(|r| r.kind == VisualRelationKind::ShownBinding)
            .count(),
        3
    );
    assert!(
        projection
            .relations
            .iter()
            .any(|r| r.kind == VisualRelationKind::SurfaceLink)
    );
    assert_eq!(graph.revision(), rev);
    assert_eq!(
        serde_json::to_value(graph.to_snapshot_at(0)).unwrap(),
        before
    );
    let shared = shared_spec(&graph, owner).unwrap();
    let decoded: QueryViewSpec = serde_json::from_str(&shared).unwrap();
    assert_eq!(decoded, spec());
    assert!(
        !shared.contains(&a.to_string()),
        "sharing carries the definition, not sender access state"
    );
}
#[test]
fn surface_buckets_preserve_parallel_assertions_without_resource_lifting() {
    let (mut graph, _, from, _) = fixture();
    let to = Uuid::from_u128(3);
    let original = graph
        .to_snapshot_at(0)
        .edges
        .into_iter()
        .find(|edge| edge.from_node_id == from.to_string() && edge.to_node_id == to.to_string())
        .unwrap();
    let mut parallel = original.clone();
    parallel.semantic.as_mut().unwrap().statements[0].statement_id =
        "urn:statement:surface-parallel".into();
    parallel.semantic.as_mut().unwrap().statements[0].provenance_iri =
        Some("urn:author:surface-other".into());
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetEdgesByIds {
            from_id: from,
            to_id: to,
            edges: vec![original, parallel],
        },
    );
    let owner = save_spec(&mut graph, None, None, Uuid::from_u128(910), &spec()).unwrap();
    let projection = project_view(&graph, owner).unwrap();
    let surface_links: Vec<_> = projection
        .relations
        .iter()
        .filter(|relation| relation.kind == VisualRelationKind::SurfaceLink)
        .collect();
    assert_eq!(surface_links.len(), 2);
    assert_ne!(
        surface_links[0].statements[0].statement_id,
        surface_links[1].statements[0].statement_id
    );
    assert_ne!(
        surface_links[0].statements[0].provenance_iri,
        surface_links[1].statements[0].provenance_iri
    );
    assert!(
        surface_links
            .iter()
            .all(|relation| relation.from == QueryNodeId::surface(from)
                && relation.source_from == relation.from
                && relation.source_to == relation.to)
    );
}
#[test]
fn unavailable_and_empty_membership_keep_coverage_and_do_not_mint_appearances() {
    let (mut graph, _, _, _) = fixture();
    let mut missing = spec();
    missing.definition.kind = SubgraphKind::Sparql {
        query: "SELECT ?member WHERE { VALUES ?member { <urn:missing:member> } }".into(),
        member_variable: "member".into(),
    };
    let owner = save_spec(&mut graph, None, None, Uuid::from_u128(11), &missing).unwrap();
    let count = graph.node_count();
    let projection = project_view(&graph, owner).unwrap();
    assert!(projection.members.is_empty());
    assert_eq!(projection.missing_resources.len(), 1);
    assert!(
        projection
            .coverage
            .limits
            .iter()
            .any(|l| l.layer == CoverageLayer::Possession)
    );
    assert!(freeze_view(&mut graph, owner, Uuid::from_u128(12)).is_err());
    let mut empty = spec();
    empty.definition.kind = SubgraphKind::Sparql {
        query: "SELECT ?member WHERE { FILTER(false) }".into(),
        member_variable: "member".into(),
    };
    let empty_id = save_spec(&mut graph, None, None, Uuid::from_u128(13), &empty).unwrap();
    graph.set_known_coverage(kernel::graph::CoverageNote {
        limits: vec![CoverageLimit::new(
            CoverageLayer::Synchronization,
            "Receiver is not synchronized",
        )],
    });
    let empty_result = project_view(&graph, empty_id).unwrap();
    assert!(empty_result.members.is_empty());
    assert!(empty_result.missing_resources.is_empty());
    assert!(!empty_result.coverage.limits.is_empty());
    assert_eq!(graph.node_count(), count);
}
#[test]
fn query_edits_refuse_stale_identity_and_invalid_definitions_without_writes() {
    let (mut graph, resource, a, _) = fixture();
    let original = spec();
    let owner = save_spec(&mut graph, None, None, Uuid::from_u128(14), &original).unwrap();
    let projection = project_view(&graph, owner).unwrap();
    projection.access_for(&graph, resource, a).unwrap();
    let mut updated = original.clone();
    updated.title = "Updated title".into();
    save_spec(
        &mut graph,
        Some(owner),
        Some(&original),
        Uuid::nil(),
        &updated,
    )
    .unwrap();
    assert!(projection.access_for(&graph, resource, a).is_err());
    let revision = graph.revision();
    assert!(
        save_spec(
            &mut graph,
            Some(owner),
            Some(&original),
            Uuid::nil(),
            &original
        )
        .is_err()
    );
    let mut invalid = original.clone();
    invalid.styles.cross_stratum_edge = StrokeCue::Solid;
    assert!(save_spec(&mut graph, None, None, Uuid::from_u128(15), &invalid).is_err());
    assert_eq!(graph.revision(), revision);
}
#[test]
fn frozen_selection_is_independent_of_later_query_edits() {
    let (mut graph, _, _, _) = fixture();
    let owner = save_spec(&mut graph, None, None, Uuid::from_u128(16), &spec()).unwrap();
    let frozen = freeze_view(&mut graph, owner, Uuid::from_u128(17)).unwrap();
    let held = graph
        .open_frozen_selection(frozen)
        .unwrap()
        .selection
        .clone();
    let mut next = spec();
    next.title = "Later query".into();
    save_spec(&mut graph, Some(owner), Some(&spec()), Uuid::nil(), &next).unwrap();
    assert_eq!(
        graph.open_frozen_selection(frozen).unwrap().selection,
        &held
    );
    assert_eq!(held.members().len(), 2);
    assert_eq!(
        held.spec(),
        &serde_json::to_value(spec().definition).unwrap()
    );
    assert!(
        project_view(&graph, owner)
            .unwrap()
            .nodes
            .iter()
            .all(|node| matches!(
                node.identity.stratum,
                GraphStratum::Resource | GraphStratum::Surface
            ))
    );
}

#[test]
fn source_identity_parallel_assertions_tags_navigation_and_reopen_survive_projection() {
    let (mut graph, resource, a, _) = fixture();
    graph
        .tag_resource_with_concept(
            resource,
            "urn:tag:alice",
            kernel::graph::resource_tags::TagConcept {
                owner_iri: "urn:author:alice".into(),
                label: "Music".into(),
            },
        )
        .unwrap();
    graph
        .tag_resource_with_concept(
            resource,
            "urn:tag:bob",
            kernel::graph::resource_tags::TagConcept {
                owner_iri: "urn:author:bob".into(),
                label: "Music".into(),
            },
        )
        .unwrap();
    let original = graph
        .to_snapshot_at(0)
        .resource_edges
        .into_iter()
        .find(|edge| {
            edge.semantic.as_ref().is_some_and(|s| {
                s.statements
                    .iter()
                    .any(|s| s.predicate != kernel::graph::resource::TAGGED_WITH_IRI)
            })
        })
        .unwrap();
    let mut parallel = original.clone();
    parallel.semantic.as_mut().unwrap().statements[0].statement_id =
        "urn:statement:parallel-query".into();
    parallel.semantic.as_mut().unwrap().statements[0].provenance_iri =
        Some("urn:author:other".into());
    let target = original.to_node_id.parse().unwrap();
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetResourceEdgesByIds {
            from_resource_id: resource,
            to_resource_id: target,
            edges: vec![original, parallel],
        },
    );
    let owner = save_spec(&mut graph, None, None, Uuid::from_u128(19), &spec()).unwrap();
    let projected = project_view(&graph, owner).unwrap();
    let concepts: Vec<_> = projected
        .nodes
        .iter()
        .filter(|n| n.title.starts_with("Music ("))
        .collect();
    assert_eq!(concepts.len(), 2);
    assert_ne!(concepts[0].identity, concepts[1].identity);
    assert!(concepts.iter().all(|n| n.held_accesses.is_empty()));
    let source_links: Vec<_> = projected
        .relations
        .iter()
        .filter(|r| {
            r.kind == VisualRelationKind::ResourceLink
                && r.source_from.id == resource
                && r.source_to.id == target
                && r.from.stratum == GraphStratum::Resource
        })
        .collect();
    assert_eq!(source_links.len(), 2);
    let lifted: Vec<_> = projected
        .relations
        .iter()
        .filter(|r| {
            r.kind == VisualRelationKind::ResourceLink
                && r.source_from.id == resource
                && r.source_to.id == target
                && r.from.stratum == GraphStratum::Surface
        })
        .collect();
    assert_eq!(lifted.len(), 4);
    assert!(
        lifted
            .iter()
            .all(|r| r.source_from.stratum == GraphStratum::Resource
                && r.kind.cue(&projected.spec.styles) == StrokeCue::Solid)
    );
    let reopened = Graph::try_from_snapshot(&graph.to_snapshot_at(0)).unwrap();
    assert_eq!(read_spec(&reopened, owner).unwrap(), spec());
    // Restore also rebuilds derived Surface Domain links. Compare the authored
    // assertions and shown bindings exactly, independently of those links.
    let reopened_projection = project_view(&reopened, owner).unwrap();
    let authored_rows = |projection: &QueryProjection| {
        let mut rows: Vec<_> = projection
            .relations
            .iter()
            .filter(|relation| {
                relation.kind != VisualRelationKind::SurfaceLink || !relation.statements.is_empty()
            })
            .map(|relation| serde_json::to_string(relation).unwrap())
            .collect();
        rows.sort();
        rows
    };
    assert_eq!(
        authored_rows(&reopened_projection),
        authored_rows(&projected)
    );
    assert!(reopened_projection.relations.iter().any(|relation| {
        relation.kind == VisualRelationKind::SurfaceLink
            && relation.families["containment"]["sub_kinds"]
                .as_array()
                .is_some_and(|kinds| kinds.contains(&serde_json::json!("Domain")))
    }));
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetShownResourceById {
            surface_id: a,
            resource_id: Some(target),
        },
    );
    assert!(projected.access_for(&graph, resource, a).is_err());
    let next = project_view(&graph, owner).unwrap();
    assert!(
        !next
            .nodes
            .iter()
            .find(|n| n.identity == QueryNodeId::resource(resource))
            .unwrap()
            .held_accesses
            .contains(&a)
    );
}
#[test]
fn malformed_nested_spec_and_budget_are_explicit_refusals() {
    let (mut graph, _, _, _) = fixture();
    let owner = save_spec(&mut graph, None, None, Uuid::from_u128(20), &spec()).unwrap();
    let mut record = graph.resource_record(owner).unwrap();
    let mut value: serde_json::Value = serde_json::from_str(
        &record
            .facets
            .iter()
            .find(|f| f.facet == QUERY_VIEW_FACET)
            .unwrap()
            .value_json,
    )
    .unwrap();
    value["definition"]["invented"] = true.into();
    record
        .facets
        .iter_mut()
        .find(|f| f.facet == QUERY_VIEW_FACET)
        .unwrap()
        .value_json = value.to_string();
    apply_graph_delta(
        &mut graph,
        GraphDelta::ReplaySetResourceRecordById {
            resource_id: owner,
            record: Some(record),
        },
    );
    assert!(
        read_spec(&graph, owner)
            .unwrap_err()
            .contains("unsupported nested")
    );
    let mut limited = spec();
    limited.max_members = 1;
    let revision = graph.revision();
    assert!(
        save_spec(&mut graph, None, None, Uuid::from_u128(21), &limited)
            .unwrap_err()
            .contains("budget")
    );
    assert_eq!(graph.revision(), revision);
}

#[test]
fn appearance_and_relation_budgets_refuse_control_and_lift_explosions() {
    let (mut graph, _, _, _) = fixture();
    for (nonce, mut limited) in [(31, spec()), (32, spec()), (33, spec())] {
        if nonce == 33 {
            limited.max_relations = 1;
        } else {
            limited.max_appearances = 1;
        }
        if nonce == 32 {
            limited.appearances = AppearancePolicy::ResourcesOnly;
        }
        let owner = save_spec(&mut graph, None, None, Uuid::from_u128(nonce), &limited).unwrap();
        let before = serde_json::to_value(graph.to_snapshot_at(0)).unwrap();
        assert!(project_view(&graph, owner).unwrap_err().contains("budget"));
        assert_eq!(
            serde_json::to_value(graph.to_snapshot_at(0)).unwrap(),
            before
        );
    }
}

#[test]
fn receiver_member_column_and_owner_scoped_tag_filters_keep_their_meaning() {
    let (mut graph, resource, _, _) = fixture();
    graph
        .tag_resource_with_concept(
            resource,
            "urn:tag:alice",
            kernel::graph::resource_tags::TagConcept {
                owner_iri: "urn:author:alice".into(),
                label: "Music".into(),
            },
        )
        .unwrap();
    let mut imported = spec();
    imported.definition.kind = SubgraphKind::Sparql {
        query: "SELECT ?item WHERE { VALUES ?item { <https://source.test/a> } }".into(),
        member_variable: "item".into(),
    };
    let owner = save_spec(&mut graph, None, None, Uuid::from_u128(34), &imported).unwrap();
    let mut edited = read_spec(&graph, owner).unwrap();
    edited.title = "Receiver title edit".into();
    edited.set_query_text(imported.query_text()).unwrap();
    save_spec(
        &mut graph,
        Some(owner),
        Some(&imported),
        Uuid::nil(),
        &edited,
    )
    .unwrap();
    assert_eq!(project_view(&graph, owner).unwrap().members, vec![resource]);
    let tag_query = QueryViewSpec::select(
        "Alice Music",
        format!(
            "SELECT ?member WHERE {{ ?member <{}> ?tag . FILTER(STR(?tag) = \"urn:tag:alice\") }}",
            kernel::graph::resource::TAGGED_WITH_IRI
        ),
    );
    let tag_owner = save_spec(&mut graph, None, None, Uuid::from_u128(35), &tag_query).unwrap();
    assert_eq!(
        project_view(&graph, tag_owner).unwrap().members,
        vec![resource]
    );
}
