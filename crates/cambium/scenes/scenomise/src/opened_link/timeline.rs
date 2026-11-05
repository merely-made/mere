// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn timeline_group(
    lane_by: LinkTimelineLane,
    assertion: Option<&LinkAssertion>,
    legacy_family: Option<&str>,
    direction: &Option<(String, String)>,
) -> String {
    if assertion.is_none_or(|item| item.first_asserted_ms.is_none()) {
        return "Undated".into();
    }
    match lane_by {
        LinkTimelineLane::Family => assertion
            .and_then(|item| item.recognized_kind.as_deref())
            .or(legacy_family)
            .or_else(|| assertion.map(|item| item.predicate.as_str()))
            .filter(|value| !value.is_empty())
            .unwrap_or("Unknown family")
            .to_owned(),
        LinkTimelineLane::Asserter => assertion
            .and_then(|item| item.asserter.as_deref())
            .filter(|value| !value.is_empty())
            .unwrap_or("Unknown asserter")
            .to_owned(),
        LinkTimelineLane::Direction => direction
            .as_ref()
            .filter(|(from, to)| !from.is_empty() || !to.is_empty())
            .map(|(from, to)| format!("{from} → {to}"))
            .unwrap_or_else(|| "Unknown direction".into()),
    }
}

pub(super) fn timeline_range(pending: &[PendingEntry]) -> Option<(u64, u64)> {
    let mut times = Vec::new();
    for item in pending {
        if let Some(first) = item.first_ms {
            times.push(first);
            if let Some(LinkSpanEnd::WithdrawalAt(end) | LinkSpanEnd::OpenAt(end)) = item.end {
                times.push(end);
            }
        }
    }
    Some((*times.iter().min()?, *times.iter().max()?))
}

pub(super) fn time_x(time: u64, minimum: u64, maximum: u64, timeline: Timeline) -> f32 {
    if minimum == maximum {
        return timeline.origin.x;
    }
    let offset = (time - minimum) as f64;
    let span = (maximum - minimum) as f64;
    timeline.origin.x + (offset / span) as f32 * timeline.axis_length
}

pub(super) fn time_x_for_pending(time: u64, range: Option<(u64, u64)>, timeline: Timeline) -> f32 {
    range.map_or(timeline.origin.x, |(minimum, maximum)| {
        time_x(time, minimum, maximum, timeline)
    })
}

pub(super) fn timeline_rows(
    pending: &[PendingEntry],
    timeline: Timeline,
) -> (Vec<(usize, usize, f32)>, Vec<String>, Vec<OpenedLinkLane>) {
    let time_range = timeline_range(pending);
    let mut grouped = BTreeMap::<(bool, String), Vec<usize>>::new();
    for (index, item) in pending.iter().enumerate() {
        let undated = item.first_ms.is_none();
        grouped
            .entry((undated, item.lane_label.clone()))
            .or_default()
            .push(index);
    }
    let mut lane_rows = vec![(0, 0, timeline.origin.y); pending.len()];
    let mut lane_labels = Vec::new();
    let mut lanes = Vec::new();
    let mut next_row = 0usize;
    for ((undated, label), mut members) in grouped {
        // Dated lanes sort before the always-explicit undated lane.
        members.sort_by_key(|index| {
            let item = &pending[*index];
            (
                item.first_ms,
                item.source.link_index,
                item.source.bucket_index,
            )
        });
        let lane_index = lanes.len();
        // Every disclosed member gets its own row within the semantic lane.
        // Different start times can still have overlapping spans, so grouping
        // rows only by equal x would let independent assertions obscure each
        // other in the host renderer.
        for (row, index) in members.iter().enumerate() {
            lane_rows[*index] = (
                lane_index,
                row,
                timeline.origin.y + (next_row + row) as f32 * timeline.row_gap,
            );
        }
        let row_count = members.len().max(1);
        lane_labels.push(label.clone());
        lanes.push(OpenedLinkLane {
            index: lane_index,
            label,
            first_row: next_row,
            row_count,
            undated,
        });
        next_row += row_count;
    }
    (lane_rows, lane_labels, lanes)
}
