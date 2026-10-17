// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Equivalence gate for incremental restyle: after every frame of drained
//! mutations the session must equal one built fresh over the same final DOM.
//!
//! The list is the Cambium shape: a row is `div(span label, span count,
//! button)`. Each scenario is a stage of one running session, so a stage also
//! proves the cascade left by the stages before it.

use super::*;
use crate::WindowDom;
use genet_livery::resolve_styles;
use genet_scripted_dom::ScriptedDom;
use layout_dom_api::{LayoutDomMut, QualName};
use paint_list_api::PaintList;

const VIEWPORT: (f32, f32) = (400.0, 300.0);

const PLAIN: &str = "
    .list { display: block; }
    .row { display: flex; height: 24px; }
    .row span { padding: 2px; }
    .row.on { background-color: rgb(0, 128, 0); }
    .row.tall { height: 40px; }
    button { width: 40px; height: 20px; }";

/// Sibling and structural selectors widen every hint to the parent, and
/// counters re-run over the whole plane.
const STRUCTURAL: &str = "
    .list { display: block; counter-reset: row; }
    .row { display: flex; height: 24px; counter-increment: row; }
    .row:nth-child(odd) { background-color: rgb(240, 240, 240); }
    .row + .row { margin-top: 2px; }
    .row:first-child { border-top-width: 3px; }
    .row:last-child { margin-bottom: 9px; }
    .row.on { background-color: rgb(0, 128, 0); }
    .row.tall { height: 40px; }
    .row span { padding: 2px; }";

fn name(local: &str) -> QualName {
    QualName::new(None, Namespace::from(""), LocalName::from(local))
}

struct Row {
    node: NodeId,
    label: NodeId,
}

struct List {
    dom: ScriptedDom,
    list: NodeId,
    rows: Vec<Row>,
    sheet: &'static str,
}

impl List {
    fn new(count: usize, sheet: &'static str) -> Self {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let list = dom.create_element(name("div"));
        dom.set_attribute(list, name("class"), "list");
        dom.append_child(root, list);
        let mut this = Self {
            dom,
            list,
            rows: Vec::new(),
            sheet,
        };
        for index in 0..count {
            let row = this.make_row(index);
            this.dom.append_child(list, row.node);
            this.rows.push(row);
        }
        // The build is not a frame's work: a session starts from this DOM.
        this.dom.drain_mutations(&mut Vec::new());
        this
    }

    fn make_row(&mut self, index: usize) -> Row {
        let dom = &mut self.dom;
        let node = dom.create_element(name("div"));
        dom.set_attribute(node, name("class"), "row");
        let mut label = None;
        for (tag, text) in [
            ("span", format!("item {index}")),
            ("span", index.to_string()),
            ("button", "Go".to_owned()),
        ] {
            let cell = dom.create_element(name(tag));
            let text = dom.create_text(&text);
            dom.append_child(cell, text);
            dom.append_child(node, cell);
            label.get_or_insert(text);
        }
        Row {
            node,
            label: label.expect("a row has a label"),
        }
    }

    fn session(&self) -> OwnedLayout {
        OwnedLayout::new(
            &self.dom,
            &[self.sheet],
            VIEWPORT.0,
            VIEWPORT.1,
            &[],
            &Default::default(),
        )
    }

    /// One host frame: drain the log, rebuild incrementally, and require the
    /// result to equal a fresh session over the same DOM.
    fn frame(&mut self, layout: &mut OwnedLayout, scrolled: bool, stage: &str) -> usize {
        let mut mutations = Vec::new();
        self.dom.drain_mutations(&mut mutations);
        assert!(!mutations.is_empty(), "{stage}: the stage mutated nothing");
        layout.rebuild(&self.dom, VIEWPORT.0, VIEWPORT.1, &mutations);
        self.assert_equivalent(layout, scrolled, stage);
        layout.style.last_stats().restyled_elements
    }

    fn assert_equivalent(&self, layout: &mut OwnedLayout, scrolled: bool, stage: &str) {
        let mut fresh = self.session();
        let mut states = [&mut *layout, &mut fresh];
        if scrolled {
            // Past the first screenful, clamped by each session's own extent.
            for state in &mut states {
                state.set_viewport_scroll((0.0, 500.0));
            }
        }
        let [layout, fresh] = states;
        let resolved = resolve_styles(
            &self.dom,
            &layout.style_set,
            &layout.device,
            &layout.interactions,
        );
        assert!(
            *layout.style.styles() == resolved,
            "{stage}: style plane differs from resolve_styles"
        );
        assert!(
            layout.styles == fresh.styles,
            "{stage}: laid-out styles differ from a fresh session"
        );
        assert_eq!(
            layout.content_extent, fresh.content_extent,
            "{stage}: content extent"
        );
        assert_eq!(
            layout.viewport_scroll(),
            fresh.viewport_scroll(),
            "{stage}: scroll"
        );
        let mut pending = vec![self.dom.document()];
        while let Some(node) = pending.pop() {
            assert_eq!(
                layout.painted_rect(&self.dom, node),
                fresh.painted_rect(&self.dom, node),
                "{stage}: geometry of {node:?}"
            );
            pending.extend(self.dom.dom_children(node));
        }
        let stride = (self.rows.len() / 40).max(1);
        for row in self.rows.iter().step_by(stride) {
            let (x, y, w, h) = fresh
                .painted_rect(&self.dom, row.node)
                .expect("every row has a box");
            let (x, y) = (x + w / 2.0, y + h / 2.0);
            assert_eq!(
                layout.hit_test(&self.dom, x, y),
                fresh.hit_test(&self.dom, x, y),
                "{stage}: hit test through {:?}",
                row.node
            );
        }
        let size = DeviceIntSize::new(VIEWPORT.0 as i32, VIEWPORT.1 as i32);
        let paint = |layout: &mut OwnedLayout| {
            let list =
                layout.emit_paint_list_with_leaves(&self.dom, size, None, |_| None, |_| None);
            format!("{:?}", list.commands())
        };
        assert!(
            paint(layout) == paint(fresh),
            "{stage}: paint commands differ"
        );
    }
}

/// Every stage of the plan's gate, in one running session. The restyle count
/// of the stage is returned so a caller can pin how local the work stayed.
fn run_stages(count: usize, sheet: &'static str, scrolled: bool) -> Vec<(&'static str, usize)> {
    let mut list = List::new(count, sheet);
    let mut layout = list.session();
    let mid = count / 2;
    let mut work = Vec::new();
    let mut stage = |list: &mut List, layout: &mut OwnedLayout, label: &'static str| {
        work.push((label, list.frame(layout, scrolled, label)));
    };

    let label = list.rows[mid].label;
    list.dom.set_text(label, "changed");
    stage(&mut list, &mut layout, "text change");

    let row = list.rows[mid].node;
    list.dom.set_attribute(row, name("class"), "row on");
    stage(&mut list, &mut layout, "class change");

    list.dom.set_attribute(row, name("class"), "row on tall");
    stage(&mut list, &mut layout, "geometry change");

    list.dom
        .set_attribute(list.rows[mid + 1].node, name("style"), "height: 55px");
    list.dom
        .remove_attribute(list.rows[mid + 1].node, name("style"));
    stage(&mut list, &mut layout, "attribute set then removed");

    let fresh = list.make_row(count);
    let before = list.rows[mid].node;
    list.dom.insert_before(list.list, fresh.node, Some(before));
    list.rows.insert(mid, fresh);
    stage(&mut list, &mut layout, "row insert");

    let tail = list.make_row(count + 1);
    list.dom.append_child(list.list, tail.node);
    list.rows.push(tail);
    stage(&mut list, &mut layout, "row append");

    let gone = list.rows.remove(count / 3);
    list.dom.remove(gone.node);
    stage(&mut list, &mut layout, "row remove");

    let last = list.rows.pop().expect("rows remain");
    let first = list.rows[0].node;
    list.dom.move_before(list.list, last.node, Some(first));
    list.rows.insert(0, last);
    stage(&mut list, &mut layout, "row move");

    // The pre-`moveBefore` spelling: Removed + Inserted of the same subtree.
    let moved = list.rows.remove(1);
    let end = list.rows.last().expect("rows remain").node;
    list.dom.insert_before(list.list, moved.node, Some(end));
    list.rows.insert(list.rows.len() - 1, moved);
    stage(&mut list, &mut layout, "row move by reinsertion");

    let (a, b) = (list.rows[5].label, list.rows[count - 5].label);
    list.dom.set_text(a, "first of two");
    list.dom.set_text(b, "second of two");
    let far = list.rows[count - 5].node;
    list.dom.set_attribute(far, name("class"), "row on");
    stage(&mut list, &mut layout, "two rows in one frame");

    work
}

#[test]
fn incremental_style_equals_a_fresh_session_at_100_rows() {
    for sheet in [PLAIN, STRUCTURAL] {
        for scrolled in [false, true] {
            run_stages(100, sheet, scrolled);
        }
    }
}

#[test]
#[cfg_attr(
    debug_assertions,
    ignore = "1,000-row layouts take minutes unoptimised; run with cargo test --release"
)]
fn incremental_style_equals_a_fresh_session_at_1000_rows() {
    run_stages(1_000, STRUCTURAL, true);
}

/// The point of the exercise: a one-row change restyles a handful of
/// elements, not the document. Also the plain-sheet equivalence run at 1,000.
#[test]
#[cfg_attr(
    debug_assertions,
    ignore = "1,000-row layouts take minutes unoptimised; run with cargo test --release"
)]
fn a_text_change_and_a_row_attribute_change_restyle_locally() {
    let work = run_stages(1_000, PLAIN, false);
    let restyled = |stage: &str| work.iter().find(|(label, _)| *label == stage).unwrap().1;
    assert_eq!(restyled("text change"), 0);
    assert!(restyled("class change") <= 5, "{work:?}");
    assert!(restyled("two rows in one frame") <= 10, "{work:?}");
}

/// A window of a forest document is laid out through `WindowDom`, whose root
/// is an element presented as the document.
#[test]
fn incremental_style_equals_a_fresh_session_under_a_window_root() {
    let mut list = List::new(40, PLAIN);
    let window = list.list;
    let build = |list: &List| {
        let view = WindowDom::new(&list.dom, window);
        OwnedLayout::new(
            &view,
            &[PLAIN],
            VIEWPORT.0,
            VIEWPORT.1,
            &[],
            &Default::default(),
        )
    };
    let mut layout = build(&list);
    let label = list.rows[7].label;
    list.dom.set_text(label, "changed");
    list.dom
        .set_attribute(list.rows[9].node, name("class"), "row tall");
    let extra = list.make_row(40);
    list.dom.append_child(list.list, extra.node);
    list.rows.push(extra);
    let mut mutations = Vec::new();
    list.dom.drain_mutations(&mut mutations);
    let view = WindowDom::new(&list.dom, window);
    layout.rebuild(&view, VIEWPORT.0, VIEWPORT.1, &mutations);
    let fresh = build(&list);
    let resolved = resolve_styles(
        &view,
        &layout.style_set,
        &layout.device,
        &layout.interactions,
    );
    assert!(*layout.style.styles() == resolved);
    assert!(layout.styles == fresh.styles);
    for row in &list.rows {
        assert_eq!(
            layout.painted_rect(&view, row.node),
            fresh.painted_rect(&view, row.node)
        );
    }
}

/// A viewport change reaches `rebuild` as a device change, which must take the
/// full path rather than reuse a plane resolved for another viewport.
#[test]
fn a_viewport_change_falls_back_to_a_full_restyle() {
    let sheet = "@media (min-width: 500px) { .row { height: 60px; } } .row { height: 24px; }";
    let mut list = List::new(30, sheet);
    let mut layout = list.session();
    let label = list.rows[3].label;
    list.dom.set_text(label, "changed");
    let mut mutations = Vec::new();
    list.dom.drain_mutations(&mut mutations);
    layout.rebuild(&list.dom, 600.0, VIEWPORT.1, &mutations);
    assert!(layout.style.last_stats().device_invalidated);
    let fresh = OwnedLayout::new(
        &list.dom,
        &[sheet],
        600.0,
        VIEWPORT.1,
        &[],
        &Default::default(),
    );
    assert!(layout.styles == fresh.styles);
    assert_eq!(
        layout.painted_rect(&list.dom, list.rows[3].node),
        fresh.painted_rect(&list.dom, list.rows[3].node)
    );
}
