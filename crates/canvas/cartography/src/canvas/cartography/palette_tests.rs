// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::collections::HashMap;

#[test]
fn public_node_color_accessors_use_the_canonical_accents() {
    fn assert_color_matches(color: &str, expected: [u8; 3]) {
        let hex = color.strip_prefix('#').expect("CSS color starts with #");
        let actual = [
            u8::from_str_radix(&hex[0..2], 16).expect("red byte"),
            u8::from_str_radix(&hex[2..4], 16).expect("green byte"),
            u8::from_str_radix(&hex[4..6], 16).expect("blue byte"),
        ];
        assert_eq!(actual, expected, "{color} should encode {expected:?}");
    }

    let mut canvas = Canvas::new();
    let key = canvas.visit("https://palette.example");
    let id = canvas.graph().get_node(key).expect("visited node").id;
    canvas.clear_selection();

    for state in [NodeState::Open, NodeState::Closed, NodeState::Idle] {
        let mut states = HashMap::new();
        states.insert(id, state);
        canvas.set_node_states(states);
        assert_color_matches(
            canvas.node_state_color(key),
            palette::accent(false, state).bg,
        );
        assert_color_matches(canvas.node_color(key), palette::accent(false, state).bg);
    }

    assert!(canvas.select_member(id));
    assert_color_matches(
        canvas.node_color(key),
        palette::accent(true, NodeState::Idle).bg,
    );
    assert_color_matches(
        canvas.node_state_color(key),
        palette::accent(false, NodeState::Idle).bg,
    );
}
