// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The graph grammar scenes' axes, read from the graph (Sets, Provenance).
//! The host computes the axis; cartography arranges by it.

use std::collections::{BTreeSet, HashMap};

use cartography::AxisValue;
use kernel::graph::{Graph, NodeKey, RelationKind};

/// The Provenance column for sources.
pub const SOURCES_COLUMN: &str = "sources";
/// The column for nodes in no derivation at all.
pub const UNRELATED_COLUMN: &str = "unrelated";

/// Each tagged node's tags as a Sets membership (`a|b`), sorted.
pub fn tag_sets(graph: &Graph) -> HashMap<NodeKey, AxisValue> {
    graph
        .nodes()
        .filter_map(|(key, _)| {
            // A visible occurrence reads the tags of its shown Resource.
            // The kernel retains raw Surface tags only as a legacy fallback.
            let tags: BTreeSet<String> = graph.node_content_tags(key)?.into_iter().collect();
            (!tags.is_empty()).then(|| {
                let joined = tags.into_iter().collect::<Vec<_>>().join("|");
                (key, AxisValue::Categorical(joined))
            })
        })
        .collect()
}

/// The derivation column a node's depth names.
pub fn derived_column(depth: usize) -> String {
    match depth {
        0 => SOURCES_COLUMN.into(),
        n => format!("derived · {n}"),
    }
}

/// Provenance columns: a node's depth is the longest chain of provenance
/// links (derived → source) beneath it, so sources stand left and each
/// column is one more step of derivation. Nodes in no provenance link go to
/// [`UNRELATED_COLUMN`]. A cycle stops counting where it closes. Returns the
/// axis and the column order.
pub fn derivation_columns(graph: &Graph) -> (HashMap<NodeKey, AxisValue>, Vec<String>) {
    let mut sources_of: HashMap<NodeKey, Vec<NodeKey>> = HashMap::new();
    let mut involved: BTreeSet<NodeKey> = BTreeSet::new();
    // Provenance lives in the resource stratum; the projection shows it between
    // the displayed nodes.
    for (_, relation) in graph.projected_relations() {
        if matches!(relation.kind, RelationKind::Provenance(_)) && relation.from != relation.to {
            sources_of
                .entry(relation.from)
                .or_default()
                .push(relation.to);
            involved.insert(relation.from);
            involved.insert(relation.to);
        }
    }
    let mut depth: HashMap<NodeKey, usize> = HashMap::new();
    for &key in &involved {
        depth_of(key, &sources_of, &mut depth, &mut BTreeSet::new());
    }
    let deepest = depth.values().copied().max().unwrap_or(0);
    let mut order: Vec<String> = (0..=deepest).map(derived_column).collect();
    order.push(UNRELATED_COLUMN.into());
    let axis = graph
        .nodes()
        .map(|(key, _)| {
            let column = depth
                .get(&key)
                .map_or_else(|| UNRELATED_COLUMN.into(), |d| derived_column(*d));
            (key, AxisValue::Categorical(column))
        })
        .collect();
    (axis, order)
}

fn depth_of(
    key: NodeKey,
    sources_of: &HashMap<NodeKey, Vec<NodeKey>>,
    memo: &mut HashMap<NodeKey, usize>,
    visiting: &mut BTreeSet<NodeKey>,
) -> usize {
    if let Some(depth) = memo.get(&key) {
        return *depth;
    }
    if !visiting.insert(key) {
        return 0;
    }
    let depth = sources_of
        .get(&key)
        .into_iter()
        .flatten()
        .map(|source| depth_of(*source, sources_of, memo, visiting) + 1)
        .max()
        .unwrap_or(0);
    visiting.remove(&key);
    memo.insert(key, depth);
    depth
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::geometry::PortablePoint;
    use kernel::graph::fixtures::GraphFixtures;
    use kernel::graph::{EdgeAssertion, ProvenanceSubKind};

    fn node(graph: &mut Graph, name: &str) -> NodeKey {
        graph.add_node(format!("https://{name}.example"), PortablePoint::zero())
    }

    #[test]
    fn tag_sets_read_public_resource_tag_assertions_instead_of_surface_fallbacks() {
        use kernel::graph::apply::{GraphDelta, apply_graph_delta};
        let mut graph = Graph::new();
        let tagged = node(&mut graph, "tagged");
        let untagged = node(&mut graph, "untagged");
        for tag in ["song", "practice"] {
            apply_graph_delta(
                &mut graph,
                GraphDelta::InsertNodeTag {
                    key: tagged,
                    tag: tag.into(),
                },
            );
        }
        assert!(
            graph.get_node(tagged).unwrap().tags.is_empty(),
            "public tagging stores Resource assertions, not Surface fallback tags"
        );
        assert!(graph.node_content_tags(tagged).unwrap().contains("song"));
        let axis = tag_sets(&graph);
        assert_eq!(
            axis.get(&tagged),
            Some(&AxisValue::Categorical("practice|song".into()))
        );
        assert!(!axis.contains_key(&untagged));
    }

    #[test]
    fn sources_stand_left_and_each_derivation_adds_a_column() {
        let mut graph = Graph::new();
        let [search, tab, progression, lyrics, lone] =
            ["search", "tab", "progression", "lyrics", "lone"].map(|n| node(&mut graph, n));
        let derive = |graph: &mut Graph, from, to| {
            graph.assert_relation(
                from,
                to,
                EdgeAssertion::Provenance {
                    sub_kind: ProvenanceSubKind::ExtractedFrom,
                },
            );
        };
        derive(&mut graph, tab, search);
        derive(&mut graph, progression, tab);
        derive(&mut graph, lyrics, progression);
        derive(&mut graph, lyrics, search);
        let (axis, order) = derivation_columns(&graph);
        let column = |key| match &axis[&key] {
            AxisValue::Categorical(column) => column.clone(),
            other => panic!("{other:?}"),
        };
        assert_eq!(column(search), SOURCES_COLUMN);
        assert_eq!(column(tab), "derived · 1");
        assert_eq!(column(lyrics), "derived · 3", "the longest chain counts");
        assert_eq!(column(lone), UNRELATED_COLUMN);
        assert_eq!(order.first().unwrap(), SOURCES_COLUMN);
        assert_eq!(order.last().unwrap(), UNRELATED_COLUMN);
        assert_eq!(order.len(), 5);
    }

    #[test]
    fn tags_become_sorted_memberships() {
        let mut graph = Graph::new();
        let a = node(&mut graph, "a");
        let _untagged = node(&mut graph, "b");
        graph
            .get_node_mut(a)
            .unwrap()
            .tags
            .extend(["song".into(), "practice".into()]);
        let sets = tag_sets(&graph);
        assert_eq!(sets.len(), 1);
        assert_eq!(sets[&a], AxisValue::Categorical("practice|song".into()));
    }
}
