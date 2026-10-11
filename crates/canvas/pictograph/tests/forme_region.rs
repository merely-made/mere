// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The public Canvas seam used by a host embedding a saved Forme.
#![cfg(feature = "canvas")]
use pictograph::canvas::{Canvas, FormeCell, FormeRegion, PointerButton, Role};
use uuid::Uuid;

fn fixture() -> (Canvas, Uuid, Uuid, Uuid) {
    let mut canvas = Canvas::with_sample_graph();
    let ids: Vec<_> = canvas.graph().nodes().map(|(_, n)| n.id).take(3).collect();
    canvas
        .set_forme_region(Some(FormeRegion {
            id: Uuid::from_u128(44),
            bounds: [0., 0., 400., 240.],
            cells: vec![
                FormeCell {
                    member: ids[0],
                    bounds: [0., 0., 0.5, 1.],
                },
                FormeCell {
                    member: ids[1],
                    bounds: [0.5, 0., 1., 1.],
                },
            ],
            locked: true,
            visible: true,
        }))
        .unwrap();
    (canvas, ids[0], ids[1], ids[2])
}
fn at(canvas: &Canvas, id: Uuid) -> (f32, f32) {
    let key = canvas.graph().get_node_key_by_id(id).unwrap();
    let p = canvas.node_position(key).unwrap();
    (p.x, p.y)
}
#[test]
fn local_holds_survive_physics_selection_and_translate_without_global_pause() {
    let (mut c, a, b, outside) = fixture();
    let before = at(&c, outside);
    for _ in 0..30 {
        c.frame(900, 600);
    }
    assert!(!c.physics_paused());
    assert_eq!(at(&c, a), (100., 120.));
    assert_eq!(at(&c, b), (300., 120.));
    assert_ne!(
        at(&c, outside),
        before,
        "outside body still participates in dynamics"
    );
    c.select_member(a);
    c.clear_selection();
    let mut region = c.forme_region().unwrap().clone();
    region.bounds = [200., 0., 600., 240.];
    c.set_forme_region(Some(region)).unwrap();
    c.frame(900, 600);
    assert_eq!(at(&c, a), (300., 120.));
    assert_eq!(at(&c, b), (500., 120.));
}
#[test]
fn explicit_pin_wins_and_unpin_rejoins_the_forme() {
    let (mut c, a, b, _) = fixture();
    c.select_member(a);
    c.pin_focused();
    let before = at(&c, a);
    let mut region = c.forme_region().unwrap().clone();
    region.bounds = [200., 0., 600., 240.];
    c.set_forme_region(Some(region)).unwrap();
    c.frame(900, 600);
    assert_eq!(at(&c, a), before);
    assert_eq!(at(&c, b), (500., 120.));
    assert_eq!(
        c.arrangement_role_of(c.graph().get_node_key_by_id(a).unwrap()),
        Role::Pinned
    );
    c.release_focused();
    c.frame(900, 600);
    assert_eq!(at(&c, a), (300., 120.));
    c.set_forme_region(None).unwrap();
    assert_ne!(
        c.arrangement_role_of(c.graph().get_node_key_by_id(b).unwrap()),
        Role::Pinned
    );
}
#[test]
fn ordinary_drag_is_withdrawn_but_click_and_shift_selection_still_work() {
    let (mut c, a, b, _) = fixture();
    c.frame(900, 600);
    let key = c.graph().get_node_key_by_id(a).unwrap();
    let (x, y) = c.screen_position_of(key).unwrap();
    c.pointer_down(PointerButton::Left, x, y);
    c.cursor_moved(x + 100., y + 100.);
    c.pointer_up(PointerButton::Left, x + 100., y + 100.);
    c.frame(900, 600);
    assert_eq!(at(&c, a), (100., 120.));
    c.select_member(a);
    c.set_shift(true);
    let key_b = c.graph().get_node_key_by_id(b).unwrap();
    let (bx, by) = c.screen_position_of(key_b).unwrap();
    c.pointer_down(PointerButton::Left, bx, by);
    c.pointer_up(PointerButton::Left, bx, by);
    c.set_shift(false);
    assert_eq!(c.selected_members().len(), 2);
    c.clear_selection();
    assert!(c.selected_members().is_empty());
    assert!(c.forme_region().is_some());
}
#[test]
fn invisible_boundary_retains_holds_and_malformed_extent_is_atomic() {
    let (mut c, a, _, _) = fixture();
    let before = c.forme_region().unwrap().clone();
    let mut bad = before.clone();
    bad.bounds[0] = f32::NAN;
    assert!(c.set_forme_region(Some(bad)).is_err());
    assert_eq!(c.forme_region(), Some(&before));
    let mut hidden = before;
    hidden.visible = false;
    c.set_forme_region(Some(hidden)).unwrap();
    c.frame(900, 600);
    assert_eq!(at(&c, a), (100., 120.));
    assert!(!c.forme_region_hovered());
}
#[test]
fn replacing_graph_drops_its_forme_instead_of_holding_foreign_members() {
    let (mut c, _, _, _) = fixture();
    let next = Canvas::with_sample_graph();
    let id = next.graph().nodes().next().unwrap().1.id;
    c.set_graph(next.graph().clone());
    assert!(c.forme_region().is_none());
    c.reseed();
    let seeded = at(&c, id);
    for _ in 0..30 {
        c.frame(900, 600);
    }
    assert_ne!(
        at(&c, id),
        seeded,
        "a reused node key must not inherit the old forme's pin"
    );
}
