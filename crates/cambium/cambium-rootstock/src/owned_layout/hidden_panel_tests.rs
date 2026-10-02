// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A closed Cambium disclosure, accordion panel or tree group is marked
//! `hidden`, and HTML's rendering section gives `[hidden]` `display: none`.
//! Laid out by the host's own Livery session, a closed panel must generate no
//! box; the open panel beside it is the positive control. Whatever supplies
//! the rule (Genet's Cambium UA sheet since genet `4ac56bbbe0b`, or an inline
//! style before it), these fail if nothing does.

use std::{cell::RefCell, rc::Rc};

use cambium::{
    AccordionConfig, AccordionItem, AccordionState, AnyView, DisclosureState, DomHandle,
    GenetAppRunner, GenetCtx, GenetElement, TreeItem, TreeState, accordion, disclosure, lens,
    tree_view,
};
use genet_scripted_dom::ScriptedDom;

use super::*;

fn find_id(dom: &ScriptedDom, node: NodeId, id: &str) -> Option<NodeId> {
    if dom.attribute(node, &Namespace::from(""), &LocalName::from("id")) == Some(id) {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|child| find_id(dom, child, id))
}

/// Lay the DOM out and report, per id, whether it has a box and its height.
fn boxes(dom: &DomHandle, ids: &[&str]) -> Vec<Option<f32>> {
    let dom = dom.borrow();
    let layout = OwnedLayout::new(&*dom, &[""], 400.0, 600.0, &[], &Default::default());
    ids.iter()
        .map(|id| {
            let node = find_id(&dom, dom.document(), id).unwrap_or_else(|| panic!("no #{id}"));
            layout.fragments().get(node).map(|fragment| fragment.height)
        })
        .collect()
}

/// One disclosure, open or closed, in a DOM of its own.
fn disclosure_panel(expanded: bool) -> Option<f32> {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let _runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |state: &DisclosureState| disclosure(state, "Panel body", DisclosureState::toggle),
        DisclosureState::new("details", "Details").expanded(expanded),
    );
    boxes(&dom, &["details-panel"])[0]
}

#[test]
fn a_closed_disclosure_panel_generates_no_box() {
    let open = disclosure_panel(true);
    assert!(
        open.is_some_and(|height| height > 0.0),
        "the open panel has a box: {open:?}"
    );
    assert_eq!(
        disclosure_panel(false),
        None,
        "a closed disclosure panel generates no box"
    );
}

#[test]
fn a_closed_accordion_panel_generates_no_box() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let items = vec![
        AccordionItem::new("one", "One", "First panel"),
        AccordionItem::new("two", "Two", "Second panel"),
    ];
    let _runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        move |state: &AccordionState<&'static str>| {
            accordion(
                state,
                &items,
                AccordionConfig::default(),
                AccordionState::toggle,
            )
        },
        AccordionState::new().single(false).with_expanded(["one"]),
    );
    let [open, shut] = boxes(
        &dom,
        &[
            "cambium-accordion-item-one-panel",
            "cambium-accordion-item-two-panel",
        ],
    )[..] else {
        unreachable!()
    };
    assert!(
        open.is_some_and(|height| height > 0.0),
        "the open panel has a box: {open:?}"
    );
    assert_eq!(shut, None, "a closed accordion panel generates no box");
}

struct TreeApp {
    tree: TreeState<&'static str>,
}

type TreeAppView = Box<dyn AnyView<TreeApp, (), GenetCtx, GenetElement>>;

#[test]
fn a_collapsed_tree_group_generates_no_box() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let items = vec![
        TreeItem::new("root", "Root").with_children([
            TreeItem::new("alpha", "Alpha"),
            TreeItem::new("beta", "Beta"),
        ]),
        TreeItem::new("other", "Other").with_children([TreeItem::new("gamma", "Gamma")]),
    ];
    let _runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        move |_: &TreeApp| -> TreeAppView {
            let items = items.clone();
            Box::new(lens(
                move |tree: &mut TreeState<&'static str>| tree_view(tree, &items),
                |app: &mut TreeApp| &mut app.tree,
            ))
        },
        TreeApp {
            tree: TreeState::new()
                .with_id("hidden-tree")
                .with_expanded(["root"]),
        },
    );
    let [open, shut] = boxes(
        &dom,
        &[
            "hidden-tree-item-root-group",
            "hidden-tree-item-other-group",
        ],
    )[..] else {
        unreachable!()
    };
    assert!(
        open.is_some_and(|height| height > 0.0),
        "the open group has a box: {open:?}"
    );
    assert_eq!(shut, None, "a collapsed tree group generates no box");
}
