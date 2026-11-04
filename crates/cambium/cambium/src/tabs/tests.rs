/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LocalName, Namespace};

use super::*;
use crate::{DomHandle, GenetAppRunner, KeyEvent};

/// The action a closable strip bubbles in these tests.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Closed(usize);
impl crate::Action for Closed {}

fn attr_of<'a>(dom: &'a ScriptedDom, node: NodeId, name: &str) -> Option<&'a str> {
    dom.attribute(node, &Namespace::default(), &LocalName::from(name))
}

fn find_attr(dom: &ScriptedDom, root: NodeId, name: &str, value: &str) -> Option<NodeId> {
    if attr_of(dom, root, name) == Some(value) {
        return Some(root);
    }
    dom.dom_children(root)
        .find_map(|child| find_attr(dom, child, name, value))
}

fn dom_handle() -> DomHandle {
    Rc::new(RefCell::new(ScriptedDom::new()))
}

/// Two items, the second keyed and tinted.
fn items() -> Vec<TabItem> {
    vec![
        TabItem::from("One"),
        TabItem::new("Two")
            .with_key("two")
            .with_accent(TabAccentColors::new([1, 2, 3], [250, 251, 252])),
    ]
}

#[test]
fn a_marked_tab_hides_its_glyph_and_announces_its_state() {
    let dom = dom_handle();
    let runner = GenetAppRunner::new(
        dom.clone(),
        |s: &TabStrip| {
            tab_strip_items::<()>(
                s,
                &[
                    TabItem::from("One"),
                    TabItem::new("Two").with_mark(TabMark::Modified),
                ],
            )
        },
        TabStrip::new(0),
    );
    let dom = dom.borrow();
    let root = runner.root();
    let mark = find_attr(&dom, root, "data-mark", "modified").expect("the mark is drawn");
    assert_eq!(attr_of(&dom, mark, "class"), Some("tab-mark"));
    assert_eq!(attr_of(&dom, mark, "aria-hidden"), Some("true"));
    let tab = dom.parent(mark).expect("the mark sits in its tab");
    assert_eq!(attr_of(&dom, tab, "role"), Some("tab"));
    assert_eq!(
        attr_of(&dom, tab, "aria-description"),
        Some("Unsaved changes")
    );
    assert_eq!(
        dom.outer_html(root).matches("aria-description").count(),
        1,
        "only the marked tab carries a description"
    );
}

#[test]
fn new_holds_selection() {
    assert_eq!(TabStrip::new(2).selected, 2);
    assert_eq!(TabStrip::default().selected, 0);
    assert_eq!(TabStrip::default().label, "Tabs");
    assert_eq!(TabStrip::new(0).with_label("Roster").label, "Roster");
    assert!(!TabStrip::default().current);
    assert!(TabStrip::new(0).with_current(true).current);
}

#[test]
fn step_wraps_both_ways() {
    let mut s = TabStrip::new(0);
    s.step(-1, 4);
    assert_eq!(s.selected, 3, "left from the first wraps to the last");
    s.step(1, 4);
    assert_eq!(s.selected, 0, "right from the last wraps to the first");
    s.step(2, 4);
    assert_eq!(s.selected, 2);
}

#[test]
fn step_is_inert_without_tabs_and_clamps_a_stale_selection() {
    let mut s = TabStrip::new(3);
    s.step(1, 0);
    assert_eq!(s.selected, 3, "no tabs: the selection is left alone");
    // A selection past the end (the caller shrank the tab set) clamps before
    // stepping rather than wrapping from a bogus index.
    let mut s = TabStrip::new(9);
    s.step(1, 3);
    assert_eq!(s.selected, 0);
}

#[test]
fn a_label_strip_keeps_the_dom_it_always_had() {
    let dom = dom_handle();
    let runner = GenetAppRunner::new(
        dom.clone(),
        |s: &TabStrip| tab_strip::<()>(s, &["One", "Two"]),
        TabStrip::new(1).with_label("Roster"),
    );
    assert_eq!(
        dom.borrow().outer_html(runner.root()),
        "<div role=\"tablist\" class=\"tablist\" aria-label=\"Roster\">\
             <div role=\"tab\" aria-selected=\"false\" tabindex=\"-1\" class=\"tab\">One</div>\
             <div role=\"tab\" aria-selected=\"true\" tabindex=\"0\" class=\"tab selected\">Two</div>\
             </div>",
    );

    // The label door is the item door: same DOM, byte for byte.
    let items = dom_handle();
    let over_items = GenetAppRunner::new(
        items.clone(),
        |s: &TabStrip| tab_strip_items::<()>(s, &[TabItem::from("One"), TabItem::from("Two")]),
        TabStrip::new(1).with_label("Roster"),
    );
    assert_eq!(
        items.borrow().outer_html(over_items.root()),
        dom.borrow().outer_html(runner.root()),
    );
}

#[test]
fn the_current_strip_is_marked_on_the_tablist() {
    let dom = dom_handle();
    let runner = GenetAppRunner::new(
        dom.clone(),
        |s: &TabStrip| tab_strip::<()>(s, &["One"]),
        TabStrip::new(0).with_current(true),
    );
    let dom = dom.borrow();
    let root = runner.root();
    assert_eq!(attr_of(&dom, root, "aria-current"), Some("true"));
    assert_eq!(attr_of(&dom, root, "class"), Some("tablist current"));

    // And a strip that is not current carries neither.
    let plain = dom_handle();
    let plain_runner = GenetAppRunner::new(
        plain.clone(),
        |s: &TabStrip| tab_strip::<()>(s, &["One"]),
        TabStrip::new(0),
    );
    let plain = plain.borrow();
    let root = plain_runner.root();
    assert_eq!(attr_of(&plain, root, "aria-current"), None);
    assert_eq!(attr_of(&plain, root, "class"), Some("tablist"));
}

#[test]
fn an_item_carries_its_key_and_paints_its_accent() {
    let dom = dom_handle();
    let runner = GenetAppRunner::new(
        dom.clone(),
        |s: &TabStrip| tab_strip_items::<()>(s, &items()),
        TabStrip::new(0),
    );
    let dom = dom.borrow();
    let root = runner.root();
    let keyed = find_attr(&dom, root, "data-tabkey", "two").expect("the keyed tab");
    assert_eq!(attr_of(&dom, keyed, "role"), Some("tab"));
    assert_eq!(
        attr_of(&dom, keyed, "style"),
        Some("background-color: rgb(1, 2, 3); color: rgb(250, 251, 252);"),
    );
    // An item with neither carries neither attribute at all.
    let plain = find_attr(&dom, root, "class", "tab selected").expect("the first tab");
    assert_eq!(attr_of(&dom, plain, "data-tabkey"), None);
    assert_eq!(attr_of(&dom, plain, "style"), None);
}

#[test]
fn a_closable_tab_carries_a_close_control() {
    let dom = dom_handle();
    let runner = GenetAppRunner::new(
        dom.clone(),
        |s: &TabStrip| tab_strip_closable(s, &items(), Closed),
        TabStrip::new(0),
    );
    let dom = dom.borrow();
    let close = find_attr(&dom, runner.root(), "aria-label", "Close Two").expect("close");
    assert_eq!(attr_of(&dom, close, "role"), Some("button"));
    assert_eq!(attr_of(&dom, close, "class"), Some("tab-close"));
    // The whole shape: label text, then the close control, inside the tab.
    let tab = find_attr(&dom, runner.root(), "data-tabkey", "two").expect("the second tab");
    assert_eq!(
        dom.outer_html(tab),
        "<div role=\"tab\" aria-selected=\"false\" tabindex=\"-1\" class=\"tab\" \
             data-tabkey=\"two\" \
             style=\"background-color: rgb(1, 2, 3); color: rgb(250, 251, 252);\">Two\
             <span class=\"tab-close\" role=\"button\" aria-label=\"Close Two\">\u{00d7}</span>\
             </div>",
    );
}

#[test]
fn a_close_reports_its_index_and_leaves_the_selection_alone() {
    let dom = dom_handle();
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        |s: &TabStrip| tab_strip_closable(s, &items(), Closed),
        TabStrip::new(0),
    );
    let close = find_attr(&dom.borrow(), runner.root(), "aria-label", "Close Two")
        .expect("the second tab's close");
    let actions = runner.dispatch_click(close, PointerClick::at((1.0, 1.0)));
    assert_eq!(actions, [Closed(1)]);
    assert_eq!(
        runner.state().selected,
        0,
        "the close stops propagation, so it does not also activate its tab"
    );
    runner.set_focus(Some(close));
    assert_eq!(
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter))),
        [Closed(1)]
    );
    assert_eq!(
        runner.state().selected,
        0,
        "keyboard close preserves background focus too"
    );
}

#[test]
fn clicking_a_tab_selects_it_and_reports_nothing() {
    let dom = dom_handle();
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        |s: &TabStrip| tab_strip_closable(s, &items(), Closed),
        TabStrip::new(0),
    );
    let second =
        find_attr(&dom.borrow(), runner.root(), "data-tabkey", "two").expect("the second tab");
    let actions = runner.dispatch_click(second, PointerClick::at((1.0, 1.0)));
    assert!(
        actions.is_empty(),
        "activation is a state change, not an action"
    );
    assert_eq!(runner.state().selected, 1);
}
