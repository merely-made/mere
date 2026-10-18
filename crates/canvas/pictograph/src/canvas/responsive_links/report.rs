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
        let from_id = from.id.to_string();
        let to_id = to.id.to_string();
        let Some((reading_index, reading, link_index, link)) = readings
            .iter()
            .enumerate()
            .find_map(|(reading_index, reading)| {
                reading
                    .links
                    .iter()
                    .enumerate()
                    .find(|(_, link)| {
                        link.key == segment.appearance.key
                            && link.buckets.iter().any(|bucket| {
                                (bucket.displayed_from == from_id && bucket.displayed_to == to_id)
                                    || (bucket.displayed_from == to_id
                                        && bucket.displayed_to == from_id)
                            })
                    })
                    .map(|(link_index, link)| (reading_index, reading, link_index, link))
            })
        else {
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
            responsive: false,
            source: reading.source.clone(),
            buckets: bucket_refs,
        });
    }
    output
}
