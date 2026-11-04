/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, NodeKind};

use crate::{
    DomHandle, GenetAppRunner, GenetCtx, GenetElement, PointerClick, View, el, lens, map_action,
    on_click,
};

/// The text of the single text child under `node`, if any.
fn text_child(dom: &ScriptedDom, node: NodeId) -> Option<String> {
    dom.dom_children(node)
        .find(|&c| dom.kind(c) == NodeKind::Text)
        .and_then(|c| dom.text(c).map(str::to_string))
}

/// Every element in `node`'s subtree (pre-order) named `name`.
fn elements_named(dom: &ScriptedDom, node: NodeId, name: &str) -> Vec<NodeId> {
    let mut out = Vec::new();
    fn walk(dom: &ScriptedDom, node: NodeId, name: &str, out: &mut Vec<NodeId>) {
        if dom.kind(node) == NodeKind::Element
            && dom
                .element_name(node)
                .is_some_and(|q| AsRef::<str>::as_ref(&q.local) == name)
        {
            out.push(node);
        }
        for c in dom.dom_children(node) {
            walk(dom, c, name, out);
        }
    }
    walk(dom, node, name, &mut out);
    out
}

// A reusable counter component over a bare `u32` sub-state; the canonical
// `lens` component shape. `+ use<>` keeps the opaque type from capturing the
// input lifetime (else it can't be a single `V` for `FnMut(&_) -> V`).
fn counter_button(count: &mut u32) -> impl View<u32, (), GenetCtx, Element = GenetElement> + use<> {
    on_click(
        el::<_, u32, ()>("button", count.to_string()),
        |c: &mut u32, _ev| *c += 1,
    )
}

struct App {
    left: u32,
    right: u32,
}

fn app_view(_s: &App) -> impl View<App, (), GenetCtx, Element = GenetElement> + use<> {
    el::<_, App, ()>(
        "div",
        (
            lens(counter_button, |s: &mut App| &mut s.left),
            lens(counter_button, |s: &mut App| &mut s.right),
        ),
    )
}

/// `lens` composes two independently-stateful counters: clicking the left
/// button moves only `App::left`, and the rebuild updates only its text.
#[test]
fn lens_isolates_substate() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner =
        GenetAppRunner::<_, _, _, ()>::new(dom.clone(), app_view, App { left: 0, right: 0 });
    let root = runner.root();

    let (left, right) = {
        let d = dom.borrow();
        let bs = elements_named(&d, root, "button");
        assert_eq!(bs.len(), 2, "two counter buttons");
        (bs[0], bs[1])
    };

    runner.dispatch_click(left, PointerClick::at((0.0, 0.0)));
    assert_eq!(
        (runner.state().left, runner.state().right),
        (1, 0),
        "only the lensed left sub-state changes"
    );
    {
        let d = dom.borrow();
        assert_eq!(text_child(&d, left).as_deref(), Some("1"));
        assert_eq!(text_child(&d, right).as_deref(), Some("0"));
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Bump;
impl crate::Action for Bump {}

fn bump_button() -> impl View<(), Bump, GenetCtx, Element = GenetElement> + use<> {
    on_click(el::<_, (), Bump>("button", "+"), |_s: &mut (), _ev| Bump)
}

struct Parent {
    count: u32,
    unit: (),
}

fn parent_view(_s: &Parent) -> impl View<Parent, (), GenetCtx, Element = GenetElement> + use<> {
    let child = lens(|_u: &mut ()| bump_button(), |p: &mut Parent| &mut p.unit);
    el::<_, Parent, ()>(
        "div",
        map_action(child, |p: &mut Parent, _a: Bump| {
            p.count += 1;
        }),
    )
}

/// `OptionalAction` + `map_action`: the child returns a `Bump` action, which
/// `map_action` turns into the parent effect `count += 1`.
#[test]
fn map_action_applies_parent_effect() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner =
        GenetAppRunner::<_, _, _, ()>::new(dom.clone(), parent_view, Parent { count: 0, unit: () });
    let root = runner.root();
    let button = {
        let d = dom.borrow();
        elements_named(&d, root, "button")[0]
    };

    runner.dispatch_click(button, PointerClick::at((0.0, 0.0)));
    assert_eq!(
        runner.state().count,
        1,
        "child Bump action maps to the parent increment"
    );
}
