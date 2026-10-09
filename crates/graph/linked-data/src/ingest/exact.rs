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
    records.push(PersistedEdge {
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
    });
    let pairs: Vec<_> = records
        .iter()
        .filter(|record| record.from_node_id == pair.0 && record.to_node_id == pair.1)
        .cloned()
        .collect();
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
