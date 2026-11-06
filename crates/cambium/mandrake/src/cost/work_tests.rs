// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cambium_rootstock::{
    NativeGpuWork, NativeWorkAttribution, NativeWorkCall, NativeWorkCallKind, NativeWorkPacket,
    NativeWorkSpan, NativeWorkerWork,
};

#[test]
fn raw_work_stays_with_its_batch_and_current_callback_without_repeating_ui_cpu() {
    let mut costs = Costs::default();
    costs.begin("work").unwrap();
    let work = NativeWorkAttribution {
        packet: Some(NativeWorkPacket {
            host: 3,
            generation: 5,
            batch_id: 7,
            proposed_sequence: 9,
        }),
        current_call: Some(NativeWorkCall {
            kind: NativeWorkCallKind::Submit,
            span: NativeWorkSpan::from_nanos(11_000, Some(0)),
        }),
        submission_call: Some(NativeWorkSpan::from_nanos(11_000, Some(0))),
        ..Default::default()
    };
    let mut profile = FrameProfile {
        native_work: work,
        native_pending: true,
        native_submit_us: 10,
        total_us: 10,
        ..Default::default()
    };
    costs
        .observe_attempt(
            0,
            Some(profile),
            Totals::default(),
            false,
            true,
            true,
            false,
        )
        .unwrap();
    profile.native_pending = false;
    profile.native_collect_us = 20;
    profile.a11y_us = 2;
    profile.native_work.current_call = Some(NativeWorkCall {
        kind: NativeWorkCallKind::Collect,
        span: NativeWorkSpan::from_nanos(22_000, Some(5_000)),
    });
    profile.native_work.worker = Some(NativeWorkerWork {
        batch_id: 7,
        core_generation: 5,
        gpu: NativeGpuWork {
            batch_id: 7,
            ..Default::default()
        },
        total: Some(NativeWorkSpan::from_nanos(80_000, None)),
        ..Default::default()
    });
    costs
        .observe_attempt(1, Some(profile), Totals::default(), false, true, true, true)
        .unwrap();
    costs.end().unwrap();
    let report = costs.report();
    let samples = report["phases"][0]["samples"].as_array().unwrap();
    let submit = &samples[0]["native_work"];
    let collect = &samples[1]["native_work"];
    assert_eq!(submit["packet"], collect["packet"]);
    assert_eq!(submit["packet"]["batch_id"], 7);
    assert_eq!(submit["current_call"]["kind"], "submit");
    assert_eq!(collect["current_call"]["kind"], "collect");
    assert_eq!(submit["current_call"]["span"]["thread_cpu_ns"], 0);
    assert_eq!(collect["current_call"]["span"]["thread_cpu_ns"], 5_000);
    assert!(submit["worker"].is_null());
    assert!(collect["worker"]["total"]["thread_cpu_ns"].is_null());
    assert!(collect["worker"]["gpu"]["recording_commands_ns"].is_null());
    assert_eq!(
        report["observed_ui_thread_us"], 32,
        "R5 callback ledger retains its meaning"
    );
}

#[test]
fn absent_attribution_is_null_and_invalid_cpu_is_not_reported_as_zero() {
    let mut costs = Costs::default();
    costs.begin("availability").unwrap();
    costs
        .observe(
            1,
            Some(FrameProfile::default()),
            Totals::default(),
            false,
            true,
            false,
        )
        .unwrap();
    let mut profile = FrameProfile::default();
    profile.native_work.current_call = Some(NativeWorkCall {
        kind: NativeWorkCallKind::Collect,
        span: NativeWorkSpan::from_nanos(10, Some(11)),
    });
    costs
        .observe(2, Some(profile), Totals::default(), false, true, false)
        .unwrap();
    costs.end().unwrap();
    let report = costs.report();
    let samples = report["phases"][0]["samples"].as_array().unwrap();
    assert!(samples[0].as_object().unwrap().contains_key("native_work"));
    assert!(samples[0]["native_work"].is_null());
    let invalid = &samples[1]["native_work"]["current_call"]["span"];
    assert!(invalid["thread_cpu_ns"].is_null());
    assert!(invalid["non_cpu_residual_ns"].is_null());
    assert_eq!(invalid["rejected_thread_cpu_ns"], 11);
}
