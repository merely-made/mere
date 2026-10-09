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
use super::{GraphContribution, ImportEnvelope, NodeContribution, SubjectIdentity};
#[cfg(not(target_arch = "wasm32"))]
use kernel::persistence::PersistedResourceRecord;

#[cfg(not(target_arch = "wasm32"))]
#[path = "exact.rs"]
mod exact;

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
    identity: impl FnMut(&NodeContribution) -> SubjectIdentity,
) -> ApplyOutcome {
    apply_inner(graph, contribution, identity, None)
}

/// Import a parsed RDF profile without inventing assertions for its definitions.
#[cfg(not(target_arch = "wasm32"))]
pub fn apply_import(graph: &mut Graph, envelope: &ImportEnvelope) -> ApplyOutcome {
    apply_import_with_identity(graph, envelope, |_| SubjectIdentity::ExactIri)
}

/// Import with explicit page identity intent and the original parser evidence.
#[cfg(not(target_arch = "wasm32"))]
pub fn apply_import_with_identity(
    graph: &mut Graph,
    envelope: &ImportEnvelope,
    identity: impl FnMut(&NodeContribution) -> SubjectIdentity,
) -> ApplyOutcome {
    apply_inner(
        graph,
        &envelope.contribution,
        identity,
        Some(&envelope.evidence),
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn apply_inner(
    graph: &mut Graph,
    contribution: &GraphContribution,
    identity: impl FnMut(&NodeContribution) -> SubjectIdentity,
    evidence: Option<&super::envelope::ImportEvidence>,
) -> ApplyOutcome {
    let admission = (graph.resource_nodes().count(), graph.node_count());
    let appearances = graph.appearance_admission_revision();
    let outcome = graph.without_pending_derivation(|graph| {
        apply_suppressed(graph, contribution, identity, evidence)
    });
    if graph.resource_nodes().count() > admission.0
        || graph.node_count() > admission.1
        || graph.appearance_admission_revision() != appearances
    {
        graph.retry_pending_links();
    }
    outcome
}

#[cfg(not(target_arch = "wasm32"))]
fn apply_suppressed(
    graph: &mut Graph,
    contribution: &GraphContribution,
    mut identity: impl FnMut(&NodeContribution) -> SubjectIdentity,
    evidence: Option<&super::envelope::ImportEvidence>,
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
    let mut defined = std::collections::HashSet::new();

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
        // Imported curated tags belong to the explicit agent, else their RDF source.
        let attributed: std::collections::BTreeSet<_> = contribution
            .edges
            .iter()
            .filter(|edge| {
                edge.subject == node.id
                    && edge.predicate == "http://www.w3.org/ns/prov#wasAttributedTo"
            })
            .map(|edge| edge.object.as_str())
            .collect();
        let source = if attributed.len() == 1 {
            *attributed.first().unwrap()
        } else {
            node.id.as_str()
        };
        if let Some(id) = graph.shown_resource_id(key) {
            for label in &node.tags {
                let concept_iri = kernel::graph::resource_tags::tag_concept_iri(source, label);
                let _ = graph.tag_resource_with_asserter(
                    id,
                    &concept_iri,
                    kernel::graph::resource_tags::TagConcept {
                        owner_iri: source.into(),
                        label: label.clone(),
                    },
                    source.into(),
                );
            }
        }
        if let Some(definition) = evidence.and_then(|evidence| evidence.definitions.get(&node.id))
            && let Some(id) = graph.shown_resource_id(key)
            && graph
                .define_tag_concept(
                    id,
                    kernel::graph::resource_tags::TagConcept {
                        owner_iri: definition.owner.clone(),
                        label: definition.label.clone(),
                    },
                )
                .is_ok()
        {
            defined.insert(node.id.as_str());
        }
        let is_definition = defined.contains(node.id.as_str());
        let properties: Vec<_> = node
            .properties
            .iter()
            .filter(|property| {
                !is_definition || !evidence.unwrap().is_plain_property(&node.id, property)
            })
            .cloned()
            .collect();
        let (carried, plain): (Vec<_>, Vec<_>) = properties.into_iter().partition(|property| {
            evidence.is_some_and(|evidence| evidence.is_carried_property(&node.id, property))
        });
        let _ = graph.append_node_properties(key, plain);
        let _ = exact::import_properties(graph, key, &carried);
        // `@type` IRIs become `rdf:type` classifications (kernel dedups them).
        let _ = graph.add_node_classifications(
            key,
            node.types
                .iter()
                .filter(|iri| !is_definition || iri.as_str() != super::envelope::SKOS_CONCEPT)
                .map(|type_iri| rdf_type_classification(type_iri))
                .collect(),
        );
        // Interpret only the complete, unambiguous SKOS tag profile. Foreign partial
        // descriptions remain ordinary RDF properties and edges.
        if evidence.is_none()
            && node
                .types
                .iter()
                .any(|iri| iri == "http://www.w3.org/2004/02/skos/core#Concept")
        {
            let descriptions: Vec<_> = node
                .properties
                .iter()
                .filter(|property| {
                    property.predicate == "http://www.w3.org/2004/02/skos/core#prefLabel"
                        && property.graph_scope == kernel::types::GraphScope::Default
                })
                .collect();
            let labels: std::collections::BTreeSet<_> = descriptions
                .iter()
                .map(|property| property.value.as_str())
                .collect();
            let supported_labels = descriptions.iter().all(|property| {
                property.lang.is_none()
                    && property.datatype.as_deref().is_none_or(|datatype| {
                        datatype == "http://www.w3.org/2001/XMLSchema#string"
                    })
            });
            let owners: std::collections::BTreeSet<_> = contribution
                .edges
                .iter()
                .filter(|edge| {
                    edge.subject == node.id
                        && edge.predicate == "http://www.w3.org/ns/prov#wasAttributedTo"
                        && edge.graph_scope == kernel::types::GraphScope::Default
                })
                .map(|edge| edge.object.as_str())
                .collect();
            if supported_labels
                && labels.len() == 1
                && owners.len() == 1
                && let Some(id) = graph.shown_resource_id(key)
            {
                let _ = graph.define_tag_concept(
                    id,
                    kernel::graph::resource_tags::TagConcept {
                        owner_iri: (*owners.first().unwrap()).into(),
                        label: (*labels.first().unwrap()).into(),
                    },
                );
            }
        }
        key_for.insert(node.id.as_str(), key);
    }

    for (index, edge) in contribution.edges.iter().enumerate() {
        if defined.contains(edge.subject.as_str())
            && evidence.is_some_and(|evidence| evidence.is_plain_edge(index))
        {
            continue;
        }
        let (Some(&from), Some(&to)) = (
            key_for.get(edge.subject.as_str()),
            key_for.get(edge.object.as_str()),
        ) else {
            outcome.edges_skipped += 1;
            continue;
        };
        let sub_kind = sub_kind_from_iri(&edge.predicate);
        if evidence.is_some() && edge.statement_id.is_some() {
            if exact::import_edge(graph, from, to, edge) {
                outcome.edges_asserted += 1;
            }
            continue;
        }
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
