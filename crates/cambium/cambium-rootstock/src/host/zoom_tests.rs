// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::{ZOOM_LADDER, fit_zoom, ladder_step};

#[test]
fn the_ladder_steps_both_ways_and_clamps_at_its_ends() {
    assert_eq!(ladder_step(1.0, true), 1.1);
    assert_eq!(ladder_step(1.0, false), 0.9);
    assert_eq!(ladder_step(3.0, true), 3.0);
    assert_eq!(ladder_step(0.5, false), 0.5);
}

#[test]
fn a_value_off_the_ladder_steps_to_the_next_rung_past_it() {
    assert_eq!(ladder_step(1.37, true), 1.5);
    assert_eq!(ladder_step(1.37, false), 1.25);
}

#[test]
fn every_rung_is_reachable_by_stepping_up_from_the_bottom() {
    let mut zoom = ZOOM_LADDER[0];
    for rung in ZOOM_LADDER.iter().skip(1) {
        zoom = ladder_step(zoom, true);
        assert_eq!(zoom, *rung);
    }
}

#[test]
fn the_fit_is_the_binding_axis() {
    // 820 tall wanted, 752 offered: height binds, width has slack.
    let zoom = fit_zoom((1100.0, 820.0), (1100.0, 752.0));
    assert!((zoom - 752.0 / 820.0).abs() < 1e-6, "{zoom}");
    // Wider design on the same surface: now width binds.
    let zoom = fit_zoom((2200.0, 820.0), (1100.0, 752.0));
    assert!((zoom - 0.5).abs() < 1e-6, "{zoom}");
}

#[test]
fn a_degenerate_design_does_not_divide_by_zero() {
    assert_eq!(fit_zoom((0.0, 0.0), (1100.0, 752.0)), 1.0);
}
