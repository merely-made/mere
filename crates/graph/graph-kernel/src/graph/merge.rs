// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Importing one graph into another as edits: what a codicil import writes
//! into a session's journal.
//!
//! Reservoir plan V2, §7 item 27. An import used to rebuild the graph from the
//! union of two snapshots and swap it in, which a journal cannot record. The
//! same union is written here as ordinary edits:
//! - a node the live graph lacks arrives whole, through the edits undo uses to
//!   bring back a removed node, and with its images;
//! - a node both graphs hold keeps its own fields, and the incoming facets
//!   overwrite its own;
//! - the incoming relations join the live ones between the same two nodes, as
//!   a snapshot load restores a union.
//!
//! The rebuild also gave every node the default-valued facets its empty legacy
//! columns import as. These edits give none: the incoming graph carries its
//! codicil's facet store whole, as a session loads its own. No codicil carries
//! column data, since canonical saves have written the columns empty since
//! before the first codicil schema.

use std::collections::BTreeSet;

use euclid::default::Point2D;
use uuid::Uuid;

use super::Graph;
use super::capture::CapturedDelta;
use super::revert::{Revert, edges_between, node_state};
use crate::persistence::PersistedEdge;

/// The edits that import `incoming` into `live`. `incoming` is a codicil's
/// graph with the codicil's facet store laid in whole, not over what its
/// snapshot's legacy columns import.
pub fn import_edits(live: &Graph, incoming: &Graph) -> Vec<CapturedDelta> {
    let mut edits = Vec::new();
    for (_, node) in incoming.nodes() {
        let (id, node_id) = (node.id, node.id.to_string());
        if live.get_node_by_id(id).is_none() {
            let mut made = Revert::default();
            made.recreate(
                id,
                &node_state(incoming, id).expect("an incoming node has a state"),
            );
            edits.extend(made.edits);
            edits.extend(node.images.iter().map(|(role, image)| {
                CapturedDelta::ReplaySetNodeImageById {
                    node_id: node_id.clone(),
                    role: *role,
                    image: *image,
                }
            }));
            continue;
        }
        let Some(facets) = incoming.facets().facets_of(&id) else {
            continue;
        };
        for (facet, value) in facets.iter() {
            if live.facets().get(&id, facet) != Some(value) {
                edits.push(CapturedDelta::ReplaySetNodeFacetById {
                    node_id: node_id.clone(),
                    facet: facet.as_str().to_string(),
                    value_json: value.to_string(),
                });
            }
        }
    }

    let id_of = |key| incoming.get_node(key).map(|node| node.id);
    let pairs: BTreeSet<(Uuid, Uuid)> = incoming
        .relations()
        .filter_map(|relation| Some((id_of(relation.from)?, id_of(relation.to)?)))
        .collect();
    for (from, to) in pairs {
        let held = edges_between(live, from, to);
        let joined = joined(from, to, &held, &edges_between(incoming, from, to));
        if joined != held {
            edits.push(CapturedDelta::ReplaySetEdgesByIds {
                from_id: from.to_string(),
                to_id: to.to_string(),
                edges: joined,
            });
        }
    }
    edits
}

/// The relations from `from` to `to` once `incoming`'s are restored after
/// `held`, worked out on a scratch pair of nodes.
fn joined(
    from: Uuid,
    to: Uuid,
    held: &[PersistedEdge],
    incoming: &[PersistedEdge],
) -> Vec<PersistedEdge> {
    let mut scratch = Graph::new();
    let origin = Point2D::new(0.0, 0.0);
    let from_key = scratch.add_node_with_id(from, format!("urn:uuid:{from}"), origin);
    let to_key = if to == from {
        from_key
    } else {
        scratch.add_node_with_id(to, format!("urn:uuid:{to}"), origin)
    };
    for edge in held.iter().chain(incoming) {
        scratch.restore_persisted_edge(from_key, to_key, edge);
    }
    scratch.persisted_edges_between(from_key, to_key)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashSet};

    use serde_json::Value;

    use chartulary::AcceptAll;

    use super::*;
    use crate::graph::revert::NodeState;
    use crate::graph::revert::tests::{add, apply_all, facet, id, relate, tag, title};
    use crate::graph::{EdgeAssertion, NavigationTrigger, NodeFacetStore, SemanticSubKind};
    use crate::persistence::GraphSnapshot;
    use crate::types::{ImageRef, ImageRole};

    type Codicil = (GraphSnapshot, NodeFacetStore);

    fn touch(n: u128, at: u64) -> CapturedDelta {
        CapturedDelta::ReplayTouchNodeLastVisitedById {
            node_id: id(n).to_string(),
            timestamp_ms: at,
        }
    }

    fn supports(from: u128, to: u128) -> CapturedDelta {
        CapturedDelta::ReplayAssertRelationByIds {
            from_id: id(from).to_string(),
            to_id: id(to).to_string(),
            assertion: EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Supports,
                label: None,
                decay_progress: None,
            },
        }
    }

    fn traverse(from: u128, to: u128, at: u64) -> CapturedDelta {
        CapturedDelta::ReplayAppendTraversalByIds {
            from_id: id(from).to_string(),
            to_id: id(to).to_string(),
            trigger: NavigationTrigger::LinkClick,
            timestamp_ms: at,
        }
    }

    fn at_url(n: u128, url: &str) -> CapturedDelta {
        CapturedDelta::ReplayAddNodeWithIdIfMissing {
            id: id(n).to_string(),
            url: url.to_string(),
            position: [0.0, 0.0],
        }
    }

    /// `members` cut from `source` as Graphshell's export cuts a codicil.
    fn codicil(source: &Graph, members: &[u128]) -> Codicil {
        let members: HashSet<Uuid> = members.iter().map(|n| id(*n)).collect();
        let ids: HashSet<String> = members.iter().map(Uuid::to_string).collect();
        let mut snapshot = source.to_snapshot();
        snapshot.nodes.retain(|node| ids.contains(&node.node_id));
        snapshot
            .edges
            .retain(|edge| ids.contains(&edge.from_node_id) && ids.contains(&edge.to_node_id));
        snapshot.import_records.clear();
        snapshot.fields.clear();
        snapshot.couplings.clear();
        snapshot.navigation = Default::default();
        let mut facets = NodeFacetStore::new();
        for (node, node_facets) in source.facets().iter() {
            if !members.contains(node) {
                continue;
            }
            for (facet, value) in node_facets.iter() {
                facets
                    .set(*node, facet.clone(), value.clone(), &AcceptAll)
                    .expect("AcceptAll admits every value");
            }
        }
        (snapshot, facets)
    }

    /// The import as Graphshell ran it before sessions: the union of the two
    /// snapshots, rebuilt, with the live facets and then the codicil's laid over.
    fn rebuilt(live: &Graph, (snapshot, facets): &Codicil) -> Graph {
        let current = live.facets().clone();
        let mut merged = live.to_snapshot();
        let mut known: HashSet<String> = merged
            .nodes
            .iter()
            .map(|node| node.node_id.clone())
            .collect();
        for node in &snapshot.nodes {
            if known.insert(node.node_id.clone()) {
                merged.nodes.push(node.clone());
            }
        }
        merged.edges.extend(snapshot.edges.iter().cloned());
        merged.timestamp_secs = snapshot.timestamp_secs;
        let mut graph = Graph::from_snapshot(&merged);
        graph.overlay_facets(current);
        graph.overlay_facets(facets.clone());
        graph
    }

    /// The same import as edits.
    fn edited(live: &Graph, (snapshot, facets): &Codicil) -> (Graph, Vec<CapturedDelta>) {
        let mut incoming = Graph::from_snapshot(snapshot);
        *incoming.facets_mut() = facets.clone();
        let edits = import_edits(live, &incoming);
        let mut graph = live.clone();
        apply_all(&mut graph, &edits);
        (graph, edits)
    }

    type Nodes = BTreeMap<Uuid, (NodeState, Vec<(ImageRole, ImageRef)>)>;
    type Relations = BTreeMap<(Uuid, Uuid), Vec<PersistedEdge>>;

    /// Every node's state and images, every relation by pair, and the
    /// relations as the graph reports them, derived ones included.
    fn whole(graph: &Graph) -> (Nodes, Relations, BTreeSet<String>) {
        let nodes: Nodes = graph
            .nodes()
            .map(|(_, node)| {
                let images = node.images.iter().map(|(role, image)| (*role, *image));
                let state = node_state(graph, node.id).expect("a listed node has a state");
                (node.id, (state, images.collect()))
            })
            .collect();
        let mut relations = Relations::new();
        for from in nodes.keys() {
            for to in nodes.keys() {
                let edges = edges_between(graph, *from, *to);
                if !edges.is_empty() {
                    relations.insert((*from, *to), edges);
                }
            }
        }
        let id_of = |key| graph.get_node(key).map(|node| node.id);
        let listed = graph
            .relations()
            .map(|relation| {
                let ends = (id_of(relation.from), id_of(relation.to));
                format!("{ends:?} {:?}", relation.kind)
            })
            .collect();
        (nodes, relations, listed)
    }

    fn facet_keys(facets: &NodeFacetStore, node: &Uuid) -> BTreeSet<String> {
        facets
            .facets_of(node)
            .map(|facets| {
                facets
                    .iter()
                    .map(|(id, _)| id.as_str().to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Whether a facet value says nothing: what an empty legacy column imports as.
    fn empty(value: &Value) -> bool {
        match value {
            Value::Null => true,
            Value::Bool(value) => !value,
            Value::Array(values) => values.is_empty(),
            Value::Object(fields) => fields.values().all(empty),
            _ => false,
        }
    }

    #[test]
    fn an_import_as_edits_matches_the_rebuilt_union() {
        let mut live = Graph::new();
        apply_all(
            &mut live,
            &[
                add(1),
                title(1, "Live one"),
                facet(1, "custom.a", "1"),
                add(2),
                relate(1, 2),
                touch(1, 1_700_000_000_000),
                touch(2, 1_700_000_001_000),
            ],
        );
        let image = ImageRef::new([7; 32], 16, 16);
        let mut source = Graph::new();
        apply_all(
            &mut source,
            &[
                add(1),
                title(1, "Source one"),
                facet(1, "custom.a", "2"),
                facet(1, "custom.b", "true"),
                add(2),
                at_url(3, "https://site.test/a"),
                title(3, "Three"),
                tag(3, "kept"),
                facet(3, "custom.c", r#"{"x":1}"#),
                at_url(4, "https://site.test/a/b"),
                supports(1, 2),
                relate(3, 4),
                traverse(3, 1, 1_700_000_002_000),
                touch(1, 1_700_000_003_000),
                touch(2, 1_700_000_004_000),
                touch(3, 1_700_000_005_000),
                touch(4, 1_700_000_006_000),
            ],
        );
        apply_all(
            &mut source,
            &[CapturedDelta::ReplaySetNodeImageById {
                node_id: id(3).to_string(),
                role: ImageRole::Favicon,
                image,
            }],
        );
        let codicil = codicil(&source, &[1, 2, 3, 4]);

        let (imported, edits) = edited(&live, &codicil);
        let (mut want_nodes, want_relations, want_listed) = whole(&rebuilt(&live, &codicil));
        // The rebuild's one extra: facets neither graph held, all of them empty.
        let mut artifacts = BTreeSet::new();
        for (node, (state, _)) in &mut want_nodes {
            let mut held = facet_keys(live.facets(), node);
            held.extend(facet_keys(&codicil.1, node));
            state.facets.retain(|facet, value| {
                let artifact = !held.contains(facet);
                if artifact {
                    assert!(empty(value), "{facet} = {value} is not an empty column");
                    artifacts.insert(*node);
                }
                !artifact
            });
        }
        assert!(
            artifacts.contains(&id(1)) && artifacts.contains(&id(3)),
            "the rebuild added empty facets to a live node and an imported one: {artifacts:?}"
        );
        let (nodes, relations, listed) = whole(&imported);

        assert_eq!(nodes, want_nodes);
        assert_eq!(relations, want_relations);
        assert_eq!(listed, want_listed);

        // What the edits did, as a positive control on the comparison.
        let (one, _) = &nodes[&id(1)];
        assert_eq!(
            one.title, "Live one",
            "a node both hold keeps its own fields"
        );
        assert_eq!(one.facets.get("custom.a"), Some(&serde_json::json!(2)));
        assert_eq!(one.facets.get("custom.b"), Some(&serde_json::json!(true)));
        assert_eq!(nodes[&id(3)].1, [(ImageRole::Favicon, image)]);
        assert_eq!(
            relations[&(id(1), id(2))][0]
                .semantic
                .as_ref()
                .map(|semantic| semantic.statements.len()),
            Some(2),
            "the incoming relation joins the live one"
        );
        assert!(
            edits.iter().all(|edit| edit.replay_delta().is_some()),
            "every edit has a replay form, so a session can journal it"
        );
    }

    #[test]
    fn importing_what_the_graph_already_holds_writes_nothing() {
        let mut live = Graph::new();
        apply_all(
            &mut live,
            &[add(1), facet(1, "custom.a", "1"), add(2), relate(1, 2)],
        );
        let (_, edits) = edited(&live, &codicil(&live, &[1, 2]));
        assert!(edits.is_empty(), "{edits:?}");
    }
}
