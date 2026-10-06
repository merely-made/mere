// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Stage 3's receipts (stack seams P2): two windows over one forest document,
//! windowless. Each window lays out, hit-tests and projects accessibility from
//! its own subtree; a click in one changes the other in the same pass; a
//! mutation that only one window shows rebuilds only that window's layout; a
//! node moved between window roots keeps its identity, is laid out by its new
//! window, and a leaf moved with it keeps its painter.

use std::cell::Cell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;

use cambium::{
    AnyView, GenetCtx, GenetElement, ProjectionId, attr_qual, clickable, el, html_qual, text,
};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom as _, LayoutDomMut as _};

use crate::{
    AppShared, Host, HostHooks, HostOptions, HostState, HostTree as _, HostWake, MultiHost,
    WindowDom, WindowTree,
};

#[derive(Default)]
struct App {
    clicks: u32,
    a_note: String,
    b_note: String,
}

type Child = Box<dyn AnyView<App, (), GenetCtx, GenetElement>>;
type Logic = Box<dyn FnMut(&App) -> Child>;
type Multi = MultiHost<App, Logic, Child>;
type WindowHost = Host<App, Logic, Child, WindowTree<App, Logic, Child>>;

const SHEET: &str = "div { display: block; height: 20px; } \
                     button { display: block; width: 80px; height: 30px; } \
                     custom-leaf { display: block; width: 40px; height: 10px; }";

/// Window `label`'s lens: its label, a button that counts, the count every
/// window shows, and a note only that window shows.
fn lens(label: &'static str) -> Logic {
    Box::new(move |app: &App| {
        let note = if label == "A" {
            &app.a_note
        } else {
            &app.b_note
        };
        Box::new(el(
            "div",
            (
                el("div", text(format!("window {label}"))),
                clickable(el("button", text("count")), |app: &mut App, _| {
                    app.clicks += 1
                }),
                el("div", text(format!("clicks:{}", app.clicks))),
                el("div", text(format!("note:{note}"))),
            ),
        )) as Child
    })
}

fn multi() -> (Multi, ProjectionId, ProjectionId) {
    let mut shared = AppShared::default();
    shared.sheet = SHEET.into();
    let hooks: HostHooks<App, Logic, Child, WindowTree<App, Logic, Child>> = HostHooks::inert();
    let mut multi = MultiHost::new(App::default(), shared, hooks);
    let open = |multi: &mut Multi, label: &'static str, zoom: f32| {
        let s = HostState::new();
        let wake = HostWake::new(s.wake_pending.clone(), Arc::new(|| {}));
        let options = HostOptions {
            ui_zoom: zoom,
            ..HostOptions::default()
        };
        multi.open(
            lens(label),
            Host::new(options, None, HostHooks::inert(), s, wake),
        )
    };
    let a = open(&mut multi, "A", 1.0);
    let b = open(&mut multi, "B", 2.0);
    (multi, a, b)
}

/// Lay window `id` out for a surface this big, as the windowless harness does.
fn layout_at(multi: &mut Multi, id: ProjectionId, width: f32, height: f32) {
    multi
        .with_window(id, |host| {
            host.set_surface_size(width, height);
            host.refresh_fit_zoom();
            let zoom = host.ui_zoom();
            host.relayout(width / zoom, height / zoom);
        })
        .expect("the window is open");
}

/// The element whose text is `needle`, anywhere under `root`.
fn find(dom: &ScriptedDom, root: NodeId, needle: &str) -> Option<NodeId> {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if dom.dom_children(node).any(|c| dom.text(c) == Some(needle)) {
            return Some(node);
        }
        stack.extend(dom.dom_children(node));
    }
    None
}

/// All text under `root`, in no particular order.
fn text_under(dom: &ScriptedDom, root: NodeId) -> String {
    let mut out = String::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if let Some(t) = dom.text(node) {
            out.push_str(t);
            out.push('|');
        }
        stack.extend(dom.dom_children(node));
    }
    out
}

/// Where `node` paints in window `id`'s layout, if it does.
fn painted(multi: &mut Multi, id: ProjectionId, node: NodeId) -> Option<(f32, f32, f32, f32)> {
    let dom = multi.dom();
    let root = multi.window_root(id)?;
    multi
        .with_window(id, |host: &mut WindowHost| {
            let dom = dom.borrow();
            host.s
                .layout
                .as_ref()?
                .painted_rect(&WindowDom::new(&dom, root), node)
        })
        .flatten()
}

#[test]
fn each_window_lays_out_its_own_subtree_at_its_own_size_and_scale() {
    let (mut multi, a, b) = multi();
    layout_at(&mut multi, a, 400.0, 300.0);
    layout_at(&mut multi, b, 300.0, 600.0);
    let dom = multi.dom();
    let (root_a, root_b) = (multi.window_root(a).unwrap(), multi.window_root(b).unwrap());
    let (label_a, label_b) = {
        let d = dom.borrow();
        (
            find(&d, root_a, "window A").unwrap(),
            find(&d, root_b, "window B").unwrap(),
        )
    };
    assert!(
        painted(&mut multi, a, label_a).is_some(),
        "A paints its own label"
    );
    assert!(
        painted(&mut multi, a, label_b).is_none(),
        "A does not lay out B's"
    );
    assert!(
        painted(&mut multi, b, label_b).is_some(),
        "B paints its own label"
    );
    assert!(
        painted(&mut multi, b, label_a).is_none(),
        "B does not lay out A's"
    );
    let (size_a, scale_a) = multi
        .with_window(a, |h| (h.s.layout_size, h.layout_scale()))
        .unwrap();
    let (size_b, scale_b) = multi
        .with_window(b, |h| (h.s.layout_size, h.layout_scale()))
        .unwrap();
    assert_eq!((size_a, scale_a), ((400.0, 300.0), 1.0));
    assert_eq!(
        (size_b, scale_b),
        ((150.0, 300.0), 2.0),
        "B lays out under its zoom"
    );
}

#[test]
fn a_click_in_one_window_changes_what_the_other_shows_in_the_same_pass() {
    let (mut multi, a, b) = multi();
    layout_at(&mut multi, a, 400.0, 300.0);
    layout_at(&mut multi, b, 300.0, 600.0);
    let dom = multi.dom();
    let button = find(&dom.borrow(), multi.window_root(a).unwrap(), "count").unwrap();
    let (x, y, w, h) = painted(&mut multi, a, button).expect("A paints its button");
    multi
        .with_window(a, |host| {
            host.pointer_moved(x + w / 2.0, y + h / 2.0);
            host.click();
            host.release();
        })
        .unwrap();
    assert_eq!(multi.state().clicks, 1);
    let root_b = multi.window_root(b).unwrap();
    assert!(
        text_under(&dom.borrow(), root_b).contains("clicks:1"),
        "B's tree shows the click A took, with no redraw of B in between"
    );
}

#[test]
fn a_change_only_one_window_shows_rebuilds_only_that_windows_layout() {
    let (mut multi, a, b) = multi();
    layout_at(&mut multi, a, 400.0, 300.0);
    layout_at(&mut multi, b, 300.0, 600.0);
    // Settled: a second pass rebuilds neither.
    layout_at(&mut multi, a, 400.0, 300.0);
    layout_at(&mut multi, b, 300.0, 600.0);
    let rebuilt =
        |multi: &mut Multi, id| multi.with_window(id, |h| h.s.last_layout_rebuilt).unwrap();
    assert!(!rebuilt(&mut multi, a) && !rebuilt(&mut multi, b));

    // A note only A shows: the document changes under A alone.
    multi
        .with_window(a, |h| {
            h.s.runner
                .as_mut()
                .unwrap()
                .update(|app| app.a_note = "first".into())
        })
        .unwrap();
    assert_eq!(multi.touched_windows(), HashSet::from([a]));
    layout_at(&mut multi, a, 400.0, 300.0);
    layout_at(&mut multi, b, 300.0, 600.0);
    assert!(rebuilt(&mut multi, a), "A's layout took the change");
    assert!(!rebuilt(&mut multi, b), "B's layout did not rebuild");

    // The control: a note only B shows moves B's count and not A's.
    multi
        .with_window(a, |h| {
            h.s.runner
                .as_mut()
                .unwrap()
                .update(|app| app.b_note = "second".into())
        })
        .unwrap();
    assert_eq!(multi.touched_windows(), HashSet::from([b]));
    layout_at(&mut multi, a, 400.0, 300.0);
    layout_at(&mut multi, b, 300.0, 600.0);
    assert!(!rebuilt(&mut multi, a));
    assert!(rebuilt(&mut multi, b));
}

/// An accessibility bridge that records the node ids of the tree the host
/// hands it, so the test reads what the host's own sync path projected.
struct Recording(Rc<std::cell::RefCell<HashSet<u64>>>);

impl crate::Accessibility for Recording {
    fn sync(
        &mut self,
        dom: &WindowDom<'_>,
        layout: &crate::OwnedLayout,
        _: &mut sprigging::LeafRegistry<u64>,
        _: &mut crate::ProducerRegistry,
        focus: Option<u64>,
        _: f64,
    ) -> Vec<crate::A11yRequest> {
        let projection = crate::document_projection(dom, layout, focus);
        *self.0.borrow_mut() = projection
            .nodes()
            .iter()
            .map(|node| node.id.get())
            .collect();
        Vec::new()
    }
}

#[test]
fn each_windows_accessibility_tree_is_its_own_subtree() {
    let (mut multi, a, b) = multi();
    let seen_a = Rc::new(std::cell::RefCell::new(HashSet::new()));
    let seen_b = Rc::new(std::cell::RefCell::new(HashSet::new()));
    multi
        .with_window(a, |h| h.s.a11y = Some(Box::new(Recording(seen_a.clone()))))
        .unwrap();
    multi
        .with_window(b, |h| h.s.a11y = Some(Box::new(Recording(seen_b.clone()))))
        .unwrap();
    layout_at(&mut multi, a, 400.0, 300.0);
    layout_at(&mut multi, b, 300.0, 600.0);
    multi.with_window(a, |h| h.sync_a11y()).unwrap();
    multi.with_window(b, |h| h.sync_a11y()).unwrap();
    let dom = multi.dom();
    let d = dom.borrow();
    let label = |root, needle| d.opaque_id(find(&d, root, needle).unwrap());
    let (root_a, root_b) = (multi.window_root(a).unwrap(), multi.window_root(b).unwrap());
    let (seen_a, seen_b) = (seen_a.borrow(), seen_b.borrow());
    assert!(seen_a.contains(&label(root_a, "window A")));
    assert!(
        !seen_a.contains(&label(root_b, "window B")),
        "A's tree holds none of B"
    );
    assert!(seen_b.contains(&label(root_b, "window B")));
    assert!(
        seen_a.is_disjoint(&seen_b),
        "no accessible node is in both windows"
    );
}

/// A leaf that counts its paints.
struct CountingLeaf(Rc<Cell<u32>>);

impl sprigging::Leaf for CountingLeaf {
    fn measure(&mut self, _: sprigging::SizeHint, _: sprigging::SizeHint) -> sprigging::Size {
        sprigging::Size {
            width: 40.0,
            height: 10.0,
        }
    }

    fn paint(&mut self, _: &mut sprigging::PaintCx<'_>) {
        self.0.set(self.0.get() + 1);
    }

    fn paint_dirty(&self) -> bool {
        false
    }
}

#[test]
fn a_node_moved_between_windows_keeps_its_identity_and_its_leaf_keeps_its_painter() {
    let (mut multi, a, b) = multi();
    let dom = multi.dom();
    let (root_a, root_b) = (multi.window_root(a).unwrap(), multi.window_root(b).unwrap());
    // A tile the test owns, under A's window root beside A's view, holding a
    // custom leaf.
    let (tile, leaf) = {
        let mut d = dom.borrow_mut();
        let tile = d.create_element(html_qual("div"));
        let label = d.create_text("tile");
        d.append_child(tile, label);
        let leaf = d.create_element(html_qual("custom-leaf"));
        d.set_attribute(leaf, attr_qual("key"), "9");
        d.append_child(tile, leaf);
        d.append_child(root_a, tile);
        (tile, leaf)
    };
    let paints = Rc::new(Cell::new(0));
    multi
        .with_window(a, |h| {
            h.s.shared
                .leaves
                .insert(9, Box::new(CountingLeaf(paints.clone())))
        })
        .unwrap();
    layout_at(&mut multi, a, 400.0, 300.0);
    layout_at(&mut multi, b, 300.0, 600.0);
    assert!(painted(&mut multi, a, tile).is_some());
    assert!(painted(&mut multi, b, tile).is_none());
    assert_eq!(
        multi.with_window(a, |h| h.s.leaf_keys.clone()).unwrap(),
        vec![9]
    );
    assert_eq!(paints.get(), 1, "A painted the leaf");

    dom.borrow_mut().move_before(root_b, tile, None);
    assert_eq!(
        multi.touched_windows(),
        HashSet::from([a, b]),
        "both windows hear of the move"
    );
    layout_at(&mut multi, a, 400.0, 300.0);
    layout_at(&mut multi, b, 300.0, 600.0);
    assert!(
        painted(&mut multi, a, tile).is_none(),
        "A no longer lays the tile out"
    );
    assert!(
        painted(&mut multi, b, tile).is_some(),
        "B lays out the same node"
    );
    assert!(painted(&mut multi, b, leaf).is_some());
    assert!(multi.with_window(a, |h| h.s.leaf_keys.is_empty()).unwrap());
    assert_eq!(
        multi.with_window(b, |h| h.s.leaf_keys.clone()).unwrap(),
        vec![9]
    );
    multi
        .with_window(b, |h| {
            assert!(
                h.s.shared.leaves.get_mut(&9).is_some(),
                "the painter is the same leaf"
            );
            assert!(
                h.s.shared.rendered.get(9).is_some(),
                "and its output stands in B"
            );
        })
        .unwrap();
}

#[test]
fn a_turn_opens_and_closes_windows_through_its_tree() {
    let (mut multi, a, b) = multi();
    let c = multi
        .with_window(a, |h| {
            let tree = h.s.runner.as_mut().unwrap();
            let c = tree.open(lens("C"), HostOptions::default());
            tree.close(a);
            c
        })
        .unwrap();
    let (opened, closed) = multi.take_requests();
    assert_eq!(
        opened.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        vec![c]
    );
    assert_eq!(closed, vec![a]);
    assert!(
        multi.take_requests().0.is_empty(),
        "requests are taken once"
    );
    for (id, options) in opened {
        let s = HostState::new();
        let wake = HostWake::new(s.wake_pending.clone(), Arc::new(|| {}));
        multi.attach(id, Host::new(options, None, HostHooks::inert(), s, wake));
    }
    for id in closed {
        assert!(multi.close(id).is_some());
    }
    assert_eq!(multi.windows().collect::<Vec<_>>(), vec![b, c]);
    layout_at(&mut multi, c, 400.0, 300.0);
    let dom = multi.dom();
    let label = find(&dom.borrow(), multi.window_root(c).unwrap(), "window C").unwrap();
    assert!(
        painted(&mut multi, c, label).is_some(),
        "C lays out its own lens"
    );
    assert!(multi.window_root(a).is_none(), "A's root left the document");
}
