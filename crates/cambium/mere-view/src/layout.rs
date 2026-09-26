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

/// Pixels between two nodes' centres, and the side of each node's square
/// target. Centres this far apart lie outside each other's targets, so every
/// press lands on the node meant, and labels do not print over each other.
pub const SEPARATION: f32 = 44.0;

/// Past this many nodes the pairwise separation pass is skipped, and nodes
/// keep the strategy's placement.
const SEPARATE_UP_TO: usize = 2000;

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
    let (width, height) = (width.max(1) as f32, height.max(1) as f32);
    let mut points: Vec<(f32, f32)> = graph
        .nodes
        .iter()
        .map(|node| {
            let (x, y) = placed
                .get(node.key.as_str())
                .map_or((0.5, 0.5), |&(x, y)| (fit(x, xs), fit(y, ys)));
            (x * width, y * height)
        })
        .collect();
    separate(&mut points, width, height);
    graph
        .nodes
        .iter()
        .zip(points)
        .map(|(node, (x, y))| (node.key.clone(), (x / width, y / height)))
        .collect()
}

/// Push apart, in pixels, every pair of nodes closer than [`SEPARATION`].
/// A strategy may place structurally equal nodes on one point; a coincident
/// pair parts along an angle fixed by its index, so the same graph always
/// comes out the same.
fn separate(points: &mut [(f32, f32)], width: f32, height: f32) {
    if points.len() > SEPARATE_UP_TO {
        return;
    }
    const GOLDEN_ANGLE: f32 = 2.399_963;
    // Clamped inside every pass, so a node held at the edge stays put and its
    // neighbour keeps moving until the pair is apart.
    let (left, right) = (width * MARGIN, (width * (1.0 - MARGIN)).max(width * MARGIN));
    let (top, bottom) = (
        height * MARGIN,
        (height * (1.0 - MARGIN)).max(height * MARGIN),
    );
    let inside = |point: &mut (f32, f32)| {
        point.0 = point.0.clamp(left, right);
        point.1 = point.1.clamp(top, bottom);
    };
    for _ in 0..96 {
        let mut moved = false;
        for i in 0..points.len() {
            for j in (i + 1)..points.len() {
                let (dx, dy) = (points[j].0 - points[i].0, points[j].1 - points[i].1);
                let distance = (dx * dx + dy * dy).sqrt();
                if distance >= SEPARATION {
                    continue;
                }
                let (ux, uy) = if distance > 0.01 {
                    (dx / distance, dy / distance)
                } else {
                    let angle = j as f32 * GOLDEN_ANGLE;
                    (angle.cos(), angle.sin())
                };
                // A quarter pixel past even, so a pair held at an edge settles.
                let push = (SEPARATION - distance) / 2.0 + 0.25;
                points[i].0 -= ux * push;
                points[i].1 -= uy * push;
                points[j].0 += ux * push;
                points[j].1 += uy * push;
                inside(&mut points[i]);
                inside(&mut points[j]);
                moved = true;
            }
        }
        if !moved {
            break;
        }
    }
    for point in points {
        inside(point);
    }
}
