// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn native_wall_partition_is_serialized_without_becoming_ui_thread_work() {
    let mut costs = Costs::default();
    costs.begin("native").unwrap();
    let profile = FrameProfile {
        total_us: 49,
        native_submit_us: 17,
        native_collect_us: 29,
        native_latency_us: 120_048,
        native_timing_available: true,
        native_preseal_us: 17,
        native_queue_us: 2,
        native_worker_us: 80_000,
        native_ready_to_collect_us: 40_000,
        native_postcollect_us: 29,
        native_timeline_us: 120_048,
        a11y_us: 3,
        a11y_publish_us: 2,
        a11y_actions_us: 1,
        ..Default::default()
    };
    costs
        .observe(1, Some(profile), Totals::default(), false, true, false)
        .unwrap();
    costs.end().unwrap();
    let phase = costs.phases[0].report(true);
    assert_eq!(phase.native_timing_frames, 1);
    assert_eq!(phase.observed_ui_thread_us, 32);
    assert_eq!(phase.all_frames_us["native_worker_us"].median, 80_000.);
    assert_eq!(
        phase.all_frames_us["native_ready_to_collect_us"].median,
        40_000.
    );
    assert_eq!(phase.all_frames_us["native_timeline_us"].median, 120_048.);
    assert_eq!(phase.all_frames_us["a11y_publish_us"].median, 2.);
    assert_eq!(phase.all_frames_us["a11y_actions_us"].median, 1.);
    let report = costs.report();
    let sample = &report["phases"][0]["samples"][0];
    assert_eq!(sample["native_timing_available"], true);
    assert_eq!(sample["times_us"]["native_queue_us"], 2);
    assert_eq!(sample["times_us"]["ui_attempt_us"], 32);
}

#[test]
fn unavailable_native_partition_is_not_reported_as_zero_latency() {
    let mut costs = Costs::default();
    costs.begin("mixed").unwrap();
    for (frame, profile) in [
        (
            1,
            FrameProfile {
                total_us: 10,
                ..Default::default()
            },
        ),
        (
            2,
            FrameProfile {
                total_us: 8,
                native_collect_us: 8,
                native_latency_us: 0,
                native_timing_available: true,
                ..Default::default()
            },
        ),
    ] {
        costs
            .observe(frame, Some(profile), Totals::default(), false, true, false)
            .unwrap();
    }
    costs.end().unwrap();
    let phase = costs.phases[0].report(true);
    assert_eq!(phase.eligible_frames, 2);
    assert_eq!(phase.native_timing_frames, 1);
    assert_eq!(phase.all_frames_us["native_queue_us"].count, 1);
    assert_eq!(phase.all_frames_us["native_queue_us"].median, 0.);
    assert!(!phase.samples[0].native_timing_available);
    assert!(phase.samples[1].native_timing_available);

    let mut legacy = Costs::default();
    legacy.begin("legacy").unwrap();
    legacy
        .observe(
            1,
            Some(FrameProfile::default()),
            Totals::default(),
            false,
            true,
            false,
        )
        .unwrap();
    legacy.end().unwrap();
    let report = legacy.phases[0].report(true);
    assert_eq!(report.native_timing_frames, 0);
    assert!(!report.all_frames_us.contains_key("native_queue_us"));
    assert!(
        !report
            .steady_after_first_us
            .contains_key("native_worker_us")
    );
}
