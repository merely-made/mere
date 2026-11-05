// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

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
    assert!(opened.entries.iter().any(|entry| {
        entry
            .span
            .is_some_and(|span| span.end == LinkSpanEnd::OpenTimeUnknown && span.x_end.is_none())
    }));
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
