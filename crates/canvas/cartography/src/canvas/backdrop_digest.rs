// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;

use pictograph::backdrop::{BackdropKind, BackdropNode, ViewDigest};

use super::Canvas;

impl Canvas {
    /// Build a wash digest from the held pause or analytic arrangement and
    /// materialized node facts in the current scope. Held signals contribute
    /// only while their keys are current. No signals or positions are computed
    /// here. A canvas without a held arrangement or pause has no digest.
    pub fn backdrop_digest(&self) -> Option<ViewDigest> {
        let anchors = self
            .paused_positions
            .as_deref()
            .or(self.strategy_positions.as_deref())?;
        let sites = self.channels.sites_fresh(&self.graph);
        let visits = self.channels.visits_fresh(&self.graph);
        let held = self
            .channels
            .community_held()
            .filter(|_| self.channels.community_is_fresh(&self.graph));
        // The producer's c0, c1, ... are local ordinals. Name an unchanged
        // group by its least member UUID so renumbering does not reorder ink.
        let groups: Vec<_> = held
            .into_iter()
            .flat_map(|held| &held.clusters)
            .filter_map(|cluster| {
                let id = cluster
                    .members
                    .iter()
                    .filter_map(|&key| self.graph.get_node(key).map(|node| node.id))
                    .min()?;
                Some((id.to_string(), &cluster.members))
            })
            .collect();
        let mut communities = HashMap::new();
        for (id, members) in &groups {
            for &member in *members {
                communities
                    .entry(member)
                    .and_modify(|previous: &mut &str| {
                        *previous = (*previous).min(id);
                    })
                    .or_insert(id.as_str());
            }
        }

        ViewDigest::from_held(anchors.iter().filter_map(|&(key, point)| {
            if !self.node_in_scope(key) {
                return None;
            }
            let node = self.graph.get_node(key)?;
            Some(BackdropNode {
                position: [point.x, point.y],
                kind: BackdropKind::classify(node.media_type.as_deref(), |tag| {
                    node.tags.contains(tag)
                }),
                community: communities.get(&key).copied(),
                site: sites.and_then(|sites| sites.get(&key)).map(String::as_str),
                visit_seconds: visits.and_then(|visits| visits.get(&key)).copied(),
            })
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::geometry::PortablePoint;
    use kernel::graph::{Graph, fixtures::GraphFixtures};

    #[test]
    fn backdrop_reads_held_positions_and_signals_without_waking_them() {
        let mut graph = Graph::new();
        let a = graph.add_node("https://a.example".into(), PortablePoint::zero());
        let b = graph.add_node("https://b.example".into(), PortablePoint::zero());
        let member = graph.get_node(a).unwrap().id;
        let mut canvas = Canvas::with_graph(graph);
        assert!(canvas.backdrop_digest().is_none());

        canvas.strategy_positions = Some(vec![
            (a, PortablePoint::new(1.0, 2.0)),
            (b, PortablePoint::new(65.0, 2.0)),
        ]);
        let runs = canvas.channels.runs();
        assert!(canvas.backdrop_digest().is_some());
        assert_eq!(canvas.channels.runs(), runs);

        canvas.channels.sites(&canvas.graph);
        canvas.channels.recency(&canvas.graph);
        canvas.channels.community(&canvas.graph);
        let held_runs = canvas.channels.runs();
        let held = canvas.backdrop_digest().unwrap();
        canvas
            .view
            .set_position(a, PortablePoint::new(600.0, 800.0));
        canvas.zoom_at((30.0, 60.0), 1.5);
        assert_eq!(canvas.backdrop_digest(), Some(held.clone()));
        assert_eq!(canvas.channels.runs(), held_runs);

        canvas.scope_to_exact_members([member]);
        assert_ne!(canvas.backdrop_digest(), Some(held));
        assert_eq!(canvas.backdrop_digest().unwrap().bounds(), [-1, -1, 2, 2]);
        assert_eq!(canvas.channels.runs(), held_runs);
    }
}
