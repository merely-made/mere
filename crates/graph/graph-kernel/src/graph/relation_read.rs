// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Relation reads across explicitly identified graph strata.

use petgraph::Direction;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use uuid::Uuid;

use super::{EdgePayload, Graph, NodeKey, RelationKey, ResourceEdgeKey};

impl Graph {
    /// Read the owning store named by this relation handle.
    pub fn get_relation(&self, key: RelationKey) -> Option<&EdgePayload> {
        match key {
            RelationKey::Surface(key) => self.get_edge(key),
            RelationKey::Resource(key) => self.get_resource_edge(key),
        }
    }

    pub(crate) fn get_relation_mut(&mut self, key: RelationKey) -> Option<&mut EdgePayload> {
        match key {
            RelationKey::Surface(key) => self.get_edge_mut(key),
            RelationKey::Resource(key) => self.resources.edge_mut(key.raw()),
        }
    }

    /// Read a resource edge without accepting a surface edge index.
    pub fn get_resource_edge(&self, key: ResourceEdgeKey) -> Option<&EdgePayload> {
        self.resources.edge(key.raw())
    }

    /// Find the first directed resource edge between resource identities.
    pub fn find_resource_edge_key(&self, from: Uuid, to: Uuid) -> Option<ResourceEdgeKey> {
        let (from, to) = (self.resources.key_of(&from)?, self.resources.key_of(&to)?);
        self.resources
            .inner()
            .find_edge(from, to)
            .map(ResourceEdgeKey::from_raw)
    }

    /// Every resource edge, with a typed handle and stable resource endpoints.
    pub fn resource_relations(
        &self,
    ) -> impl Iterator<Item = (ResourceEdgeKey, Uuid, Uuid, &EdgePayload)> {
        self.resources.inner().edge_references().map(|edge| {
            (
                ResourceEdgeKey::from_raw(edge.id()),
                self.resources
                    .node(edge.source())
                    .expect("resource endpoint exists")
                    .id(),
                self.resources
                    .node(edge.target())
                    .expect("resource endpoint exists")
                    .id(),
                edge.weight(),
            )
        })
    }

    /// Directed surface edges followed by edges between the resources shown.
    /// Parallel records and self-loops each appear once per directed pair.
    pub fn projected_relations_between(
        &self,
        from: NodeKey,
        to: NodeKey,
    ) -> impl Iterator<Item = (RelationKey, &EdgePayload)> {
        let surface_pair = self
            .inner
            .contains_node(from)
            .then_some(from)
            .zip(self.inner.contains_node(to).then_some(to));
        let resource_pair = self
            .shown_resource_id(from)
            .zip(self.shown_resource_id(to))
            .and_then(|(from, to)| self.resources.key_of(&from).zip(self.resources.key_of(&to)));
        let surface = surface_pair.into_iter().flat_map(move |(from, to)| {
            self.inner
                .inner()
                .edges_connecting(from, to)
                .map(|edge| (RelationKey::Surface(edge.id()), edge.weight()))
        });
        let resource = resource_pair.into_iter().flat_map(move |(from, to)| {
            self.resources
                .inner()
                .edges_connecting(from, to)
                .map(|edge| {
                    (
                        RelationKey::Resource(ResourceEdgeKey::from_raw(edge.id())),
                        edge.weight(),
                    )
                })
        });
        surface.chain(resource)
    }

    /// Outgoing relations lifted to every surface showing their target resource.
    pub fn projected_outgoing_relations(
        &self,
        from: NodeKey,
    ) -> impl Iterator<Item = (NodeKey, RelationKey, &EdgePayload)> {
        self.projected_adjacent_relations(from, Direction::Outgoing)
    }

    /// Incoming relations lifted from every surface showing their source resource.
    pub fn projected_incoming_relations(
        &self,
        to: NodeKey,
    ) -> impl Iterator<Item = (NodeKey, RelationKey, &EdgePayload)> {
        self.projected_adjacent_relations(to, Direction::Incoming)
    }

    fn projected_adjacent_relations(
        &self,
        surface: NodeKey,
        direction: Direction,
    ) -> impl Iterator<Item = (NodeKey, RelationKey, &EdgePayload)> {
        let surface_key = self.inner.contains_node(surface).then_some(surface);
        let resource_key = self
            .shown_resource_id(surface)
            .and_then(|id| self.resources.key_of(&id));
        let direct = surface_key.into_iter().flat_map(move |surface| {
            self.inner
                .inner()
                .edges_directed(surface, direction)
                .map(move |edge| {
                    let neighbor = match direction {
                        Direction::Outgoing => edge.target(),
                        Direction::Incoming => edge.source(),
                    };
                    (neighbor, RelationKey::Surface(edge.id()), edge.weight())
                })
        });
        let lifted = resource_key
            .into_iter()
            .flat_map(move |resource| self.resources.inner().edges_directed(resource, direction))
            .flat_map(move |edge| {
                let neighbor = match direction {
                    Direction::Outgoing => edge.target(),
                    Direction::Incoming => edge.source(),
                };
                let id = self
                    .resources
                    .node(neighbor)
                    .expect("resource endpoint exists")
                    .id();
                self.shown_resources
                    .iter()
                    .filter_map(move |(surface, resource)| {
                        (*resource == id)
                            .then(|| self.get_node_key_by_id(*surface))
                            .flatten()
                            .map(|surface| {
                                (
                                    surface,
                                    RelationKey::Resource(ResourceEdgeKey::from_raw(edge.id())),
                                    edge.weight(),
                                )
                            })
                    })
            });
        direct.chain(lifted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{EdgeAssertion, ResourceNode, SemanticStatement, SemanticSubKind};
    use crate::persistence::PersistedResourceRecord;
    use crate::types::GraphScope;
    use euclid::default::Point2D;
    use petgraph::stable_graph::EdgeIndex;

    fn surface(graph: &mut Graph, iri: &str) -> NodeKey {
        graph.add_node(iri.into(), Point2D::zero())
    }

    fn resource(graph: &mut Graph, iri: &str) -> Uuid {
        let node = ResourceNode::for_term(iri);
        assert!(graph.set_resource_record(
            node.id(),
            Some(PersistedResourceRecord {
                canonical_iri: iri.into(),
                facets: vec![],
            })
        ));
        node.id()
    }

    fn payload(id: &str, predicate: &str) -> EdgePayload {
        let mut payload = EdgePayload::new();
        payload.push_persisted_semantic_statement(SemanticStatement {
            statement_id: id.into(),
            predicate: predicate.into(),
            recognized_sub_kind: None,
            label: None,
            graph_scope: GraphScope::Default,
            provenance_iri: Some("https://author.test/".into()),
            asserted_at_ms: Some(17),
        });
        payload
    }

    fn pair(graph: &mut Graph, from: Uuid, to: Uuid, ids: &[&str], predicate: &str) {
        let edges: Vec<_> = ids
            .iter()
            .map(|id| {
                super::super::snapshot::persisted_edge_for_ids(from, to, &payload(id, predicate))
            })
            .collect();
        assert!(graph.set_resource_edges_between(from, to, &edges));
    }

    #[test]
    fn relation_read_equal_indices_select_only_the_named_store() {
        let mut graph = Graph::new();
        let a = surface(&mut graph, "https://surface.test/a");
        let b = surface(&mut graph, "https://surface.test/b");
        let surface_handle = graph
            .assert_relation(
                a,
                b,
                EdgeAssertion::Semantic {
                    sub_kind: SemanticSubKind::UserGrouped,
                    label: None,
                    decay_progress: None,
                },
            )
            .unwrap();
        let RelationKey::Surface(surface_key) = surface_handle else {
            panic!("surface writer handle")
        };
        let ra = resource(&mut graph, "https://resource.test/a");
        let rb = resource(&mut graph, "https://resource.test/b");
        pair(
            &mut graph,
            ra,
            rb,
            &["resource-cites"],
            super::super::predicate_iri(SemanticSubKind::Cites),
        );
        let resource_key = graph.find_resource_edge_key(ra, rb).unwrap();
        assert_eq!(surface_key.index(), 0);
        assert_eq!(resource_key.raw().index(), 0);
        assert!(
            graph
                .get_edge(surface_key)
                .unwrap()
                .semantic_statements()
                .iter()
                .any(|s| s.recognized_sub_kind == Some(SemanticSubKind::UserGrouped))
        );
        assert_eq!(
            graph.get_relation(surface_handle),
            graph.get_edge(surface_key)
        );
        assert_eq!(
            graph.get_relation(RelationKey::Resource(resource_key)),
            graph.get_resource_edge(resource_key)
        );
        assert_eq!(
            graph
                .get_resource_edge(resource_key)
                .unwrap()
                .semantic_statements()[0]
                .statement_id,
            "resource-cites"
        );
        assert_eq!(graph.find_edge_key(a, b), Some(surface_key));
        let c = surface(&mut graph, "https://surface.test/c");
        let RelationKey::Surface(surface_only) = graph
            .assert_relation(
                a,
                c,
                EdgeAssertion::Semantic {
                    sub_kind: SemanticSubKind::UserGrouped,
                    label: None,
                    decay_progress: None,
                },
            )
            .unwrap()
        else {
            panic!("surface writer handle")
        };
        assert!(graph.get_edge(surface_only).is_some());
        let absent_resource = ResourceEdgeKey::from_raw(surface_only);
        assert!(graph.get_resource_edge(absent_resource).is_none());
        assert!(
            graph
                .get_relation(RelationKey::Resource(absent_resource))
                .is_none()
        );
        assert!(graph.get_edge(EdgeIndex::new(99)).is_none());
        assert_eq!(graph.resource_relations().count(), 1);
    }

    #[test]
    fn relation_read_projection_preserves_parallel_direction_and_binding_fanout() {
        let mut graph = Graph::new();
        let a = surface(&mut graph, "https://surface.test/a");
        let b = surface(&mut graph, "https://surface.test/b");
        let alias = surface(&mut graph, "https://surface.test/alias");
        let unbound = surface(&mut graph, "https://surface.test/unbound");
        let ra = resource(&mut graph, "https://resource.test/a");
        let rb = resource(&mut graph, "https://resource.test/b");
        let iri = "https://schema.org/citation";
        pair(&mut graph, ra, rb, &["forward-1", "forward-2"], iri);
        pair(&mut graph, rb, ra, &["backward"], iri);
        pair(&mut graph, ra, ra, &["self"], iri);
        graph.assert_relation(
            a,
            b,
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::UserGrouped,
                label: None,
                decay_progress: None,
            },
        );
        assert_eq!(graph.projected_relations_between(a, b).count(), 1);
        assert_eq!(graph.projected_outgoing_relations(unbound).count(), 0);
        assert!(graph.set_shown_resource(graph.get_node(a).unwrap().id, Some(ra)));
        assert!(graph.set_shown_resource(graph.get_node(b).unwrap().id, Some(rb)));
        assert!(graph.set_shown_resource(graph.get_node(alias).unwrap().id, Some(rb)));
        assert_eq!(graph.projected_relations_between(a, b).count(), 3);
        assert_eq!(graph.projected_relations_between(a, alias).count(), 2);
        assert_eq!(graph.projected_relations_between(b, a).count(), 1);
        assert_eq!(graph.projected_relations_between(a, a).count(), 1);
        let outgoing: Vec<_> = graph.projected_outgoing_relations(a).collect();
        assert_eq!(outgoing.len(), 6); // direct + two parallel edges to two surfaces + self
        assert_eq!(
            outgoing
                .iter()
                .filter(|(neighbor, key, _)| *neighbor == alias
                    && matches!(key, RelationKey::Resource(_)))
                .count(),
            2
        );
        assert_eq!(graph.projected_incoming_relations(b).count(), 3);
        assert_eq!(graph.projected_incoming_relations(a).count(), 3); // two sources + self
        assert!(graph.set_shown_resource(graph.get_node(alias).unwrap().id, None));
        assert_eq!(graph.projected_outgoing_relations(a).count(), 4);
        assert_eq!(graph.projected_relations_between(a, alias).count(), 0);
        assert_eq!(graph.resource_relations().count(), 4);
    }

    #[test]
    fn relation_read_display_role_uses_shown_content_with_surface_control() {
        let mut graph = Graph::new();
        let source = surface(&mut graph, "https://surface.test/source");
        let target = surface(&mut graph, "urn:mere:bnode:document:_:author");
        let legacy = surface(&mut graph, "urn:mere:bnode:document:_:publisher");
        graph.assert_semantic_predicate(source, legacy, "https://schema.org/publisher".into());
        assert_eq!(graph.node_display_label(legacy), "publisher");
        let from = resource(&mut graph, "https://resource.test/page");
        let to = resource(&mut graph, "urn:mere:bnode:document:_:author");
        pair(
            &mut graph,
            from,
            to,
            &["author-role"],
            "https://schema.org/author",
        );
        assert_eq!(graph.node_display_label(target), "node _:author");
        assert!(graph.set_shown_resource(graph.get_node(source).unwrap().id, Some(from)));
        assert!(graph.set_shown_resource(graph.get_node(target).unwrap().id, Some(to)));
        assert_eq!(graph.node_display_label(target), "author");
        assert_eq!(graph.node_display_label(legacy), "publisher");
    }
}
