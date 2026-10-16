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
            for (summary_index, summary) in bucket.legacy_summaries.iter().enumerate() {
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

fn timeline_group(
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

fn timeline_range(pending: &[PendingEntry]) -> Option<(u64, u64)> {
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

fn time_x(time: u64, minimum: u64, maximum: u64, timeline: Timeline) -> f32 {
    if minimum == maximum {
        return timeline.origin.x;
    }
    let offset = (time - minimum) as f64;
    let span = (maximum - minimum) as f64;
    timeline.origin.x + (offset / span) as f32 * timeline.axis_length
}

fn time_x_for_pending(time: u64, range: Option<(u64, u64)>, timeline: Timeline) -> f32 {
    range.map_or(timeline.origin.x, |(minimum, maximum)| {
        time_x(time, minimum, maximum, timeline)
    })
}

fn timeline_rows(
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
mod tests {
    use super::*;
    use sceno::Stack;
    use scenograph::{
        PublicSourceRevision, SourceBinding,
        link::{
            LinkAssertionBasis, LinkBucket, LinkCoverage, LinkCoverageLayer, LinkCoverageLimit,
            LinkEndpoint, LinkGraphScope, LinkKey, LinkLegacyMember, LinkReading, LinkSource,
            LinkSourceRevision, LinkStratum, LinkWithdrawal,
        },
    };

    fn reading() -> LinkPairReading {
        let assertion = |id: &str,
                         at: Option<u64>,
                         end: Option<Option<u64>>,
                         asserter: Option<&str>| LinkAssertion {
            id: id.into(),
            predicate: "urn:pred:related".into(),
            recognized_kind: Some("citation".into()),
            label: None,
            scope: LinkGraphScope::Default,
            asserter: asserter.map(str::to_owned),
            first_asserted_ms: at,
            asserted_at_ms: at,
            withdrawn: end.map(|at_ms| LinkWithdrawal { at_ms, by: None }),
            basis: LinkAssertionBasis::default(),
        };
        LinkPairReading {
            source: LinkSource::Available {
                binding: SourceBinding {
                    authority: "mere.test".into(),
                    domain: "graph".into(),
                    resource: "g1".into(),
                },
                revision: LinkSourceRevision::Available {
                    revision: PublicSourceRevision::new("rev-7"),
                },
            },
            links: vec![
                LinkReading {
                    key: LinkKey::new(LinkStratum::Surface, "a", "b"),
                    buckets: vec![
                        LinkBucket {
                            displayed_from: "a".into(),
                            displayed_to: "b".into(),
                            source_from: LinkEndpoint {
                                id: "a".into(),
                                address: None,
                            },
                            source_to: LinkEndpoint {
                                id: "b".into(),
                                address: None,
                            },
                            assertions: vec![
                                assertion("id-1", Some(10), None, Some("writer-a")),
                                assertion("id-2", Some(10), Some(Some(30)), None),
                            ],
                            legacy_members: vec![LinkLegacyMember {
                                family: "old-family".into(),
                                sub_kind: None,
                            }],
                            legacy_summaries: Vec::new(),
                        },
                        LinkBucket {
                            displayed_from: "b".into(),
                            displayed_to: "a".into(),
                            source_from: LinkEndpoint {
                                id: "b".into(),
                                address: None,
                            },
                            source_to: LinkEndpoint {
                                id: "a".into(),
                                address: None,
                            },
                            assertions: vec![assertion("id-3", None, None, None)],
                            legacy_members: Vec::new(),
                            legacy_summaries: Vec::new(),
                        },
                        LinkBucket {
                            displayed_from: "a".into(),
                            displayed_to: "b".into(),
                            source_from: LinkEndpoint {
                                id: "a-parallel".into(),
                                address: None,
                            },
                            source_to: LinkEndpoint {
                                id: "b-parallel".into(),
                                address: None,
                            },
                            assertions: vec![assertion("id-5", Some(10), None, Some("writer-a"))],
                            legacy_members: Vec::new(),
                            legacy_summaries: Vec::new(),
                        },
                    ],
                },
                LinkReading {
                    key: LinkKey::new(LinkStratum::Resource, "urn:r:a", "urn:r:b"),
                    buckets: vec![LinkBucket {
                        displayed_from: "a".into(),
                        displayed_to: "b".into(),
                        source_from: LinkEndpoint {
                            id: "ra".into(),
                            address: Some("urn:r:a".into()),
                        },
                        source_to: LinkEndpoint {
                            id: "rb".into(),
                            address: Some("urn:r:b".into()),
                        },
                        assertions: vec![assertion("id-4", Some(20), None, Some("writer-b"))],
                        legacy_members: Vec::new(),
                        legacy_summaries: Vec::new(),
                    }],
                },
            ],
            coverage: LinkCoverage {
                limits: vec![LinkCoverageLimit {
                    layer: LinkCoverageLayer::Disclosure,
                    reason: "partial fixture".into(),
                    resources: Vec::new(),
                    surfaces: vec!["a".into()],
                    count: Some(1),
                }],
            },
        }
    }

    #[test]
    fn timeline_preserves_exact_parallel_antiparallel_and_legacy_references() {
        let input = reading();
        let layout = OpenedLinkLayout::Timeline {
            lane_by: LinkTimelineLane::Direction,
            timeline: Timeline {
                origin: Vec2::new(20.0, 30.0),
                axis_length: 400.0,
                row_gap: 32.0,
                fallback: TimelineFallback::LeaveInPlace,
            },
        };
        let opened = compile_opened_link(&input, &layout, OpenedLinkOptions::default()).unwrap();
        assert_eq!(opened.entries.len(), 6);
        assert_eq!(opened.scene.items.len(), 6);
        assert_eq!(opened.reading, input);
        assert_eq!(opened.time_range_ms, Some((10, 30)));
        let ids: Vec<_> = opened
            .entries
            .iter()
            .filter_map(|entry| match opened.resolve(entry) {
                Some(OpenedLinkMember::Assertion(assertion)) => Some(assertion.id.as_str()),
                _ => None,
            })
            .collect();
        assert!(
            ids.contains(&"id-1")
                && ids.contains(&"id-2")
                && ids.contains(&"id-3")
                && ids.contains(&"id-5")
        );
        assert!(opened.entries.iter().any(|entry| matches!(
            opened.resolve(entry),
            Some(OpenedLinkMember::LegacyMember(_))
        )));
        assert!(
            opened
                .lanes
                .iter()
                .any(|lane| lane.label == "Undated" && lane.undated)
        );
        assert!(
            opened
                .entries
                .iter()
                .any(|entry| entry.span.is_some_and(|span| {
                    span.end == LinkSpanEnd::WithdrawalAt(30) && span.x_end == Some(420.0)
                }))
        );
        assert!(
            opened
                .entries
                .iter()
                .any(|entry| entry.span.is_some_and(|span| {
                    span.end == LinkSpanEnd::OpenTimeUnknown && span.x_end.is_none()
                }))
        );
        for entry in &opened.entries {
            let item = &opened.scene.items[entry.instance.0 as usize];
            if let Some(span) = entry.span {
                assert_eq!(item.transform.translate.x, span.x_start);
                assert_eq!(item.transform.translate.y, span.y);
            }
        }
        let same_instant_rows: Vec<_> = opened
            .entries
            .iter()
            .filter(|entry| {
                matches!(
                    opened.resolve(entry),
                    Some(OpenedLinkMember::Assertion(assertion))
                        if matches!(assertion.id.as_str(), "id-1" | "id-2" | "id-5")
                )
            })
            .map(|entry| entry.span.expect("dated member").y)
            .collect();
        assert_eq!(same_instant_rows.len(), 3);
        assert_ne!(same_instant_rows[0], same_instant_rows[1]);
        assert_ne!(same_instant_rows[1], same_instant_rows[2]);
    }

    #[test]
    fn explicit_now_ends_only_held_spans_and_never_stamps_the_reading() {
        let input = reading();
        let now = 40;
        let layout = OpenedLinkLayout::Timeline {
            lane_by: LinkTimelineLane::Asserter,
            timeline: Timeline::default(),
        };
        let opened =
            compile_opened_link(&input, &layout, OpenedLinkOptions { now_ms: Some(now) }).unwrap();
        assert_eq!(opened.reading, input);
        assert!(opened.entries.iter().any(|entry| {
            entry
                .span
                .is_some_and(|span| span.end == LinkSpanEnd::OpenAt(now))
        }));
        assert!(
            opened
                .lanes
                .iter()
                .any(|lane| lane.label == "Unknown asserter")
        );
    }

    #[test]
    fn all_dated_members_share_the_full_interval_axis_at_large_clock_values() {
        let mut input = reading();
        let base = u64::MAX - 100;
        for link in &mut input.links {
            for bucket in &mut link.buckets {
                for assertion in &mut bucket.assertions {
                    assertion.first_asserted_ms = Some(base);
                    assertion.withdrawn = None;
                }
            }
        }
        input.links[0].buckets[0].assertions[0].withdrawn = Some(LinkWithdrawal {
            at_ms: Some(base + 10),
            by: None,
        });
        let layout = OpenedLinkLayout::Timeline {
            lane_by: LinkTimelineLane::Family,
            timeline: Timeline {
                origin: Vec2::new(5.0, 0.0),
                axis_length: 300.0,
                row_gap: 30.0,
                fallback: TimelineFallback::StackBelowOrigin,
            },
        };
        let opened = compile_opened_link(&input, &layout, OpenedLinkOptions::default()).unwrap();
        let first = opened
            .entries
            .iter()
            .find(|entry| {
                matches!(
                    opened.resolve(entry),
                    Some(OpenedLinkMember::Assertion(assertion)) if assertion.id == "id-1"
                )
            })
            .unwrap()
            .span
            .unwrap();
        let second = opened
            .entries
            .iter()
            .find(|entry| {
                matches!(
                    opened.resolve(entry),
                    Some(OpenedLinkMember::Assertion(assertion)) if assertion.id == "id-2"
                )
            })
            .unwrap()
            .span
            .unwrap();
        assert!(first.x_end.unwrap() > first.x_start);
        assert_eq!(first.x_start, second.x_start);
        assert_eq!(opened.time_range_ms, Some((base, base + 10)));
    }

    #[test]
    fn timeline_scene_item_start_matches_span_at_large_clock_values() {
        let mut input = reading();
        let base = u64::MAX - 9_000;
        for link in &mut input.links {
            for bucket in &mut link.buckets {
                for assertion in &mut bucket.assertions {
                    assertion.first_asserted_ms = Some(base);
                    assertion.withdrawn = None;
                }
            }
        }
        input.links[0].buckets[0].assertions[0].withdrawn = Some(LinkWithdrawal {
            at_ms: Some(base + 8_000),
            by: None,
        });
        let layout = OpenedLinkLayout::Timeline {
            lane_by: LinkTimelineLane::Family,
            timeline: Timeline {
                origin: Vec2::new(17.0, 0.0),
                axis_length: 240.0,
                row_gap: 24.0,
                fallback: TimelineFallback::StackBelowOrigin,
            },
        };
        let opened = compile_opened_link(&input, &layout, OpenedLinkOptions::default()).unwrap();
        let entry = opened
            .entries
            .iter()
            .find(|entry| {
                matches!(
                    opened.resolve(entry),
                    Some(OpenedLinkMember::Assertion(assertion)) if assertion.id == "id-1"
                )
            })
            .unwrap();
        let span = entry.span.unwrap();
        let item = &opened.scene.items[entry.instance.0 as usize];
        assert_eq!(item.transform.translate.x, span.x_start);
        assert!(span.x_end.unwrap() > span.x_start);
    }

    #[test]
    fn timeline_preserves_backward_withdrawal_and_observation_times() {
        let mut input = reading();
        let withdrawn = &mut input.links[0].buckets[0].assertions[0];
        withdrawn.first_asserted_ms = Some(40);
        withdrawn.withdrawn = Some(LinkWithdrawal {
            at_ms: Some(20),
            by: Some("moderator".into()),
        });
        let held = &mut input.links[0].buckets[0].assertions[1];
        held.first_asserted_ms = Some(50);
        held.asserted_at_ms = Some(50);
        held.withdrawn = None;

        let layout = OpenedLinkLayout::Timeline {
            lane_by: LinkTimelineLane::Asserter,
            timeline: Timeline::default(),
        };
        let opened =
            compile_opened_link(&input, &layout, OpenedLinkOptions { now_ms: Some(10) }).unwrap();
        let withdrawn_entry = opened
            .entries
            .iter()
            .find(|entry| {
                matches!(
                    opened.resolve(entry),
                    Some(OpenedLinkMember::Assertion(assertion)) if assertion.id == "id-1"
                )
            })
            .unwrap();
        let withdrawn_span = withdrawn_entry.span.unwrap();
        assert_eq!(withdrawn_span.start_ms, 40);
        assert_eq!(withdrawn_span.end, LinkSpanEnd::WithdrawalAt(20));
        assert!(withdrawn_span.x_end.unwrap() < withdrawn_span.x_start);
        assert_eq!(
            opened.scene.items[withdrawn_entry.instance.0 as usize]
                .transform
                .translate
                .x,
            withdrawn_span.x_start
        );

        let held_entry = opened
            .entries
            .iter()
            .find(|entry| {
                matches!(
                    opened.resolve(entry),
                    Some(OpenedLinkMember::Assertion(assertion)) if assertion.id == "id-2"
                )
            })
            .unwrap();
        let held_span = held_entry.span.unwrap();
        assert_eq!(held_span.start_ms, 50);
        assert_eq!(held_span.end, LinkSpanEnd::OpenAt(10));
        assert!(held_span.x_end.unwrap() < held_span.x_start);
        assert_eq!(opened.time_range_ms, Some((10, 50)));
        assert_eq!(
            opened.reading, input,
            "the disclosed source times stay intact"
        );
    }

    #[test]
    fn overlapping_spans_with_different_starts_get_distinct_rows() {
        let mut input = reading();
        let first = &mut input.links[0].buckets[0].assertions[0];
        first.first_asserted_ms = Some(10);
        first.withdrawn = Some(LinkWithdrawal {
            at_ms: Some(40),
            by: None,
        });
        let second = &mut input.links[0].buckets[0].assertions[1];
        second.first_asserted_ms = Some(20);
        second.withdrawn = Some(LinkWithdrawal {
            at_ms: Some(30),
            by: None,
        });

        let opened = compile_opened_link(
            &input,
            &OpenedLinkLayout::Timeline {
                lane_by: LinkTimelineLane::Direction,
                timeline: Timeline::default(),
            },
            OpenedLinkOptions::default(),
        )
        .unwrap();
        let first_entry = opened
            .entries
            .iter()
            .find(|entry| {
                matches!(
                    opened.resolve(entry),
                    Some(OpenedLinkMember::Assertion(assertion)) if assertion.id == "id-1"
                )
            })
            .unwrap();
        let second_entry = opened
            .entries
            .iter()
            .find(|entry| {
                matches!(
                    opened.resolve(entry),
                    Some(OpenedLinkMember::Assertion(assertion)) if assertion.id == "id-2"
                )
            })
            .unwrap();
        let first_span = first_entry.span.unwrap();
        let second_span = second_entry.span.unwrap();
        assert_eq!(first_entry.lane_index, second_entry.lane_index);
        assert_ne!(first_entry.row, second_entry.row);
        assert_ne!(first_span.y, second_span.y);
        assert!(first_span.x_start < second_span.x_end.unwrap());
        assert!(second_span.x_start < first_span.x_end.unwrap());
        assert_eq!(
            opened.scene.items[first_entry.instance.0 as usize]
                .transform
                .translate
                .y,
            first_span.y
        );
        assert_eq!(
            opened.scene.items[second_entry.instance.0 as usize]
                .transform
                .translate
                .y,
            second_span.y
        );
    }

    #[test]
    fn timeline_rejects_negative_axis_length_and_row_gap() {
        let input = reading();
        for timeline in [
            Timeline {
                axis_length: -1.0,
                ..Timeline::default()
            },
            Timeline {
                row_gap: -1.0,
                ..Timeline::default()
            },
        ] {
            assert_eq!(
                compile_opened_link(
                    &input,
                    &OpenedLinkLayout::Timeline {
                        lane_by: LinkTimelineLane::Family,
                        timeline,
                    },
                    OpenedLinkOptions::default(),
                )
                .unwrap_err(),
                OpenedLinkCompileError::NegativeTimelineGeometry
            );
        }
    }

    #[test]
    fn default_layout_counts_legacy_and_undated_members_as_undated() {
        let input = reading();
        assert!(matches!(
            OpenedLinkLayout::default_for(&input),
            OpenedLinkLayout::Timeline { .. }
        ));
        let mut legacy_majority = input.clone();
        legacy_majority.links[0].buckets[0]
            .legacy_members
            .push(LinkLegacyMember {
                family: "another-legacy-family".into(),
                sub_kind: None,
            });
        legacy_majority.links[0].buckets[0]
            .legacy_members
            .push(LinkLegacyMember {
                family: "one-more-legacy-family".into(),
                sub_kind: None,
            });
        assert!(matches!(
            OpenedLinkLayout::default_for(&legacy_majority),
            OpenedLinkLayout::List { .. }
        ));
        let mut all_dated = input.clone();
        for link in &mut all_dated.links {
            for bucket in &mut link.buckets {
                bucket.legacy_members.clear();
                bucket.legacy_summaries.clear();
                for assertion in &mut bucket.assertions {
                    assertion.first_asserted_ms = Some(10);
                }
            }
        }
        assert!(matches!(
            OpenedLinkLayout::default_for(&all_dated),
            OpenedLinkLayout::Timeline { .. }
        ));
    }

    #[test]
    fn list_uses_stack_and_groups_by_source_without_merging_members() {
        let input = reading();
        let layout = OpenedLinkLayout::List {
            group_by: scenograph::link::LinkListGroup::Source,
            stack: Stack::default(),
        };
        let opened = compile_opened_link(&input, &layout, OpenedLinkOptions::default()).unwrap();
        assert_eq!(opened.entries.len(), 6);
        assert_eq!(opened.lanes.len(), 2);
        assert_eq!(opened.scene.items.len(), 6);
        assert_eq!(opened.reading.source, input.source);
        assert_eq!(opened.reading.coverage, input.coverage);
    }

    #[test]
    fn link_scope_compiles_only_matching_stratum_and_keeps_original_provenance() {
        let input = reading();
        let selected = input.links[1].key.clone();
        let scope = Scope::Link(selected.clone());
        let layout = OpenedLinkLayout::List {
            group_by: scenograph::link::LinkListGroup::Source,
            stack: Stack::default(),
        };
        let opened =
            compile_opened_link_scope(&scope, &input, &layout, OpenedLinkOptions::default())
                .unwrap();

        assert_eq!(opened.reading, input, "retain the whole source disclosure");
        assert_eq!(opened.reading.links.len(), 2);
        assert_eq!(opened.reading.coverage, reading().coverage);
        assert_eq!(opened.entries.len(), 1);
        let entry = &opened.entries[0];
        assert_eq!(entry.source.link_index, 1, "keep the original source index");
        assert_eq!(opened.reading.links[entry.source.link_index].key, selected);
        assert!(matches!(
            opened.resolve(entry),
            Some(OpenedLinkMember::Assertion(assertion)) if assertion.id == "id-4"
        ));
    }

    #[test]
    fn link_scope_rejects_wrong_scope_and_missing_key() {
        let input = reading();
        let layout = OpenedLinkLayout::List {
            group_by: scenograph::link::LinkListGroup::Family,
            stack: Stack::default(),
        };
        assert_eq!(
            compile_opened_link_scope(
                &Scope::Node("a".into()),
                &input,
                &layout,
                OpenedLinkOptions::default(),
            )
            .unwrap_err(),
            OpenedLinkCompileError::WrongScope
        );
        assert_eq!(
            compile_opened_link_scope(
                &Scope::Link(LinkKey::new(LinkStratum::Surface, "missing-a", "missing-b")),
                &input,
                &layout,
                OpenedLinkOptions::default(),
            )
            .unwrap_err(),
            OpenedLinkCompileError::MissingLink
        );
    }
}
