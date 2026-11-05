// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shared list and Timeline compositions for an opened link reading.

use std::collections::BTreeMap;

use sceno::{
    Arrangement, AxisValue, Footprint, InstanceId, Placement, Representation, Scene, Score,
    ScoreItem, Size2, SourceRef, Timeline, TimelineFallback, Vec2,
};
use scenograph::link::{
    LinkAssertion, LinkKey, LinkPairReading, LinkSpanEnd, LinkSpanGeometry, LinkTimelineLane,
    OpenedLinkEntryRef, OpenedLinkLayout, OpenedLinkMember, OpenedLinkMemberRef, OpenedLinkOptions,
};
use scenograph::swatch::Scope;

/// A named Timeline row. `Undated` is always explicit and sorts last.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenedLinkLane {
    pub index: usize,
    pub label: String,
    pub first_row: usize,
    pub row_count: usize,
    pub undated: bool,
}

/// One placed member, still addressable through its exact source reading.
#[derive(Clone, Debug, PartialEq)]
pub struct OpenedLinkEntry {
    pub source: OpenedLinkEntryRef,
    pub instance: InstanceId,
    /// Display label derived from the exact source member.
    pub label: String,
    pub lane_index: usize,
    pub lane_label: String,
    pub row: usize,
    pub direction: Option<(String, String)>,
    /// Present for dated assertion records. `x_end == None` means the
    /// interval end is unknown, not that the assertion is held.
    pub span: Option<LinkSpanGeometry>,
}

/// The original disclosure, a real Sceno scene, and a stable map from each
/// scene item back to the exact bucket/member it depicts.
#[derive(Clone, Debug, PartialEq)]
pub struct OpenedLinkScene {
    pub reading: LinkPairReading,
    pub scene: Scene,
    pub entries: Vec<OpenedLinkEntry>,
    pub lanes: Vec<OpenedLinkLane>,
    /// Inclusive domain used to map every dated start and known end onto the
    /// same Timeline axis across all lanes.
    pub time_range_ms: Option<(u64, u64)>,
}

impl OpenedLinkScene {
    /// Borrow the exact assertion, legacy member, or legacy summary represented
    /// by one entry. Returned assertion IDs are the only assertion action keys.
    pub fn resolve(&self, entry: &OpenedLinkEntry) -> Option<OpenedLinkMember<'_>> {
        entry.source.resolve(&self.reading)
    }

    pub fn resolve_ref(&self, source: &OpenedLinkEntryRef) -> Option<OpenedLinkMember<'_>> {
        source.resolve(&self.reading)
    }
}

/// Invalid finite layout geometry refused before any scene is returned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenedLinkCompileError {
    NonFiniteLayoutGeometry,
    NegativeTimelineGeometry,
    WrongScope,
    MissingLink,
    TooManyMembers,
}

impl std::fmt::Display for OpenedLinkCompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonFiniteLayoutGeometry => {
                f.write_str("opened-link layout geometry must be finite")
            },
            Self::NegativeTimelineGeometry => {
                f.write_str("opened-link Timeline axis length and row gap must be nonnegative")
            },
            Self::WrongScope => f.write_str("opened-link compilation requires a Link scope"),
            Self::MissingLink => f.write_str("the scoped link is absent from the source reading"),
            Self::TooManyMembers => {
                f.write_str("opened-link member count exceeds scene index bounds")
            },
        }
    }
}

impl std::error::Error for OpenedLinkCompileError {}

#[derive(Clone, Debug)]
struct PendingEntry {
    source: OpenedLinkEntryRef,
    source_ref: SourceRef,
    label: String,
    lane_label: String,
    direction: Option<(String, String)>,
    first_ms: Option<u64>,
    end: Option<LinkSpanEnd>,
    group_index: usize,
}

/// Compile a disclosed link into list or Timeline placement while retaining
/// the exact source reading. The compiler neither reads graph truth nor
/// supplies a clock; callers pass `now_ms` explicitly when they want held
/// spans to end at a known observation time.
pub fn compile_opened_link(
    reading: &LinkPairReading,
    layout: &OpenedLinkLayout,
    options: OpenedLinkOptions,
) -> Result<OpenedLinkScene, OpenedLinkCompileError> {
    compile_opened_link_filtered(reading, layout, options, None)
}

/// Compile only the exact stratum link named by a swatch scope while retaining
/// the complete disclosed reading as the authority for source and coverage.
/// Entry references continue to use indexes into the original reading.
pub fn compile_opened_link_scope(
    scope: &Scope,
    reading: &LinkPairReading,
    layout: &OpenedLinkLayout,
    options: OpenedLinkOptions,
) -> Result<OpenedLinkScene, OpenedLinkCompileError> {
    let Scope::Link(key) = scope else {
        return Err(OpenedLinkCompileError::WrongScope);
    };
    if !reading.links.iter().any(|link| &link.key == key) {
        return Err(OpenedLinkCompileError::MissingLink);
    }
    compile_opened_link_filtered(reading, layout, options, Some(key))
}

fn compile_opened_link_filtered(
    reading: &LinkPairReading,
    layout: &OpenedLinkLayout,
    options: OpenedLinkOptions,
    link_filter: Option<&LinkKey>,
) -> Result<OpenedLinkScene, OpenedLinkCompileError> {
    match layout {
        OpenedLinkLayout::List { stack, .. } => {
            if ![
                stack.layer_gap,
                stack.row_gap,
                stack.center.x,
                stack.center.y,
            ]
            .into_iter()
            .all(f32::is_finite)
            {
                return Err(OpenedLinkCompileError::NonFiniteLayoutGeometry);
            }
        },
        OpenedLinkLayout::Timeline { timeline, .. } => {
            if ![
                timeline.origin.x,
                timeline.origin.y,
                timeline.axis_length,
                timeline.row_gap,
            ]
            .into_iter()
            .all(f32::is_finite)
            {
                return Err(OpenedLinkCompileError::NonFiniteLayoutGeometry);
            }
            if timeline.axis_length < 0.0 || timeline.row_gap < 0.0 {
                return Err(OpenedLinkCompileError::NegativeTimelineGeometry);
            }
        },
    }

    let mut pending = collect_entries(reading, layout, options, link_filter);
    if matches!(layout, OpenedLinkLayout::List { .. }) {
        // The accessible list is grouped first, then reads each group in time
        // order with undated legacy/current records last.
        pending.sort_by_key(|item| {
            (
                item.lane_label.clone(),
                item.first_ms.is_none(),
                item.first_ms.unwrap_or_default(),
            )
        });
    }
    if pending.len() > u32::MAX as usize {
        return Err(OpenedLinkCompileError::TooManyMembers);
    }

    let mut labels: Vec<String> = pending.iter().map(|item| item.lane_label.clone()).collect();
    labels.sort();
    labels.dedup();
    let label_index: BTreeMap<String, usize> = labels
        .iter()
        .enumerate()
        .map(|(index, label)| (label.clone(), index))
        .collect();
    for item in &mut pending {
        item.group_index = label_index[&item.lane_label];
    }

    let time_range_ms = match layout {
        OpenedLinkLayout::Timeline { .. } => timeline_range(&pending),
        OpenedLinkLayout::List { .. } => None,
    };
    let arrangement = match layout {
        OpenedLinkLayout::List { stack, .. } => Arrangement::Stack(*stack),
        OpenedLinkLayout::Timeline { timeline, .. } => {
            let mut configured = *timeline;
            if let Some((minimum, maximum)) = time_range_ms {
                let first_min = pending.iter().filter_map(|item| item.first_ms).min();
                let first_max = pending.iter().filter_map(|item| item.first_ms).max();
                if let (Some(first_min), Some(first_max)) = (first_min, first_max) {
                    configured.origin.x = time_x(first_min, minimum, maximum, *timeline);
                    configured.axis_length =
                        time_x(first_max, minimum, maximum, *timeline) - configured.origin.x;
                }
            }
            configured.fallback = TimelineFallback::StackBelowOrigin;
            Arrangement::Timeline(configured)
        },
    };

    let mut score = Score::new(arrangement);
    let first_minimum = pending.iter().filter_map(|item| item.first_ms).min();
    score.items = pending
        .iter()
        .enumerate()
        .map(|(ordinal, item)| ScoreItem {
            source: item.source_ref.clone(),
            ordinal: ordinal as u32,
            footprint: Footprint::Rect {
                size: Size2::new(40.0, 24.0),
            },
            representation: Representation::Card,
            placement: Placement::Ordinal,
            layer: 0,
            visible: true,
            axis: match layout {
                OpenedLinkLayout::List { .. } => Some(AxisValue::Numeric(item.group_index as f64)),
                OpenedLinkLayout::Timeline { .. } => item.first_ms.map(|value| {
                    AxisValue::Numeric((value - first_minimum.unwrap_or(value)) as f64)
                }),
            },
            embedding: None,
            weight: None,
        })
        .collect();
    let mut scene = crate::solve(&score);

    let mut lanes = Vec::new();
    let mut entries = Vec::with_capacity(pending.len());
    match layout {
        OpenedLinkLayout::List { .. } => {
            let mut row_by_lane = BTreeMap::<String, usize>::new();
            for (index, item) in pending.iter().enumerate() {
                let row = row_by_lane.entry(item.lane_label.clone()).or_default();
                let entry_row = *row;
                *row += 1;
                lanes.push_once(item.group_index, &item.lane_label, entry_row, false);
                entries.push(make_entry(
                    index,
                    item,
                    item.group_index,
                    entry_row,
                    &item.lane_label,
                    item.direction.clone(),
                    None,
                ));
            }
            lanes.sort_by_key(|lane| lane.index);
        },
        OpenedLinkLayout::Timeline { timeline, .. } => {
            let (lane_rows, lane_indexes, lane_specs) = timeline_rows(&pending, *timeline);
            lanes = lane_specs;
            for (index, item) in pending.iter().enumerate() {
                let (lane_index, row, y) = lane_rows[index];
                let x_start = item.first_ms.map_or(timeline.origin.x, |start_ms| {
                    time_x_for_pending(start_ms, time_range_ms, *timeline)
                });
                // The shared Timeline solver positions dated score items on
                // its own axis scale. Use the same integer-relative mapping as
                // span geometry so renderers and entry geometry agree exactly,
                // including when absolute clock values exceed f64 precision.
                scene.items[index].transform.translate.x = x_start;
                scene.items[index].transform.translate.y = y;
                let span = item.first_ms.map(|start_ms| {
                    let end = item.end.unwrap_or(LinkSpanEnd::OpenTimeUnknown);
                    let end_ms = match end {
                        LinkSpanEnd::WithdrawalAt(time) | LinkSpanEnd::OpenAt(time) => Some(time),
                        LinkSpanEnd::WithdrawalTimeUnknown | LinkSpanEnd::OpenTimeUnknown => None,
                    };
                    LinkSpanGeometry {
                        start_ms,
                        end,
                        x_start,
                        x_end: end_ms
                            .map(|time| time_x_for_pending(time, time_range_ms, *timeline)),
                        y,
                    }
                });
                entries.push(make_entry(
                    index,
                    item,
                    lane_index,
                    row,
                    &lane_indexes[lane_index],
                    item.direction.clone(),
                    span,
                ));
            }
        },
    }
    if scene.items.iter().any(|item| {
        !item.transform.translate.x.is_finite() || !item.transform.translate.y.is_finite()
    }) || entries.iter().any(|entry| {
        entry.span.is_some_and(|span| {
            !span.x_start.is_finite()
                || span.x_end.is_some_and(|x| !x.is_finite())
                || !span.y.is_finite()
        })
    }) {
        return Err(OpenedLinkCompileError::NonFiniteLayoutGeometry);
    }
    update_scene_bounds(&mut scene);

    Ok(OpenedLinkScene {
        reading: reading.clone(),
        scene,
        entries,
        lanes,
        time_range_ms,
    })
}

fn make_entry(
    index: usize,
    item: &PendingEntry,
    lane_index: usize,
    row: usize,
    lane_label: &str,
    direction: Option<(String, String)>,
    span: Option<LinkSpanGeometry>,
) -> OpenedLinkEntry {
    OpenedLinkEntry {
        source: item.source.clone(),
        instance: InstanceId(index as u32),
        label: item.label.clone(),
        lane_index,
        lane_label: lane_label.to_owned(),
        row,
        direction,
        span,
    }
}

trait PushLaneOnce {
    fn push_once(&mut self, index: usize, label: &str, row: usize, undated: bool);
}

impl PushLaneOnce for Vec<OpenedLinkLane> {
    fn push_once(&mut self, index: usize, label: &str, row: usize, undated: bool) {
        if let Some(lane) = self.iter_mut().find(|lane| lane.index == index) {
            lane.row_count = lane.row_count.max(row + 1);
        } else {
            self.push(OpenedLinkLane {
                index,
                label: label.to_owned(),
                first_row: 0,
                row_count: row + 1,
                undated,
            });
        }
    }
}

fn collect_entries(
    reading: &LinkPairReading,
    layout: &OpenedLinkLayout,
    options: OpenedLinkOptions,
    link_filter: Option<&LinkKey>,
) -> Vec<PendingEntry> {
    let mut pending = Vec::new();
    for (link_index, link) in reading.links.iter().enumerate() {
        if link_filter.is_some_and(|key| key != &link.key) {
            continue;
        }
        for (bucket_index, bucket) in link.buckets.iter().enumerate() {
            let direction = Some((bucket.displayed_from.clone(), bucket.displayed_to.clone()));
            for (assertion_index, assertion) in bucket.assertions.iter().enumerate() {
                let group = match layout {
                    OpenedLinkLayout::List { group_by, .. } => list_group(
                        *group_by,
                        link.key.stratum,
                        &link.key.first,
                        &link.key.second,
                        Some(assertion),
                        None,
                    ),
                    OpenedLinkLayout::Timeline { lane_by, .. } => {
                        timeline_group(*lane_by, Some(assertion), None, &direction)
                    },
                };
                let source = OpenedLinkEntryRef {
                    link_index,
                    bucket_index,
                    member: OpenedLinkMemberRef::Assertion(assertion_index),
                };
                pending.push(PendingEntry {
                    source_ref: scene_source(&source),
                    source,
                    label: assertion.predicate.clone(),
                    lane_label: group,
                    direction: direction.clone(),
                    first_ms: assertion.first_asserted_ms,
                    end: Some(match &assertion.withdrawn {
                        Some(withdrawal) => withdrawal.at_ms.map_or(
                            LinkSpanEnd::WithdrawalTimeUnknown,
                            LinkSpanEnd::WithdrawalAt,
                        ),
                        None => options
                            .now_ms
                            .map_or(LinkSpanEnd::OpenTimeUnknown, LinkSpanEnd::OpenAt),
                    }),
                    group_index: 0,
                });
            }
            for (member_index, member) in bucket.legacy_members.iter().enumerate() {
                let group = match layout {
                    OpenedLinkLayout::List { group_by, .. } => list_group(
                        *group_by,
                        link.key.stratum,
                        &link.key.first,
                        &link.key.second,
                        None,
                        Some(&member.family),
                    ),
                    OpenedLinkLayout::Timeline { lane_by, .. } => {
                        timeline_group(*lane_by, None, Some(&member.family), &direction)
                    },
                };
                let source = OpenedLinkEntryRef {
                    link_index,
                    bucket_index,
                    member: OpenedLinkMemberRef::LegacyMember(member_index),
                };
                pending.push(PendingEntry {
                    source_ref: scene_source(&source),
                    source,
                    label: member.family.clone(),
                    lane_label: group,
                    direction: direction.clone(),
                    first_ms: None,
                    end: None,
                    group_index: 0,
                });
            }
            for (summary_index, summary) in bucket.unrepresented_legacy_summaries() {
                let group = match layout {
                    OpenedLinkLayout::List { group_by, .. } => list_group(
                        *group_by,
                        link.key.stratum,
                        &link.key.first,
                        &link.key.second,
                        None,
                        Some(&summary.family),
                    ),
                    OpenedLinkLayout::Timeline { lane_by, .. } => {
                        timeline_group(*lane_by, None, Some(&summary.family), &direction)
                    },
                };
                let source = OpenedLinkEntryRef {
                    link_index,
                    bucket_index,
                    member: OpenedLinkMemberRef::LegacySummary(summary_index),
                };
                pending.push(PendingEntry {
                    source_ref: scene_source(&source),
                    source,
                    label: summary.family.clone(),
                    lane_label: group,
                    direction: direction.clone(),
                    first_ms: None,
                    end: None,
                    group_index: 0,
                });
            }
        }
    }
    pending
}

fn scene_source(source: &OpenedLinkEntryRef) -> SourceRef {
    SourceRef::new(
        "scenograph.opened_link",
        serde_json::to_string(source).expect("entry references serialize"),
    )
}

fn list_group(
    group_by: scenograph::link::LinkListGroup,
    stratum: scenograph::link::LinkStratum,
    first: &str,
    second: &str,
    assertion: Option<&LinkAssertion>,
    legacy_family: Option<&str>,
) -> String {
    use scenograph::link::LinkListGroup;
    match group_by {
        LinkListGroup::Family => assertion
            .and_then(|item| item.recognized_kind.as_deref())
            .or(legacy_family)
            .or_else(|| assertion.map(|item| item.predicate.as_str()))
            .filter(|value| !value.is_empty())
            .unwrap_or("Unknown family")
            .to_owned(),
        LinkListGroup::Source => format!(
            "{} · {} ↔ {}",
            match stratum {
                scenograph::link::LinkStratum::Resource => "Resource",
                scenograph::link::LinkStratum::Surface => "Surface",
            },
            first,
            second
        ),
    }
}

fn update_scene_bounds(scene: &mut Scene) {
    let mut bounds = None;
    for item in &scene.items {
        let Some(local) = item.footprint.bounds() else {
            continue;
        };
        let transform = item.transform;
        let origin = transform.apply(local.origin);
        let opposite = transform.apply(Vec2::new(
            local.origin.x + local.size.w,
            local.origin.y + local.size.h,
        ));
        let rect = sceno::Rect::new(
            Vec2::new(origin.x.min(opposite.x), origin.y.min(opposite.y)),
            Size2::new((origin.x - opposite.x).abs(), (origin.y - opposite.y).abs()),
        );
        bounds = Some(bounds.map_or(rect, |old: sceno::Rect| old.union(rect)));
    }
    scene.bounds = bounds.unwrap_or_default();
}

#[cfg(test)]
#[path = "opened_link/tests.rs"]
mod tests;

#[path = "opened_link/timeline.rs"]
mod timeline;

use timeline::timeline_group;

use timeline::timeline_range;

use timeline::time_x;

use timeline::time_x_for_pending;

use timeline::timeline_rows;
