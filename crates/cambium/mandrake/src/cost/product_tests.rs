// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

fn details(epoch: u64, calls: u64) -> serde_json::Value {
    serde_json::json!({
        "schema":1,
        "main_scene":{"cost_epoch":epoch,"stages":{
            "draw":{"calls":calls,"wall_ns":calls*13,"thread_cpu_ns":null}
        }},
        "previews":[]
    })
}

#[test]
fn optional_product_snapshots_remain_immutable_on_pending_and_accepted_attempts() {
    let mut costs = Costs::default();
    costs.begin("producer").unwrap();
    let mut supplied = details(4, 1);
    let profile = FrameProfile {
        native_pending: true,
        native_submit_us: 10,
        total_us: 10,
        ..Default::default()
    };
    costs
        .observe_attempt_with_details(
            0,
            Some(profile),
            Totals::default(),
            false,
            true,
            false,
            false,
            Some(supplied.clone()),
        )
        .unwrap();
    supplied["main_scene"]["cost_epoch"] = 99.into();
    costs
        .observe_attempt_with_details(
            1,
            Some(FrameProfile {
                native_collect_us: 20,
                native_submit_us: 10,
                total_us: 30,
                ..Default::default()
            }),
            Totals::default(),
            false,
            true,
            false,
            true,
            Some(details(4, 1)),
        )
        .unwrap();
    costs.end().unwrap();
    let report = costs.report();
    let phase = &report["phases"][0];
    let samples = phase["samples"].as_array().unwrap();
    assert_eq!(samples[0]["product_work"], details(4, 1));
    assert_eq!(samples[1]["product_work"], details(4, 1));
    assert_eq!(phase["pending_attempts"], 1);
    assert_eq!(phase["eligible_frames"], 1);
    assert_eq!(report["observed_ui_thread_us"], 30);
    assert_eq!(
        phase["all_attempt_counter_totals"]["actual_scene_redraws"], 0,
        "opaque snapshots never manufacture product counter increments"
    );
}

#[test]
fn absent_details_remain_unavailable_and_epoch_resets_are_preserved_verbatim() {
    let mut costs = Costs::default();
    costs.begin("epochs").unwrap();
    let profile = Some(FrameProfile {
        total_us: 7,
        ..Default::default()
    });
    costs
        .observe_attempt(1, profile, Totals::default(), false, true, false, true)
        .unwrap();
    for (frame, epoch, calls) in [(2, 8, 5), (3, 9, 0)] {
        costs
            .observe_attempt_with_details(
                frame,
                profile,
                Totals::default(),
                false,
                true,
                false,
                true,
                Some(details(epoch, calls)),
            )
            .unwrap();
    }
    costs.end().unwrap();
    let report = costs.report();
    let samples = report["phases"][0]["samples"].as_array().unwrap();
    assert!(samples[0].get("product_work").is_none());
    assert_eq!(samples[1]["product_work"], details(8, 5));
    assert_eq!(samples[2]["product_work"], details(9, 0));
    assert!(samples[1]["product_work"]["main_scene"]["stages"]["draw"]["thread_cpu_ns"].is_null());
    assert_eq!(report["observed_ui_thread_us"], 21);
}

#[test]
fn phase_baseline_retains_the_immediately_preceding_actual_attempt() {
    let mut costs = Costs::default();
    let profile = Some(FrameProfile {
        total_us: 7,
        ..Default::default()
    });
    costs
        .observe_attempt_with_details(
            1,
            profile,
            Totals::default(),
            false,
            true,
            false,
            true,
            Some(details(4, 5)),
        )
        .unwrap();
    costs.begin("initial_work").unwrap();
    costs
        .observe_attempt_with_details(
            2,
            profile,
            Totals::default(),
            false,
            true,
            false,
            true,
            Some(details(4, 6)),
        )
        .unwrap();
    costs.end().unwrap();
    let report = costs.report();
    let phase = &report["phases"][0];
    let before = &phase["product_work_before_phase"];
    let first = &phase["samples"][0];
    assert_eq!(before["attempt"], 1);
    assert_eq!(before["product_work"], details(4, 5));
    assert_eq!(first["attempt"], 2);
    assert_eq!(first["product_work"], details(4, 6));
    assert_eq!(
        first["product_work"]["main_scene"]["stages"]["draw"]["calls"]
            .as_u64()
            .unwrap()
            - before["product_work"]["main_scene"]["stages"]["draw"]["calls"]
                .as_u64()
                .unwrap(),
        1,
        "first sample work is retained rather than subtracting it from itself"
    );
    costs
        .observe_attempt(3, profile, Totals::default(), false, true, false, true)
        .unwrap();
    costs.begin("unavailable_prior").unwrap();
    costs
        .observe_attempt(4, profile, Totals::default(), false, true, false, true)
        .unwrap();
    costs.end().unwrap();
    let report = costs.report();
    let before = &report["phases"][1]["product_work_before_phase"];
    assert_eq!(before["attempt"], 3);
    assert!(
        before["product_work"].is_null(),
        "no stale prior Some reading"
    );
}
