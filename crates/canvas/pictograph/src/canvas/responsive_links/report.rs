// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Current resting geometry with canonical source references for host inspection.

use scenograph::link::LinkPairReading;

use super::{Canvas, EvaluatedLinkPath, LinkPathSourceRef, SourceBucketRef, edge_cells};

pub(super) fn base_paths(
    canvas: &Canvas,
    segments: &[edge_cells::LinkAppearanceSegment],
    readings: &[LinkPairReading],
) -> Vec<EvaluatedLinkPath> {
    // Index source correspondence once; paint/picking never build this report.
    let mut sources = std::collections::HashMap::new();
    for (reading_index, reading) in readings.iter().enumerate() {
        for (link_index, link) in reading.links.iter().enumerate() {
            for bucket in &link.buckets {
                let (Ok(from), Ok(to)) = (
                    bucket.displayed_from.parse::<uuid::Uuid>(),
                    bucket.displayed_to.parse::<uuid::Uuid>(),
                ) else {
                    continue;
                };
                sources
                    .entry((link.key.clone(), from.min(to), from.max(to)))
                    .or_insert((reading_index, reading, link_index, link));
            }
        }
    }
    let mut output = Vec::new();
    for segment in segments {
        let (Some(from), Some(to)) = (
            canvas.graph.get_node(segment.appearance.display_from),
            canvas.graph.get_node(segment.appearance.display_to),
        ) else {
            continue;
        };
        if !canvas.node_visible_in_canvas(segment.appearance.display_from)
            || !canvas.node_visible_in_canvas(segment.appearance.display_to)
        {
            continue;
        }
        let Some(&(reading_index, reading, link_index, link)) = sources.get(&(
            segment.appearance.key.clone(),
            from.id.min(to.id),
            from.id.max(to.id),
        )) else {
            continue;
        };
        let bucket_refs = link
            .buckets
            .iter()
            .enumerate()
            .map(|(bucket_index, bucket)| LinkPathSourceRef {
                displayed_from: bucket.displayed_from.clone(),
                displayed_to: bucket.displayed_to.clone(),
                assertion_ids: bucket
                    .assertions
                    .iter()
                    .map(|assertion| assertion.id.clone())
                    .collect(),
                legacy_members: bucket
                    .legacy_members
                    .iter()
                    .map(|member| member.family.clone())
                    .collect(),
                bucket: SourceBucketRef {
                    reading_index,
                    link_index,
                    bucket_index,
                },
            })
            .collect();
        output.push(EvaluatedLinkPath {
            key: segment.appearance.key.clone(),
            displayed_from: from.id,
            displayed_to: to.id,
            stratum: segment.appearance.key.stratum,
            points: segment
                .points
                .iter()
                .map(|point| (point.x, point.y))
                .collect(),
            responsive: segment.responsive,
            source: reading.source.clone(),
            buckets: bucket_refs,
        });
    }
    output
}
