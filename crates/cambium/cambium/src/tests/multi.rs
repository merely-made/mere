/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, NodeKind};

use crate::{DomHandle, El, GenetMultiRunner, OnClick, PointerClick, el, on_click};

struct Counter {
    count: u32,
}

fn bump(state: &mut Counter, _click: PointerClick) {
    state.count += 1;
}

type ClickView = OnClick<El<String, Counter, ()>, Counter, (), fn(&mut Counter, PointerClick)>;

fn click_view(state: &Counter) -> ClickView {
    on_click(
        el::<_, Counter, ()>("div", state.count.to_string()),
        bump as fn(&mut Counter, PointerClick),
    )
}

fn fresh_dom() -> DomHandle {
    Rc::new(RefCell::new(ScriptedDom::new()))
}

fn text_of(dom: &DomHandle, root: NodeId) -> String {
    let dom = dom.borrow();
    dom.dom_children(root)
        .find(|&c| dom.kind(c) == NodeKind::Text)
        .and_then(|c| dom.text(c).map(str::to_string))
        .unwrap_or_default()
}

/// The thesis receipt: one state, N windows, nothing to synchronize — a
/// single `update` lands in every projection's dom.
#[test]
fn one_update_projects_into_every_window() {
    let mut runner = GenetMultiRunner::<_, _, _, ()>::new(Counter { count: 0 });
    let (dom_a, dom_b) = (fresh_dom(), fresh_dom());
    let a = runner.push_projection(dom_a.clone(), click_view);
    let b = runner.push_projection(dom_b.clone(), click_view);
    let (root_a, root_b) = (runner.root(a).unwrap(), runner.root(b).unwrap());
    assert_eq!(text_of(&dom_a, root_a), "0");
    assert_eq!(text_of(&dom_b, root_b), "0");

    runner.update(|s| s.count += 1);

    assert_eq!(text_of(&dom_a, root_a), "1");
    assert_eq!(text_of(&dom_b, root_b), "1");
}

/// `update_local` rebuilds only the named projection — the per-window path.
/// The one state still changes, but the other window's dom is left until it
/// rebuilds, so a per-window snapshot (an orrery render, pane rows) does not
/// churn N windows every frame.
#[test]
fn update_local_rebuilds_only_that_projection() {
    let mut runner = GenetMultiRunner::<_, _, _, ()>::new(Counter { count: 0 });
    let (dom_a, dom_b) = (fresh_dom(), fresh_dom());
    let a = runner.push_projection(dom_a.clone(), click_view);
    let b = runner.push_projection(dom_b.clone(), click_view);
    let (root_a, root_b) = (runner.root(a).unwrap(), runner.root(b).unwrap());

    runner.update_local(a, |s| s.count += 1);

    assert_eq!(runner.state().count, 1, "the one state changed");
    assert_eq!(text_of(&dom_a, root_a), "1", "the named window rebuilt");
    assert_eq!(
        text_of(&dom_b, root_b),
        "0",
        "the other window was not rebuilt"
    );
}

/// A dispatch into window A mutates the one truth and window B reflects it
/// in the same pass — the mirror fan-outs' replacement.
#[test]
fn dispatch_in_one_window_updates_the_others() {
    let mut runner = GenetMultiRunner::<_, _, _, ()>::new(Counter { count: 0 });
    let (dom_a, dom_b) = (fresh_dom(), fresh_dom());
    let a = runner.push_projection(dom_a.clone(), click_view);
    let b = runner.push_projection(dom_b.clone(), click_view);
    let (root_a, root_b) = (runner.root(a).unwrap(), runner.root(b).unwrap());

    runner.dispatch_click(a, root_a, PointerClick::at((0.0, 0.0)));

    assert_eq!(runner.state().count, 1);
    assert_eq!(text_of(&dom_a, root_a), "1");
    assert_eq!(text_of(&dom_b, root_b), "1", "the other window sees it too");
}

/// Focus and pointer capture are per-window interaction state, not shared.
#[test]
fn focus_is_per_window() {
    let mut runner = GenetMultiRunner::<_, _, _, ()>::new(Counter { count: 0 });
    let a = runner.push_projection(fresh_dom(), click_view);
    let b = runner.push_projection(fresh_dom(), click_view);
    let root_a = runner.root(a).unwrap();

    runner.set_focus(a, Some(root_a));
    assert_eq!(runner.focus(a), Some(root_a));
    assert_eq!(runner.focus(b), None, "window B's focus is its own");
}

/// Removing a projection tears its tree down and detaches its root; the
/// shared state and the other windows are untouched.
#[test]
fn remove_projection_tears_down_its_tree_only() {
    let mut runner = GenetMultiRunner::<_, _, _, ()>::new(Counter { count: 0 });
    let (dom_a, dom_b) = (fresh_dom(), fresh_dom());
    let a = runner.push_projection(dom_a.clone(), click_view);
    let b = runner.push_projection(dom_b.clone(), click_view);
    let root_a = runner.root(a).unwrap();
    let root_b = runner.root(b).unwrap();

    runner.remove_projection(a);

    assert_eq!(runner.projection_count(), 1);
    {
        let dom = dom_a.borrow();
        assert!(
            dom.dom_children(dom.document()).next().is_none(),
            "window A's document is empty after teardown"
        );
    }
    runner.update(|s| s.count += 5);
    assert_eq!(text_of(&dom_b, root_b), "5", "window B lives on");
    // A stale handle resolves to nothing rather than panicking.
    assert!(runner.root(a).is_none());
    assert_eq!(
        runner.dispatch_click(a, root_a, PointerClick::at((0.0, 0.0))),
        vec![]
    );
}
