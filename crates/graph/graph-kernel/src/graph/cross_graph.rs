// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Cross-graph node copy — the kernel half of the tear-out "fork" / cross-graph
//! composability gesture (tear-out brief §7.5; tearout-composability plan C4).
//!
//! A node→node derivation *within* one graph rides a petgraph `Provenance`
//! edge. A cross-graph copy's source node lives in a *different* graph, so the
//! relation cannot be a petgraph edge; the derivation is recorded on the minted
//! node as a [`NodeDerivation`] instead (see `node.rs` / `types.rs`). That
//! record is the lineage pointer back to the source and projects to a
//! `<copy> {provenance predicate} <source>` statement under the RDF projection.
//!
//! WASM-clean: must compile to `wasm32-unknown-unknown` (mirrors `graph/mod.rs`).

use euclid::default::Point2D;
use uuid::Uuid;

use super::node_facets::{PROVENANCE_DERIVATIONS, VisitHistoryFacet};
use super::{Graph, Node, NodeKey, ProvenanceSubKind};
use crate::types::NodeDerivation;

impl Graph {
    /// Copy `source`'s content into this graph, minting a fresh node and
    /// recording cross-graph derivation provenance back to the source.
    ///
    /// `source` is a donor [`Node`] from another [`Graph`]; `source_graph` is the
    /// donor graph's id (the constellation `GraphId` rendered as a string), or
    /// `None` when unknown / same-graph. The copy is a *new entity*: it gets a
    /// fresh identity, and its **content** (title, tags, classifications, open
    /// properties, address, visuals, viewer preferences) is cloned, while its
    /// **identity / runtime / session / arrangement** state is reset (fresh id,
    /// unpinned, no frame hints; browser-runtime state lives in the host's
    /// `BrowserNodeState` sidecar and never travels with a copy). A
    /// [`NodeDerivation`] records `(CopiedFrom, source.id, source_graph)` so the
    /// lineage survives and projects to a `wasDerivedFrom` statement.
    ///
    /// The donor's `import_provenance` is intentionally *not* carried over: that
    /// describes how the donor entered *its* graph from an external source; the
    /// copy's provenance is the derivation record, not the donor's import.
    ///
    /// Not available on `wasm32` (mints a random id, like [`add_node`]); use
    /// [`copy_node_from_with_id`](Self::copy_node_from_with_id) with a
    /// host-provided UUID there.
    ///
    /// [`add_node`]: Self::add_node
    #[cfg(not(target_arch = "wasm32"))]
    pub fn copy_node_from(
        &mut self,
        source: &Node,
        source_graph: Option<String>,
        position: Point2D<f32>,
    ) -> NodeKey {
        self.copy_node_from_with_id(Uuid::new_v4(), source, source_graph, position)
    }

    /// [`copy_node_from`](Self::copy_node_from) with a caller-supplied id (the
    /// wasm-clean entry point, mirroring
    /// [`add_node_with_id`](Self::add_node_with_id)).
    pub fn copy_node_from_with_id(
        &mut self,
        id: Uuid,
        source: &Node,
        source_graph: Option<String>,
        _position: Point2D<f32>,
    ) -> NodeKey {
        let derivation = NodeDerivation {
            sub_kind: ProvenanceSubKind::CopiedFrom,
            source_node: source.id.to_string(),
            source_graph,
        };
        let url = source.primary_address().as_url_str().to_string();
        let mut container = source.container.clone();
        container.id = id;
        // Deliberately NOT carried: a copy is un-resided. Carrying the LogId
        // would leave two nodes bearing one world file.
        container.nested = None;

        let key = self.inner.insert(Node {
            // --- container content: cloned from the source ---
            container,
            // References copy freely: the blob is content-addressed and shared,
            // so a cross-graph copy costs ~40 bytes per role, not a duplicated
            // image.
            images: source.images.clone(),
        });
        self.set_node_facet(key, PROVENANCE_DERIVATIONS, &vec![derivation]);
        self.set_node_facet(
            key,
            super::node_facets::VISIT_HISTORY,
            &VisitHistoryFacet {
                last_visited_ms: Some(Self::epoch_ms()),
                last_session_visited: 0,
            },
        );

        self.url_to_nodes.entry(url).or_default().push(key);
        self.bump_revision();
        self.bump_content_revision();
        key
    }

    /// [`copy_node_from`](Self::copy_node_from) taking plain `x` / `y` screen-graph
    /// coords, so a host can place the copy without depending on `euclid`. (Tear-out
    /// gestures G5.)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn copy_node_from_xy(
        &mut self,
        source: &Node,
        source_graph: Option<String>,
        x: f32,
        y: f32,
    ) -> NodeKey {
        self.copy_node_from(source, source_graph, Point2D::new(x, y))
    }

    /// Copy the connected component containing `seed` (a node in `source`) into this
    /// graph: the kernel half of a tear-out **fork** (the reachable-subgraph snapshot,
    /// tear-out brief §4.3). The component is [`component_members`]' (every family,
    /// both strata, shown bindings at zero hops; ruling 38), so surfaces joined only by
    /// a resource relation copy together. Each surface is minted fresh (with a
    /// `CopiedFrom` [`NodeDerivation`] back to its source); surface edges among the
    /// copies are re-pointed onto the new nodes by cloning each payload verbatim.
    /// Resources keep their ids (shared content, as selection Copy retains them): the
    /// walked resources and the predicate-declaration resources their content needs
    /// travel as whole records, each copied surface keeps its shown binding, and the
    /// resource edges among them travel verbatim. A resource or resource pair this
    /// graph already holds keeps this graph's truth. Edges leaving the component are
    /// dropped (the snapshot is closed under the component).
    /// Returns the [`ComponentCopy`]: the new keys plus the `source id → new id`
    /// remap the host's facet-carry maps donor facets through (G4-R R0 — positions
    /// and every other per-node character ride `arrangement.*` / `web.*` / … facets,
    /// not the graph, so the fork's layout carry needs this mapping).
    /// (Tear-out gestures G4.)
    ///
    /// [`component_members`]: Self::component_members
    #[cfg(not(target_arch = "wasm32"))]
    pub fn copy_component_from(
        &mut self,
        source: &Graph,
        seed: Uuid,
        source_graph: Option<String>,
    ) -> ComponentCopy {
        use petgraph::visit::{EdgeRef, IntoEdgeReferences};
        let members = source.component_members(seed, &[]);
        // Copy each node, mapping its old key to the fresh one for edge re-pointing
        // (and its old id to the fresh id for the host's facet-carry).
        let mut key_remap: std::collections::HashMap<NodeKey, NodeKey> =
            std::collections::HashMap::with_capacity(members.len());
        let mut copy = ComponentCopy {
            new_keys: Vec::with_capacity(members.len()),
            id_remap: Vec::with_capacity(members.len()),
        };
        for (old_key, node) in members.iter().filter_map(|id| source.get_node_by_id(*id)) {
            let source_id = node.id;
            // Position is no longer a node field; the copy is placed by the
            // destination's layout (seiche / arrangement facets), not carried over.
            let new_key = self.copy_node_from(node, source_graph.clone(), Point2D::zero());
            let new_id = self.inner.node(new_key).expect("inserted copy").id;
            // Content facets travel with a component copy. Runtime,
            // arrangement, visit, and import provenance do not; the host
            // may separately carry view/foreign facets through `id_remap`.
            for facet in [
                super::node_facets::PRESENTATION_TAGS,
                super::node_facets::SEMANTIC_CLASSIFICATIONS,
                super::node_facets::SEMANTIC_PROPERTIES,
            ] {
                let facet_id = chartulary::FacetId::new(facet);
                if let Some(value) = source.facets.get(&source_id, &facet_id) {
                    self.facets
                        .set(new_id, facet_id, value.clone(), &chartulary::AcceptAll)
                        .expect("AcceptAll cannot reject copied content");
                }
            }
            key_remap.insert(old_key, new_key);
            copy.new_keys.push(new_key);
            copy.id_remap.push((source_id, new_id));
        }
        // Re-point the component's internal surface edges (both endpoints copied):
        // clone each edge's payload verbatim onto the new node pair.
        for edge in source.inner.inner().edge_references() {
            if let (Some(&from), Some(&to)) =
                (key_remap.get(&edge.source()), key_remap.get(&edge.target()))
            {
                self.inner.connect(from, to, edge.weight().clone());
            }
        }
        self.copy_component_resources(source, &key_remap);
        self.bump_revision();
        copy
    }

    /// The resource half of [`copy_component_from`](Self::copy_component_from):
    /// records, shown bindings and resource edges for the copied surfaces.
    #[cfg(not(target_arch = "wasm32"))]
    fn copy_component_resources(
        &mut self,
        source: &Graph,
        key_remap: &std::collections::HashMap<NodeKey, NodeKey>,
    ) {
        use super::predicate_declarations::PREDICATE_DECLARATIONS_FACET;
        use super::resource_content::RESOURCE_PROPERTIES;
        use petgraph::Direction;
        use petgraph::visit::{EdgeRef, IntoEdgeReferences};
        use std::collections::BTreeSet;

        // The walked resources: every resource reachable from a copied surface's
        // shown resource through resource edges (the walk `component_members` took).
        let mut resources = BTreeSet::new();
        let mut stack: Vec<Uuid> = key_remap
            .keys()
            .filter_map(|key| source.shown_resource_id(*key))
            .filter(|id| source.resource(*id).is_some())
            .collect();
        while let Some(id) = stack.pop() {
            if !resources.insert(id) {
                continue;
            }
            let key = source
                .resources
                .key_of(&id)
                .expect("walked resource exists");
            for direction in [Direction::Outgoing, Direction::Incoming] {
                for edge in source.resources.inner().edges_directed(key, direction) {
                    let other = match direction {
                        Direction::Outgoing => edge.target(),
                        Direction::Incoming => edge.source(),
                    };
                    stack.push(source.resources.node(other).expect("endpoint").id());
                }
            }
        }
        // Declarations the copied statements and properties need, to a fixpoint
        // (a declaration resource can itself carry declared properties).
        let declarations = chartulary::FacetId::new(PREDICATE_DECLARATIONS_FACET);
        let declared = |predicate: &str| {
            let id = chartulary::resource_id_from_canonical_iri(predicate);
            source
                .resource_facets
                .get(&id, &declarations)
                .is_some()
                .then_some(id)
        };
        let mut predicates: Vec<String> = source
            .inner
            .inner()
            .edge_references()
            .filter(|edge| {
                key_remap.contains_key(&edge.source()) && key_remap.contains_key(&edge.target())
            })
            .flat_map(|edge| edge.weight().semantic_statements())
            .map(|statement| statement.predicate.clone())
            .chain(
                key_remap
                    .keys()
                    .flat_map(|key| source.node_properties(*key).unwrap_or_default())
                    .map(|property| property.predicate),
            )
            .collect();
        let properties = chartulary::FacetId::new(RESOURCE_PROPERTIES);
        loop {
            let before = resources.len();
            for (from, to, payload) in source.resource_edges() {
                if resources.contains(&from.id()) && resources.contains(&to.id()) {
                    predicates.extend(
                        payload
                            .semantic_statements()
                            .iter()
                            .map(|statement| statement.predicate.clone()),
                    );
                }
            }
            for id in &resources {
                if let Some(value) = source.resource_facets.get(id, &properties) {
                    predicates.extend(
                        serde_json::from_value::<Vec<crate::types::NodeProperty>>(value.clone())
                            .unwrap_or_default()
                            .into_iter()
                            .map(|property| property.predicate),
                    );
                }
            }
            let needed: Vec<Uuid> = predicates.drain(..).filter_map(|p| declared(&p)).collect();
            resources.extend(needed);
            if resources.len() == before {
                break;
            }
        }
        // Records first (a binding or edge needs its resource), then bindings, then
        // resource edges. Truth this graph already holds wins.
        for id in &resources {
            if self.resource(*id).is_none() {
                self.set_resource_record(*id, source.resource_record(*id));
            }
        }
        for (old_key, new_key) in key_remap {
            if let (Some(resource), Some(new_id)) = (
                source.shown_resource_id(*old_key),
                self.inner.node(*new_key).map(|node| node.id),
            ) {
                self.set_shown_resource(new_id, Some(resource));
            }
        }
        let pairs: BTreeSet<(Uuid, Uuid)> = source
            .resource_edges()
            .map(|(from, to, _)| (from.id(), to.id()))
            .filter(|(from, to)| resources.contains(from) && resources.contains(to))
            .collect();
        for (from, to) in pairs {
            if self.persisted_resource_edges_between(from, to).is_empty() {
                self.set_resource_edges_between(
                    from,
                    to,
                    &source.persisted_resource_edges_between(from, to),
                );
            }
        }
    }
}

/// The result of [`Graph::copy_component_from`]: the minted keys plus the
/// `(source id, new id)` pairs, in copy order. The id remap is the seam the
/// host's facet-carry maps donor facets through (a forked node keeps its whole
/// per-node character — `arrangement.*`, `web.*`, foreign namespaces — by
/// copying facets from `source` to `new`).
#[derive(Clone, Debug, Default)]
pub struct ComponentCopy {
    /// The fresh node keys in this graph, in copy order.
    pub new_keys: Vec<NodeKey>,
    /// `(source node id, minted node id)` per copied node, in copy order.
    pub id_remap: Vec<(Uuid, Uuid)>,
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn donor_with_content() -> (Graph, NodeKey) {
        let mut a = Graph::new();
        let key = a.add_node(
            "https://example.com/article".to_string(),
            Point2D::new(1.0, 2.0),
        );
        {
            let node = a.inner.node_mut(key).unwrap();
            node.title = "An Article".to_string();
            node.tags = HashSet::from(["read-later".to_string(), "research".to_string()]);
        }
        a.append_node_property(
            key,
            crate::types::NodeProperty::new(
                "https://schema.org/datePublished".to_string(),
                "2026-01-01".to_string(),
            ),
        );
        a.set_node_pinned(key, true);
        (a, key)
    }

    #[test]
    fn copy_mints_a_fresh_node_cloning_content_with_derivation() {
        let (a, src_key) = donor_with_content();
        let source = a.get_node(src_key).unwrap().clone();
        let source_id = source.id;

        let mut b = Graph::new();
        let rev_before = b.revision();
        let copy_key =
            b.copy_node_from(&source, Some("graph-A".to_string()), Point2D::new(9.0, 9.0));
        assert!(
            b.revision() > rev_before,
            "copying a node in is structural and advances the revision"
        );
        let copy = b.get_node(copy_key).unwrap();

        // Fresh identity, content cloned.
        assert_ne!(copy.id, source_id, "copy is a new entity");
        assert_eq!(copy.title, "An Article");
        assert_eq!(copy.tags, source.tags);
        assert_eq!(copy.url(), source.url(), "same content, same address");

        // Position is not a node field (not inherited, not carried); the copy is
        // placed by the destination's layout.
        assert_eq!(
            b.node_is_pinned(copy_key),
            Some(false),
            "pin is per-placement"
        );

        // Derivation records the lineage back to the source.
        let derivations = b.node_derivations(copy_key).unwrap();
        assert_eq!(derivations.len(), 1);
        let d = &derivations[0];
        assert_eq!(d.sub_kind, ProvenanceSubKind::CopiedFrom);
        assert_eq!(d.source_node, source_id.to_string());
        assert_eq!(d.source_graph.as_deref(), Some("graph-A"));

        // The copy is discoverable by its address in the destination index.
        assert_eq!(b.get_nodes_by_url(source.url()).len(), 1);
    }

    #[test]
    fn copy_does_not_carry_the_donor_import_provenance() {
        let (mut a, src_key) = donor_with_content();
        a.set_node_import_provenance(
            src_key,
            vec![crate::types::NodeImportProvenance {
                source_id: "firefox".to_string(),
                source_label: "Firefox bookmarks".to_string(),
            }],
        );
        let source = a.get_node(src_key).unwrap().clone();

        let mut b = Graph::new();
        let copy_key = b.copy_node_from(&source, None, Point2D::zero());
        let copy = b.get_node(copy_key).unwrap();

        assert!(
            b.node_import_provenance(copy_key).unwrap().is_empty(),
            "donor import provenance is the donor's, not the copy's"
        );
        assert_eq!(
            b.node_derivations(copy_key).unwrap()[0].source_graph,
            None,
            "same-graph / unknown source graph"
        );
    }

    #[test]
    fn copy_component_clones_the_connected_subgraph_with_edges_and_provenance() {
        use crate::graph::{EdgeAssertion, SemanticSubKind};
        let mut a = Graph::new();
        let k1 = a.add_node("https://a.com/1".to_string(), Point2D::new(0.0, 0.0));
        let k2 = a.add_node("https://a.com/2".to_string(), Point2D::new(1.0, 0.0));
        let _k3 = a.add_node("https://a.com/3".to_string(), Point2D::new(9.0, 9.0)); // disconnected
        a.assert_relation(
            k1,
            k2,
            EdgeAssertion::Semantic {
                // A Surface relation; the Resource case is the test below.
                sub_kind: SemanticSubKind::UserGrouped,
                label: None,
                decay_progress: None,
            },
        );
        let seed = a.get_node(k1).unwrap().id;

        let mut b = Graph::new();
        let copy = b.copy_component_from(&a, seed, Some("graph-A".to_string()));

        // The connected component is {k1, k2}; the disconnected k3 is not pulled in.
        assert_eq!(
            copy.new_keys.len(),
            2,
            "copied the 2-node component, not the lone disconnected node"
        );
        assert_eq!(b.nodes().count(), 2);
        // The component's internal edge is re-pointed onto the copies.
        assert_eq!(b.relations().count(), 1, "the internal edge is re-pointed");
        // Each copy records `CopiedFrom` provenance back to the source graph.
        for (key, _) in b.nodes() {
            let derivations = b.node_derivations(key).unwrap();
            assert_eq!(derivations.len(), 1);
            assert_eq!(derivations[0].sub_kind, ProvenanceSubKind::CopiedFrom);
            assert_eq!(derivations[0].source_graph.as_deref(), Some("graph-A"));
        }
        // The id remap pairs each source id with its minted copy's id — the seam
        // the host's facet-carry maps donor facets through (G4-R R0).
        assert_eq!(copy.id_remap.len(), 2);
        for (source_id, new_id) in &copy.id_remap {
            assert_ne!(source_id, new_id, "a copy is a new entity");
            let (_, source_node) = a.get_node_by_id(*source_id).map(|(k, n)| (k, n)).unwrap();
            let (_, new_node) = b.get_node_by_id(*new_id).map(|(k, n)| (k, n)).unwrap();
            assert_eq!(source_node.url(), new_node.url(), "pairs align by content");
            assert_eq!(
                b.node_derivations(b.get_node_key_by_id(*new_id).unwrap())
                    .unwrap()[0]
                    .source_node,
                source_id.to_string()
            );
        }
    }

    #[test]
    fn copy_component_carries_a_resource_relation_whole() {
        use crate::graph::{EdgeAssertion, SemanticSubKind};
        let mut a = Graph::new();
        let k1 = a.add_node("https://r.com/1".to_string(), Point2D::zero());
        let k2 = a.add_node("https://r.com/2".to_string(), Point2D::zero());
        let _k3 = a.add_node("https://r.com/3".to_string(), Point2D::zero()); // disconnected
        a.assert_relation(
            k1,
            k2,
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Hyperlink,
                label: None,
                decay_progress: None,
            },
        );
        // Precondition: the relation lives in the resource stratum only.
        assert_eq!(a.edge_count(), 0, "no surface edge");
        assert_eq!(a.resource_edges().count(), 1, "one resource edge");
        assert_eq!(
            a.weakly_connected_components().len(),
            3,
            "the surface partition cannot see the relation"
        );
        let seed = a.get_node(k1).unwrap().id;

        let mut b = Graph::new();
        let copy = b.copy_component_from(&a, seed, None);

        assert_eq!(
            copy.new_keys.len(),
            2,
            "the resource-joined pair copies whole"
        );
        assert_eq!(b.nodes().count(), 2);
        // Bindings carried: each copy shows its donor's resource (ids retained).
        for (source_id, new_id) in &copy.id_remap {
            let source_key = a.get_node_key_by_id(*source_id).unwrap();
            let new_key = b.get_node_key_by_id(*new_id).unwrap();
            let resource = a.shown_resource_id(source_key).expect("donor binding");
            assert_eq!(b.shown_resource_id(new_key), Some(resource));
            assert_eq!(b.resource_record(resource), a.resource_record(resource));
        }
        // The relation is present in the copy, verbatim, between the new surfaces.
        let donor_edges: Vec<_> = a
            .resource_edges()
            .map(|(f, t, p)| (f.id(), t.id(), p.clone()))
            .collect();
        let copied_edges: Vec<_> = b
            .resource_edges()
            .map(|(f, t, p)| (f.id(), t.id(), p.clone()))
            .collect();
        assert_eq!(
            copied_edges, donor_edges,
            "the resource edge travels verbatim"
        );
        let projected: Vec<_> = b.projected_relations().map(|(_, row)| row).collect();
        assert_eq!(
            projected.len(),
            1,
            "the relation projects between the copies"
        );
        let ids: HashSet<_> = copy.id_remap.iter().map(|(_, new)| *new).collect();
        let from = b.get_node(projected[0].from).unwrap().id;
        let to = b.get_node(projected[0].to).unwrap().id;
        assert!(ids.contains(&from) && ids.contains(&to) && from != to);
        assert_eq!(
            b.component_members(copy.id_remap[0].1, &[]).len(),
            2,
            "the copy is one component in its own graph"
        );
    }
}
