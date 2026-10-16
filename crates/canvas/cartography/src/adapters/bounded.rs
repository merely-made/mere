// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Adapters for the share arrangements: Partition and Sets fill the request's
//! target size, centred on the origin; Mosaic grows on the infinite canvas.
//! All three read their groups from the axis the host threads onto the
//! intent.

use std::collections::{BTreeMap, HashMap};

use kernel::graph::NodeKey;
use sceno::{Arrangement, Rect, SET_SEPARATOR, Size2, Vec2};

use super::{Disclosures, project_arrangement, project_score, score_from_request};
use crate::projection::Projection;
use crate::request::{AxisValue, ProjectionRequest};
use crate::signals::WEIGHT_DEGREE;
use crate::strategy::LayoutStrategy;

/// The request's target size as an area centred on the origin.
fn viewport(request: &ProjectionRequest<'_>) -> Rect {
    let size = request.intent.target_size.logical_size();
    Rect::new(
        Vec2::new(-size.width / 2.0, -size.height / 2.0),
        Size2::new(size.width, size.height),
    )
}

/// Shares of a measured whole subdividing the viewport. The axis names each
/// node's group; `weight.degree` is its size where the host disclosed it,
/// else every node counts as one.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PartitionAdapter {
    pub config: sceno::Partition,
}

impl PartitionAdapter {
    pub const PROJECTION_ID: &'static str = "partition.default";
}

impl LayoutStrategy for PartitionAdapter {
    fn projection_id(&self) -> &'static str {
        Self::PROJECTION_ID
    }

    fn project(&self, request: &ProjectionRequest<'_>) -> Projection {
        let mut disclosures = Disclosures::from_intent(request);
        if let Ok(weights) = request.signals.weights(WEIGHT_DEGREE) {
            // Degree zero still holds a share: an isolated node is not nothing.
            disclosures = disclosures.with_weight(
                weights
                    .weights
                    .iter()
                    .map(|(key, w)| (*key, w.max(0.0) + 1.0))
                    .collect(),
            );
        }
        project_arrangement(
            Self::PROJECTION_ID,
            request,
            Arrangement::Partition(sceno::Partition {
                bounds: viewport(request),
                ..self.config.clone()
            }),
            &disclosures,
            Vec::new(),
        )
    }
}

/// Fixed set regions filling the viewport. The axis lists each node's sets
/// (`a|b`); without configured sets the four most common are drawn. The
/// focused node sits in the core, everything else in the rim.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SetsAdapter {
    pub config: sceno::Setscape,
}

impl SetsAdapter {
    pub const PROJECTION_ID: &'static str = "sets.default";
    /// How many sets are drawn when none are configured.
    pub const DEFAULT_SETS: usize = 4;
}

/// The `limit` most common set names on `axis`, most common first.
pub fn common_sets(axis: &HashMap<NodeKey, AxisValue>, limit: usize) -> Vec<String> {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for value in axis.values() {
        if let AxisValue::Categorical(tags) = value {
            for tag in tags
                .split(SET_SEPARATOR)
                .map(str::trim)
                .filter(|t| !t.is_empty())
            {
                *counts.entry(tag).or_default() += 1;
            }
        }
    }
    let mut ranked: Vec<(&str, usize)> = counts.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    ranked
        .into_iter()
        .take(limit)
        .map(|(tag, _)| tag.to_owned())
        .collect()
}

impl LayoutStrategy for SetsAdapter {
    fn projection_id(&self) -> &'static str {
        Self::PROJECTION_ID
    }

    fn project(&self, request: &ProjectionRequest<'_>) -> Projection {
        let axis = request.intent.axis_values.clone().unwrap_or_default();
        let sets = if self.config.sets.is_empty() {
            common_sets(&axis, Self::DEFAULT_SETS)
        } else {
            self.config.sets.clone()
        };
        let tier = request
            .graph
            .nodes()
            .map(|(key, _)| {
                (
                    key,
                    if request.intent.focus == Some(key) {
                        1.0
                    } else {
                        0.0
                    },
                )
            })
            .collect();
        project_arrangement(
            Self::PROJECTION_ID,
            request,
            Arrangement::Setscape(sceno::Setscape {
                bounds: viewport(request),
                sets,
            }),
            &Disclosures::from_intent(request).with_weight(tier),
            Vec::new(),
        )
    }
}

/// Tiles by weight on the infinite canvas, grouped by the axis. A tile's
/// weight is `weight.degree` plus one, times the person's size factor; the
/// person's order leads within each group.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MosaicAdapter {
    pub config: sceno::Mosaic,
    /// Nodes in the person's order; unlisted nodes follow in score order.
    pub order: Vec<NodeKey>,
    /// Size factors the person set, by node.
    pub scale: HashMap<NodeKey, f32>,
}

impl MosaicAdapter {
    pub const PROJECTION_ID: &'static str = "mosaic.default";
}

impl LayoutStrategy for MosaicAdapter {
    fn projection_id(&self) -> &'static str {
        Self::PROJECTION_ID
    }

    fn project(&self, request: &ProjectionRequest<'_>) -> Projection {
        let degree: HashMap<NodeKey, f32> = request
            .signals
            .weights(WEIGHT_DEGREE)
            .map(|weights| weights.weights.iter().copied().collect())
            .unwrap_or_default();
        let weight = request
            .graph
            .nodes()
            .map(|(key, _)| {
                let base = degree.get(&key).copied().unwrap_or(0.0).max(0.0) + 1.0;
                (key, base * self.scale.get(&key).copied().unwrap_or(1.0))
            })
            .collect();
        let disclosures = Disclosures::from_intent(request).with_weight(weight);
        let arrangement = Arrangement::Mosaic(self.config.clone());
        let (mut score, keys) = match score_from_request(request, arrangement, &disclosures) {
            Ok(scored) => scored,
            Err(fault) => {
                return super::faulted_projection(Self::PROJECTION_ID, vec![fault])
                    .with_graph_coverage(request.graph);
            },
        };
        if keys.is_empty() {
            return super::empty_projection(Self::PROJECTION_ID).with_graph_coverage(request.graph);
        }
        // The person's order first, then the score's, as fresh ordinals.
        let rank: HashMap<NodeKey, usize> = self
            .order
            .iter()
            .enumerate()
            .map(|(i, key)| (*key, i))
            .collect();
        let mut paired: Vec<_> = score.items.drain(..).zip(keys).collect();
        paired.sort_by_key(|(item, key)| {
            (rank.get(key).copied().unwrap_or(usize::MAX), item.ordinal)
        });
        let mut keys = Vec::with_capacity(paired.len());
        for (ordinal, (mut item, key)) in paired.into_iter().enumerate() {
            item.ordinal = ordinal as u32;
            score.items.push(item);
            keys.push(key);
        }
        project_score(Self::PROJECTION_ID, request, &score, &keys)
    }
}
