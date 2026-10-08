// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

#[cfg(not(target_arch = "wasm32"))]
use kernel::graph::apply::{self as graph_apply, GraphDelta, GraphDeltaResult, apply_graph_delta};
#[cfg(not(target_arch = "wasm32"))]
use kernel::graph::{
    Graph, NodeKey, ProvenanceSubKind, ResourceNode, SemanticStatement, SemanticStatementSpec,
    SemanticSubKind, sub_kind_from_iri,
};
#[cfg(not(target_arch = "wasm32"))]
use kernel::types::{
    ClassificationProvenance, ClassificationScheme, ClassificationStatus, NodeClassification,
    NodeDerivation,
};

#[cfg(not(target_arch = "wasm32"))]
use super::{GraphContribution, NodeContribution, SubjectIdentity};
#[cfg(not(target_arch = "wasm32"))]
use kernel::persistence::PersistedResourceRecord;

/// What [`apply_contribution`] did.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ApplyOutcome {
    pub nodes_created: usize,
    pub edges_asserted: usize,
    /// Edges whose subject or object node could not be resolved (should be zero
    /// for a self-contained contribution).
    pub edges_skipped: usize,
}

/// Materialize a [`GraphContribution`] into `graph`: a node per subject/object
/// (reused if matched by URL, else created), and one `Semantic` edge per edge —
/// recognized predicate → typed sub-kind + canonical IRI; unrecognized → open
/// predicate via [`Graph::assert_semantic_predicate`]. Curated literals
/// (`title` / `tags`) are written onto the node; `@type` becomes a classification.
/// Generic RDF binds new subjects to exact-IRI resources. Page extraction uses
/// [`apply_contribution_with_identity`] to identify its page subjects explicitly.
///
/// `not(wasm32)`: `add_node` mints a UUID. A wasm host materializes from the same
/// contribution using `add_node_with_id` with a host-provided UUID.
#[cfg(not(target_arch = "wasm32"))]
pub fn apply_contribution(graph: &mut Graph, contribution: &GraphContribution) -> ApplyOutcome {
    apply_contribution_with_identity(graph, contribution, |_| SubjectIdentity::ExactIri)
}

/// Apply mixed RDF subjects with caller-owned resource identity intent.
/// The callback identifies pages explicitly; RDF types and the parse base do not.
/// Existing shown bindings and resource records are preserved.
#[cfg(not(target_arch = "wasm32"))]
pub fn apply_contribution_with_identity(
    graph: &mut Graph,
    contribution: &GraphContribution,
    mut identity: impl FnMut(&NodeContribution) -> SubjectIdentity,
) -> ApplyOutcome {
    use std::collections::HashMap;

    /// An ingested `@type` IRI as a classification under the `rdf:type` scheme
    /// (the full type IRI is the value — lossless).
    fn rdf_type_classification(type_iri: &str) -> NodeClassification {
        NodeClassification {
            scheme: ClassificationScheme::Custom("rdf:type".to_string()),
            value: type_iri.to_string(),
            label: None,
            confidence: 1.0,
            provenance: ClassificationProvenance::Imported,
            status: ClassificationStatus::Imported,
            primary: false,
        }
    }

    let mut outcome = ApplyOutcome::default();
    let mut key_for: HashMap<&str, NodeKey> = HashMap::new();

    for node in &contribution.nodes {
        let subject_identity = identity(node);
        let key = graph
            .get_node_by_url(&node.id)
            .map(|(key, _)| key)
            .unwrap_or_else(|| {
                // The `@id` is the node's identity here, so mint a deterministic
                // UUIDv5 from it: two hosts ingesting the same document agree on
                // node ids, so a federated merge needs no reconciliation.
                let id = Graph::node_namespace_id(&node.id);
                if subject_identity == SubjectIdentity::Page {
                    outcome.nodes_created += 1;
                    graph_apply::add_node(graph, Some(id), node.id.clone(), Default::default())
                } else {
                    // The legacy Add grammar leaves binding to the exact typed captures below.
                    match apply_graph_delta(
                        graph,
                        GraphDelta::ReplayAddNodeWithIdIfMissing {
                            id,
                            url: node.id.clone(),
                            position: Default::default(),
                        },
                    ) {
                        GraphDeltaResult::NodeMaybeAdded(Some(key)) => {
                            outcome.nodes_created += 1;
                            key
                        },
                        GraphDeltaResult::NodeMaybeAdded(None) => graph
                            .get_node_key_by_id(id)
                            .expect("an existing deterministic subject has a surface"),
                        _ => unreachable!("ReplayAddNode returns NodeMaybeAdded"),
                    }
                }
            });
        if graph.shown_resource_id(key).is_none() {
            let resource = match subject_identity {
                SubjectIdentity::ExactIri => ResourceNode::for_term(&node.id),
                SubjectIdentity::Page => ResourceNode::new(&node.id),
            };
            if graph.resource(resource.id()).is_none() {
                apply_graph_delta(
                    graph,
                    GraphDelta::ReplaySetResourceRecordById {
                        resource_id: resource.id(),
                        record: Some(PersistedResourceRecord {
                            canonical_iri: resource.canonical_iri().into(),
                            facets: vec![],
                        }),
                    },
                );
            }
            let surface_id = graph.get_node(key).unwrap().id;
            apply_graph_delta(
                graph,
                GraphDelta::ReplaySetShownResourceById {
                    surface_id,
                    resource_id: Some(resource.id()),
                },
            );
        }
        if let Some(title) = &node.title {
            let _ = apply_graph_delta(
                graph,
                GraphDelta::SetNodeTitle {
                    key,
                    title: title.clone(),
                },
            );
        }
        // Batched rather than one delta per item: each single-item write
        // reserializes the node's whole facet array, so a document with P
        // properties on a node cost Theta(P^2) JSON work. The batch entry
        // points read each facet once and record the same per-item captured
        // deltas the `GraphDelta` path would have, so the journal is unchanged
        // — which is also why they must not be wrapped in `apply_graph_delta`.
        let _ = graph.insert_node_tags(key, node.tags.clone());
        let _ = graph.append_node_properties(key, node.properties.clone());
        // `@type` IRIs become `rdf:type` classifications (kernel dedups them).
        let _ = graph.add_node_classifications(
            key,
            node.types
                .iter()
                .map(|type_iri| rdf_type_classification(type_iri))
                .collect(),
        );
        key_for.insert(node.id.as_str(), key);
    }

    for edge in &contribution.edges {
        let (Some(&from), Some(&to)) = (
            key_for.get(edge.subject.as_str()),
            key_for.get(edge.object.as_str()),
        ) else {
            outcome.edges_skipped += 1;
            continue;
        };
        let sub_kind = sub_kind_from_iri(&edge.predicate);
        let asserter = edge
            .provenance_iri
            .clone()
            .unwrap_or_else(|| graph.write_author().asserter_iri());
        let has_statement_metadata = edge.statement_id.is_some()
            || edge.label.is_some()
            || edge.provenance_iri.is_some()
            || edge.asserted_at_ms.is_some();
        if has_statement_metadata {
            // A reified statement: write it statement-aware so the fact handle
            // and metadata survive (the Phase 2 round-trip contract). A carried
            // id is preserved verbatim; a foreign reifier's fact gets a fresh
            // kernel-minted id.
            let asserted = match &edge.statement_id {
                Some(id) => graph
                    .assert_persisted_semantic_statement(
                        from,
                        to,
                        SemanticStatement {
                            statement_id: id.clone(),
                            predicate: edge.predicate.clone(),
                            recognized_sub_kind: sub_kind,
                            label: edge.label.clone(),
                            graph_scope: edge.graph_scope.clone(),
                            provenance_iri: Some(asserter.clone()),
                            asserted_at_ms: edge.asserted_at_ms,
                        },
                    )
                    .is_some(),
                None => graph
                    .assert_semantic_statement(
                        from,
                        to,
                        SemanticStatementSpec {
                            predicate: edge.predicate.clone(),
                            recognized_sub_kind: sub_kind,
                            label: edge.label.clone(),
                            graph_scope: edge.graph_scope.clone(),
                            provenance_iri: Some(asserter.clone()),
                            asserted_at_ms: edge.asserted_at_ms,
                        },
                    )
                    .is_some(),
            };
            if asserted {
                outcome.edges_asserted += 1;
            }
            continue;
        }
        let asserted = if let Some(sub_kind) = sub_kind {
            // Recognized: typed Semantic statement in the supplied graph scope.
            let semantic_ok = graph_apply::assert_semantic_relation_in_scope(
                graph,
                from,
                to,
                sub_kind,
                None,
                edge.graph_scope.clone(),
            )
            .is_some();
            // A harvested hyperlink also records derivation provenance on the
            // target: it was `ExtractedFrom` the source page (capture plan C3).
            // Recorded as a node derivation (like cross-graph `CopiedFrom`), so it
            // feeds the provenance trail without polluting the link graph's
            // out-edges — a channel distinct from the `Hyperlink` semantic edge.
            if sub_kind == SemanticSubKind::Hyperlink
                && let Some(source_node) = graph.get_node(from).map(|n| n.id.to_string())
            {
                let _ = apply_graph_delta(
                    graph,
                    GraphDelta::RecordNodeDerivation {
                        key: to,
                        derivation: NodeDerivation {
                            sub_kind: ProvenanceSubKind::ExtractedFrom,
                            source_node,
                            source_graph: None,
                        },
                    },
                );
            }
            semantic_ok
        } else {
            // Unrecognized: an open-predicate Semantic edge (raw IRI).
            graph_apply::assert_semantic_predicate_in_scope(
                graph,
                from,
                to,
                edge.predicate.clone(),
                edge.graph_scope.clone(),
            )
            .is_some()
        };
        if asserted {
            outcome.edges_asserted += 1;
        }
    }

    outcome
}
