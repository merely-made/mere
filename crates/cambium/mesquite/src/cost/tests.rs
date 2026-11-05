// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
fn profile(total: u64) -> Option<FrameProfile> {
    Some(FrameProfile {
        total_us: total,
        ..FrameProfile::default()
    })
}
#[test]
fn percentile_uses_nearest_rank_and_even_median_is_not_truncated() {
    assert!(distribution([]).is_none());
    let d = distribution([7]);
    assert_eq!(d.unwrap().p95, 7);
    let d = distribution([4, 1, 3, 2]).unwrap();
    assert_eq!(d.median, 2.5);
    assert_eq!(d.p95, 4);
    assert_eq!(d.max, 4);
    let d = distribution((1..=100).rev()).unwrap();
    assert_eq!(d.median, 50.5);
    assert_eq!(d.p95, 95);
    assert_eq!(d.max, 100);
}
#[test]
fn capture_and_invalid_outliers_do_not_pollute_first_or_steady_summary() {
    let mut c = Costs::default();
    c.begin("yaw").unwrap();
    for (i, total, capture, valid) in [
        (1, 9000, true, true),
        (2, 8000, false, false),
        (3, 100, false, true),
        (4, 20, false, true),
        (5, 30, false, true),
        (6, 10, false, true),
    ] {
        c.observe(
            i,
            profile(total),
            Totals {
                redraws: i,
                mesh_bytes: i * 8,
                instance_bytes: i * 4,
            },
            capture,
            valid,
            true,
        )
        .unwrap();
    }
    c.end().unwrap();
    let r = c.phases[0].report(true);
    assert_eq!(r.eligible_frames, 4);
    assert_eq!(r.capture_frames, 1);
    assert_eq!(r.invalid_frames, 1);
    assert_eq!(r.first_frame.unwrap().frame, 3);
    assert_eq!(r.all_frames_us["total_us"].median, 25.);
    assert_eq!(r.steady_after_first_us["total_us"].median, 20.);
    assert_eq!(r.steady_after_first_us["total_us"].p95, 30);
    assert_eq!(r.eligible_counter_totals["actual_scene_redraws"], 4);
    assert_eq!(r.eligible_counter_totals["mesh_upload_bytes"], 32);
}
#[test]
fn retained_calls_remain_distinct_from_redraws_and_startup_frames() {
    let mut c = Costs::default();
    c.observe(1, profile(200), Totals::default(), false, true, false)
        .unwrap();
    c.observe(
        2,
        profile(100),
        Totals {
            redraws: 1,
            mesh_bytes: 40,
            instance_bytes: 20,
        },
        false,
        true,
        true,
    )
    .unwrap();
    c.begin("idle").unwrap();
    let mut p = FrameProfile::default();
    p.producers.render_calls = 1;
    c.observe(
        3,
        Some(p),
        Totals {
            redraws: 1,
            mesh_bytes: 40,
            instance_bytes: 20,
        },
        false,
        true,
        true,
    )
    .unwrap();
    c.end().unwrap();
    let r = c.phases[0].report(true);
    assert_eq!(r.eligible_counter_totals["producer_calls"], 1);
    assert_eq!(r.eligible_counter_totals["actual_scene_redraws"], 0);
    assert_eq!(r.eligible_counter_totals["mesh_upload_bytes"], 0);
    assert_eq!(c.first_document.as_ref().unwrap().frame, 1);
    assert_eq!(c.first_populated.as_ref().unwrap().frame, 2);
    assert!(r.steady_after_first_us.is_empty());
}
#[test]
fn phase_and_sample_limits_refuse_instead_of_silently_truncating() {
    let mut c = Costs::default();
    c.begin("bounded").unwrap();
    assert!(c.begin("nested").is_err());
    assert!(c.finish().is_err());
    for i in 0..MAX_SAMPLES {
        c.observe(i as u64, profile(1), Totals::default(), false, true, false)
            .unwrap();
    }
    assert!(
        c.observe(
            MAX_SAMPLES as u64,
            profile(1),
            Totals::default(),
            false,
            true,
            false
        )
        .is_err()
    );
    assert!(c.failed);
    assert_eq!(c.active.as_ref().unwrap().samples.len(), MAX_SAMPLES);
    c.end().unwrap();
    assert!(c.begin("again").is_err());
    let mut c = Costs::default();
    c.begin("empty").unwrap();
    assert!(c.end().is_err());
    assert!(c.begin("empty").is_err());
    assert!(c.begin("bad/name").is_err());
}

#[test]
fn pending_attempts_keep_raw_work_and_latency_but_do_not_become_eligible_frames() {
    let mut costs = Costs::default();
    costs.begin("native").unwrap();
    let totals = Totals {
        redraws: 1,
        mesh_bytes: 40,
        instance_bytes: 20,
    };
    costs
        .observe_attempt(
            0,
            Some(FrameProfile {
                native_pending: true,
                total_us: 7,
                native_submit_us: 7,
                ..Default::default()
            }),
            totals,
            false,
            true,
            false,
            false,
        )
        .unwrap();
    costs
        .observe_attempt(
            0,
            Some(FrameProfile {
                native_pending: true,
                total_us: 2,
                native_collect_us: 2,
                ..Default::default()
            }),
            totals,
            false,
            true,
            false,
            false,
        )
        .unwrap();
    costs
        .observe_attempt(
            1,
            Some(FrameProfile {
                total_us: 10,
                native_submit_us: 7,
                native_collect_us: 3,
                native_latency_us: 80_000,
                ..Default::default()
            }),
            totals,
            false,
            true,
            true,
            true,
        )
        .unwrap();
    // A failed acquisition can have no pending batch; it is still not presented.
    costs
        .observe_attempt(1, profile(9000), totals, false, true, true, false)
        .unwrap();
    costs.end().unwrap();
    let report = costs.phases[0].report(true);
    assert_eq!(report.samples.len(), 4);
    assert_eq!(report.eligible_frames, 1);
    assert_eq!(report.pending_attempts, 2);
    assert_eq!(report.unpresented_attempts, 3);
    assert_eq!(report.all_frames_us["total_us"].median, 10.0);
    assert_eq!(report.all_frames_us["native_latency_us"].median, 80_000.0);
    assert_eq!(report.all_attempt_counter_totals["actual_scene_redraws"], 1);
    assert_eq!(report.eligible_counter_totals["actual_scene_redraws"], 0);
    assert_eq!(costs.first_document.as_ref().unwrap().attempt, 3);
    assert_eq!(costs.first_populated.as_ref().unwrap().attempt, 3);
    assert_eq!(report.observed_ui_thread_us, 9012);
    assert_eq!(costs.ui_thread_us, 9012);
    assert_eq!(costs.attempts, 4);
    assert_eq!(costs.native_pending_attempts, 2);
    assert_eq!(costs.unpresented_attempts, 3);
}

#[test]
fn a_phase_of_only_pending_attempts_cannot_qualify_as_a_presented_timing_phase() {
    let mut costs = Costs::default();
    costs.begin("pending").unwrap();
    costs
        .observe_attempt(
            0,
            Some(FrameProfile {
                native_pending: true,
                ..Default::default()
            }),
            Totals::default(),
            false,
            true,
            false,
            false,
        )
        .unwrap();
    assert!(costs.end().is_err());
    assert!(costs.first_document.is_none());
    assert!(costs.first_populated.is_none());
    assert_eq!(costs.phases[0].report(true).eligible_frames, 0);
}
