// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The three memory levels' read-model: classifying a node as short-term or
//! long-term, and computing which short-term nodes an eviction policy would drop.
//!
//! Moved back from Alembic 2026-10-31 (distillery-extraction D12). Since engine
//! split D45 this is the graph adapter: the levels, the policy and the census
//! are `eidetic::memory::levels`, generic over node records; this module reads
//! a mere's graph into them. This is the spine of Alembic slice C
//! (the memory model). It is pure logic over
//! a graph's persisted nodes plus per-node last-visit timing the host supplies, so
//! it is fully testable without a live graph, store, or clock. The host wiring (the
//! Alembic Recent / Saved sections rendering over this, the policy as a setting, the
//! actual eviction pass) layers on top once it lands.
//!
//! The model (Alembic plan, decision #2, confirmed with Mark): **a tag
//! promotes a node to long-term**, retained and never evicted. A physics position
//! pin is independent of retention. Everything else is
//! short-term working memory, subject to the eviction policy. Codicils (slice A) are
//! the third level and are not nodes, so they are out of scope here.

use std::collections::HashMap;

use kernel::graph::{Graph, NodeKey};

use eidetic::memory::levels;
pub use eidetic::memory::levels::{EvictionPolicy, MemoryCensus, MemoryLevel, MemoryRecord};

/// One graph node as the memory program sees it.
struct GraphNodeMemory<'a> {
    graph: &'a Graph,
    key: NodeKey,
}

impl MemoryRecord for GraphNodeMemory<'_> {
    fn record_id(&self) -> String {
        self.graph
            .get_node(self.key)
            .map(|node| node.id.to_string())
            .unwrap_or_default()
    }

    fn promoted(&self) -> bool {
        is_promoted(self.graph, self.key)
    }

    fn last_session_visited(&self) -> u64 {
        self.graph
            .node_last_session_visited(self.key)
            .unwrap_or_default()
    }
}

fn records(graph: &Graph) -> impl Iterator<Item = GraphNodeMemory<'_>> {
    graph
        .nodes()
        .map(move |(key, _)| GraphNodeMemory { graph, key })
}

/// Whether a node has been promoted to long-term. Promotion is an affirmative act:
/// at least one tag (decision #2 — "tagging adds to long-term, retained"). `is_pinned`
/// is deliberately *not* a promotion signal: it pins a node's physics position, an
/// orthogonal spatial choice, not a memory-keep. The graph's content-tag projection
/// reads canonical resource tags and supports unbound legacy nodes.
pub fn is_promoted(graph: &Graph, key: NodeKey) -> bool {
    graph
        .node_content_tags(key)
        .is_some_and(|tags| !tags.is_empty())
}

/// The memory level of a node.
pub fn level_of(graph: &Graph, key: NodeKey) -> MemoryLevel {
    levels::level_of(&GraphNodeMemory { graph, key })
}

/// The node ids of the short-term nodes `policy` would evict as of `now_ms` /
/// `current_session`, given each node's last-visit time (`last_visit_ms`, keyed by
/// `node_id`, supplied by the host from the graph's navigation history) for the
/// by-time policies, or the node's own `last_session_visited` stamp for
/// [`KeepSessions`](EvictionPolicy::KeepSessions). See
/// [`levels::evictable_short_term`].
pub fn evictable_short_term(
    graph: &Graph,
    last_visit_ms: &HashMap<String, u64>,
    policy: EvictionPolicy,
    now_ms: u64,
    current_session: u64,
) -> Vec<String> {
    levels::evictable_short_term(
        records(graph),
        last_visit_ms,
        policy,
        now_ms,
        current_session,
    )
}

/// Count a graph's memory levels and how many the `policy` would evict now.
pub fn census(
    graph: &Graph,
    last_visit_ms: &HashMap<String, u64>,
    policy: EvictionPolicy,
    now_ms: u64,
    current_session: u64,
) -> MemoryCensus {
    levels::census(
        records(graph),
        last_visit_ms,
        policy,
        now_ms,
        current_session,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use euclid::default::Point2D;
    use kernel::graph::fixtures::GraphFixtures;
    use kernel::graph::node_facets::{ARRANGEMENT_PIN, VISIT_HISTORY, VisitHistoryFacet};
    use uuid::Uuid;

    const DAY_MS: u64 = 86_400_000;

    fn node(graph: &mut Graph, id: &str) -> NodeKey {
        graph.add_node_with_id(
            Uuid::new_v5(&Uuid::NAMESPACE_URL, id.as_bytes()),
            format!("https://{id}.example"),
            Point2D::zero(),
        )
    }

    fn set_session(graph: &mut Graph, key: NodeKey, session: u64) {
        let id = graph.get_node(key).unwrap().id;
        graph
            .facets_mut()
            .set(
                id,
                portolan::FacetId::new(VISIT_HISTORY),
                serde_json::to_value(VisitHistoryFacet {
                    last_visited_ms: None,
                    last_session_visited: session,
                })
                .unwrap(),
                &portolan::AcceptAll,
            )
            .unwrap();
    }

    #[test]
    fn a_tag_promotes_to_long_term_but_a_position_pin_does_not() {
        let mut graph = Graph::new();
        let plain = node(&mut graph, "plain");
        let tagged = node(&mut graph, "tagged");
        let pinned = node(&mut graph, "pinned");
        graph.insert_node_tag(tagged, "keep".to_string());
        let pinned_id = graph.get_node(pinned).unwrap().id;
        graph
            .facets_mut()
            .set(
                pinned_id,
                portolan::FacetId::new(ARRANGEMENT_PIN),
                serde_json::json!(true),
                &portolan::AcceptAll,
            )
            .unwrap();

        assert_eq!(level_of(&graph, plain), MemoryLevel::ShortTerm);
        assert_eq!(level_of(&graph, tagged), MemoryLevel::LongTerm);
        assert_eq!(
            level_of(&graph, pinned),
            MemoryLevel::ShortTerm,
            "a position-pin is orthogonal to memory level",
        );
    }

    #[test]
    fn keep_forever_evicts_nothing() {
        let mut graph = Graph::new();
        node(&mut graph, "a");
        node(&mut graph, "b");
        let mut times = HashMap::new();
        times.insert("a".to_string(), 0u64); // ancient, but policy keeps all
        let out =
            evictable_short_term(&graph, &times, EvictionPolicy::KeepForever, 100 * DAY_MS, 0);
        assert!(out.is_empty(), "KeepForever never evicts");
    }

    #[test]
    fn keep_days_evicts_only_stale_undated_safe_and_promoted_exempt() {
        let now = 100 * DAY_MS;
        let mut graph = Graph::new();
        let stale = node(&mut graph, "stale");
        let fresh = node(&mut graph, "fresh");
        node(&mut graph, "undated");
        let promoted = node(&mut graph, "promoted");
        graph.insert_node_tag(promoted, "keep".to_string());

        let mut times = HashMap::new();
        times.insert(
            graph.get_node(stale).unwrap().id.to_string(),
            now - 40 * DAY_MS,
        );
        times.insert(
            graph.get_node(fresh).unwrap().id.to_string(),
            now - 5 * DAY_MS,
        );
        times.insert(
            graph.get_node(promoted).unwrap().id.to_string(),
            now - 90 * DAY_MS,
        );
        // "undated" has no visit record -> never dropped (we don't drop what we can't date)

        let out = evictable_short_term(&graph, &times, EvictionPolicy::KeepDays(30), now, 0);
        assert_eq!(
            out,
            vec![graph.get_node(stale).unwrap().id.to_string()],
            "only the dated, stale, un-promoted node"
        );
    }

    #[test]
    fn keep_sessions_evicts_only_stale_unstamped_safe_and_promoted_exempt() {
        let current_session = 10u64;
        let mut graph = Graph::new();
        let stale = node(&mut graph, "stale");
        let fresh = node(&mut graph, "fresh");
        node(&mut graph, "unstamped");
        let promoted = node(&mut graph, "promoted");
        set_session(&mut graph, stale, 2);
        set_session(&mut graph, fresh, 9);
        set_session(&mut graph, promoted, 1);
        graph.insert_node_tag(promoted, "keep".to_string());

        let out = evictable_short_term(
            &graph,
            &HashMap::new(),
            EvictionPolicy::KeepSessions(3),
            0,
            current_session,
        );
        assert_eq!(
            out,
            vec![graph.get_node(stale).unwrap().id.to_string()],
            "only the dated, stale, un-promoted node"
        );
    }

    #[test]
    fn keep_sessions_evicts_by_the_launch_stamp() {
        use kernel::graph::apply::{GraphDelta, apply_graph_delta};

        let mut graph = Graph::new();
        let visited = node(&mut graph, "visited");
        graph.set_current_session(5);
        apply_graph_delta(
            &mut graph,
            GraphDelta::NavigateNode {
                key: visited,
                url: "https://visited.example/next".into(),
            },
        );
        assert_eq!(graph.node_last_session_visited(visited), Some(5));

        let id = graph.get_node(visited).unwrap().id.to_string();
        let evicted = |sessions| {
            let policy = EvictionPolicy::KeepSessions(sessions);
            evictable_short_term(&graph, &HashMap::new(), policy, 0, 7)
        };
        assert_eq!(
            evicted(2),
            vec![id],
            "two launches on is out of a window of two"
        );
        assert!(
            evicted(3).is_empty(),
            "two launches on is inside a window of three"
        );
    }

    #[test]
    fn census_counts_levels_and_evictable() {
        let now = 100 * DAY_MS;
        let mut graph = Graph::new();
        let s1 = node(&mut graph, "s1");
        let s2 = node(&mut graph, "s2");
        let kept = node(&mut graph, "kept");
        graph.insert_node_tag(kept, "keep".to_string());

        let mut times = HashMap::new();
        times.insert(
            graph.get_node(s1).unwrap().id.to_string(),
            now - 60 * DAY_MS,
        );
        times.insert(graph.get_node(s2).unwrap().id.to_string(), now - 2 * DAY_MS);

        let c = census(&graph, &times, EvictionPolicy::KeepDays(30), now, 0);
        assert_eq!(c.short_term, 2);
        assert_eq!(c.long_term, 1);
        assert_eq!(c.evictable, 1, "only s1 is stale enough");
    }
}
