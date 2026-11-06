// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use serde_json::{Value, json};

type Navigation = ScopeNavigation<String, String>;

fn visit(place: &str, anchor: &str) -> ScopeVisit<String, String> {
    ScopeVisit {
        place: place.to_owned(),
        camera: Some(ScopeCamera {
            x: 3.0,
            y: 7.0,
            zoom: 1.25,
        }),
        anchor: Some(anchor.to_owned()),
    }
}

#[test]
fn back_then_visit_keeps_the_old_branch_and_moves_the_current_cursor() {
    let mut nav = Navigation::default();
    nav.visit(visit("a", "a initial"));
    nav.visit(visit("b", "b initial"));
    assert_eq!(nav.destination(false).map(|v| v.place.as_str()), Some("a"));
    assert!(nav.step(false));

    nav.visit(visit("c", "c initial"));
    assert_eq!(nav.current().map(|v| v.place.as_str()), Some("c"));
    assert_eq!(nav.destination(false).map(|v| v.place.as_str()), Some("a"));
    assert_eq!(
        nav.visits().count(),
        6,
        "three lineage visits and three bookmarks remain stored"
    );

    assert!(nav.step(false));
    assert_eq!(nav.current().map(|v| v.place.as_str()), Some("a"));
    assert_eq!(nav.destination(true).map(|v| v.place.as_str()), Some("c"));
    assert!(nav.step(true));
    assert_eq!(nav.current().map(|v| v.place.as_str()), Some("c"));
}

#[test]
fn capture_updates_only_the_current_view_bookmark() {
    let mut nav = Navigation::default();
    nav.visit(visit("route at member 2", "first anchor"));
    let before = nav.lineage.visits[0].context.clone();

    nav.capture(visit("route at member 3", "updated anchor"));

    assert_eq!(nav.lineage.visits.len(), 1);
    assert_eq!(nav.lineage.visits[0].context, before);
    assert_eq!(nav.current().unwrap().place, "route at member 3");
    assert_eq!(
        nav.current().unwrap().anchor.as_deref(),
        Some("updated anchor")
    );
    assert_eq!(
        nav.saved_matching(|place| place.ends_with("3"))
            .unwrap()
            .place,
        "route at member 3"
    );
}

#[test]
fn serialized_lineage_keeps_cursor_and_per_visit_bookmarks() {
    let mut nav = Navigation::default();
    nav.visit(visit("a", "a old"));
    nav.visit(visit("b", "b retained"));
    assert!(nav.step(false));
    nav.capture(visit("a refreshed", "a current"));

    let encoded = serde_json::to_string(&nav).unwrap();
    let reopened: Navigation = serde_json::from_str(&encoded).unwrap();
    assert_eq!(reopened, nav);
    assert_eq!(
        reopened.current().unwrap().anchor.as_deref(),
        Some("a current")
    );
    assert_eq!(
        reopened.destination(true).unwrap().anchor.as_deref(),
        Some("b retained")
    );
    assert_eq!(
        reopened.visits().count(),
        5,
        "two visit bookmarks plus three remembered places"
    );
}

#[test]
fn legacy_entries_keep_duplicate_positions_and_restore_forward_cursor() {
    let first = serde_json::to_value(visit("same place", "first")).unwrap();
    let second = serde_json::to_value(visit("same place", "second")).unwrap();
    let legacy = json!({ "entries": [first, second], "cursor": 0, "remembered": [] });
    let mut nav: Navigation = serde_json::from_value(legacy).unwrap();

    assert_eq!(nav.current().unwrap().anchor.as_deref(), Some("first"));
    assert_eq!(
        nav.destination(true).unwrap().anchor.as_deref(),
        Some("second")
    );
    assert_eq!(
        nav.visits().count(),
        2,
        "preserve both positions without inventing remembered places"
    );
    assert!(nav.step(true));
    assert_eq!(nav.current().unwrap().anchor.as_deref(), Some("second"));
}

#[test]
fn current_wire_rejects_bad_references_and_invalid_camera() {
    let mut nav = Navigation::default();
    nav.visit(visit("a", "a"));
    let mut bad_reference = serde_json::to_value(&nav).unwrap();
    bad_reference["lineage"]["visits"][0]["entry"] = json!(42);
    assert!(serde_json::from_value::<Navigation>(bad_reference).is_err());

    let mut bad_camera: Value = serde_json::to_value(&nav).unwrap();
    bad_camera["bookmarks"][0]["value"]["camera"]["zoom"] = json!(0.0);
    assert!(serde_json::from_value::<Navigation>(bad_camera).is_err());

    let mut cyclic = serde_json::to_value(&nav).unwrap();
    cyclic["lineage"]["visits"][0]["parent"] = json!(0);
    assert!(serde_json::from_value::<Navigation>(cyclic).is_err());
}

#[test]
fn host_validation_checks_immutable_places_and_latest_bookmarks() {
    let mut nav = Navigation::default();
    nav.visit(visit("same graph: original", "anchor"));
    nav.capture(visit("foreign graph: focused member", "anchor"));

    assert!(
        nav.validate(|place| place.starts_with("same graph"), |_| true)
            .is_err()
    );
    assert!(nav.validate(|_| true, |anchor| anchor != "anchor").is_err());

    let mut bad_camera = visit("same graph: original", "anchor");
    bad_camera.camera = Some(ScopeCamera {
        x: 0.0,
        y: 0.0,
        zoom: 0.0,
    });
    nav.capture(bad_camera);
    assert!(nav.validate(|_| true, |_| true).is_err());
}

#[test]
fn reconstruction_rejects_corrupt_tree_and_owner_links_before_supplier_access() {
    let mut nav = Navigation::default();
    nav.visit(visit("a", "a"));
    nav.visit(visit("b", "b"));
    let wire = serde_json::to_value(&nav).unwrap();
    let cases: Vec<(&str, Value)> = vec![
        ("/lineage/visits/1/parent", json!(1)),
        ("/lineage/visits/0/children", json!([])),
        ("/lineage/visits/0/bindings/0/forward_child", json!(0)),
        ("/lineage/visits/1/bindings/0/owner", json!(12)),
        ("/lineage/owners/0/current", Value::Null),
        ("/lineage/owners/0/owned_visits", json!([0, 0])),
        ("/lineage/entries/0/visit_count", json!(500)),
        ("/bookmarks/1/visit", json!(0)),
    ];
    for (path, replacement) in cases {
        let mut damaged = wire.clone();
        *damaged.pointer_mut(path).unwrap() = replacement;
        assert!(
            serde_json::from_value::<Navigation>(damaged).is_err(),
            "{path}"
        );
    }
}

#[test]
fn try_map_preserves_branches_cursor_and_mapped_bookmarks() {
    let mut nav = Navigation::default();
    nav.visit(visit("a", "a anchor"));
    nav.visit(visit("b", "b anchor"));
    assert!(nav.step(false));
    nav.visit(visit("c", "c anchor"));
    assert_eq!(nav.lineage.visits.len(), 3);

    let mut mapped: ScopeNavigation<String, u8> = nav
        .try_map(|visit| {
            Ok(ScopeVisit {
                place: format!("runtime:{}", visit.place),
                camera: visit.camera,
                anchor: visit.anchor.as_ref().map(|anchor| anchor.len() as u8),
            })
        })
        .unwrap();

    assert_eq!(mapped.lineage.visits.len(), nav.lineage.visits.len());
    for (mapped_visit, old_visit) in mapped.lineage.visits.iter().zip(&nav.lineage.visits) {
        assert_eq!(mapped_visit.entry, old_visit.entry);
        assert_eq!(mapped_visit.parent, old_visit.parent);
        assert_eq!(mapped_visit.children, old_visit.children);
        assert_eq!(mapped_visit.bindings, old_visit.bindings);
        assert_eq!(
            mapped_visit.context.place,
            format!("runtime:{}", old_visit.context.place)
        );
        assert_eq!(
            mapped_visit.context.anchor,
            old_visit
                .context
                .anchor
                .as_ref()
                .map(|anchor| anchor.len() as u8)
        );
    }
    assert_eq!(mapped.current().unwrap().place, "runtime:c");
    assert_eq!(mapped.current().unwrap().anchor, Some(8));
    assert_eq!(mapped.visits().count(), nav.visits().count());
    assert!(mapped.step(false));
    assert_eq!(mapped.current().unwrap().place, "runtime:a");
    assert_eq!(mapped.destination(true).unwrap().place, "runtime:c");
}

#[test]
fn failed_try_map_does_not_change_the_source_navigation() {
    let mut nav = Navigation::default();
    nav.visit(visit("a", "a anchor"));
    nav.visit(visit("b", "b anchor"));
    assert!(nav.step(false));
    let before = nav.clone();

    let result: Result<ScopeNavigation<String, u8>, _> = nav.try_map(|visit| {
        if visit.place == "b" {
            Err("runtime place is unavailable".into())
        } else {
            Ok(ScopeVisit {
                place: visit.place.clone(),
                camera: visit.camera,
                anchor: None,
            })
        }
    });

    assert_eq!(result.unwrap_err(), "runtime place is unavailable");
    assert_eq!(nav, before);
}

#[test]
fn live_bookmark_mapping_preserves_exact_immutable_encounters_and_branches() {
    let mut nav = Navigation::default();
    nav.visit(visit("a", "a anchor"));
    nav.visit(visit("b", "b anchor"));
    assert!(nav.step(false));
    nav.visit(visit("c", "c anchor"));
    nav.remember(visit("separate remembered place", "remembered anchor"));
    let before = nav.clone();

    let mut mapped = nav
        .try_map_bookmarks(|visit| {
            let mut bookmark = visit.clone();
            bookmark.anchor = visit
                .anchor
                .as_ref()
                .map(|anchor| format!("{anchor}:closed"));
            Ok(bookmark)
        })
        .unwrap();

    assert_eq!(nav, before);
    assert_eq!(mapped.lineage, before.lineage);
    assert!(mapped.visits().all(|visit| {
        visit
            .anchor
            .as_deref()
            .is_some_and(|anchor| anchor.ends_with(":closed"))
    }));
    assert_eq!(
        mapped.current().unwrap().anchor.as_deref(),
        Some("c anchor:closed")
    );
    assert_eq!(
        mapped.lineage.visits[1].context.anchor.as_deref(),
        Some("b anchor")
    );
    let restored: Navigation =
        serde_json::from_value(serde_json::to_value(&mapped).unwrap()).unwrap();
    assert_eq!(restored, mapped);
    assert!(mapped.step(false));
    assert_eq!(mapped.current().unwrap().place, "a");
    assert_eq!(mapped.destination(true).unwrap().place, "c");
    assert_eq!(mapped.lineage.visits[0].children.len(), 2);
}

#[test]
fn failed_live_bookmark_mapping_leaves_original_state_intact() {
    let mut nav = Navigation::default();
    nav.visit(visit("a", "a anchor"));
    nav.visit(visit("b", "b anchor"));
    let before = nav.clone();
    let result = nav.try_map_bookmarks(|visit| {
        if visit.place == "b" {
            Err("view is unavailable".into())
        } else {
            let mut bookmark = visit.clone();
            bookmark.anchor = None;
            Ok(bookmark)
        }
    });
    assert_eq!(result.unwrap_err(), "view is unavailable");
    assert_eq!(nav, before);
}
