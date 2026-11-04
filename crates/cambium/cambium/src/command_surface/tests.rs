/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{cell::RefCell, rc::Rc};

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LocalName, Namespace};

use super::*;
use crate::{DomHandle, GenetAppRunner, KeyEvent};

fn items() -> Vec<CommandItem> {
    vec![
        CommandItem::new("Open").with_id("open"),
        CommandItem::new("Export").with_id("export").with_children([
            CommandItem::new("Plain text"),
            CommandItem::new("PDF").disabled_because("PDF support is unavailable"),
            CommandItem::new("JSON"),
        ]),
        CommandItem::new("Close").disabled_because("Keep one document open"),
    ]
}

fn attr<'a>(dom: &'a ScriptedDom, node: NodeId, name: &str) -> Option<&'a str> {
    dom.attribute(node, &Namespace::from(""), &LocalName::from(name))
}

fn find_attr(dom: &ScriptedDom, root: NodeId, name: &str, value: &str) -> Option<NodeId> {
    if attr(dom, root, name) == Some(value) {
        return Some(root);
    }
    dom.dom_children(root)
        .find_map(|child| find_attr(dom, child, name, value))
}

/// The DOM id of the row carrying `key`, or `None` if no row does.
fn row_id_for_key(dom: &ScriptedDom, root: NodeId, key: &str) -> Option<String> {
    let node = find_attr(dom, root, "data-key", key)?;
    attr(dom, node, "id").map(str::to_string)
}

#[test]
fn every_row_carries_its_own_id_for_targeting() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let runner = GenetAppRunner::new(
        dom.clone(),
        |state: &CommandState| command_picker(state, &items()),
        CommandState::default(),
    );
    let dom = dom.borrow();
    for key in ["open", "export", "Close"] {
        let node = find_attr(&dom, runner.root(), "data-key", key)
            .unwrap_or_else(|| panic!("no row carries the key {key:?}"));
        // Contains, not equals: the selected row also carries `selected`.
        assert!(
            attr(&dom, node, "class").is_some_and(|c| c.contains("command-item")),
            "the key belongs to the row, not to something inside it"
        );
    }
    // An item that was never given an explicit id still carries one: it
    // defaults to the label, so a driver never has to know which kind of
    // item it is looking at.
    assert!(find_attr(&dom, runner.root(), "data-key", "Close").is_some());
}

#[test]
fn a_rows_key_holds_still_where_its_dom_id_does_not() {
    // Why the key exists. `item_dom_id` names a row by its index in the
    // caller's list, so a list that gains an entry ahead of it renames every
    // row after. A roster sorted by id does that whenever a persona is
    // added, and a driver targeting `...-item-1` would then be pointing at
    // somebody else.
    fn picker_over(names: Vec<&'static str>) -> (DomHandle, NodeId) {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let items: Vec<CommandItem> = names
            .into_iter()
            .map(|name| CommandItem::new(name).with_id(name))
            .collect();
        let runner = GenetAppRunner::new(
            dom.clone(),
            move |state: &CommandState| command_picker(state, &items),
            CommandState::default(),
        );
        let root = runner.root();
        (dom, root)
    }

    let (before, before_root) = picker_over(vec!["alt", "work"]);
    let (after, after_root) = picker_over(vec!["alt", "burner", "work"]);

    let before = before.borrow();
    let after = after.borrow();
    assert_ne!(
        row_id_for_key(&before, before_root, "work"),
        row_id_for_key(&after, after_root, "work"),
        "the positional id moved, which is the hazard"
    );
    assert!(
        row_id_for_key(&after, after_root, "work").is_some(),
        "and the key still finds the same row after the move"
    );
}

#[test]
fn palette_filters_and_skips_disabled_items() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        |state: &CommandState| command_palette(state, &items()),
        CommandState::default(),
    );
    runner.set_focus(Some(runner.root()));
    runner.dispatch_key(KeyEvent::new(Key::Character("cl".into())));
    assert_eq!(runner.state().query, "cl");
    let actions = runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter)));
    assert!(actions.is_empty(), "the only match is disabled");
    let disabled =
        find_attr(&dom.borrow(), runner.root(), "aria-disabled", "true").expect("disabled option");
    assert_eq!(
        attr(&dom.borrow(), disabled, "aria-description"),
        Some("Keep one document open")
    );
}

#[test]
fn menu_opens_and_closes_a_submenu_with_standard_arrows() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        |state: &CommandState| command_menu(state, &items(), 12.0, 16.0),
        CommandState::default(),
    );
    runner.set_focus(Some(runner.root()));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowDown)));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowRight)));
    assert_eq!(runner.state().submenu, Some(1));
    let parent =
        find_attr(&dom.borrow(), runner.root(), "aria-expanded", "true").expect("expanded parent");
    assert_eq!(attr(&dom.borrow(), parent, "role"), Some("menuitem"));

    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowDown)));
    let actions = runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter)));
    assert!(
        actions.is_empty(),
        "a disabled menu item is focusable but inert"
    );
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::End)));
    let actions = runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter)));
    assert_eq!(actions, [CommandEvent::Activate(vec![1, 2])]);

    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowLeft)));
    assert_eq!(runner.state().submenu, None);
    let actions = runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Escape)));
    assert_eq!(actions, [CommandEvent::Dismiss]);
}

#[test]
fn picker_uses_listbox_semantics_and_home_end() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        |state: &CommandState| command_picker(state, &items()),
        CommandState::default(),
    );
    assert_eq!(attr(&dom.borrow(), runner.root(), "role"), Some("listbox"));
    runner.set_focus(Some(runner.root()));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::End)));
    let actions = runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter)));
    assert_eq!(actions, [CommandEvent::Activate(vec![1])]);
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Home)));
    let actions = runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter)));
    assert_eq!(actions, [CommandEvent::Activate(vec![0])]);
}
