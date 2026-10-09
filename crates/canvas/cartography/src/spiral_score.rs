// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mere's graph-to-Scenograph Spiral adapter.
//!
//! The solver is portable (`scenomise::solve`). This module is deliberately
//! not: it places the graph in the order it is handed, chooses the browser's
//! LOD rungs from the recency it is handed, and maps `NodeKey`s back into the
//! local canvas projection. That is the boundary P3 exists to prove. The order
//! and the recency are the host's channel registry's, read from keyed signals
//! like every other arrangement's facts (`order.recency` or `order.timeline`,
//! and `weight.recency`; dynamics grammar plan, G2b and F132).

use std::collections::HashMap;

use kernel::geometry::{PortablePoint, PortableRect, PortableSize};
use kernel::graph::{Graph, NodeKey};
use sceno::{
    Arrangement, Footprint, Placement, Representation, Score, ScoreItem, Size2, SourceRef, Spiral,
};

use crate::representation::{RepresentationState, default_graph_representation_registry};
use crate::scene_out::MERE_GRAPH_ADAPTER;
use crate::signals::{IntelligenceSignals, SignalFault, WEIGHT_RECENCY};
use crate::{PositionedEdge, PositionedNode, Projection, ProjectionMetadata};

const SPIRAL_ID: &str = "phyllotaxis.default";

/// The persisted score plus the ordinary Mere canvas projection it realizes.
#[derive(Clone, Debug, PartialEq)]
pub struct MereSpiralProjection {
    pub score: Score,
    pub projection: Projection,
}

/// Build and realize Mere's P3 pane-spiral score.
///
/// `signals` carry what the host's channel registry computed: the order the
/// spiral places the nodes in, under `order` (`order.recency`, most recent
/// first, or `order.timeline`, the graph's own), and each node's recency in
/// `0..=1` under `weight.recency`, which picks its rung (F132). `extents` are
/// the host's measured node faces. The portable score retains only the
/// ordinal, never a Mere timestamp. A channel missing or of another kind is
/// reported on the projection's faults, with nothing placed.
pub fn project_spiral_score(
    graph: &Graph,
    signals: &IntelligenceSignals,
    order: &str,
    extents: Option<&HashMap<NodeKey, (f32, f32)>>,
    focus: Option<NodeKey>,
) -> MereSpiralProjection {
    project_spiral_score_for_view(graph, signals, order, extents, focus, 1.0, None)
}

/// Build and realize Mere's P3 pane-spiral score for one declared view.
///
/// Representation conditions stay in Cartography's host registry. The score
/// records only the selected rung. `previous` supplies prior selections for
/// hysteresis; it never changes source identity, order, placement, or geometry.
pub fn project_spiral_score_for_view(
    graph: &Graph,
    signals: &IntelligenceSignals,
    order: &str,
    extents: Option<&HashMap<NodeKey, (f32, f32)>>,
    focus: Option<NodeKey>,
    zoom_level: f32,
    previous: Option<&Score>,
) -> MereSpiralProjection {
    let order = signals.order(order);
    let weights = signals.weights(WEIGHT_RECENCY);
    let (ordered, recency) = match (order, weights) {
        (Ok(order), Ok(weights)) => (
            order.order.as_slice(),
            weights
                .weights
                .iter()
                .copied()
                .collect::<HashMap<NodeKey, f32>>(),
        ),
        (order, weights) => {
            let faults: Vec<SignalFault> =
                [order.err(), weights.err()].into_iter().flatten().collect();
            let mut projection = crate::adapters::empty_projection(SPIRAL_ID);
            projection.metadata.faults = faults;
            return MereSpiralProjection {
                score: Score::new(Arrangement::Spiral(Spiral::default())),
                projection,
            };
        },
    };
    let registry = default_graph_representation_registry();
    let previous: HashMap<&str, &Representation> = previous
        .into_iter()
        .flat_map(|score| score.items.iter())
        .filter(|item| item.source.adapter == MERE_GRAPH_ADAPTER)
        .map(|item| (item.source.id.as_str(), &item.representation))
        .collect();
    let zoom_level = if zoom_level.is_finite() && zoom_level > 0.0 {
        zoom_level
    } else {
        1.0
    };
    let mut score = Score::new(Arrangement::Spiral(Spiral::default()));
    score.generation = graph.revision();
    for (ordinal, key) in ordered.iter().enumerate() {
        let node = graph
            .get_node(*key)
            .expect("node keys came from this graph");
        let extent = extents
            .and_then(|items| items.get(key).copied())
            .unwrap_or((0.0, 0.0));
        let source = SourceRef::new(MERE_GRAPH_ADAPTER, node.id.to_string());
        let tags = graph.node_content_tags(*key).unwrap_or_default();
        let profile = registry.resolve_classes(tags.iter().map(String::as_str));
        let state = RepresentationState {
            screen_width: extent.0 * zoom_level,
            screen_height: extent.1 * zoom_level,
            zoom_level,
            recency: recency.get(key).copied().unwrap_or(0.0),
            focused: focus == Some(*key),
        };
        score.items.push(ScoreItem {
            representation: profile
                .ladder
                .select(state, previous.get(source.id.as_str()).copied()),
            source,
            ordinal: ordinal as u32,
            footprint: footprint_for(extent),
            placement: Placement::Ordinal,
            layer: 0,
            visible: true,
            // A spiral places by ordinal alone. The disclosure fields exist for
            // the arrangements that read them; this adapter has nothing to
            // disclose, and `None` says so rather than guessing a zero.
            axis: None,
            embedding: None,
            weight: None,
        });
    }

    let scene = scenomise::solve(&score);
    let positions: HashMap<NodeKey, PortablePoint> = ordered
        .iter()
        .copied()
        .zip(scene.items.iter())
        .map(|(key, item)| {
            (
                key,
                PortablePoint::new(item.transform.translate.x, item.transform.translate.y),
            )
        })
        .collect();
    let nodes = ordered
        .iter()
        .filter_map(|key| {
            positions.get(key).copied().map(|position| PositionedNode {
                node: *key,
                position,
                radius: 0.0,
            })
        })
        .collect();
    let edges = graph
        .projected_relations()
        .map(|(_, view)| view)
        .filter(|relation| relation.from != relation.to)
        .map(|relation| PositionedEdge {
            edge: None,
            from: relation.from,
            to: relation.to,
            path: vec![
                positions.get(&relation.from).copied().unwrap_or_default(),
                positions.get(&relation.to).copied().unwrap_or_default(),
            ],
            weight: 1.0,
        })
        .collect();
    let projection = Projection {
        nodes,
        edges,
        overlays: Vec::new(),
        minimap: None,
        content_bounds: PortableRect::new(
            PortablePoint::new(scene.bounds.origin.x, scene.bounds.origin.y),
            PortableSize::new(scene.bounds.size.w, scene.bounds.size.h),
        ),
        metadata: ProjectionMetadata {
            strategy_id: Some(SPIRAL_ID.to_string()),
            settled: true,
            faults: Vec::new(),
        },
    };
    MereSpiralProjection { score, projection }
}

fn footprint_for((w, h): (f32, f32)) -> Footprint {
    if w > 0.0 && h > 0.0 {
        Footprint::Rect {
            size: Size2::new(w, h),
        }
    } else {
        Footprint::Point
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::graph::apply::add_node;
    use uuid::Uuid;

    fn fixture(ids: std::ops::RangeInclusive<u128>) -> (Graph, Vec<NodeKey>) {
        let mut graph = Graph::new();
        let keys = ids
            .map(|id| {
                add_node(
                    &mut graph,
                    Some(Uuid::from_u128(id)),
                    format!("fixture://{id}"),
                    PortablePoint::zero(),
                )
            })
            .collect();
        (graph, keys)
    }

    use crate::signals::{
        ImportanceWeights, NodeOrder, ORDER_RECENCY, ORDER_TIMELINE, Signal, SignalKind,
    };

    /// The registry's recency (`order.recency`, `weight.recency`) is computed
    /// host-side; these tests hand the spiral the signals it would produce.
    fn keyed(ordered: &[NodeKey], recency: &HashMap<NodeKey, f32>) -> IntelligenceSignals {
        IntelligenceSignals::new()
            .with(
                ORDER_RECENCY,
                Signal::Order(NodeOrder {
                    order: ordered.to_vec(),
                }),
            )
            .with(
                WEIGHT_RECENCY,
                Signal::Weights(ImportanceWeights {
                    weights: recency.iter().map(|(k, v)| (*k, *v)).collect(),
                }),
            )
    }

    fn newest(keys: &[NodeKey]) -> IntelligenceSignals {
        keyed(keys, &keys.iter().map(|key| (*key, 1.0)).collect())
    }

    /// F132: the Spiral reports a missing order or recency, or one of another
    /// kind, and places nothing; with both it places every node (the control).
    #[test]
    fn the_spiral_reports_a_missing_or_mistyped_channel_and_places_nothing() {
        let (graph, keys) = fixture(1..=3);
        let whole = newest(&keys);
        let placed = project_spiral_score(&graph, &whole, ORDER_RECENCY, None, None);
        assert_eq!(placed.projection.nodes.len(), 3);
        assert!(placed.projection.metadata.faults.is_empty());

        let no_timeline = project_spiral_score(&graph, &whole, ORDER_TIMELINE, None, None);
        assert!(no_timeline.projection.nodes.is_empty());
        assert!(no_timeline.score.items.is_empty());
        assert_eq!(
            no_timeline.projection.metadata.faults,
            vec![SignalFault::Missing {
                id: ORDER_TIMELINE.into()
            }]
        );

        let mistyped = IntelligenceSignals::new()
            .with(
                ORDER_RECENCY,
                Signal::Order(NodeOrder {
                    order: keys.clone(),
                }),
            )
            .with(
                WEIGHT_RECENCY,
                Signal::Order(NodeOrder {
                    order: keys.clone(),
                }),
            );
        let wrong = project_spiral_score(&graph, &mistyped, ORDER_RECENCY, None, None);
        assert!(wrong.projection.nodes.is_empty());
        assert_eq!(
            wrong.projection.metadata.faults,
            vec![SignalFault::Mistyped {
                id: WEIGHT_RECENCY.into(),
                expected: SignalKind::Weights,
                found: SignalKind::Order,
            }]
        );
        assert_eq!(
            wrong.projection.metadata.strategy_id.as_deref(),
            Some(SPIRAL_ID)
        );
    }

    #[test]
    fn the_handed_order_is_the_portable_order_and_recency_selects_declared_lod_rungs() {
        let (graph, keys) = fixture(1..=4);
        let ordered = [keys[3], keys[2], keys[1], keys[0]];
        let recency = HashMap::from([
            (keys[0], 0.0),
            (keys[1], 1.0 / 3.0),
            (keys[2], 2.0 / 3.0),
            (keys[3], 1.0),
        ]);
        let extents = HashMap::from([
            (keys[0], (36.0, 36.0)),
            (keys[1], (52.0, 52.0)),
            (keys[2], (80.0, 80.0)),
            (keys[3], (88.0, 88.0)),
        ]);
        let projected = project_spiral_score(
            &graph,
            &keyed(&ordered, &recency),
            ORDER_RECENCY,
            Some(&extents),
            Some(keys[3]),
        );
        assert_eq!(
            projected.score.items[0].source.id,
            Uuid::from_u128(4).to_string()
        );
        assert_eq!(
            projected.score.items[0].representation,
            Representation::LivePane
        );
        assert!(
            projected
                .score
                .items
                .iter()
                .any(|item| item.representation == Representation::Card)
        );
        assert!(
            projected
                .score
                .items
                .iter()
                .any(|item| item.representation == Representation::Glyph)
        );
        assert_eq!(projected.projection.nodes.len(), 4);
    }

    #[test]
    fn one_graph_at_two_zooms_selects_different_rungs_without_moving_items() {
        let (graph, keys) = fixture(1..=2);
        let (newest_key, oldest) = (keys[0], keys[1]);
        let ordered = [newest_key, oldest];
        let recency = HashMap::from([(newest_key, 1.0), (oldest, 0.0)]);
        let signals = keyed(&ordered, &recency);
        let extents = HashMap::from([(newest_key, (64.0, 64.0)), (oldest, (64.0, 64.0))]);

        let near = project_spiral_score_for_view(
            &graph,
            &signals,
            ORDER_RECENCY,
            Some(&extents),
            None,
            1.0,
            None,
        );
        let far = project_spiral_score_for_view(
            &graph,
            &signals,
            ORDER_RECENCY,
            Some(&extents),
            None,
            0.5,
            None,
        );

        assert_eq!(near.score.items[0].representation, Representation::Card);
        assert_eq!(far.score.items[0].representation, Representation::Glyph);
        assert_eq!(near.projection, far.projection);
        assert_eq!(near.score.items[0].source, far.score.items[0].source);
        assert_eq!(near.score.items[0].ordinal, far.score.items[0].ordinal);
    }

    #[test]
    fn an_unmeasured_item_does_not_claim_a_card() {
        let (graph, keys) = fixture(1..=1);
        let projected = project_spiral_score_for_view(
            &graph,
            &newest(&keys),
            ORDER_RECENCY,
            None,
            None,
            2.0,
            None,
        );
        assert_eq!(
            projected.score.items[0].representation,
            Representation::Glyph
        );
        assert_eq!(projected.score.items[0].footprint, Footprint::Point);
    }

    #[test]
    fn prior_score_supplies_hysteresis_and_focus_stays_live() {
        let (graph, keys) = fixture(1..=1);
        let key = keys[0];
        let signals = newest(&keys);
        let extents = HashMap::from([(key, (64.0, 64.0))]);
        let card = project_spiral_score_for_view(
            &graph,
            &signals,
            ORDER_RECENCY,
            Some(&extents),
            None,
            1.0,
            None,
        );
        let retained = project_spiral_score_for_view(
            &graph,
            &signals,
            ORDER_RECENCY,
            Some(&extents),
            None,
            0.95,
            Some(&card.score),
        );
        let released = project_spiral_score_for_view(
            &graph,
            &signals,
            ORDER_RECENCY,
            Some(&extents),
            None,
            0.89,
            Some(&retained.score),
        );
        let focused = project_spiral_score_for_view(
            &graph,
            &signals,
            ORDER_RECENCY,
            Some(&extents),
            Some(key),
            0.2,
            Some(&released.score),
        );

        assert_eq!(retained.score.items[0].representation, Representation::Card);
        assert_eq!(
            released.score.items[0].representation,
            Representation::Glyph
        );
        assert_eq!(
            focused.score.items[0].representation,
            Representation::LivePane
        );
    }
}
