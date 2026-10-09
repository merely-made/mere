// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Exact carried assertions use record captures, not live content deduplication.

use super::super::EdgeContribution;
use kernel::graph::apply::{GraphDelta, apply_graph_delta};
use kernel::graph::predicate_registry::GraphStratum;
use kernel::graph::resource_content::RESOURCE_PROPERTIES;
use kernel::graph::resource_properties::merge_resource_properties;
use kernel::graph::{Graph, NodeKey, sub_kind_from_iri};
use kernel::persistence::{
    GraphSnapshot, PersistedEdge, PersistedEdgeFamily, PersistedResourceFacet,
    PersistedSemanticEdgeData, PersistedSemanticStatement, PersistedSemanticSubKind,
};
use kernel::types::NodeProperty;

fn preflight(snapshot: &GraphSnapshot) -> bool {
    Graph::try_from_recorded_snapshot(snapshot)
        .is_ok_and(|graph| graph.validate_active_resource_assertion_handles().is_ok())
}

pub(super) fn import_properties(
    graph: &mut Graph,
    key: NodeKey,
    incoming: &[NodeProperty],
) -> bool {
    if incoming.is_empty() {
        return true;
    }
    let Some(id) = graph.shown_resource_id(key) else {
        return false;
    };
    if incoming.iter().any(|property| {
        graph
            .find_semantic_statement(&property.statement_id)
            .is_some()
            || graph.nodes().any(|(key, _)| {
                graph
                    .legacy_node_properties(key)
                    .unwrap_or_default()
                    .iter()
                    .any(|held| held.statement_id == property.statement_id)
            })
    }) {
        return false;
    }
    let Ok(properties) = merge_resource_properties(&graph.resource_properties(id), incoming) else {
        return false;
    };
    let mut snapshot = graph.to_snapshot();
    let iri = graph.resource(id).unwrap().canonical_iri();
    let Some(record) = snapshot
        .resources
        .iter_mut()
        .find(|record| record.canonical_iri == iri)
    else {
        return false;
    };
    record
        .facets
        .retain(|facet| facet.facet != RESOURCE_PROPERTIES);
    record.facets.push(PersistedResourceFacet {
        facet: RESOURCE_PROPERTIES.into(),
        value_json: serde_json::to_string(&properties).expect("properties serialize"),
    });
    let record = record.clone();
    if !preflight(&snapshot) {
        return false;
    }
    apply_graph_delta(
        graph,
        GraphDelta::ReplaySetResourceRecordById {
            resource_id: id,
            record: Some(record),
        },
    );
    incoming
        .iter()
        .all(|property| graph.resource_properties(id).contains(property))
}

fn persisted_kind(kind: kernel::graph::SemanticSubKind) -> PersistedSemanticSubKind {
    use kernel::graph::SemanticSubKind as Kind;
    match kind {
        Kind::Hyperlink => PersistedSemanticSubKind::Hyperlink,
        Kind::UserGrouped => PersistedSemanticSubKind::UserGrouped,
        Kind::AgentDerived => PersistedSemanticSubKind::AgentDerived,
        Kind::Cites => PersistedSemanticSubKind::Cites,
        Kind::Quotes => PersistedSemanticSubKind::Quotes,
        Kind::Summarizes => PersistedSemanticSubKind::Summarizes,
        Kind::Elaborates => PersistedSemanticSubKind::Elaborates,
        Kind::ExampleOf => PersistedSemanticSubKind::ExampleOf,
        Kind::Supports => PersistedSemanticSubKind::Supports,
        Kind::Contradicts => PersistedSemanticSubKind::Contradicts,
        Kind::Questions => PersistedSemanticSubKind::Questions,
        Kind::SameEntityAs => PersistedSemanticSubKind::SameEntityAs,
        Kind::DuplicateOf => PersistedSemanticSubKind::DuplicateOf,
        Kind::CanonicalMirrorOf => PersistedSemanticSubKind::CanonicalMirrorOf,
        Kind::DependsOn => PersistedSemanticSubKind::DependsOn,
        Kind::Blocks => PersistedSemanticSubKind::Blocks,
        Kind::NextStep => PersistedSemanticSubKind::NextStep,
    }
}

pub(super) fn import_edge(
    graph: &mut Graph,
    from: NodeKey,
    to: NodeKey,
    edge: &EdgeContribution,
) -> bool {
    let Some(id) = &edge.statement_id else {
        return false;
    };
    if graph.nodes().any(|(key, _)| {
        graph
            .legacy_node_properties(key)
            .unwrap_or_default()
            .iter()
            .any(|held| &held.statement_id == id)
    }) || graph.resource_nodes().any(|resource| {
        graph
            .resource_properties(resource.id())
            .iter()
            .any(|held| &held.statement_id == id)
    }) {
        return false;
    }
    let statement = PersistedSemanticStatement {
        statement_id: id.clone(),
        predicate: edge.predicate.clone(),
        recognized_sub_kind: sub_kind_from_iri(&edge.predicate).map(persisted_kind),
        label: edge.label.clone(),
        graph_scope: edge.graph_scope.clone(),
        provenance_iri: edge.provenance_iri.clone(),
        asserted_at_ms: edge.asserted_at_ms,
    };
    let surface_pair = (
        graph.get_node(from).unwrap().id.to_string(),
        graph.get_node(to).unwrap().id.to_string(),
    );
    let resource_pair = graph
        .shown_resource_id(from)
        .zip(graph.shown_resource_id(to))
        .map(|(from, to)| (from.to_string(), to.to_string()));
    let mut snapshot = graph.to_snapshot();
    let mut held_stratum = None;
    for (stratum, records, pair) in [
        (GraphStratum::Surface, &snapshot.edges, Some(&surface_pair)),
        (
            GraphStratum::Resource,
            &snapshot.resource_edges,
            resource_pair.as_ref(),
        ),
    ] {
        for record in records {
            for held in record.semantic.iter().flat_map(|bucket| &bucket.statements) {
                let pair_matches = pair.is_some_and(|(from, to)| {
                    record.from_node_id == *from && record.to_node_id == *to
                });
                if held.statement_id == *id {
                    return pair_matches && *held == statement;
                }
                if pair_matches
                    && held.predicate == statement.predicate
                    && held.recognized_sub_kind == statement.recognized_sub_kind
                    && held.graph_scope == statement.graph_scope
                    && held.provenance_iri == statement.provenance_iri
                {
                    if held_stratum.is_some_and(|held| held != stratum) {
                        return false;
                    }
                    held_stratum = Some(stratum);
                }
            }
        }
    }
    let stratum = match held_stratum {
        Some(stratum) => stratum,
        None => match graph.effective_predicate_stratum(&edge.predicate) {
            Ok(stratum) => stratum,
            Err(_) => return false,
        },
    };
    let pair = match stratum {
        GraphStratum::Surface => surface_pair,
        GraphStratum::Resource => match resource_pair {
            Some(pair) => pair,
            None => return false,
        },
    };
    let records = match stratum {
        GraphStratum::Surface => &mut snapshot.edges,
        GraphStratum::Resource => &mut snapshot.resource_edges,
    };
    let incoming = PersistedEdge {
        from_node_id: pair.0.clone(),
        to_node_id: pair.1.clone(),
        families: vec![PersistedEdgeFamily::Semantic],
        semantic: Some(PersistedSemanticEdgeData {
            sub_kinds: statement.recognized_sub_kind.into_iter().collect(),
            label: statement.label.clone(),
            predicate: Some(statement.predicate.clone()),
            agent_decay_progress: None,
            statements: vec![statement],
        }),
        traversal: None,
        containment: None,
        arrangement: None,
        imported: None,
        provenance: None,
    };
    let pairs = match stratum {
        GraphStratum::Surface => {
            let mut pairs = graph.persisted_edges_between(from, to);
            pairs.push(incoming.clone());
            // Preflight the same complete rows that will be replayed, including
            // live arrangements absent from the durable snapshot projection.
            records.retain(|record| record.from_node_id != pair.0 || record.to_node_id != pair.1);
            records.extend(pairs.iter().cloned());
            pairs
        },
        GraphStratum::Resource => {
            records.push(incoming);
            records
                .iter()
                .filter(|record| record.from_node_id == pair.0 && record.to_node_id == pair.1)
                .cloned()
                .collect()
        },
    };
    if !preflight(&snapshot) {
        return false;
    }
    let delta = match stratum {
        GraphStratum::Surface => GraphDelta::ReplaySetEdgesByIds {
            from_id: graph.get_node(from).unwrap().id,
            to_id: graph.get_node(to).unwrap().id,
            edges: pairs,
        },
        GraphStratum::Resource => GraphDelta::ReplaySetResourceEdgesByIds {
            from_resource_id: graph.shown_resource_id(from).unwrap(),
            to_resource_id: graph.shown_resource_id(to).unwrap(),
            edges: pairs,
        },
    };
    apply_graph_delta(graph, delta);
    graph.find_semantic_statement(id).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::persistence::{
        PersistedNavigationTrigger, PersistedTraversalEdgeData, PersistedTraversalMetrics,
        PersistedTraversalRecord,
    };
    use kernel::types::GraphScope;

    #[test]
    fn carried_import_refuses_conflicting_and_cross_store_handles_without_writes() {
        let mut graph = Graph::new();
        let from = kernel::graph::apply::add_node(
            &mut graph,
            None,
            "https://from.test/".into(),
            Default::default(),
        );
        let to = kernel::graph::apply::add_node(
            &mut graph,
            None,
            "https://to.test/".into(),
            Default::default(),
        );
        let mut held = NodeProperty::new("urn:predicate:literal".into(), "held".into());
        held.statement_id = "held-handle".into();
        assert!(import_properties(&mut graph, from, &[held.clone()]));
        let truth = |graph: &Graph| {
            let mut snapshot = graph.to_snapshot();
            snapshot.timestamp_secs = 0;
            serde_json::to_value(snapshot).unwrap()
        };
        let before = truth(&graph);
        let mut conflicting = held.clone();
        conflicting.value = "conflict".into();
        assert!(!import_properties(&mut graph, from, &[conflicting]));
        assert_eq!(truth(&graph), before);
        assert!(!import_properties(&mut graph, to, &[held.clone()]));
        assert_eq!(truth(&graph), before);
        let mut edge = EdgeContribution {
            subject: "https://from.test/".into(),
            object: "https://to.test/".into(),
            predicate: "urn:predicate:edge".into(),
            graph_scope: GraphScope::Default,
            statement_id: Some(held.statement_id.clone()),
            label: None,
            provenance_iri: None,
            asserted_at_ms: None,
        };
        assert!(!import_edge(&mut graph, from, to, &edge));
        assert_eq!(truth(&graph), before);
        edge.statement_id = Some("edge-handle".into());
        assert!(
            import_edge(&mut graph, from, to, &edge),
            "distinct handle control"
        );
        let before = truth(&graph);
        let mut property = held.clone();
        property.statement_id = "edge-handle".into();
        assert!(!import_properties(&mut graph, from, &[property]));
        assert_eq!(truth(&graph), before);
        edge.label = Some("conflicting edge".into());
        assert!(!import_edge(&mut graph, from, to, &edge));
        assert_eq!(truth(&graph), before);
        let resource = graph.shown_resource_id(from).unwrap();
        assert_eq!(graph.resource_properties(resource), vec![held.clone()]);
        held.statement_id = "fresh-handle".into();
        assert!(
            import_properties(&mut graph, to, &[held.clone()]),
            "distinct property control"
        );
        assert_eq!(
            graph.resource_properties(graph.shown_resource_id(to).unwrap()),
            vec![held]
        );
    }

    #[test]
    fn carried_surface_import_preserves_parallel_rows_and_traversals() {
        let mut graph = Graph::new();
        let from = kernel::graph::apply::add_node(
            &mut graph,
            None,
            "https://from.test/".into(),
            Default::default(),
        );
        let to = kernel::graph::apply::add_node(
            &mut graph,
            None,
            "https://to.test/".into(),
            Default::default(),
        );
        graph
            .declare_predicate("urn:predicate:surface", GraphStratum::Surface)
            .unwrap();
        let mut snapshot = graph.to_snapshot();
        let mut first = PersistedEdge {
            from_node_id: graph.get_node(from).unwrap().id.to_string(),
            to_node_id: graph.get_node(to).unwrap().id.to_string(),
            families: vec![PersistedEdgeFamily::Semantic],
            semantic: Some(PersistedSemanticEdgeData {
                sub_kinds: vec![],
                label: Some("held".into()),
                predicate: Some("urn:predicate:held".into()),
                agent_decay_progress: None,
                statements: vec![PersistedSemanticStatement {
                    statement_id: "held-one".into(),
                    predicate: "urn:predicate:held".into(),
                    recognized_sub_kind: None,
                    label: Some("held".into()),
                    graph_scope: GraphScope::User,
                    provenance_iri: Some("urn:author:held".into()),
                    asserted_at_ms: Some(10),
                }],
            }),
            traversal: None,
            containment: None,
            arrangement: None,
            imported: None,
            provenance: None,
        };
        first.families.push(PersistedEdgeFamily::Traversal);
        first.traversal = Some(PersistedTraversalEdgeData {
            traversals: vec![PersistedTraversalRecord {
                timestamp_ms: 11,
                trigger: PersistedNavigationTrigger::LinkClick,
            }],
            metrics: PersistedTraversalMetrics {
                total_navigations: 1,
                forward_navigations: 1,
                backward_navigations: 0,
                last_navigated_at: Some(11),
            },
        });
        let mut second = first.clone();
        second.semantic.as_mut().unwrap().statements[0].statement_id = "held-two".into();
        let traversal = second.traversal.as_mut().unwrap();
        traversal.traversals[0].timestamp_ms = 22;
        traversal.traversals[0].trigger = PersistedNavigationTrigger::Back;
        traversal.metrics.forward_navigations = 0;
        traversal.metrics.backward_navigations = 1;
        traversal.metrics.last_navigated_at = Some(22);
        snapshot.edges = vec![first.clone(), second.clone()];
        let mut graph = Graph::try_from_recorded_snapshot(&snapshot).unwrap();
        let before = graph.to_snapshot().edges;
        assert!(
            before.contains(&first) && before.contains(&second),
            "recorded load control"
        );
        for sub_kind in [
            kernel::graph::ArrangementSubKind::TileGroup,
            kernel::graph::ArrangementSubKind::SplitPair,
        ] {
            apply_graph_delta(
                &mut graph,
                GraphDelta::AssertRelation {
                    from,
                    to,
                    assertion: kernel::graph::EdgeAssertion::Arrangement { sub_kind },
                    asserter_iri: "urn:author:layout".into(),
                },
            );
        }
        let before = graph.to_snapshot().edges;
        let before_payloads: Vec<_> = graph
            .edges_between_undirected(from, to)
            .map(|(_, payload)| payload.clone())
            .collect();
        assert_eq!(
            before_payloads[0]
                .arrangement_data()
                .unwrap()
                .sub_kinds
                .len(),
            2
        );
        let incoming = EdgeContribution {
            subject: "https://from.test/".into(),
            predicate: "urn:predicate:surface".into(),
            object: "https://to.test/".into(),
            graph_scope: GraphScope::User,
            statement_id: Some("incoming".into()),
            label: None,
            provenance_iri: Some("urn:author:incoming".into()),
            asserted_at_ms: None,
        };
        assert!(import_edge(&mut graph, from, to, &incoming));
        let after_payloads: Vec<_> = graph
            .edges_between_undirected(from, to)
            .map(|(_, payload)| payload.clone())
            .collect();
        assert_eq!(
            after_payloads[0], before_payloads[0],
            "retain the existing first-arc priority and session arrangements"
        );
        for held in before_payloads {
            assert!(
                after_payloads.contains(&held),
                "lost complete live Surface payload: {held:?}"
            );
        }
        let after = graph.to_snapshot().edges;
        assert_eq!(after.len(), before.len() + 1);
        for held in before {
            assert!(
                after.contains(&held),
                "lost complete Surface record: {held:?}"
            );
        }
        let statement = graph.find_semantic_statement("incoming").unwrap().1;
        assert_eq!(statement.provenance_iri, incoming.provenance_iri);
        assert_eq!(statement.asserted_at_ms, None);
        assert_eq!(statement.predicate, incoming.predicate);
        let before_retry = graph.to_snapshot();
        assert!(import_edge(&mut graph, from, to, &incoming));
        let mut after_retry = graph.to_snapshot();
        after_retry.timestamp_secs = before_retry.timestamp_secs;
        assert_eq!(
            serde_json::to_value(after_retry).unwrap(),
            serde_json::to_value(before_retry).unwrap()
        );
    }
}
