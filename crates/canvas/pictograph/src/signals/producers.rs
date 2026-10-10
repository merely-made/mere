// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Graph work whose result is a disclosure an arrangement reads: ring indices
//! from a breadth-first walk, spectral coordinates from the graph Laplacian,
//! degree weights, the recency order and the enumeration order. Moved from
//! cartography (`adapters/producers.rs` and `spiral_score.rs`) unchanged in
//! arithmetic, so every arrangement's output holds (dynamics grammar plan,
//! G2b, F38). The [`ChannelRegistry`](super::ChannelRegistry) runs them once
//! per key; these are the computations it caches.
//!
//! Each reduces to a small per-item value, so the walk happens once beside the
//! graph, and what crosses into a score is a ring index, a weight or a pair of
//! coordinates: `sceno`'s solvers never learn a source's native truth.

use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

use kernel::graph::{Graph, NodeKey};

fn projected_neighbors(graph: &Graph, key: NodeKey) -> impl Iterator<Item = NodeKey> + '_ {
    graph
        .projected_outgoing_relations(key)
        .chain(graph.projected_incoming_relations(key))
        .map(|(neighbor, _, _)| neighbor)
}

/// Breadth-first ring index from `focus`. The focus is ring zero.
///
/// Nodes unreachable from the focus are absent from the map rather than given a
/// sentinel ring: "no ring" is what `RadialUnreachablePolicy` exists to answer,
/// and a sentinel would have to be a number the solver could not tell from a
/// real ring.
pub fn radial_rings(graph: &Graph, focus: NodeKey) -> HashMap<NodeKey, u32> {
    let mut ring_of: HashMap<NodeKey, u32> = HashMap::new();
    if graph.get_node(focus).is_none() {
        return ring_of;
    }
    ring_of.insert(focus, 0);

    let mut queue: VecDeque<NodeKey> = VecDeque::from([focus]);
    while let Some(key) = queue.pop_front() {
        let next = ring_of[&key] + 1;
        for neighbour in projected_neighbors(graph, key) {
            if let std::collections::hash_map::Entry::Vacant(slot) = ring_of.entry(neighbour) {
                slot.insert(next);
                queue.push_back(neighbour);
            }
        }
    }
    ring_of
}

/// Undirected degree plus one, as the angular weight for
/// `sceno::RadialAngularPolicy::Weighted`.
///
/// The plus-one guarantees a zero-degree node still gets a slot, and it matches
/// the solver's own default of `1.0` for an item that disclosed no weight, so
/// an isolated node and an undisclosed one occupy the same arc. Self-loops do
/// not count, and every edge does, hidden or not: this is the graph's degree,
/// not the physics view's (which is why it is `weight.degree`, not
/// `mass.degree`).
pub fn degree_weights(graph: &Graph) -> HashMap<NodeKey, f32> {
    graph
        .nodes()
        .map(|(key, _)| {
            let degree = projected_neighbors(graph, key)
                .filter(|neighbour| *neighbour != key)
                .count();
            (key, (degree + 1) as f32)
        })
        .collect()
}

/// Per-node coordinates from the two smallest non-trivial Laplacian
/// eigenvectors, normalized into roughly `[-1, 1]`.
///
/// Normalizing here rather than in the solver is deliberate: eigenvector
/// components come out a few thousandths wide, and only this side knows that.
/// `sceno::Embedded` then applies the caller's origin and scale.
///
/// Returns an empty map when the graph has no structure to project — an
/// edgeless or perfectly symmetric graph collapses every component to zero.
/// Empty rather than all-zeros, so `EmbeddingFallback` decides what happens to
/// nodes with nothing to place them by, instead of stacking them all at one
/// point.
pub fn spectral_coords(graph: &Graph, iterations: usize) -> HashMap<NodeKey, (f32, f32)> {
    let keys: Vec<NodeKey> = graph.nodes().map(|(key, _)| key).collect();
    if keys.is_empty() {
        return HashMap::new();
    }
    let adjacency = weighted_adjacency(graph, &keys);
    let vectors = smallest_laplacian_eigenvectors(&adjacency, 2, iterations);
    let coords: Vec<(f64, f64)> = (0..keys.len())
        .map(|index| (vectors[0][index], vectors[1][index]))
        .collect();

    let max_abs = coords
        .iter()
        .flat_map(|(x, y)| [x.abs(), y.abs()])
        .fold(0.0_f64, f64::max);
    if max_abs <= 1e-9 {
        return HashMap::new();
    }
    keys.iter()
        .zip(&coords)
        .map(|(key, (x, y))| (*key, ((x / max_abs) as f32, (y / max_abs) as f32)))
        .collect()
}

/// Index-based weighted undirected adjacency: multigraph multiplicity summed,
/// self-loops dropped, rows sorted for determinism.
fn weighted_adjacency(graph: &Graph, keys: &[NodeKey]) -> Vec<Vec<(usize, f64)>> {
    let index: HashMap<NodeKey, usize> =
        keys.iter().enumerate().map(|(i, key)| (*key, i)).collect();
    let mut rows: Vec<HashMap<usize, f64>> = vec![HashMap::new(); keys.len()];
    for (i, key) in keys.iter().enumerate() {
        for neighbour in projected_neighbors(graph, *key) {
            if let Some(&j) = index.get(&neighbour)
                && i != j
            {
                *rows[i].entry(j).or_insert(0.0) += 1.0;
            }
        }
    }
    rows.into_iter()
        .map(|row| {
            let mut row: Vec<(usize, f64)> = row.into_iter().collect();
            row.sort_unstable_by_key(|(j, _)| *j);
            row
        })
        .collect()
}

/// Power-iterate the `count` smallest non-trivial Laplacian eigenvectors via
/// `B = cI - L`, whose largest eigenvectors are `L`'s smallest, deflating
/// against the all-ones vector and each previously found one.
///
/// `c = 2·max_degree` is the tight Gershgorin bound, so `B` is
/// positive-semidefinite and the iteration converges to the wanted end. Each
/// returned vector is unit-norm with zero mean, or all-zero when the graph is
/// too small or degenerate to support one.
fn smallest_laplacian_eigenvectors(
    adjacency: &[Vec<(usize, f64)>],
    count: usize,
    iterations: usize,
) -> Vec<Vec<f64>> {
    let n = adjacency.len();
    let degree: Vec<f64> = adjacency
        .iter()
        .map(|row| row.iter().map(|(_, weight)| weight).sum())
        .collect();
    let c = degree.iter().copied().fold(0.0_f64, f64::max) * 2.0;

    let mut found: Vec<Vec<f64>> = Vec::new();
    for eigen_index in 0..count {
        let mut vector: Vec<f64> = (0..n).map(|i| start_value(i, eigen_index, n)).collect();
        orthonormalize(&mut vector, &found);
        // No edges leaves `B = 0`, so iteration cannot find structure. The
        // zeroed vector falls through to the caller's empty-map return.
        if c > 0.0 {
            for _ in 0..iterations {
                let mut next = vec![0.0; n];
                for i in 0..n {
                    let mut sum = (c - degree[i]) * vector[i];
                    for (j, weight) in &adjacency[i] {
                        sum += weight * vector[*j];
                    }
                    next[i] = sum;
                }
                vector = next;
                orthonormalize(&mut vector, &found);
            }
        }
        found.push(vector);
    }
    found
}

/// Subtract the all-ones component and the projection onto each found vector,
/// then normalize. Leaves the vector all-zero if it collapses.
fn orthonormalize(vector: &mut [f64], found: &[Vec<f64>]) {
    let n = vector.len() as f64;
    let mean = vector.iter().sum::<f64>() / n;
    for value in vector.iter_mut() {
        *value -= mean;
    }
    for previous in found {
        let dot: f64 = vector.iter().zip(previous).map(|(a, b)| a * b).sum();
        for (value, component) in vector.iter_mut().zip(previous) {
            *value -= dot * component;
        }
    }
    let norm = vector.iter().map(|value| value * value).sum::<f64>().sqrt();
    if norm > 1e-12 {
        for value in vector.iter_mut() {
            *value /= norm;
        }
    }
}

/// A deterministic starting vector: a low-frequency cosine that overlaps the
/// corresponding Laplacian eigenvector — a path graph's eigenvectors *are*
/// cosines — varied by index so successive starts are not parallel.
fn start_value(i: usize, eigen_index: usize, n: usize) -> f64 {
    let t = i as f64 / n.max(1) as f64;
    ((eigen_index + 1) as f64 * std::f64::consts::PI * t).cos()
}

/// The graph's own enumeration order: what every score's ordinal follows, and
/// the order a timeline lays its axis along.
pub fn enumeration_order(graph: &Graph) -> Vec<NodeKey> {
    graph.nodes().map(|(key, _)| key).collect()
}

/// The recency channel: every node, most recently visited first, and each
/// node's recency normalized into `0..=1` (newest `1.0`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Recency {
    /// Most recent first. Visit facets persist millisecond timestamps, so
    /// nodes created in one tick commonly tie; a tie goes to the larger
    /// stable id, so the order is portable instead of inheriting local graph
    /// iteration.
    pub order: Vec<NodeKey>,
    /// Each node's visit time across the graph's span, in `f64` seconds, then
    /// `f32` (F51: the spiral's arithmetic). A graph with one distinct visit
    /// time reads every node as newest.
    pub values: HashMap<NodeKey, f32>,
}

pub fn recency(graph: &Graph) -> Recency {
    let mut order = enumeration_order(graph);
    order.sort_by_key(|key| {
        let node = graph
            .get_node(*key)
            .expect("node keys came from this graph");
        (
            std::cmp::Reverse(graph.node_last_visited(*key).unwrap_or(UNIX_EPOCH)),
            std::cmp::Reverse(node.id),
        )
    });
    let values = normalized_recency(graph, &order);
    Recency { order, values }
}

fn normalized_recency(graph: &Graph, keys: &[NodeKey]) -> HashMap<NodeKey, f32> {
    let times: Vec<_> = keys
        .iter()
        .copied()
        .map(|key| {
            let seconds = graph
                .node_last_visited(key)
                .and_then(|time: SystemTime| time.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs_f64())
                .unwrap_or(0.0);
            (key, seconds)
        })
        .collect();
    let minimum = times
        .iter()
        .map(|(_, time)| *time)
        .fold(f64::INFINITY, f64::min);
    let maximum = times
        .iter()
        .map(|(_, time)| *time)
        .fold(f64::NEG_INFINITY, f64::max);
    let span = maximum - minimum;
    times
        .into_iter()
        .map(|(key, time)| {
            let value = if span.is_finite() && span > f64::EPSILON {
                ((time - minimum) / span) as f32
            } else {
                1.0
            };
            (key, value)
        })
        .collect()
}
