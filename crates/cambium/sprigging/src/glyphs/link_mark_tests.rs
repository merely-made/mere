// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
#[test]
fn direction_heads_and_opposition_marks_follow_the_authored_path() {
    let marks = relation_mark_paths(&[(0.0, 0.0), (50.0, 0.0), (50.0, 50.0)], true, true, true);
    assert_eq!(marks.len(), 4);
    assert!(
        marks[0][0].0 > marks[0][1].0,
        "reverse arrow points toward route start"
    );
    assert!(
        marks[1][0].1 < marks[1][1].1,
        "forward arrow follows final vertical segment"
    );
    assert!(marks[2].iter().all(|p| p.0.is_finite() && p.1.is_finite()));
    assert!(relation_mark_paths(&[(0.0, 0.0), (0.0, 0.0)], true, true, true).is_empty());
}
