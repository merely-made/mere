// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Projection: the output handed to canvas swatches.

use kernel::geometry::{PortablePoint, PortableRect};
use kernel::graph::{CoverageLayer, CoverageLimit, CoverageNote, EdgeKey, Graph, NodeKey};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::minimap::MinimapDescriptor;
use crate::overlay::Overlay;

/// Output of a strategy projection.
///
/// Carries everything a canvas swatch needs to render the result:
/// positioned nodes, positioned edges, overlay vocabulary, and an
/// optional minimap descriptor.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Projection {
    pub nodes: Vec<PositionedNode>,
    pub edges: Vec<PositionedEdge>,
    pub overlays: Vec<Overlay>,
    pub minimap: Option<MinimapDescriptor>,
    /// Total content bounds for this projection. May exceed the
    /// requested `TargetSize` (canvas decides scroll/zoom strategy).
    pub content_bounds: PortableRect,
    /// Strategy-specific metadata. Free-form to keep the contract
    /// open to strategies that need to communicate state out-of-band.
    pub metadata: ProjectionMetadata,
}

impl Projection {
    /// An empty projection — what strategies should return for an
    /// empty graph or fully-filtered-out input.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Refresh runtime coverage from the actual output, independently of geometry caches.
    pub fn with_graph_coverage(mut self, graph: &Graph) -> Self {
        let mut coverage = graph.coverage_note();
        let nodes: HashSet<_> = self.nodes.iter().map(|node| node.node).collect();
        let resources: HashSet<_> = nodes
            .iter()
            .filter_map(|&node| graph.shown_resource_id(node))
            .collect();
        coverage.add_count(
            CoverageLayer::Projection,
            "Surfaces omitted from projection",
            graph
                .nodes()
                .filter(|(key, _)| !nodes.contains(key))
                .count(),
        );
        coverage.add_count(
            CoverageLayer::Projection,
            "held Resources without a projected appearance",
            graph
                .resource_nodes()
                .filter(|resource| !resources.contains(&resource.id()))
                .count(),
        );
        let pair = |a, b| if a <= b { (a, b) } else { (b, a) };
        let represented: HashSet<_> = self
            .edges
            .iter()
            .filter(|edge| nodes.contains(&edge.from) && nodes.contains(&edge.to))
            .map(|edge| pair(edge.from, edge.to))
            .collect();
        let held: HashSet<_> = graph
            .projected_relations()
            .map(|(_, relation)| pair(relation.from, relation.to))
            .collect();
        coverage.add_count(
            CoverageLayer::Projection,
            "relation pairs omitted from projection",
            held.difference(&represented).count(),
        );
        if !self.metadata.faults.is_empty() {
            coverage.add_count(
                CoverageLayer::Projection,
                "required projection channels unavailable",
                self.metadata.faults.len(),
            );
        }
        // Preserve only projection-local reasons across refresh, never stale host context.
        for limit in &self.metadata.coverage.limits {
            if limit.layer == CoverageLayer::Projection
                && matches!(
                    limit.reason.as_str(),
                    "projection focus unavailable" | "projection strategy unavailable"
                )
            {
                coverage.push(limit.clone());
            }
        }
        self.metadata.coverage = coverage;
        self
    }

    pub fn unavailable(graph: &Graph, reason: &str) -> Self {
        let mut projection = Self::empty();
        projection
            .metadata
            .coverage
            .push(CoverageLimit::new(CoverageLayer::Projection, reason));
        projection.with_graph_coverage(graph)
    }
}

/// Per-projection metadata that doesn't fit elsewhere.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectionMetadata {
    /// `projection_id()` of the strategy that produced this
    /// projection. Set by the strategy; consumers can inspect to
    /// route per-strategy debug overlays.
    pub strategy_id: Option<String>,
    /// Whether the strategy considers this projection settled
    /// (force-directed converged, tree fully placed, etc.).
    /// Hosts can use this to suppress redundant re-projection.
    pub settled: bool,
    /// The channels the strategy needed and could not read, each missing or
    /// of the wrong kind (dynamics grammar plan, F86). A projection with a
    /// fault places nothing.
    #[serde(default)]
    pub faults: Vec<crate::signals::SignalFault>,
    /// Known limits of this supplied graph and this projection.
    #[serde(default)]
    pub coverage: CoverageNote,
}

/// One node with a positioned point.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PositionedNode {
    pub node: NodeKey,
    pub position: PortablePoint,
    /// Suggested visual radius in projection-space units. Strategies
    /// that don't care about radius emit 0.0; canvases substitute a
    /// theme-default.
    pub radius: f32,
}

/// One edge with a positioned start/end. Edges may pass through
/// additional waypoints for routed strategies (orthogonal routing,
/// bundled edges) — represented here as a polyline; strategies that
/// don't route emit just `[from, to]`.
///
/// `edge` is `Option<EdgeKey>` because some strategies derive edges
/// from node identity rather than from a stable graph edge (e.g.
/// analytic strategies that draw spring connections without going
/// through the graph's edge table; the kernel graph's `relations()`
/// iterator yields `RelationView`s that don't carry an `EdgeKey`
/// because one `EdgeKey` can project to multiple `RelationView`s
/// when an edge payload carries several typed sidecars).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PositionedEdge {
    pub edge: Option<EdgeKey>,
    pub from: NodeKey,
    pub to: NodeKey,
    pub path: Vec<PortablePoint>,
    /// Relative edge weight, driving the stroke thickness: `1.0` is the normal stroke; a
    /// heavier edge (e.g. a node pair connected by several statements — the multigraph
    /// multiplicity) paints thicker. `1.0` for an un-weighted edge, so the default look is
    /// unchanged. (Graph signals — edge-weight encoding.)
    pub weight: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_empty_has_no_geometry() {
        let p = Projection::empty();
        assert!(p.nodes.is_empty());
        assert!(p.edges.is_empty());
        assert!(p.overlays.is_empty());
        assert!(p.minimap.is_none());
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use kernel::graph::{
        CoverageLayer, CoverageLimit, CoverageNote, Graph, SemanticStatementSpec, SemanticSubKind,
        predicate_iri,
    };

    #[test]
    fn output_omissions_and_scene_lowering_preserve_coverage() {
        let mut graph = Graph::new();
        let a = kernel::graph::apply::add_node(
            &mut graph,
            None,
            "https://example.org/a".into(),
            Default::default(),
        );
        let b = kernel::graph::apply::add_node(
            &mut graph,
            None,
            "https://example.org/b".into(),
            Default::default(),
        );
        graph
            .try_assert_semantic_statement(
                a,
                a,
                SemanticStatementSpec {
                    predicate: predicate_iri(SemanticSubKind::Cites).into(),
                    recognized_sub_kind: Some(SemanticSubKind::Cites),
                    ..Default::default()
                },
            )
            .unwrap();
        let known = CoverageNote {
            limits: CoverageLayer::ALL
                .into_iter()
                .map(|layer| CoverageLimit::new(layer, "host boundary"))
                .collect(),
        };
        graph.set_known_coverage(known.clone());
        let mut projection = Projection {
            nodes: vec![PositionedNode {
                node: a,
                position: Default::default(),
                radius: 0.0,
            }],
            ..Projection::empty()
        }
        .with_graph_coverage(&graph);
        assert!(
            known
                .limits
                .iter()
                .all(|limit| projection.metadata.coverage.limits.contains(limit))
        );
        for reason in [
            "Surfaces omitted from projection",
            "held Resources without a projected appearance",
            "relation pairs omitted from projection",
        ] {
            assert!(
                projection
                    .metadata
                    .coverage
                    .limits
                    .iter()
                    .any(|limit| limit.reason == reason && limit.count == Some(1)),
                "{reason}"
            );
        }
        let scene = crate::scene_from_projection(
            &projection,
            |key| graph.get_node(key).unwrap().id.to_string(),
            |_| None,
        );
        assert_eq!(scene.coverage, projection.metadata.coverage);
        assert_eq!(scene.scene.items.len(), 1);
        let geometry = projection.nodes.clone();
        graph.set_known_coverage(Default::default());
        projection = projection.with_graph_coverage(&graph);
        assert_eq!(projection.nodes, geometry);
        assert!(
            !projection
                .metadata
                .coverage
                .limits
                .iter()
                .any(|limit| limit.reason == "host boundary")
        );
        projection.nodes.push(PositionedNode {
            node: b,
            position: Default::default(),
            radius: 0.0,
        });
        projection.edges.push(PositionedEdge {
            edge: None,
            from: a,
            to: a,
            path: vec![],
            weight: 1.0,
        });
        assert!(
            projection
                .with_graph_coverage(&graph)
                .metadata
                .coverage
                .limits
                .is_empty()
        );
    }
    #[test]
    fn missing_channels_on_an_empty_graph_are_a_projection_limit() {
        use crate::{IntelligenceSignals, LayoutStrategy, ProjectionRequest, ViewIntent};
        let graph = Graph::new();
        let signals = IntelligenceSignals::default();
        let request = ProjectionRequest {
            graph: &graph,
            signals: &signals,
            intent: ViewIntent::default(),
        };
        let projection = crate::adapters::GridAdapter::default().project(&request);
        assert!(!projection.metadata.faults.is_empty());
        assert!(
            projection
                .metadata
                .coverage
                .limits
                .iter()
                .any(|limit| limit.layer == CoverageLayer::Projection
                    && limit.reason == "required projection channels unavailable")
        );
        let spiral =
            crate::project_spiral_score(&graph, &signals, crate::ORDER_TIMELINE, None, None);
        assert!(
            spiral
                .projection
                .metadata
                .coverage
                .limits
                .iter()
                .any(|limit| limit.reason == "required projection channels unavailable")
        );
    }
}
