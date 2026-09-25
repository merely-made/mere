/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Laying a host's graph out by one of cartography's graph-only strategies.

use std::collections::HashMap;

use cartography::adapters::project_graph_only;
use cartography::{IntelligenceSignals, Projection, ProjectionRequest, TargetSize, ViewIntent};
use kernel::geometry::PortablePoint;
use kernel::graph::apply::{add_node, assert_relation};
use kernel::graph::{EdgeAssertion, Graph, SemanticSubKind};
use uuid::Uuid;

use crate::model::GraphModel;

/// The layout used when the host names none, or one that cannot place a graph
/// from its nodes and relations alone.
pub const DEFAULT_LAYOUT: &str = "spectral.default";

/// The share of the graph's area left clear at each edge.
const MARGIN: f32 = 0.08;

/// Every node's position, normalized to `0..=1` within a graph area of
/// `width` by `height`, keyed by the node's key.
///
/// The strategy sees only the nodes and which ones are related, so a switch
/// of layout moves nodes without touching their keys. A node the strategy
/// leaves out sits at the centre.
pub fn lay_out(
    graph: &GraphModel,
    layout: &str,
    width: u32,
    height: u32,
) -> HashMap<String, (f32, f32)> {
    let mut scratch = Graph::new();
    let mut key_of = HashMap::with_capacity(graph.nodes.len());
    let mut node_of = HashMap::with_capacity(graph.nodes.len());
    for (index, node) in graph.nodes.iter().enumerate() {
        // An explicit id: a browser build has no random ids, and a fixed one
        // keeps a strategy's placement the same for the same graph.
        let id = Uuid::from_u128(index as u128 + 1);
        let at = add_node(
            &mut scratch,
            Some(id),
            format!("mere-view:node/{index}"),
            PortablePoint::new(0.0, 0.0),
        );
        key_of.insert(at, node.key.as_str());
        node_of.insert(node.key.as_str(), at);
    }
    for relation in &graph.relations {
        let (Some(&from), Some(&to)) = (
            node_of.get(relation.from.as_str()),
            node_of.get(relation.to.as_str()),
        ) else {
            continue;
        };
        // Strategies read adjacency, not kind.
        let link = EdgeAssertion::Semantic {
            sub_kind: SemanticSubKind::Hyperlink,
            label: None,
            decay_progress: None,
        };
        assert_relation(&mut scratch, from, to, link);
    }

    let signals = IntelligenceSignals::default();
    let request = ProjectionRequest {
        graph: &scratch,
        signals: &signals,
        intent: ViewIntent {
            target_size: TargetSize::Pixels {
                width: width.max(1),
                height: height.max(1),
            },
            ..ViewIntent::default()
        },
    };
    let projection = project_graph_only(layout, &request)
        .or_else(|| project_graph_only(DEFAULT_LAYOUT, &request))
        .unwrap_or_else(Projection::empty);

    let placed: HashMap<&str, (f32, f32)> = projection
        .nodes
        .iter()
        .filter_map(|node| {
            let key = key_of.get(&node.node)?;
            Some((*key, (node.position.x, node.position.y)))
        })
        .collect();
    let span = |axis: fn(&(f32, f32)) -> f32| {
        placed
            .values()
            .map(axis)
            .fold(None, |range, value| match range {
                None => Some((value, value)),
                Some((low, high)) => Some((f32::min(low, value), f32::max(high, value))),
            })
    };
    let fit = |value: f32, range: Option<(f32, f32)>| match range {
        Some((low, high)) if high - low > f32::EPSILON => {
            MARGIN + (value - low) / (high - low) * (1.0 - 2.0 * MARGIN)
        },
        _ => 0.5,
    };
    let (xs, ys) = (span(|at| at.0), span(|at| at.1));
    graph
        .nodes
        .iter()
        .map(|node| {
            let at = placed
                .get(node.key.as_str())
                .map_or((0.5, 0.5), |&(x, y)| (fit(x, xs), fit(y, ys)));
            (node.key.clone(), at)
        })
        .collect()
}
