// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Graph query + traversal — node / edge lookups, neighbor
//! iteration, BFS / shortest-path / connected-components / SCC, and
//! the derived-edge views (semantic / arrangement / containment).
//!
//! Extracted from `graph/mod.rs` per the 2026-05-11 kernel
//! decomposition pass. These are the read-only and read-mostly
//! methods on `Graph`; the mutating counterparts live in `mod.rs`
//! (node-property setters), `edge_ops.rs` (edge mutators), and
//! `snapshot.rs` (serialization).

use std::collections::{HashMap, HashSet, VecDeque};

use petgraph::Direction;
use petgraph::algo::kosaraju_scc;
use petgraph::graph::DiGraph;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use uuid::Uuid;

use super::identity::{EdgeKey, NodeKey, RelationKey};
use super::node::Node;
use super::snapshot::containment_parent_url;
use super::{ArrangementEdgeView, ContainmentEdgeView, Graph, RelationView, SemanticEdgeView};
use mere::model::edge_payload::EdgePayload;
use mere::model::edge_taxonomy::{ContainmentSubKind, EdgeAssertion, RelationSelector};

/// Expand one pair-local edge bucket into held [`RelationView`] rows: one
/// per held semantic statement (`Semantic(sub_kind)` when recognized, else
/// `OpenPredicate`), one for traversal-sidecar presence, and one per
/// sub-kind in each of the remaining families. Shared by every relation
/// iterator so per-node and whole-graph reads agree row for row.
///
/// Row families agree with [`EdgePayload::families`] for every edge
/// shape (Q1 ruling, 2026-09-12; pinned by `row_family_parity_tests`).
fn relation_rows(from: NodeKey, to: NodeKey, payload: &EdgePayload) -> Vec<RelationView> {
    use mere::model::edge_taxonomy::RelationKind;
    let mut out: Vec<RelationView> = Vec::new();
    for statement in payload.held_assertions() {
        out.push(RelationView {
            from,
            to,
            kind: statement.relation_kind(),
        });
    }
    // Traversal is event-shaped, no sub-kind. Presence of the
    // traversal sidecar yields one row, matching
    // `has_family(Traversal)`; an event-free sidecar (reachable
    // only by deserializing such a snapshot) still counts.
    // Family-aware view callers reach for the typed payload for
    // actual events.
    if payload.traversal_data().is_some() {
        out.push(RelationView {
            from,
            to,
            kind: RelationKind::Traversal,
        });
    }
    out
}

#[cfg(test)]
#[path = "query/projected_relation_tests.rs"]
mod projected_relation_tests;

impl Graph {
    /// Get a node by key
    pub fn get_node(&self, key: NodeKey) -> Option<&Node> {
        self.inner.node(key)
    }

    /// Get a mutable node by key.
    pub(crate) fn get_node_mut(&mut self, key: NodeKey) -> Option<&mut Node> {
        self.inner.node_mut(key)
    }

    /// Find a node whose AddressClaims include the given address (any
    /// `AddressRole`). Returns an arbitrary matching node when multiple
    /// nodes share the address — the caller is expected to handle
    /// duplicates explicitly when relevant.
    ///
    /// Canonical address lookup per the [node identity + duplicates
    /// brief](https://github.com/merely-made/mere/blob/main/design_docs/mere_docs/research/2026-05-18_node_identity_and_duplicates_brief.md);
    /// supersedes the URL-string-keyed `get_node_by_url` (which is
    /// retained as a transitional shim — see below).
    pub fn find_node_by_address(
        &self,
        address: &crate::address::Address,
    ) -> Option<(NodeKey, &Node)> {
        let url = address.as_url_str();
        let key = self.url_to_nodes.get(url)?.last().copied()?;
        Some((key, self.inner.node(key)?))
    }

    /// Convenience wrapper over [`Graph::find_node_by_address`] for
    /// callers that have a URL string rather than a typed [`Address`].
    /// Builds the [`Address`] internally and delegates.
    ///
    /// Use this for tests and quick lookups; prefer
    /// [`Graph::find_node_by_address`] in code paths that already have a
    /// typed [`Address`] in hand.
    pub fn get_node_by_url(&self, url: &str) -> Option<(NodeKey, &Node)> {
        let address = crate::address::address_from_url(url);
        self.find_node_by_address(&address)
    }

    /// Get all node keys currently mapped to a URL.
    pub fn get_nodes_by_url(&self, url: &str) -> Vec<NodeKey> {
        self.url_to_nodes.get(url).cloned().unwrap_or_default()
    }

    /// Get a node by UUID.
    pub fn get_node_by_id(&self, id: Uuid) -> Option<(NodeKey, &Node)> {
        let key = self.inner.key_of(&id)?;
        Some((key, self.inner.node(key)?))
    }

    /// Get node key by UUID.
    pub fn get_node_key_by_id(&self, id: Uuid) -> Option<NodeKey> {
        self.inner.key_of(&id)
    }

    /// Iterate over all nodes as (key, node) pairs
    pub fn nodes(&self) -> impl Iterator<Item = (NodeKey, &Node)> {
        self.inner.nodes()
    }

    /// Iterate over all stored relations, one row per recognized
    /// statement/family carried by the pair-local edge bucket.
    /// One `EdgePayload` between two nodes may therefore yield
    /// multiple semantic rows.
    ///
    /// Canonical read surface per the 2026-05-11 relation-taxonomy
    /// plan §2. Stage 4 removed the legacy `EdgeType`-flavoured
    /// `edges()` iterator that this replaces.
    pub fn relations(&self) -> impl Iterator<Item = RelationView> + '_ {
        self.inner
            .inner()
            .edge_references()
            .flat_map(|edge| relation_rows(edge.source(), edge.target(), edge.weight()))
    }

    /// Retained Surface buckets without expanding statements or lifting
    /// Resource relations. Each handle names its exact bucket, including
    /// parallel pairs. This is a raw retained-history view; use held relation
    /// iterators for ordinary reads. Tombstone-only buckets remain included.
    pub fn surface_relation_buckets(
        &self,
    ) -> impl Iterator<Item = (EdgeKey, NodeKey, NodeKey, &EdgePayload)> + '_ {
        self.inner
            .inner()
            .edge_references()
            .map(|edge| (edge.id(), edge.source(), edge.target(), edge.weight()))
    }

    /// Held Surface buckets outgoing from `from`, including every parallel
    /// bucket without expanding its assertions. Cost is proportional to the
    /// node's out-degree.
    pub fn surface_outgoing_relation_buckets(
        &self,
        from: NodeKey,
    ) -> impl Iterator<Item = (EdgeKey, NodeKey, NodeKey, &EdgePayload)> + '_ {
        self.inner
            .inner()
            .edges_directed(from, Direction::Outgoing)
            .map(|edge| (edge.id(), edge.source(), edge.target(), edge.weight()))
    }

    /// Surface relations followed by content relations lifted to every pair of
    /// surfaces showing their endpoints. The handle retains its owning store;
    /// projection does not copy resource assertions into the surface graph.
    pub fn projected_relations(&self) -> impl Iterator<Item = (RelationKey, RelationView)> + '_ {
        let surfaces = self.inner.inner().edge_references().flat_map(|edge| {
            let key = RelationKey::Surface(edge.id());
            relation_rows(edge.source(), edge.target(), edge.weight())
                .into_iter()
                .map(move |row| (key, row))
        });
        let resources = self
            .resource_relations()
            .flat_map(|(key, from, to, payload)| {
                let sources = self.surface_ids_showing_resource(from);
                let targets = self.surface_ids_showing_resource(to);
                let mut rows = Vec::new();
                for source in sources {
                    let Some(source) = self.get_node_key_by_id(source) else {
                        continue;
                    };
                    for target in &targets {
                        let Some(target) = self.get_node_key_by_id(*target) else {
                            continue;
                        };
                        rows.extend(
                            relation_rows(source, target, payload)
                                .into_iter()
                                .map(|row| (RelationKey::Resource(key), row)),
                        );
                    }
                }
                rows
            });
        surfaces.chain(resources)
    }

    /// Every exact held assertion lifted to its displayed endpoint pairs.
    /// Traversal remains an event rollup and is not an assertion record.
    pub fn projected_assertions(
        &self,
    ) -> impl Iterator<Item = (RelationKey, NodeKey, NodeKey, &super::SemanticStatement)> + '_ {
        let surfaces = self.inner.inner().edge_references().flat_map(|edge| {
            let (key, from, to) = (
                RelationKey::Surface(edge.id()),
                edge.source(),
                edge.target(),
            );
            edge.weight()
                .held_assertions()
                .map(move |record| (key, from, to, record))
        });
        let resources =
            self.resource_relations()
                .flat_map(|(key, from, to, payload)| {
                    let mut rows = Vec::new();
                    for source in self.surface_ids_showing_resource(from) {
                        let Some(source) = self.get_node_key_by_id(source) else {
                            continue;
                        };
                        for target in self.surface_ids_showing_resource(to) {
                            let Some(target) = self.get_node_key_by_id(target) else {
                                continue;
                            };
                            rows.extend(payload.held_assertions().map(|record| {
                                (RelationKey::Resource(key), source, target, record)
                            }));
                        }
                    }
                    rows
                });
        surfaces.chain(resources)
    }

    /// Relations whose `from` endpoint is `key`, expanded with exactly the
    /// row semantics of [`Self::relations`] but visiting only that node's
    /// outgoing edges. Cost is proportional to the node's out-degree, not
    /// the graph's edge count.
    pub fn outgoing_relations(&self, key: NodeKey) -> impl Iterator<Item = RelationView> + '_ {
        self.inner
            .inner()
            .edges_directed(key, Direction::Outgoing)
            .flat_map(|edge| relation_rows(edge.source(), edge.target(), edge.weight()))
    }

    /// Relations whose `to` endpoint is `key`, expanded with exactly the
    /// row semantics of [`Self::relations`] but visiting only that node's
    /// incoming edges. Cost is proportional to the node's in-degree, not
    /// the graph's edge count.
    pub fn incoming_relations(&self, key: NodeKey) -> impl Iterator<Item = RelationView> + '_ {
        self.inner
            .inner()
            .edges_directed(key, Direction::Incoming)
            .flat_map(|edge| relation_rows(edge.source(), edge.target(), edge.weight()))
    }

    pub fn semantic_edges(&self) -> impl Iterator<Item = SemanticEdgeView> + '_ {
        self.inner.inner().edge_references().flat_map(|edge| {
            let from = edge.source();
            let to = edge.target();
            let payload = edge.weight();
            payload
                .held_semantic_statements()
                .filter_map(|statement| {
                    statement
                        .recognized_sub_kind
                        .map(|sub_kind| SemanticEdgeView {
                            from,
                            to,
                            sub_kind,
                            label: statement.label.clone(),
                        })
                })
                .collect::<Vec<_>>()
                .into_iter()
        })
    }

    pub fn arrangement_edges(&self) -> impl Iterator<Item = ArrangementEdgeView> + '_ {
        self.inner.inner().edge_references().flat_map(|e| {
            let from = e.source();
            let to = e.target();
            e.weight()
                .arrangement_data()
                .map(|data| {
                    data.sub_kinds
                        .into_iter()
                        .map(move |sub_kind| ArrangementEdgeView { from, to, sub_kind })
                })
                .into_iter()
                .flatten()
        })
    }

    pub fn containment_edges(&self) -> impl Iterator<Item = ContainmentEdgeView> + '_ {
        self.inner.inner().edge_references().flat_map(|e| {
            let from = e.source();
            let to = e.target();
            e.weight()
                .containment_data()
                .map(|data| {
                    data.sub_kinds
                        .into_iter()
                        .map(move |sub_kind| ContainmentEdgeView { from, to, sub_kind })
                })
                .into_iter()
                .flatten()
        })
    }

    /// Derive the containment this one node's address implies, at the moment
    /// the node is created.
    ///
    /// Current relation routing stores this assertion on Resources. Ordinary
    /// snapshot loading separately rebuilds compatibility Surface containment;
    /// recorded loading preserves stored relations without that rebuild. Both
    /// strata are included in snapshots.
    ///
    /// The **UrlPath half** of [`rebuild_derived_containment_relations`],
    /// applied to a single node. It exists because that rebuild runs only on
    /// snapshot load, which used to mean a node visited under a folder had no
    /// containment edge until the session was saved and reloaded. Anything
    /// reading containment live (a graph behavior's watch, which names a
    /// region by its container) saw nothing there. Deriving at creation makes
    /// the local relation available before the first save.
    ///
    /// The **Domain half is deliberately not here.** A domain anchor is the
    /// shallowest node on a host, so minting one node can re-anchor every
    /// other node sharing that host: a whole-graph fact, which stays with the
    /// whole-graph rebuild. This one is purely local, which is what makes it
    /// cheap enough to run per node.
    pub fn derive_containment_for(&mut self, key: NodeKey) {
        let Some(node) = self.get_node(key) else {
            return;
        };
        let Ok(parsed) = url::Url::parse(node.url()) else {
            return;
        };
        let Some(parent_url) = containment_parent_url(&parsed) else {
            return;
        };
        let Some((parent_key, _)) = self.get_node_by_url(&parent_url) else {
            return;
        };
        if parent_key == key {
            return;
        }
        // Child to container, the direction the rebuild uses.
        let _ = self.assert_relation(
            key,
            parent_key,
            EdgeAssertion::Containment {
                sub_kind: ContainmentSubKind::UrlPath,
            },
        );
    }

    pub(crate) fn rebuild_derived_containment_relations(&mut self) {
        let edge_ids: Vec<EdgeKey> = self.inner.inner().edge_indices().collect();
        let mut empty_edges = Vec::new();
        // This rebuild replaces only held, positively rule-marked containment
        // rows on `inner` (bypassing the bumping API), so track the structural
        // change and bump once. The `assert_relation` calls below bump on their
        // own when they re-derive the edges.
        let mut changed = false;
        for edge_id in edge_ids {
            if let Some(payload) = self.get_edge_mut(edge_id) {
                let before = payload.assertions().len();
                payload.assertions.statements.retain(|statement| {
                    !(statement.is_held()
                        && is_derived_containment_record(statement)
                        && matches!(
                            statement.relation_kind(),
                            super::RelationKind::Containment(
                                ContainmentSubKind::UrlPath | ContainmentSubKind::Domain
                            )
                        ))
                });
                let removed_any = payload.assertions().len() != before;
                if removed_any {
                    payload.assertions.rebuild_compat();
                    changed = true;
                }
                if removed_any && payload.is_empty() {
                    empty_edges.push(edge_id);
                }
            }
        }
        for edge_id in empty_edges {
            let _ = self.inner.disconnect(edge_id);
        }
        if changed {
            self.bump_revision();
        }

        // Resource assertions keep their recorded owner and metadata. The
        // compatibility rebuild below retains its prior Surface-only behavior.

        let mut domain_anchor: HashMap<String, (NodeKey, usize, Uuid)> = HashMap::new();
        for (key, node) in self.nodes() {
            let Ok(parsed) = url::Url::parse(node.url()) else {
                continue;
            };
            let depth = parsed.path_segments().map_or(0, |segments| {
                segments.filter(|segment| !segment.is_empty()).count()
            });

            let Some(host) = parsed.host_str() else {
                continue;
            };
            let host = host.to_ascii_lowercase();
            let candidate = (key, depth, node.id);
            domain_anchor
                .entry(host)
                .and_modify(|current| {
                    if candidate.1 < current.1
                        || (candidate.1 == current.1 && candidate.2 < current.2)
                    {
                        *current = candidate;
                    }
                })
                .or_insert(candidate);
        }

        let mut url_parent_edges = Vec::new();
        let mut domain_edges = Vec::new();
        for (key, node) in self.nodes() {
            let Ok(parsed) = url::Url::parse(node.url()) else {
                continue;
            };

            if let Some(parent_url) = containment_parent_url(&parsed)
                && let Some((parent_key, _)) = self.get_node_by_url(&parent_url)
                && parent_key != key
            {
                url_parent_edges.push((key, parent_key));
            }

            if let Some(host) = parsed.host_str() {
                let host = host.to_ascii_lowercase();
                if let Some((anchor_key, _, _)) = domain_anchor.get(&host)
                    && *anchor_key != key
                {
                    domain_edges.push((key, *anchor_key));
                }
            }
        }

        for (from, to) in url_parent_edges {
            let _ = self.assert_surface_relation_as(
                from,
                to,
                EdgeAssertion::Containment {
                    sub_kind: ContainmentSubKind::UrlPath,
                },
                self.write_author().asserter_iri(),
                None,
            );
        }
        for (from, to) in domain_edges {
            let _ = self.assert_surface_relation_as(
                from,
                to,
                EdgeAssertion::Containment {
                    sub_kind: ContainmentSubKind::Domain,
                },
                self.write_author().asserter_iri(),
                None,
            );
        }
    }

    /// Check if a directed edge exists from `from` to `to`
    pub fn has_edge_between(&self, from: NodeKey, to: NodeKey) -> bool {
        self.inner
            .inner()
            .edges_connecting(from, to)
            .any(|edge| edge.weight().has_held_topology())
    }

    /// Count of nodes in the graph
    pub fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    /// Count of retained Surface buckets, including tombstone-only history.
    /// Use held topology queries when asking how many relations are active.
    pub fn edge_count(&self) -> usize {
        self.inner.edge_count()
    }
}

/// Only rows positively identified as outputs of the containment rule may be
/// replaced by a current derivation. Same-predicate authored/imported rows
/// remain retained graph facts.
pub(crate) fn is_derived_containment_record(statement: &super::SemanticStatement) -> bool {
    statement.basis.method.as_ref().is_some_and(|method| {
        method.iri == "urn:mere:method:derived-containment"
            && method.version.as_deref() == Some("1")
    })
}

#[cfg(test)]
#[path = "query/derivation_tests.rs"]
mod derivation_tests;

/// Q1 (ruled 2026-09-12): `has_family` / `families()` and the row set
/// from `relation_rows` agree for every family on every edge shape,
/// including the two that used to diverge (open-predicate-only Semantic,
/// event-free Traversal sidecar).
#[cfg(test)]
#[path = "query/row_family_parity_tests.rs"]
mod row_family_parity_tests;

#[path = "query/traversal.rs"]
mod traversal;
