/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;
use crate::{DomHandle, GenetAppRunner};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LocalName, Namespace};
use std::{cell::RefCell, rc::Rc};

fn items() -> Vec<CommandItem> {
    vec![
        CommandItem::new("File").with_id("file").with_children([
            CommandItem::new("Open")
                .with_id("open")
                .with_shortcut("Ctrl+O"),
            CommandItem::new("Export").with_id("export").with_children([
                CommandItem::new("Plain text").with_id("plain"),
                CommandItem::new("PDF")
                    .with_id("pdf")
                    .disabled_because("PDF unavailable"),
            ]),
            CommandItem::new("Close")
                .with_id("close")
                .disabled_because("Keep one document open"),
        ]),
        CommandItem::new("Edit")
            .with_id("edit")
            .with_children([CommandItem::new("Undo").with_id("undo")]),
    ]
}

fn attr<'a>(dom: &'a ScriptedDom, node: NodeId, name: &str) -> Option<&'a str> {
    dom.attribute(node, &Namespace::from(""), &LocalName::from(name))
}

fn find(dom: &ScriptedDom, root: NodeId, name: &str, value: &str) -> Option<NodeId> {
    if attr(dom, root, name) == Some(value) {
        return Some(root);
    }
    dom.dom_children(root)
        .find_map(|child| find(dom, child, name, value))
}

#[test]
fn top_roving_and_command_activation_keep_original_path() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        |state: &CommandMenuBarState| command_menu_bar(state, &items(), false),
        CommandMenuBarState::default(),
    );
    assert_eq!(attr(&dom.borrow(), runner.root(), "role"), Some("menubar"));
    runner.set_focus(Some(runner.root()));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowRight)));
    assert_eq!(runner.state().active, 1);
    let edit = find(&dom.borrow(), runner.root(), "data-key", "edit").expect("Edit item");
    assert_eq!(attr(&dom.borrow(), edit, "tabindex"), Some("0"));
    assert_eq!(runner.focus(), Some(edit));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Home)));
    assert_eq!(runner.state().active, 0);
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowDown)));
    let row = find(&dom.borrow(), runner.root(), "data-key", "open").expect("open row");
    assert_eq!(attr(&dom.borrow(), row, "role"), Some("menuitem"));
    assert_eq!(
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter))),
        [CommandEvent::Activate(vec![0, 0])]
    );
}

#[test]
fn submenu_disabled_item_is_described_and_inert() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        |state: &CommandMenuBarState| command_menu_bar(state, &items(), false),
        CommandMenuBarState::default(),
    );
    runner.set_focus(Some(runner.root()));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowDown)));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowDown)));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowRight)));
    assert_eq!(runner.state().open_path, vec![0, 1]);
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowDown)));
    let row = find(&dom.borrow(), runner.root(), "data-key", "pdf").expect("PDF row");
    assert_eq!(
        attr(&dom.borrow(), row, "aria-description"),
        Some("PDF unavailable")
    );
    assert!(
        runner
            .dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter)))
            .is_empty()
    );
    assert!(
        runner
            .dispatch_click(row, PointerClick::at((1.0, 1.0)))
            .is_empty()
    );
    assert!(runner.state().open);
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Home)));
    assert_eq!(
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter))),
        [CommandEvent::Activate(vec![0, 1, 0])]
    );
}

#[test]
fn compact_overflow_is_one_button_and_keeps_paths() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        |state: &CommandMenuBarState| command_menu_bar(state, &items(), true),
        CommandMenuBarState::default(),
    );
    let button = find(&dom.borrow(), runner.root(), "data-key", "menu").expect("Menu button");
    assert!(find(&dom.borrow(), runner.root(), "data-key", "file").is_none());
    runner.dispatch_click(button, PointerClick::at((1.0, 1.0)));
    assert!(find(&dom.borrow(), runner.root(), "data-key", "file").is_some());
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter)));
    assert_eq!(runner.state().open_path, vec![0]);
    assert_eq!(
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter))),
        [CommandEvent::Activate(vec![0, 0])]
    );
}

#[test]
fn entry_keys_and_escape() {
    let mut state = CommandMenuBarState::default();
    assert!(state.handle_entry_key(&KeyEvent::new(Key::Named(NamedKey::F10))));
    assert!(state.handle_entry_key(&KeyEvent::new(Key::Named(NamedKey::Alt))));
    assert!(!state.handle_entry_key(&KeyEvent::new(Key::Character("a".into()))));
    assert!(!state.handle_entry_key(&KeyEvent::with_mods(
        Key::Named(NamedKey::F10),
        crate::Modifiers {
            shift: true,
            ..Default::default()
        }
    )));
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom,
        |state: &CommandMenuBarState| command_menu_bar(state, &items(), false),
        state,
    );
    runner.set_focus(Some(runner.root()));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowDown)));
    assert_eq!(
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Escape))),
        [CommandEvent::Dismiss]
    );
    assert!(!runner.state().open);
}

#[test]
fn entry_request_rearms_after_focus_leaves_the_bar() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        |state: &CommandMenuBarState| command_menu_bar(state, &items(), false),
        CommandMenuBarState::default(),
    );
    let file = find(&dom.borrow(), runner.root(), "data-key", "file").expect("File item");
    runner.update(|state| state.enter());
    assert_eq!(runner.focus(), Some(file));
    runner.set_focus(None);
    assert!(
        !runner.state().focus_active,
        "blur resets the retained edge"
    );
    runner.update(|state| state.enter());
    assert_eq!(runner.focus(), Some(file), "F10 can enter again");

    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowDown)));
    assert!(runner.state().open);
    runner.set_focus(None);
    assert!(!runner.state().open, "blur dismisses the open popup");
    assert!(!runner.state().focus_active);
    runner.update(|state| state.enter());
    assert_eq!(runner.focus(), Some(file));
}

#[test]
fn width_mode_change_normalizes_an_open_menu() {
    let items = items();
    let mut state = CommandMenuBarState::default();
    state.show(&items, true);
    assert!(state.open_path.is_empty());
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        move |state: &CommandMenuBarState| command_menu_bar(state, &items, false),
        state,
    );
    // The wide view projects the stale compact disclosure closed. The
    // next interaction commits wide mode before opening a group.
    assert!(find(&dom.borrow(), runner.root(), "data-key", "open").is_none());
    runner.set_focus(Some(runner.root()));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowDown)));
    assert_eq!(runner.state().open_path, vec![0]);
    assert!(find(&dom.borrow(), runner.root(), "data-key", "open").is_some());
}

#[test]
fn disabled_top_items_do_not_take_home_end_tabstop() {
    let items = vec![
        CommandItem::new("File").disabled_because("Unavailable"),
        CommandItem::new("Edit").with_children([CommandItem::new("Undo")]),
        CommandItem::new("View").disabled(true),
    ];
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        move |state: &CommandMenuBarState| command_menu_bar(state, &items, false),
        CommandMenuBarState::default(),
    );
    let edit = find(&dom.borrow(), runner.root(), "data-key", "Edit").expect("Edit");
    assert_eq!(attr(&dom.borrow(), edit, "tabindex"), Some("0"));
    runner.set_focus(Some(runner.root()));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Home)));
    assert_eq!(runner.state().active, 1);
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::End)));
    assert_eq!(runner.state().active, 1);
    assert_eq!(runner.focus(), Some(edit));
}
