// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

fn b(x: f32, y: f32, w: f32, h: f32) -> Box2 {
    Box2 { x, y, w, h }
}

/// A control directly below beats a nearer one off to the side: "down" means
/// down the column you are looking at.
#[test]
fn alignment_beats_raw_distance() {
    let from = b(0.0, 0.0, 100.0, 40.0);
    let below = score(from, b(0.0, 60.0, 100.0, 40.0), Direction::Down).expect("in direction");
    let aside = score(from, b(300.0, 45.0, 100.0, 40.0), Direction::Down).expect("in direction");
    assert!(
        below < aside,
        "aligned {below} must beat off-to-the-side {aside}",
    );
}

/// Nothing behind you is a candidate.
#[test]
fn the_opposite_direction_is_never_a_candidate() {
    let from = b(0.0, 100.0, 100.0, 40.0);
    assert!(score(from, b(0.0, 0.0, 100.0, 40.0), Direction::Down).is_none());
    assert!(score(from, b(0.0, 0.0, 100.0, 40.0), Direction::Up).is_some());
}

/// A control on the same visual row is not "below" its neighbour, so Down
/// from one column does not slide sideways.
#[test]
fn the_same_row_is_not_below() {
    let from = b(0.0, 0.0, 100.0, 40.0);
    assert!(score(from, b(120.0, 0.0, 100.0, 40.0), Direction::Down).is_none());
    assert!(score(from, b(120.0, 0.0, 100.0, 40.0), Direction::Right).is_some());
}
