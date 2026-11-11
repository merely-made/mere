// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// F22: an item's role overrides the recipe's, and the explicit pin is the
/// item-level pinned case. Realized: the seeded item gets no spring.
#[test]
fn an_item_role_overrides_the_recipe_role() {
    let (mut canvas, keys, _) = spiral_canvas();
    canvas.set_arrangement_role(Role::Anchored);
    let member = canvas.graph().get_node(keys[2]).unwrap().id;
    assert!(canvas.set_member_role(member, Some(Role::Seeded)));
    assert_eq!(
        canvas.arrangement_role_of(keys[0]),
        Role::Anchored,
        "recipe"
    );
    assert_eq!(canvas.arrangement_role_of(keys[2]), Role::Seeded, "item");
    canvas.select_only(keys[0]);
    assert!(canvas.pin_focused());
    assert_eq!(
        canvas.arrangement_role_of(keys[0]),
        Role::Pinned,
        "explicit pin"
    );
    assert!(canvas.release_focused());
    assert_eq!(canvas.arrangement_role_of(keys[0]), Role::Anchored);
    play(&mut canvas, 1);
    assert_eq!(
        canvas.physics.anchor_count(),
        N - 1,
        "all anchored but the seeded item"
    );
    assert!(canvas.set_member_role(member, None));
    assert_eq!(
        canvas.arrangement_role_of(keys[2]),
        Role::Anchored,
        "back to the recipe"
    );
}

/// F22: a group's role overrides the recipe's, and an item's overrides its
/// group's. Groups are sites: three pages of one site and one of another.
/// Realized: the group's pinned items hold exactly while the rest plays.
#[test]
fn an_item_role_overrides_its_group_role() {
    let mut graph = Graph::new();
    let urls = [
        "https://same.example/a",
        "https://same.example/b",
        "https://same.example/c",
        "https://other.example/d",
    ];
    let keys: Vec<_> = urls
        .iter()
        .map(|url| graph.add_node(url.to_string(), PortablePoint::new(0.0, 0.0)))
        .collect();
    for pair in keys.windows(2) {
        graph.assert_relation(pair[0], pair[1], crate::canvas::build::hyperlink());
    }
    let mut canvas = Canvas::with_graph(graph);
    canvas.resize(800, 600);
    canvas.set_physics_paused(true);
    canvas.set_layout_strategy(Some("test.groups".to_string()));
    let slots: Vec<_> = keys
        .iter()
        .enumerate()
        .map(|(i, k)| (*k, PortablePoint::new(i as f32 * 60.0, 0.0)))
        .collect();
    canvas.apply_strategy_positions(&slots);
    let site = canvas.role_group_of(keys[0]).unwrap();
    assert_eq!(
        canvas.role_group_of(keys[2]).as_deref(),
        Some(site.as_str())
    );
    assert_ne!(
        canvas.role_group_of(keys[3]).as_deref(),
        Some(site.as_str())
    );
    canvas.set_group_role(&site, Some(Role::Pinned));
    let member = canvas.graph().get_node(keys[2]).unwrap().id;
    assert!(canvas.set_member_role(member, Some(Role::Seeded)));
    assert_eq!(canvas.arrangement_role_of(keys[0]), Role::Pinned, "group");
    assert_eq!(canvas.arrangement_role_of(keys[1]), Role::Pinned, "group");
    assert_eq!(
        canvas.arrangement_role_of(keys[2]),
        Role::Seeded,
        "item over group"
    );
    assert_eq!(
        canvas.arrangement_role_of(keys[3]),
        Role::Seeded,
        "the recipe's"
    );
    play(&mut canvas, 120);
    for i in [0, 1] {
        assert_eq!(
            canvas.view.position_of(keys[i]),
            Some(slots[i].1),
            "pinned {i}"
        );
    }
    assert!(
        canvas.view.position_of(keys[2]) != Some(slots[2].1),
        "the overridden item moved"
    );
    assert!(canvas.set_member_role(member, None));
    assert_eq!(
        canvas.arrangement_role_of(keys[2]),
        Role::Pinned,
        "back to its group"
    );
}
