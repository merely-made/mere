// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Actual retained Workshop keyboard routing, without GPU or a window.
use cambium_genet_winit_host::Harness;
use genet_scripted_dom::NodeId;
use tabard_workshop::{WorkshopState, WorkshopView, workshop_stylesheet, workshop_view};
use taproot::Selector;
use winit::keyboard::NamedKey;

type Host = Harness<WorkshopState, fn(&WorkshopState) -> WorkshopView, WorkshopView>;

fn hue(host: &Host) -> NodeId {
    host.with_dom(|dom| {
        let nodes = taproot::matching(dom, &Selector::role("slider").containing("Primary hue"));
        assert_eq!(
            nodes.len(),
            1,
            "real Workshop must have one Primary hue control"
        );
        nodes[0]
    })
}
fn reach_hue(host: &mut Host, held: bool) {
    for _ in 0..150 {
        if host.focus() == Some(hue(host)) && host.tab_held() == held {
            return;
        }
        if held {
            host.hold_tab(true);
            if host.focus() == Some(hue(host)) {
                return;
            }
            host.release_tab();
        } else {
            host.tab(true);
        }
    }
    panic!("real Tab traversal cannot reach Workshop hue with held={held}");
}

#[test]
fn held_tab_arrow_steers_focus_but_released_tab_arrow_edits_actual_workshop_slider() {
    let mut host = Harness::new(
        workshop_stylesheet(),
        WorkshopState::in_memory(),
        workshop_view as fn(&WorkshopState) -> WorkshopView,
    );
    host.layout_at(1280.0, 1100.0);
    let active = host.state().registry().active_theme();
    reach_hue(&mut host, true);
    assert!(host.tab_held());
    let prior = host.state().draft_theme().seeds.primary;
    let focused = host.focus();
    // There are real channel controls below Hue. The held arrow goes through
    // production spatial navigation, rather than editing a slider value.
    host.key_named(NamedKey::ArrowDown);
    assert_ne!(
        host.focus(),
        focused,
        "held Tab must steer actual retained focus"
    );
    assert_eq!(host.state().draft_theme().seeds.primary, prior);
    assert!(host.tab_held());
    host.release_tab();
    assert!(!host.tab_held());
    reach_hue(&mut host, false);
    let focused = host.focus();
    let prior = host.state().draft_theme().seeds.primary;
    host.key_named(NamedKey::ArrowRight);
    assert_eq!(
        host.focus(),
        focused,
        "released Tab leaves arrows to the focused control"
    );
    assert_ne!(
        host.state().draft_theme().seeds.primary,
        prior,
        "the real Workshop slider must update its application seed"
    );
    assert_eq!(
        host.state().registry().active_theme(),
        active,
        "editing must preserve host appearance authority"
    );
}
