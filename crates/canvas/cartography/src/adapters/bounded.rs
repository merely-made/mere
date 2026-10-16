// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Adapters for the bounded arrangements (Partition, Sets): both fill the
//! request's target size, centred on the origin, and both read their groups
//! from the axis the host threads onto the intent.

use std::collections::{BTreeMap, HashMap};

use kernel::graph::NodeKey;
use sceno::{Arrangement, Rect, SET_SEPARATOR, Size2, Vec2};

use super::{Disclosures, project_arrangement};
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
