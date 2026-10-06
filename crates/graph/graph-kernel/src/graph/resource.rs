// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Resource identity beneath browsing surfaces.

use super::{Graph, NodeKey};
use crate::persistence::{PersistedEdge, PersistedResourceFacet, PersistedResourceRecord};
use chartulary::{Address, Addressed, Identified};
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use uuid::Uuid;

/// A tagging assertion points from the tagged resource to its concept resource.
pub const TAGGED_WITH_IRI: &str = "https://mere.computer/ns/rel#taggedWith";

/// One resource identified by a canonical IRI.
///
/// Identity and address are immutable together. Content records will be
/// attached through the kernel's recorded write path.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceNode {
    container: chartulary::Container<Uuid>,
}

impl ResourceNode {
    /// Identify a resource using the common canonicalizer and UUID namespace.
    pub fn new(iri: &str) -> Self {
        let canonical = chartulary::canonical_url(iri);
        Self::from_canonical_iri(&canonical)
    }

    /// Identify a vocabulary term by its exact IRI, including fragment and case.
    /// Page resources use [`Self::new`] instead.
    pub fn for_term(iri: &str) -> Self {
        Self::from_canonical_iri(iri)
    }

    pub(crate) fn from_canonical_iri(canonical: &str) -> Self {
        Self {
            container: chartulary::Container::with_identity(
                chartulary::resource_id_from_canonical_iri(canonical),
            )
            .with_address_record(Address::new(canonical)),
        }
    }

    /// The stable identity, shared by every surface showing this resource.
    pub fn id(&self) -> Uuid {
        self.container.id
    }

    /// The canonical IRI from which the stable identity was derived.
    pub fn canonical_iri(&self) -> &str {
        self.container.addresses[0].as_str()
    }
}

impl Graph {
    /// Read a resource by its stable identity.
    pub fn resource(&self, id: Uuid) -> Option<&ResourceNode> {
        self.resources.get(&id)
    }

    /// The resources currently held in this graph.
    pub fn resource_nodes(&self) -> impl Iterator<Item = &ResourceNode> {
        self.resources.nodes().map(|(_, resource)| resource)
    }

    /// Resource edges with their resource endpoints. Surface indices never cross this seam.
    pub fn resource_edges(
        &self,
    ) -> impl Iterator<Item = (&ResourceNode, &ResourceNode, &super::EdgePayload)> {
        self.resources.inner().edge_references().map(|edge| {
            (
                self.resources
                    .node(edge.source())
                    .expect("resource endpoint exists"),
                self.resources
                    .node(edge.target())
                    .expect("resource endpoint exists"),
                edge.weight(),
            )
        })
    }

    /// Resource metadata, separate from surface metadata.
    pub fn resource_facets(&self) -> &chartulary::FacetStore<Uuid> {
        &self.resource_facets
    }

    /// The resource explicitly shown by a surface, when recorded.
    pub fn shown_resource_id(&self, surface: NodeKey) -> Option<Uuid> {
        self.shown_resources
            .get(&self.get_node(surface)?.id)
            .copied()
    }

    /// Surface identities showing a resource, in stable order.
    pub fn surface_ids_showing_resource(&self, resource_id: Uuid) -> Vec<Uuid> {
        self.shown_resources
            .iter()
            .filter_map(|(surface, resource)| (*resource == resource_id).then_some(*surface))
            .collect()
    }

    pub(crate) fn resource_record(&self, id: Uuid) -> Option<PersistedResourceRecord> {
        let resource = self.resource(id)?;
        let facets = self
            .resource_facets
            .facets_of(&id)
            .into_iter()
            .flat_map(|facets| facets.iter())
            .map(|(facet, value)| PersistedResourceFacet {
                facet: facet.as_str().to_string(),
                value_json: serde_json::to_string(value).expect("JSON value serializes"),
            })
            .collect();
        Some(PersistedResourceRecord {
            canonical_iri: resource.canonical_iri().to_string(),
            facets,
        })
    }

    pub(crate) fn set_resource_record(
        &mut self,
        id: Uuid,
        record: Option<PersistedResourceRecord>,
    ) -> bool {
        let Some(record) = record else {
            let Some(key) = self.resources.key_of(&id) else {
                return false;
            };
            if self
                .shown_resources
                .values()
                .any(|resource| *resource == id)
                || self
                    .resources
                    .inner()
                    .edge_references()
                    .any(|edge| edge.source() == key || edge.target() == key)
            {
                return false;
            }
            self.resources.remove(key);
            self.resource_facets.remove_node(&id);
            self.bump_revision();
            return true;
        };
        let resource = ResourceNode::from_canonical_iri(&record.canonical_iri);
        if resource.id() != id {
            return false;
        }
        let mut facets = std::collections::BTreeMap::new();
        for facet in record.facets {
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&facet.value_json) else {
                return false;
            };
            if facets
                .insert(chartulary::FacetId::new(facet.facet), value)
                .is_some()
            {
                return false;
            }
        }
        if self
            .resource(id)
            .is_some_and(|existing| existing == &resource)
            && self
                .resource_facets
                .facets_of(&id)
                .map(|values| {
                    values
                        .iter()
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect::<std::collections::BTreeMap<_, _>>()
                })
                .unwrap_or_default()
                == facets
        {
            return false;
        }
        self.resources.insert(resource);
        self.resource_facets.remove_node(&id);
        for (facet, value) in facets {
            self.resource_facets
                .set(id, facet, value, &chartulary::AcceptAll)
                .expect("permissive facet validator");
        }
        self.bump_revision();
        true
    }

    pub(crate) fn set_shown_resource(&mut self, surface: Uuid, resource: Option<Uuid>) -> bool {
        if self.inner.key_of(&surface).is_none()
            || resource.is_some_and(|id| self.resource(id).is_none())
        {
            return false;
        }
        if self.shown_resources.get(&surface).copied() == resource {
            return false;
        }
        if let Some(resource) = resource {
            self.shown_resources.insert(surface, resource);
        } else {
            self.shown_resources.remove(&surface);
        }
        self.bump_revision();
        true
    }

    pub(crate) fn persisted_resource_edges_between(
        &self,
        from: Uuid,
        to: Uuid,
    ) -> Vec<PersistedEdge> {
        let (Some(source), Some(target)) =
            (self.resources.key_of(&from), self.resources.key_of(&to))
        else {
            return Vec::new();
        };
        self.resources
            .inner()
            .edges_connecting(source, target)
            .map(|edge| super::snapshot::persisted_edge_for_ids(from, to, edge.weight()))
            .collect()
    }

    pub(crate) fn set_resource_edges_between(
        &mut self,
        from: Uuid,
        to: Uuid,
        edges: &[PersistedEdge],
    ) -> bool {
        let (Some(source), Some(target)) =
            (self.resources.key_of(&from), self.resources.key_of(&to))
        else {
            return false;
        };
        if edges
            .iter()
            .any(|edge| edge.from_node_id != from.to_string() || edge.to_node_id != to.to_string())
        {
            return false;
        }
        if edges
            .iter()
            .filter_map(|edge| edge.semantic.as_ref())
            .any(|semantic| {
                semantic.statements.is_empty()
                    && (!semantic.sub_kinds.is_empty() || semantic.predicate.is_some())
            })
        {
            return false;
        }
        let mut statements = std::collections::BTreeMap::new();
        for statement in edges
            .iter()
            .filter_map(|edge| edge.semantic.as_ref())
            .flat_map(|semantic| &semantic.statements)
        {
            if let Some(previous) = statements.insert(&statement.statement_id, statement)
                && previous != statement
            {
                return false;
            }
        }
        for edge in self.resources.inner().edge_references() {
            if edge.source() == source && edge.target() == target {
                continue;
            }
            if edge
                .weight()
                .semantic_statements()
                .iter()
                .any(|statement| statements.contains_key(&statement.statement_id))
            {
                return false;
            }
        }
        if self.inner.inner().edge_weights().any(|edge| {
            edge.semantic_statements()
                .iter()
                .any(|statement| statements.contains_key(&statement.statement_id))
        }) {
            return false;
        }
        let payloads: Vec<_> = edges
            .iter()
            .map(super::snapshot::payload_from_persisted)
            .collect();
        let current: Vec<_> = self
            .resources
            .inner()
            .edges_connecting(source, target)
            .map(|edge| edge.weight())
            .collect();
        if current.len() == payloads.len()
            && current.iter().zip(&payloads).all(|(old, new)| *old == new)
        {
            return false;
        }
        let old: Vec<_> = self
            .resources
            .inner()
            .edges_connecting(source, target)
            .map(|edge| edge.id())
            .collect();
        for edge in old {
            self.resources.disconnect(edge);
        }
        for payload in payloads.into_iter().rev() {
            self.resources.connect(source, target, payload);
        }
        self.bump_revision();
        true
    }
}

impl Identified for ResourceNode {
    type Id = Uuid;

    fn id(&self) -> &Uuid {
        &self.container.id
    }
}

impl Addressed for ResourceNode {
    fn addresses(&self) -> Vec<Address> {
        self.container.addresses.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Graph, SurfaceNode};

    fn record(iri: &str, value_json: &str) -> PersistedResourceRecord {
        PersistedResourceRecord {
            canonical_iri: chartulary::canonical_url(iri),
            facets: vec![PersistedResourceFacet {
                facet: "test.content".into(),
                value_json: value_json.into(),
            }],
        }
    }

    #[test]
    fn resource_record_validation_is_atomic_and_replacement_clears_old_facets() {
        let mut graph = Graph::new();
        let iri = "https://example.com/resource";
        let id = chartulary::resource_id(iri);
        let original = record(iri, "{\"title\":\"original\"}");
        assert!(graph.set_resource_record(id, Some(original.clone())));
        assert_eq!(graph.resource_record(id), Some(original.clone()));
        assert!(!graph.set_resource_record(id, Some(original.clone())));
        assert!(!graph.set_resource_record(Uuid::nil(), Some(original.clone())));
        assert!(!graph.set_resource_record(id, Some(record(iri, "invalid JSON"))));
        let mut duplicate = original.clone();
        duplicate.facets.push(duplicate.facets[0].clone());
        assert!(!graph.set_resource_record(id, Some(duplicate)));
        assert_eq!(graph.resource_record(id), Some(original));
        assert!(graph.set_resource_record(
            id,
            Some(PersistedResourceRecord {
                canonical_iri: iri.into(),
                facets: vec![]
            })
        ));
        assert!(graph.resource_record(id).unwrap().facets.is_empty());
        assert!(graph.set_resource_record(id, None));
        assert!(graph.resource(id).is_none());
        assert!(graph.resource_facets.facets_of(&id).is_none());
    }

    #[test]
    fn shown_resource_is_explicit_and_references_guard_inverse_creation() {
        let mut graph = Graph::new();
        let first = graph.add_node(
            "https://example.com/page".into(),
            euclid::default::Point2D::new(0.0, 0.0),
        );
        let second = graph.add_node(
            "https://example.com/page#part".into(),
            euclid::default::Point2D::new(1.0, 0.0),
        );
        let surface = graph.get_node(first).unwrap().id;
        let alias = graph.get_node(second).unwrap().id;
        let id = chartulary::resource_id("https://example.com/page");
        assert!(graph.shown_resource_id(first).is_none());
        assert!(!graph.set_shown_resource(surface, Some(id)));
        assert!(graph.set_resource_record(id, Some(record("https://example.com/page", "null"))));
        assert!(graph.set_shown_resource(surface, Some(id)));
        assert!(graph.set_shown_resource(alias, Some(id)));
        let mut expected = vec![surface, alias];
        expected.sort();
        assert_eq!(graph.surface_ids_showing_resource(id), expected);
        assert!(!graph.set_shown_resource(surface, Some(id)));
        assert!(!graph.set_resource_record(id, None));
        assert!(graph.remove_node(first));
        assert_eq!(graph.surface_ids_showing_resource(id), vec![alias]);
        assert!(graph.set_shown_resource(alias, None));
        assert!(graph.set_resource_record(id, None));
    }

    #[test]
    fn resource_pair_rejects_conflicting_handles_before_replacing_truth() {
        use crate::graph::{EdgePayload, SemanticStatement, SemanticSubKind};
        let mut graph = Graph::new();
        let ids: Vec<_> = ["https://a.test/", "https://b.test/", "https://c.test/"]
            .into_iter()
            .map(|iri| {
                let id = chartulary::resource_id(iri);
                assert!(graph.set_resource_record(id, Some(record(iri, "null"))));
                id
            })
            .collect();
        let mut payload = EdgePayload::new();
        payload.push_persisted_semantic_statement(SemanticStatement {
            statement_id: "stable-handle".into(),
            predicate: crate::graph::predicate_iri(SemanticSubKind::Cites).into(),
            recognized_sub_kind: Some(SemanticSubKind::Cites),
            label: Some("original".into()),
            graph_scope: crate::types::GraphScope::Default,
            provenance_iri: Some("https://alice.test/".into()),
            asserted_at_ms: Some(17),
        });
        let original = super::super::snapshot::persisted_edge_for_ids(ids[0], ids[1], &payload);
        let mut aggregate_only = original.clone();
        aggregate_only.semantic.as_mut().unwrap().statements.clear();
        assert!(!graph.set_resource_edges_between(ids[0], ids[1], &[aggregate_only.clone()]));
        assert!(!graph.set_resource_edges_between(ids[0], ids[1], &[aggregate_only]));
        assert!(
            graph
                .persisted_resource_edges_between(ids[0], ids[1])
                .is_empty()
        );
        assert!(graph.set_resource_edges_between(ids[0], ids[1], &[original.clone()]));
        assert!(!graph.set_resource_edges_between(ids[0], ids[1], &[original.clone()]));
        let mut conflicting = original.clone();
        conflicting.semantic.as_mut().unwrap().statements[0].label = Some("conflict".into());
        assert!(!graph.set_resource_edges_between(
            ids[0],
            ids[1],
            &[original.clone(), conflicting]
        ));
        assert_eq!(
            graph.persisted_resource_edges_between(ids[0], ids[1]),
            vec![original.clone()]
        );
        let mut other_pair = original.clone();
        other_pair.to_node_id = ids[2].to_string();
        assert!(!graph.set_resource_edges_between(ids[0], ids[2], &[other_pair.clone()]));
        assert!(
            graph
                .persisted_resource_edges_between(ids[0], ids[2])
                .is_empty()
        );
        other_pair.semantic.as_mut().unwrap().statements[0].statement_id = "other-handle".into();
        assert!(graph.set_resource_edges_between(ids[0], ids[2], &[other_pair.clone()]));
        assert_eq!(
            graph.persisted_resource_edges_between(ids[0], ids[2]),
            vec![other_pair.clone()]
        );
        let first = graph.add_node(
            "https://surface.test/a".into(),
            euclid::default::Point2D::new(0.0, 0.0),
        );
        let second = graph.add_node(
            "https://surface.test/b".into(),
            euclid::default::Point2D::new(1.0, 0.0),
        );
        let mut collision = other_pair.clone();
        collision.semantic.as_mut().unwrap().statements[0].statement_id = "surface-handle".into();
        graph.inner.connect(
            first,
            second,
            super::super::snapshot::payload_from_persisted(&collision),
        );
        assert!(!graph.set_resource_edges_between(ids[0], ids[2], &[collision.clone()]));
        assert_eq!(
            graph.persisted_resource_edges_between(ids[0], ids[2]),
            vec![other_pair]
        );
        collision.semantic.as_mut().unwrap().statements[0].statement_id =
            "fresh-resource-handle".into();
        assert!(graph.set_resource_edges_between(ids[0], ids[2], &[collision.clone()]));
        assert_eq!(
            graph.persisted_resource_edges_between(ids[0], ids[2]),
            vec![collision]
        );
        assert!(!graph.set_resource_record(ids[0], None));
        assert!(graph.set_resource_edges_between(ids[0], ids[1], &[]));
        assert!(graph.set_resource_edges_between(ids[0], ids[2], &[]));
        assert!(graph.set_resource_record(ids[0], None));
    }

    #[test]
    fn resource_identity_is_common_and_canonical_without_renumbering_surfaces() {
        let iri = "https://example.com/page?id=7";
        let alias = "https://EXAMPLE.COM:443/page?utm_source=news&id=7#part";
        let resource = ResourceNode::new(alias);
        assert_eq!(resource.canonical_iri(), iri);
        assert_eq!(resource.id(), chartulary::resource_id(iri));
        assert_eq!(*Identified::id(&resource), resource.id());
        assert_eq!(resource.primary_address(), Some(Address::new(iri)));
        assert_eq!(resource, ResourceNode::new(iri));
        assert_ne!(
            resource.id(),
            ResourceNode::new("https://example.com/page?id=8").id()
        );
        assert_ne!(resource.id(), Graph::node_namespace_id(iri));

        let first = SurfaceNode::test_stub(iri);
        let second = SurfaceNode::test_stub(alias);
        assert_ne!(first.id, second.id);
        assert_eq!(
            ResourceNode::new(first.url()),
            ResourceNode::new(second.url())
        );
    }

    #[test]
    fn vocabulary_term_identity_preserves_fragments_query_and_case() {
        let cat = ResourceNode::for_term("https://vocab.test/concepts#Cat");
        for iri in [
            "https://vocab.test/concepts#Dog",
            "https://vocab.test/concepts#cat",
            "https://VOCAB.test/concepts#Cat",
            "https://vocab.test/concepts?edition=1#Cat",
        ] {
            let term = ResourceNode::for_term(iri);
            assert_eq!(term.canonical_iri(), iri);
            assert_eq!(term.id(), chartulary::resource_id_from_canonical_iri(iri));
            assert_ne!(term.id(), cat.id());
        }
        assert_eq!(cat, ResourceNode::for_term(cat.canonical_iri()));
        assert_eq!(
            ResourceNode::new("https://vocab.test/concepts#Cat"),
            ResourceNode::new("https://VOCAB.test:443/concepts#Dog")
        );
        assert_ne!(cat.id(), ResourceNode::new(cat.canonical_iri()).id());
    }

    #[test]
    fn exact_term_records_and_resource_edge_endpoints_survive_checked_load() {
        use crate::graph::{EdgePayload, SemanticStatement, SemanticSubKind};
        let mut graph = Graph::new();
        let terms = [
            ResourceNode::for_term("https://vocab.test/concepts#Cat"),
            ResourceNode::for_term("https://vocab.test/concepts#Dog"),
        ];
        for term in &terms {
            assert!(graph.set_resource_record(
                term.id(),
                Some(PersistedResourceRecord {
                    canonical_iri: term.canonical_iri().into(),
                    facets: vec![],
                })
            ));
        }
        let mut payload = EdgePayload::new();
        payload.push_persisted_semantic_statement(SemanticStatement {
            statement_id: "exact-vocabulary-assertion".into(),
            predicate: crate::graph::predicate_iri(SemanticSubKind::SameEntityAs).into(),
            recognized_sub_kind: Some(SemanticSubKind::SameEntityAs),
            label: None,
            graph_scope: crate::types::GraphScope::Default,
            provenance_iri: Some("https://author.test/".into()),
            asserted_at_ms: Some(19),
        });
        let edge =
            super::super::snapshot::persisted_edge_for_ids(terms[0].id(), terms[1].id(), &payload);
        assert!(graph.set_resource_edges_between(terms[0].id(), terms[1].id(), &[edge]));
        let restored = Graph::try_from_snapshot(&graph.to_snapshot()).unwrap();
        for term in &terms {
            assert_eq!(restored.resource(term.id()), Some(term));
        }
        let edges: Vec<_> = restored.resource_edges().collect();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].0, &terms[0]);
        assert_eq!(edges[0].1, &terms[1]);
        assert_eq!(edges[0].2, &payload);
        assert_eq!(
            restored.edge_count(),
            0,
            "resource edges do not create surface edges"
        );
    }

    #[test]
    fn chartulary_indexes_resources_by_the_shared_identity() {
        let mut graph = chartulary::Graph::<ResourceNode, ()>::new();
        let first = graph.insert(ResourceNode::new("https://example.com/page"));
        let alias = graph.insert(ResourceNode::new("https://EXAMPLE.COM/page#part"));
        assert_eq!(first, alias);
        assert_eq!(graph.node_count(), 1);
        let other = graph.insert(ResourceNode::new("https://example.com/other"));
        assert_ne!(first, other);
        assert_eq!(graph.node_count(), 2);
        let clone = graph.clone();
        assert_eq!(
            clone.get(&chartulary::resource_id("https://example.com/page")),
            graph.node(first)
        );
        assert_eq!(
            clone.get(&chartulary::resource_id("https://example.com/other")),
            graph.node(other)
        );
    }
}
