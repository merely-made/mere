// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn native_form_state_change_rebuilds_only_its_owning_window() {
    let (mut multi, a, b) = multi();
    let dom = multi.dom();
    let root_a = multi.window_root(a).unwrap();
    let input = {
        let mut d = dom.borrow_mut();
        let input = d.create_element(html_qual("input"));
        d.set_attribute(input, attr_qual("value"), "default");
        d.set_attribute(
            input,
            attr_qual("style"),
            "display:block;width:120px;height:20px",
        );
        d.append_child(root_a, input);
        input
    };
    for _ in 0..2 {
        layout_at(&mut multi, a, 400.0, 300.0);
        layout_at(&mut multi, b, 300.0, 600.0);
    }
    let rebuilt =
        |multi: &mut Multi, id| multi.with_window(id, |h| h.s.last_layout_rebuilt).unwrap();
    assert!(!rebuilt(&mut multi, a) && !rebuilt(&mut multi, b));

    dom.borrow_mut()
        .set_form_control_value(input, "current")
        .unwrap();
    assert_eq!(
        dom.borrow().attribute(
            input,
            &layout_dom_api::Namespace::from(""),
            &layout_dom_api::LocalName::from("value")
        ),
        Some("default"),
        "the live-value change produces no value attribute change"
    );
    assert_eq!(multi.touched_windows(), HashSet::from([a]));
    layout_at(&mut multi, a, 400.0, 300.0);
    layout_at(&mut multi, b, 300.0, 600.0);
    assert!(
        rebuilt(&mut multi, a),
        "the owning window consumes the form-state event"
    );
    assert!(
        !rebuilt(&mut multi, b),
        "the other window keeps its settled layout"
    );
}

#[test]
fn native_form_metadata_notifications_reach_only_the_owning_window() {
    let (mut multi, a, b) = multi();
    let dom = multi.dom();
    let root = multi.window_root(a).unwrap();
    let (input, option, button) = {
        let mut document = dom.borrow_mut();
        let input = document.create_element(html_qual("input"));
        document.append_child(root, input);
        let select = document.create_element(html_qual("select"));
        document.append_child(root, select);
        let option = document.create_element(html_qual("option"));
        document.append_child(select, option);
        let button = document.create_element(html_qual("button"));
        document.append_child(root, button);
        (input, option, button)
    };
    for _ in 0..2 {
        layout_at(&mut multi, a, 400.0, 300.0);
        layout_at(&mut multi, b, 300.0, 600.0);
    }
    assert!(multi.touched_windows().is_empty());
    for notification in 0..3 {
        {
            let mut document = dom.borrow_mut();
            match notification {
                0 => assert!(document.set_form_control_interaction_state(
                    input,
                    layout_dom_api::FormControlInteractionState {
                        last_change_by_user: true,
                        ..Default::default()
                    }
                )),
                1 => assert!(document.set_option_selected_state(
                    option,
                    layout_dom_api::SelectOptionState {
                        selected: true,
                        dirty: true
                    }
                )),
                _ => assert!(document.set_custom_validity(button, "Choose an option")),
            }
        }
        assert_eq!(
            multi.touched_windows(),
            HashSet::from([a]),
            "form notification {notification} must reach its owner without invalidating the sibling"
        );
        layout_at(&mut multi, a, 400.0, 300.0);
        layout_at(&mut multi, b, 300.0, 600.0);
        assert!(
            multi.touched_windows().is_empty(),
            "the owner consumes its notification"
        );
    }
}

#[test]
fn native_interaction_option_and_custom_validity_reach_only_their_window() {
    use layout_dom_api::{FormControlInteractionState, SelectOptionState};
    let (mut multi, a, b) = multi();
    let dom = multi.dom();
    let (input, option, button) = {
        let mut d = dom.borrow_mut();
        let input = d.create_element(html_qual("input"));
        d.set_attribute(input, attr_qual("required"), "");
        d.append_child(multi.window_root(a).unwrap(), input);
        let select = d.create_element(html_qual("select"));
        let option = d.create_element(html_qual("option"));
        d.append_child(select, option);
        d.append_child(multi.window_root(a).unwrap(), select);
        // Buttons use the new independent custom-validity state; text inputs
        // retain the older FormControlStateChanged compatibility mutation.
        let button = d.create_element(html_qual("button"));
        d.append_child(multi.window_root(b).unwrap(), button);
        (input, option, button)
    };
    for _ in 0..2 {
        layout_at(&mut multi, a, 400.0, 300.0);
        layout_at(&mut multi, b, 300.0, 600.0);
    }
    assert!(multi.touched_windows().is_empty());
    for (kind, owner) in [(0, a), (1, a), (2, b)] {
        let changed = match kind {
            0 => dom.borrow_mut().set_form_control_interaction_state(
                input,
                FormControlInteractionState {
                    user_validity: true,
                    last_change_by_user: true,
                    ..Default::default()
                },
            ),
            1 => dom.borrow_mut().set_option_selected_state(
                option,
                SelectOptionState {
                    selected: true,
                    dirty: true,
                },
            ),
            _ => dom
                .borrow_mut()
                .set_custom_validity(button, "Confirm this choice"),
        };
        assert!(changed, "fixture emits native form mutation {kind}");
        assert_eq!(
            multi.touched_windows(),
            HashSet::from([owner]),
            "form mutation {kind} stays in its window"
        );
        layout_at(&mut multi, a, 400.0, 300.0);
        layout_at(&mut multi, b, 300.0, 600.0);
        let other = if owner == a { b } else { a };
        assert!(
            !multi
                .with_window(other, |h| h.s.last_layout_rebuilt)
                .unwrap(),
            "unrelated window does not rebuild for form mutation {kind}"
        );
        assert!(
            multi.touched_windows().is_empty(),
            "owner consumed mutation {kind}"
        );
    }
}

#[test]
fn queued_secondary_release_and_cancel_follow_host_capture_routing() {
    use crate::HostPointer;
    use cambium::{PointerEvent, PointerPhase, on_pointer};
    let mut shared = AppShared::default();
    shared.sheet = SHEET.into();
    let mut multi: Multi = MultiHost::new(App::default(), shared, HostHooks::inert());
    let logic: Logic = Box::new(|_: &App| {
        Box::new(on_pointer(
            el("button", text("drag target")),
            |app: &mut App, event: PointerEvent| {
                if event.phase == PointerPhase::Down {
                    event.capture_pointer();
                }
                app.pointer_phases.push(event.phase);
            },
        ))
    });
    let state = HostState::new();
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let window = multi.open(
        logic,
        Host::new(
            HostOptions::default(),
            None,
            HostHooks::inert(),
            state,
            wake,
        ),
    );
    layout_at(&mut multi, window, 400.0, 300.0);
    let dom = multi.dom();
    let node = find(
        &dom.borrow(),
        multi.window_root(window).unwrap(),
        "drag target",
    )
    .unwrap();
    let (x, y, w, h) = painted(&mut multi, window, node).unwrap();
    let (x, y) = (x + w / 2.0, y + h / 2.0);
    multi
        .with_window(window, |host| {
            host.s.pending_pointer.extend([
                HostPointer::SecondaryPress(x, y),
                HostPointer::Moved(x + 2.0, y),
                HostPointer::Release(x + 2.0, y),
            ]);
            host.drain_pointer();
            assert!(
                host.s.runner.as_ref().unwrap().pointer_capture().is_some(),
                "unrelated primary release preserves secondary capture"
            );
            host.s
                .pending_pointer
                .push(HostPointer::SecondaryRelease(x + 2.0, y));
            host.drain_pointer();
            assert!(host.s.runner.as_ref().unwrap().pointer_capture().is_none());
        })
        .unwrap();
    assert_eq!(
        multi.state().pointer_phases,
        [PointerPhase::Down, PointerPhase::Move, PointerPhase::Up]
    );
    multi
        .with_window(window, |host| {
            host.s.pending_pointer.extend([
                HostPointer::SecondaryPress(x, y),
                HostPointer::Cancel,
                HostPointer::SecondaryRelease(x, y),
            ]);
            host.drain_pointer();
            assert!(host.s.runner.as_ref().unwrap().pointer_capture().is_none());
        })
        .unwrap();
    assert_eq!(
        multi.state().pointer_phases,
        [
            PointerPhase::Down,
            PointerPhase::Move,
            PointerPhase::Up,
            PointerPhase::Down,
            PointerPhase::Cancel
        ]
    );
}
