/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{cell::RefCell, rc::Rc};

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LocalName, Namespace};

use super::*;
use crate::{AnyView, DomHandle, GenetAppRunner, KeyEvent, PointerClick, lens};

fn attr<'a>(dom: &'a ScriptedDom, node: NodeId, name: &str) -> Option<&'a str> {
    dom.attribute(node, &Namespace::from(""), &LocalName::from(name))
}

fn find_attr(dom: &ScriptedDom, node: NodeId, name: &str, value: &str) -> Option<NodeId> {
    if attr(dom, node, name) == Some(value) {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|child| find_attr(dom, child, name, value))
}

#[test]
fn disclosure_links_and_toggles_its_panel_by_pointer_and_keyboard() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |state: &DisclosureState| disclosure(state, "Panel body", DisclosureState::toggle),
        DisclosureState::new("details", "Details"),
    );
    let trigger = find_attr(&dom.borrow(), runner.root(), "id", "details-trigger")
        .expect("disclosure trigger");
    assert_eq!(attr(&dom.borrow(), trigger, "aria-expanded"), Some("false"));
    let panel =
        find_attr(&dom.borrow(), runner.root(), "id", "details-panel").expect("disclosure panel");
    assert_eq!(attr(&dom.borrow(), panel, "hidden"), Some("true"));

    runner.dispatch_click(trigger, PointerClick::at((4.0, 4.0)));
    assert!(runner.state().expanded);
    assert_eq!(attr(&dom.borrow(), trigger, "aria-expanded"), Some("true"));
    assert_eq!(attr(&dom.borrow(), panel, "hidden"), None);

    runner.set_focus(Some(trigger));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Space)));
    assert!(!runner.state().expanded);
}

struct TurnsApp {
    turns: DisclosureState,
    selected: Option<&'static str>,
    selections: usize,
}

type TurnsAppView = Box<dyn AnyView<TurnsApp, (), GenetCtx, GenetElement>>;

/// A panel whose rows call an application verb — the shape isometry's
/// Turns section needs, and the one the old `View<DisclosureState, ()>`
/// signature could not express.
fn turns_app_view(app: &TurnsApp) -> TurnsAppView {
    let rows: Vec<Box<dyn AnyView<TurnsApp, (), GenetCtx, GenetElement>>> = ["ada", "grace"]
        .into_iter()
        .map(|id| {
            Box::new(on_click(
                el::<_, TurnsApp, ()>("button", id).attr("id", format!("row-{id}")),
                move |app: &mut TurnsApp, _click: PointerClick| {
                    app.selected = Some(id);
                    app.selections += 1;
                },
            )) as Box<dyn AnyView<TurnsApp, (), GenetCtx, GenetElement>>
        })
        .collect();
    Box::new(disclosure(&app.turns, rows, |app: &mut TurnsApp| {
        app.turns.toggle()
    }))
}

#[test]
fn disclosure_content_acts_on_application_state_beside_the_widget_state() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        turns_app_view,
        TurnsApp {
            turns: DisclosureState::new("turns", "Turns").expanded(true),
            selected: None,
            selections: 0,
        },
    );
    let row = find_attr(&dom.borrow(), runner.root(), "id", "row-grace").expect("turn row");
    runner.dispatch_click(row, PointerClick::at((4.0, 4.0)));
    assert_eq!(runner.state().selected, Some("grace"));
    assert_eq!(runner.state().selections, 1);
    assert!(
        runner.state().turns.expanded,
        "a row acting on the application does not disturb the disclosure's own state"
    );

    // The two states stay independent in the other direction as well: the
    // widget's ARIA wiring and its keyboard toggle are what they were, and
    // collapsing leaves the application's selection alone.
    let trigger = find_attr(&dom.borrow(), runner.root(), "id", "turns-trigger").expect("trigger");
    let panel = find_attr(&dom.borrow(), runner.root(), "id", "turns-panel").expect("panel");
    assert_eq!(
        attr(&dom.borrow(), trigger, "aria-controls"),
        Some("turns-panel")
    );
    assert_eq!(attr(&dom.borrow(), trigger, "aria-expanded"), Some("true"));
    assert_eq!(
        attr(&dom.borrow(), panel, "aria-labelledby"),
        Some("turns-trigger")
    );
    runner.set_focus(Some(trigger));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Space)));
    assert!(!runner.state().turns.expanded);
    assert_eq!(attr(&dom.borrow(), trigger, "aria-expanded"), Some("false"));
    assert_eq!(attr(&dom.borrow(), panel, "hidden"), Some("true"));
    assert_eq!(
        runner.state().selected,
        Some("grace"),
        "collapsing the panel does not reach into the application's state"
    );
}

#[test]
fn trigger_announces_state_and_toggles_host_state_with_the_keyboard() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |open: &bool| {
            disclosure_trigger(
                if *open { "Hide list" } else { "Show list" },
                *open,
                |open: &mut bool| *open = !*open,
            )
        },
        false,
    );
    let trigger = runner.root();
    assert_eq!(attr(&dom.borrow(), trigger, "aria-expanded"), Some("false"));
    assert_eq!(attr(&dom.borrow(), trigger, "tabindex"), None);
    runner.dispatch_click(trigger, PointerClick::at((1.0, 1.0)));
    assert!(*runner.state());
    assert_eq!(attr(&dom.borrow(), trigger, "aria-expanded"), Some("true"));
    runner.set_focus(Some(trigger));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Space)));
    assert!(!*runner.state(), "Space toggles the trigger like a button");
}

fn accordion_items() -> Vec<AccordionItem<&'static str>> {
    vec![
        AccordionItem::new("one", "One", "First panel"),
        AccordionItem::new("two", "Two", "Second panel"),
    ]
}

#[test]
fn accordion_composes_heading_buttons_over_the_disclosure_control() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |state: &AccordionState<&'static str>| {
            accordion(
                state,
                &accordion_items(),
                AccordionConfig::default().with_heading_level(2),
                AccordionState::toggle,
            )
        },
        AccordionState::new().single(false).with_expanded(["one"]),
    );
    let first = find_attr(
        &dom.borrow(),
        runner.root(),
        "id",
        "cambium-accordion-item-one-trigger",
    )
    .expect("first accordion trigger");
    assert_eq!(attr(&dom.borrow(), first, "aria-expanded"), Some("true"));
    assert_eq!(attr(&dom.borrow(), first, "aria-disabled"), Some("true"));
    assert_eq!(
        attr(&dom.borrow(), first, "data-disclosure-control"),
        Some("true")
    );
    let heading = dom.borrow().parent(first).expect("heading parent");
    assert_eq!(attr(&dom.borrow(), heading, "role"), Some("heading"));
    assert_eq!(attr(&dom.borrow(), heading, "aria-level"), Some("2"));

    runner.dispatch_click(first, PointerClick::at((4.0, 4.0)));
    assert!(runner.state().is_expanded(&"one"));
    let second = find_attr(
        &dom.borrow(),
        runner.root(),
        "id",
        "cambium-accordion-item-two-trigger",
    )
    .expect("second accordion trigger");
    runner.dispatch_click(second, PointerClick::at((4.0, 4.0)));
    assert_eq!(runner.state().expanded, ["two"]);
}

fn tree_items() -> Vec<TreeItem<&'static str>> {
    vec![
        TreeItem::new("root", "Root").with_children([
            TreeItem::new("alpha", "Alpha"),
            TreeItem::new("beta", "Beta"),
        ]),
        TreeItem::new("other", "Other"),
    ]
}

struct TreeApp {
    tree: TreeState<&'static str>,
}

type TreeAppView = Box<dyn AnyView<TreeApp, (), GenetCtx, GenetElement>>;

fn tree_app_view(_: &TreeApp) -> TreeAppView {
    Box::new(lens(
        |tree: &mut TreeState<&'static str>| tree_view(tree, &tree_items()),
        |app: &mut TreeApp| &mut app.tree,
    ))
}

#[test]
fn recursive_tree_uses_disclosure_and_standard_hierarchical_navigation() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        tree_app_view,
        TreeApp {
            tree: TreeState::new()
                .with_id("test-tree")
                .with_expanded(["root"])
                .with_selected("root"),
        },
    );
    let root_item = find_attr(&dom.borrow(), runner.root(), "id", "test-tree-item-root")
        .expect("root tree item");
    assert_eq!(attr(&dom.borrow(), runner.root(), "role"), Some("tree"));
    assert_eq!(attr(&dom.borrow(), root_item, "role"), Some("treeitem"));
    assert_eq!(
        attr(&dom.borrow(), root_item, "data-disclosure-control"),
        Some("true")
    );
    assert_eq!(
        attr(&dom.borrow(), root_item, "aria-expanded"),
        Some("true")
    );
    let alpha = find_attr(&dom.borrow(), runner.root(), "id", "test-tree-item-alpha")
        .expect("alpha tree item");
    runner.dispatch_click(alpha, PointerClick::at((4.0, 4.0)));
    assert!(
        runner.state().tree.is_expanded(&"root"),
        "a child click does not bubble into the parent disclosure"
    );
    assert_eq!(runner.state().tree.selected(), Some(&"alpha"));
    runner.update(|app| {
        app.tree.active = Some("root");
        app.tree.selected = Some("root");
        app.tree.focus_active = true;
    });
    runner.set_focus(Some(root_item));

    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowRight)));
    assert_eq!(runner.state().tree.active(), Some(&"alpha"));
    assert_eq!(runner.focus(), Some(alpha));

    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowDown)));
    assert_eq!(runner.state().tree.active(), Some(&"beta"));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowLeft)));
    assert_eq!(runner.state().tree.active(), Some(&"root"));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowLeft)));
    assert!(!runner.state().tree.is_expanded(&"root"));
    assert_eq!(
        attr(&dom.borrow(), root_item, "aria-expanded"),
        Some("false")
    );
    let group = find_attr(
        &dom.borrow(),
        runner.root(),
        "id",
        "test-tree-item-root-group",
    )
    .expect("root child group");
    assert_eq!(attr(&dom.borrow(), group, "hidden"), Some("true"));
}

#[test]
fn explicit_tree_selection_stays_distinct_from_roving_focus() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        tree_app_view,
        TreeApp {
            tree: TreeState::new()
                .with_id("test-tree")
                .with_expanded(["root"])
                .with_selected("root")
                .with_selection_mode(TreeSelectionMode::Explicit),
        },
    );
    let root_item = find_attr(&dom.borrow(), runner.root(), "id", "test-tree-item-root")
        .expect("root tree item");
    runner.set_focus(Some(root_item));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowRight)));
    assert_eq!(runner.state().tree.active(), Some(&"alpha"));
    assert_eq!(runner.state().tree.selected(), Some(&"root"));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Space)));
    assert_eq!(runner.state().tree.selected(), Some(&"alpha"));
}
