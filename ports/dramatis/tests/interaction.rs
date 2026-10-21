// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Production retained views, layout, focus and action routing.
use cambium_genet_winit_host::{Harness, Init, Key, KeyPress, NamedKey};
use dramatis::{
    roster::{Roster, RosterEntry},
    view::VaultLockView,
};
use dramatis_app::{
    Request, Snapshot, State,
    views::{Child, Logic, SHEET, root},
};
use graphshell_endpoint::door::quarantine::{QuarantineCard, ReviewAction};
use personae::ProfileId;
use taproot::Selector;
fn state(locked: bool, unsupported: bool) -> State {
    let roster = Roster {
        chosen: ProfileId("family".into()),
        description: String::new(),
        entries: vec![
            RosterEntry {
                id: ProfileId("family".into()),
                display_name: "Family".into(),
                slot_count: 0,
                chosen: true,
            },
            RosterEntry {
                id: ProfileId("work".into()),
                display_name: "Work".into(),
                slot_count: 0,
                chosen: false,
            },
        ],
    };
    let item = chatelaine::Item {
        id: chatelaine::ItemId::from_random([1; 16]),
        source_id: None,
        title: "Imported payment card for review".into(),
        subtitle: None,
        scope: None,
        tags: vec![],
        favorite: false,
        created_at: None,
        modified_at: None,
        credentials: vec![chatelaine::Credential {
            id: chatelaine::CredentialId::from_random([2; 16]),
            kind: chatelaine::CredentialKind::CreditCard {
                card_type: None,
                expiry: None,
                last_four: None,
            },
        }],
        state: chatelaine::ItemState::Quarantined,
    };
    State {
        snapshot: Some(Snapshot {
            lock: if locked {
                VaultLockView::Locked
            } else {
                VaultLockView::Unlocked
            },
            roster,
            unassigned: vec![],
            cards: if locked {
                vec![]
            } else {
                vec![QuarantineCard {
                    store: ProfileId("family".into()),
                    persona: *personae::PersonaId::new().as_uuid(),
                    item,
                    revision: "exact-reviewed-metadata".into(),
                    accept_blocked: unsupported
                        .then(|| "This credential type is not supported.".into()),
                }]
            },
        }),
        ..Default::default()
    }
}
fn harness(state: State) -> Harness<State, Logic, Child> {
    Harness::with_hooks(
        Init {
            state,
            logic: root as Logic,
            sheet: SHEET.into(),
            fonts: vec![],
            images: vec![],
        },
        dramatis_app::host_hooks(),
    )
}
fn host(locked: bool, unsupported: bool, width: f32) -> Harness<State, Logic, Child> {
    let mut h = harness(state(locked, unsupported));
    h.layout_at(width, 720.0);
    h
}
#[test]
fn keyboard_reconnect_and_pointer_persona_selection_use_the_real_controls() {
    let mut h = host(false, false, 860.0);
    h.tab(true);
    assert!(h.focus().is_some());
    h.press_key(&KeyPress {
        key: Key::Named(NamedKey::Enter),
        text: None,
        modifiers: Default::default(),
        repeat: false,
    });
    assert!(matches!(h.state().request, Some(Request::Refresh)));
    h.update(|s| {
        s.busy = false;
        s.request = None;
    });
    assert!(h.click_on(&Selector::role("button").containing("Work")));
    assert!(matches!(h.state().request,Some(Request::Select(ProfileId(ref p))) if p=="work"));
}
#[test]
fn unsupported_acceptance_cannot_queue_but_delete_names_the_exact_card() {
    let mut h = host(false, true, 420.0);
    h.click_on(&Selector::role("button").containing("Accept"));
    assert!(h.state().request.is_none());
    assert!(h.click_on(&Selector::role("button").containing("Delete")));
    assert!(
        matches!(h.state().request,Some(Request::Review(ref r)) if r.action==ReviewAction::Delete && r.profile.0=="family" && r.revision=="exact-reviewed-metadata")
    );
}
#[test]
fn locked_persona_buttons_cannot_queue_a_switch() {
    let mut h = host(true, false, 420.0);
    h.click_on(&Selector::role("button").containing("Work"));
    assert!(h.state().request.is_none());
    assert!(h.click_on(&Selector::role("button").containing("Unlock with Djinn")));
    assert!(matches!(h.state().request, Some(Request::Unlock)));
}
#[test]
fn narrow_layout_and_zoom_keep_a_review_action_reachable() {
    let mut h = host(false, false, 420.0);
    h.set_ui_zoom(2.0);
    for _ in 0..12 {
        h.tab(true);
        let focus = h.focus().expect("keyboard has a focused control");
        assert!(
            h.visible_rect(focus).is_some(),
            "every tabbed control is revealed at double zoom"
        );
    }
    assert!(h.focus().is_some());
    let node = h.focus().unwrap();
    let (_, _, w, height) = h
        .visible_rect(node)
        .expect("focused control is revealed in viewport");
    assert!(w > 0.0 && height > 0.0);
    h.set_ui_zoom(1.0);
    h.layout_at(860.0, 720.0);
    assert!(h.click_on(&Selector::role("button").containing("Accept")));
    assert!(
        matches!(h.state().request,Some(Request::Review(ref r)) if r.action==ReviewAction::Accept)
    );
}

#[test]
fn older_import_assignment_names_the_destination_and_exact_revision() {
    let mut s = state(false, false);
    let persona = *personae::PersonaId::new().as_uuid();
    s.snapshot.as_mut().unwrap().unassigned.push(
        graphshell_endpoint::door::quarantine::UnassignedImport {
            store: ProfileId("family".into()),
            persona,
            titles: vec!["Older imported card".into()],
            item_count: 1,
            revision: "older-reviewed-metadata".into(),
        },
    );
    let mut h = harness(s);
    h.layout_at(860.0, 1100.0);
    assert!(h.click_on(&Selector::role("button").containing("Assign to this persona")));
    assert!(
        matches!(h.state().request,Some(Request::Assign(ref r)) if r.profile.0=="family" && r.persona==persona && r.revision=="older-reviewed-metadata")
    );
}

#[test]
fn focus_follow_preserves_manual_scrolling_until_focus_changes() {
    let mut h = host(false, false, 420.0);
    h.set_ui_zoom(2.0);
    let delete = h.with_dom(|dom| {
        taproot::matching_estimated_fixture(dom, &Selector::role("button").containing("Delete"))[0]
    });
    for _ in 0..12 {
        if h.focus() == Some(delete) {
            break;
        }
        h.tab(true);
    }
    assert_eq!(h.focus(), Some(delete));
    let focus = delete;
    assert!(h.visible_rect(focus).is_some());
    let before = h.element_scroll_total();
    assert!(before > 0.0);
    h.move_to(100.0, 100.0);
    h.wheel(0.0, -2000.0);
    assert_eq!(h.focus(), Some(focus));
    assert!(
        h.element_scroll_total() < before,
        "manual scroll is not undone by focus following"
    );
    h.tab(true);
    assert!(h.visible_rect(h.focus().unwrap()).is_some());
}

#[test]
fn locked_persona_controls_are_excluded_from_keyboard_order() {
    let mut h = host(true, false, 420.0);
    let disabled = h.with_dom(|dom| {
        let mut nodes = taproot::matching_estimated_fixture(
            dom,
            &Selector::role("button").containing("Family"),
        );
        nodes.extend(taproot::matching_estimated_fixture(
            dom,
            &Selector::role("button").containing("Work"),
        ));
        nodes
    });
    for _ in 0..8 {
        h.tab(true);
        assert!(
            !disabled.contains(&h.focus().unwrap()),
            "locked choices are not keyboard stops"
        );
    }
}

#[test]
fn authentication_control_is_a_public_request_and_disabled_when_locked() {
    for locked in [false, true] {
        let mut h = host(locked, false, 860.);
        let activated = h.click_on(&Selector::role("button").containing("Change passphrase"));
        if !locked {
            assert!(activated);
            assert!(matches!(h.state().request, Some(Request::ChangePassphrase)));
        } else {
            assert!(h.state().request.is_none());
        }
    }
}
