// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{GenetAppRunner, PointerClick, Slot, WorkspaceModel, el, on_click};
use genet_scripted_dom::ScriptedDom;
use std::cell::RefCell;
use workbench::{TileId, TileTree, Workspace};

#[test]
fn actual_owner_keeps_dom_and_state_after_destination_first_mount() {
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mount = RetainedMount::new(dom.clone());
    let mut owner = GenetAppRunner::new_at(
        dom.clone(),
        mount.node(),
        |count: &usize| {
            on_click(
                el::<_, usize, ()>("button", count.to_string()),
                |count: &mut usize, _: PointerClick| {
                    *count += 1;
                },
            )
        },
        0usize,
    );
    let button = owner.root();
    let tile = workbench::Tile {
        id: TileId(7),
        title: "Owner".into(),
        content: workbench::ContentSource::Open {
            kind: "test".into(),
            id: "7".into(),
        },
        accent: None,
    };
    let workspace = Workspace::new(TileTree::single(tile));
    let source_mount = mount.clone();
    let mut source = GenetAppRunner::new(
        dom.clone(),
        move |enabled: &bool| -> Box<dyn crate::AnyView<bool, (), GenetCtx, GenetElement>> {
            let fill = source_mount.clone();
            let enabled = *enabled;
            Box::new(crate::workspace_view(
                &WorkspaceModel {
                    workspace: &workspace,
                    current: Some(TileId(7)),
                    float_layer_visible: true,
                },
                |_: &mut bool, _| {},
                move |_| {
                    if enabled {
                        Slot::View(Box::new(fill.view::<bool, ()>()))
                    } else {
                        Slot::Surface
                    }
                },
            ))
        },
        true,
    );
    let old_parent = dom.borrow().parent(mount.node()).unwrap();
    let destination_mount = mount.clone();
    let mut destination = GenetAppRunner::new(
        dom.clone(),
        move |_: &()| destination_mount.view::<(), ()>(),
        (),
    );
    assert_ne!(dom.borrow().parent(mount.node()), Some(old_parent));
    // Explicit source teardown cannot withdraw a destination-first body.
    source.update(|enabled| *enabled = false);
    destination.update(|_| {});
    assert_eq!(dom.borrow().parent(button), Some(mount.node()));
    assert_eq!(owner.root(), button);
    owner.dispatch_click(button, PointerClick::at((1.0, 1.0)));
    assert_eq!(*owner.state(), 1);
    assert_eq!(owner.root(), button);
    assert_eq!(dom.borrow().parent(mount.node()), Some(destination.root()));
}

#[test]
fn source_first_withdrawal_gc_and_remount_preserve_actual_runner() {
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mount = RetainedMount::new(dom.clone());
    let mut owner = GenetAppRunner::new_at(
        dom.clone(),
        mount.node(),
        |count: &usize| {
            crate::on_key(
                on_click(
                    el::<_, usize, ()>("button", count.to_string()),
                    |count: &mut usize, _: PointerClick| *count += 1,
                ),
                |_: &mut usize, _: crate::KeyEvent| {},
            )
        },
        0usize,
    );
    let button = owner.root();
    owner.set_focus(Some(button));
    owner.dispatch_click(button, PointerClick::at((1.0, 1.0)));
    assert_eq!(
        owner.focus(),
        Some(button),
        "the actual keyboard-bearing owner is focused before withdrawal"
    );
    let source_mount = mount.clone();
    let mut source = GenetAppRunner::new(
        dom.clone(),
        move |enabled: &bool| -> Box<dyn crate::AnyView<bool, (), GenetCtx, GenetElement>> {
            if *enabled {
                Box::new(source_mount.view::<bool, ()>())
            } else {
                Box::new(el::<_, bool, ()>("div", "withdrawn"))
            }
        },
        true,
    );
    source.update(|enabled| *enabled = false);
    // Exercise the actual eager remove boundary and host collection while
    // the source wrapper is gone and the destination does not exist yet.
    dom.borrow_mut().collect([]);
    assert!(dom.borrow().is_live(mount.node()));
    assert!(dom.borrow().is_live(button));
    assert_eq!(dom.borrow().parent(mount.node()), Some(mount.0.parking));
    assert_eq!(owner.focus(), Some(button));
    let target_mount = mount.clone();
    let target = GenetAppRunner::new(dom.clone(), move |_: &()| target_mount.view::<(), ()>(), ());
    dom.borrow_mut().collect([]);
    owner.dispatch_click(button, PointerClick::at((1.0, 1.0)));
    assert_eq!(*owner.state(), 2);
    assert_eq!(owner.root(), button);
    assert_eq!(owner.focus(), Some(button));
    assert_eq!(dom.borrow().parent(mount.node()), Some(target.root()));
}

#[test]
fn final_owner_releases_after_actual_runner_state_and_cleans_parking() {
    struct State {
        dom: DomHandle,
        root: NodeId,
        destroyed: Rc<std::cell::Cell<bool>>,
    }
    impl Drop for State {
        fn drop(&mut self) {
            assert!(
                self.dom.borrow().is_live(self.root),
                "runner state owns a live subtree until it drops"
            );
            self.destroyed.set(true);
        }
    }
    // Host field order is part of custody: the actual runner is released
    // before the retained mount's final strong owner.
    struct Owned<Runner> {
        _runner: Runner,
        _mount: RetainedMount,
    }
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mount = RetainedMount::new(dom.clone());
    let root = mount.node();
    let parking = mount.0.parking;
    let destroyed = Rc::new(std::cell::Cell::new(false));
    let runner = GenetAppRunner::new_at(
        dom.clone(),
        root,
        |_: &State| el::<_, State, ()>("button", "real runner"),
        State {
            dom: dom.clone(),
            root,
            destroyed: destroyed.clone(),
        },
    );
    let body = runner.root();
    let owned = Owned {
        _runner: runner,
        _mount: mount,
    };
    assert!(dom.borrow().is_live(body));
    drop(owned);
    assert!(destroyed.get());
    assert!(!dom.borrow().is_live(body));
    assert!(!dom.borrow().is_live(root));
    assert!(!dom.borrow().is_live(parking));
}

#[test]
fn handled_tab_arrows_move_real_focus_but_programmatic_sync_does_not() {
    use crate::{Key, KeyEvent, NamedKey, TabStrip, tab_strip};
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mount = RetainedMount::new(dom.clone());
    let mut runner = GenetAppRunner::new_at(
        dom.clone(),
        mount.node(),
        |state: &TabStrip| tab_strip::<()>(state, &["One", "Two", "Three"]),
        TabStrip::new(0),
    );
    let tabs = dom.borrow().all_with_class(runner.root(), "tab");
    assert_eq!(tabs.len(), 3);
    assert_eq!(runner.focus(), None);
    runner.update(|state| state.selected = 1);
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowRight)));
    assert_eq!(
        runner.focus(),
        None,
        "an unfocused group cannot claim a key"
    );
    assert_eq!(runner.state().selected, 1);
    runner.set_focus(Some(tabs[1]));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowLeft)));
    assert_eq!(runner.state().selected, 0);
    assert_eq!(runner.focus(), Some(tabs[0]));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowRight)));
    assert_eq!(runner.state().selected, 1);
    assert_eq!(
        runner.focus(),
        Some(tabs[1]),
        "the next arrow starts from the actual current seat"
    );
    runner.update(|state| state.selected = 2);
    assert_eq!(
        runner.focus(),
        Some(tabs[1]),
        "programmatic selection does not request focus"
    );
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowRight)));
    assert_eq!(runner.state().selected, 2);
    assert_eq!(
        runner.focus(),
        Some(tabs[2]),
        "an explicit stock arrow moves focus even when selection already matches"
    );
    drop(runner);
    drop(mount);
}

#[test]
fn prevented_manual_tab_arrow_retains_the_unselected_focus_owner() {
    use crate::{Key, KeyEvent, NamedKey, on_key};
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mount = RetainedMount::new(dom.clone());
    let mut runner = GenetAppRunner::new_at(
        dom.clone(),
        mount.node(),
        |_: &()| {
            el::<_, (), ()>(
                "div",
                (
                    el::<_, (), ()>("div", "Selected A")
                        .attr("role", "tab")
                        .attr("aria-selected", "true")
                        .attr("tabindex", "0"),
                    on_key(
                        el::<_, (), ()>("div", "Focused B")
                            .attr("class", "manual-tab")
                            .attr("role", "tab")
                            .attr("aria-selected", "false")
                            .attr("tabindex", "0"),
                        |_: &mut (), event| event.prevent_default(),
                    ),
                ),
            )
            .attr("role", "tablist")
        },
        (),
    );
    let focused = dom.borrow().all_with_class(runner.root(), "manual-tab")[0];
    runner.set_focus(Some(focused));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowRight)));
    assert_eq!(
        runner.focus(),
        Some(focused),
        "preventing an arrow does not activate a manual tab"
    );
    drop(runner);
    drop(mount);
}

#[test]
fn tab_focus_request_cannot_escape_into_a_foreign_host_group() {
    use crate::{Key, KeyEvent, NamedKey, on_key};
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mount = RetainedMount::new(dom.clone());
    dom.borrow_mut()
        .set_attribute(mount.node(), crate::attr_qual("role"), "tablist");
    let mut runner = GenetAppRunner::new_at(
        dom.clone(),
        mount.node(),
        |_: &()| {
            on_key(
                el::<_, (), ()>("div", "Owned tab").attr("role", "tab"),
                |_: &mut (), event| {
                    event.prop.request_tab_focus(1);
                    event.prevent_default();
                },
            )
        },
        (),
    );
    let mut foreign = GenetAppRunner::new_at(
        dom.clone(),
        mount.node(),
        |_: &()| {
            on_key(
                el::<_, (), ()>("div", "Foreign tab")
                    .attr("role", "tab")
                    .attr("aria-selected", "true")
                    .attr("tabindex", "0"),
                |_: &mut (), _| {},
            )
        },
        (),
    );
    runner.set_focus(Some(runner.root()));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowRight)));
    assert_eq!(runner.focus(), Some(runner.root()));
    assert_eq!(foreign.focus(), None);
    foreign.set_focus(Some(foreign.root()));
    assert_ne!(runner.focus(), foreign.focus());
    drop(foreign);
    drop(runner);
    drop(mount);
}

#[test]
fn custom_tab_selection_change_does_not_imply_a_focus_request() {
    use crate::{Key, KeyEvent, NamedKey, on_key};
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mount = RetainedMount::new(dom.clone());
    let mut runner = GenetAppRunner::new_at(
        dom.clone(),
        mount.node(),
        |second: &bool| {
            el::<_, bool, ()>(
                "div",
                (
                    on_key(
                        el::<_, bool, ()>("div", "A")
                            .attr("role", "tab")
                            .attr("aria-selected", if *second { "false" } else { "true" })
                            .attr("tabindex", if *second { "-1" } else { "0" }),
                        |second: &mut bool, event| {
                            *second = true;
                            event.prevent_default();
                        },
                    ),
                    on_key(
                        el::<_, bool, ()>("div", "B")
                            .attr("role", "tab")
                            .attr("aria-selected", if *second { "true" } else { "false" })
                            .attr("tabindex", if *second { "0" } else { "-1" }),
                        |_: &mut bool, _| {},
                    ),
                ),
            )
            .attr("role", "tablist")
        },
        false,
    );
    let first = dom
        .borrow()
        .dom_children(runner.root())
        .next()
        .expect("first tab");
    runner.set_focus(Some(first));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowRight)));
    assert!(*runner.state());
    assert_eq!(
        runner.focus(),
        Some(first),
        "custom ARIA changes do not request focus"
    );
    drop(runner);
    drop(mount);
}

#[test]
fn reused_key_event_cannot_carry_a_tab_request_between_dispatches() {
    use crate::{Key, KeyEvent, NamedKey, on_key};
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mount = RetainedMount::new(dom.clone());
    let mut runner = GenetAppRunner::new_at(
        dom.clone(),
        mount.node(),
        |_: &()| {
            el::<_, (), ()>(
                "div",
                (
                    on_key(
                        el::<_, (), ()>("div", "A").attr("role", "tab"),
                        |_: &mut (), event| event.prevent_default(),
                    ),
                    on_key(
                        el::<_, (), ()>("div", "B")
                            .attr("role", "tab")
                            .attr("aria-selected", "true")
                            .attr("tabindex", "0"),
                        |_: &mut (), event| event.prevent_default(),
                    ),
                ),
            )
            .attr("role", "tablist")
        },
        (),
    );
    let tabs: Vec<_> = dom.borrow().dom_children(runner.root()).collect();
    runner.set_focus(Some(tabs[0]));
    let event = KeyEvent::new(Key::Named(NamedKey::ArrowRight));
    event.prop.set_key_target(Some((tabs[0], runner.root())));
    event.prop.request_tab_focus(1);
    runner.dispatch_key(event.clone());
    assert_eq!(runner.focus(), Some(tabs[0]));
    assert!(event.prop.take_tab_focus().is_none());
    runner.dispatch_key(event);
    assert_eq!(runner.focus(), Some(tabs[0]));
    drop(runner);
    drop(mount);
}

#[test]
fn explicit_focus_request_outranks_deferred_stock_tab_motion() {
    use crate::{Key, KeyEvent, NamedKey, on_key, request_focus};
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mount = RetainedMount::new(dom.clone());
    let mut runner = GenetAppRunner::new_at(
        dom.clone(),
        mount.node(),
        |requested: &bool| {
            el::<_, bool, ()>(
                "div",
                (
                    on_key(
                        request_focus(
                            el::<_, bool, ()>("div", "A").attr("role", "tab"),
                            *requested,
                        ),
                        |requested: &mut bool, event| {
                            *requested = true;
                            event.prop.request_tab_focus(1);
                            event.prevent_default();
                        },
                    ),
                    on_key(
                        el::<_, bool, ()>("div", "B")
                            .attr("role", "tab")
                            .attr("aria-selected", "true")
                            .attr("tabindex", "0"),
                        |_: &mut bool, _| {},
                    ),
                ),
            )
            .attr("role", "tablist")
        },
        false,
    );
    let first = dom
        .borrow()
        .dom_children(runner.root())
        .next()
        .expect("first tab");
    runner.set_focus(Some(first));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowRight)));
    assert!(*runner.state());
    assert_eq!(
        runner.focus(),
        Some(first),
        "even a same-node explicit focus request has priority"
    );
    drop(runner);
    drop(mount);
}

#[test]
fn retired_target_cannot_satisfy_a_deferred_tab_request() {
    use crate::{Key, KeyEvent, NamedKey, on_key};
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mount = RetainedMount::new(dom.clone());
    let mut runner = GenetAppRunner::new_at(
        dom.clone(),
        mount.node(),
        |visible: &bool| {
            el::<_, bool, ()>(
                "div",
                (
                    on_key(
                        el::<_, bool, ()>("div", "A").attr("role", "tab"),
                        |visible: &mut bool, event| {
                            *visible = false;
                            event.prop.request_tab_focus(1);
                            event.prevent_default();
                        },
                    ),
                    visible.then(|| {
                        on_key(
                            el::<_, bool, ()>("div", "B")
                                .attr("role", "tab")
                                .attr("aria-selected", "true")
                                .attr("tabindex", "0"),
                            |_: &mut bool, _| {},
                        )
                    }),
                ),
            )
            .attr("role", "tablist")
        },
        true,
    );
    let children: Vec<_> = dom.borrow().dom_children(runner.root()).collect();
    runner.set_focus(Some(children[0]));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowRight)));
    assert_eq!(runner.focus(), Some(children[0]));
    assert!(!dom.borrow().is_live(children[1]));
    drop(runner);
    drop(mount);
}

#[test]
#[should_panic(expected = "retained mount must share the projection document")]
fn unrelated_document_cannot_mount_a_foreign_owner_identity() {
    let owner = RetainedMount::new(Rc::new(RefCell::new(ScriptedDom::new())));
    let other = Rc::new(RefCell::new(ScriptedDom::new()));
    let _ = GenetAppRunner::new(other, move |_: &()| owner.view::<(), ()>(), ());
}
