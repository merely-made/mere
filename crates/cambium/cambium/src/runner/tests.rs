/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{DomMutation, LayoutDom, LayoutDomMut, NodeKind};

use crate::{CompositionEvent, DomHandle, El, FocusPhase, el};

use super::{GenetAppRunner, is_true_token};

/// A counter: the canonical Stage 1b app state.
struct Counter {
    count: u32,
}

/// The app logic: a `<div>` holding the count as text. The element type is
/// the uniform `GenetElement`, so the view type is concrete (no boxing).
fn counter_view(s: &Counter) -> El<String, Counter, ()> {
    el::<_, Counter, ()>("div", s.count.to_string())
}

/// The text data of the (single) text child under `node`.
fn text_child(dom: &ScriptedDom, node: NodeId) -> Option<String> {
    dom.dom_children(node)
        .find(|&c| dom.kind(c) == NodeKind::Text)
        .and_then(|c| dom.text(c).map(str::to_string))
}

fn drain(dom: &DomHandle) -> Vec<DomMutation<NodeId>> {
    let mut out = Vec::new();
    dom.borrow_mut().drain_mutations(&mut out);
    out
}

#[test]
fn deferred_paint_updates_leave_semantics_until_reconciled() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(dom.clone(), counter_view, Counter { count: 0 });
    drain(&dom);
    for _ in 0..60 {
        runner.update_deferred(|state| state.count += 1);
    }
    assert_eq!(runner.state().count, 60);
    assert!(drain(&dom).is_empty());
    assert_eq!(
        text_child(&dom.borrow(), runner.root()).as_deref(),
        Some("0")
    );
    runner.update(|_| {});
    assert_eq!(
        text_child(&dom.borrow(), runner.root()).as_deref(),
        Some("60")
    );
    assert!(!drain(&dom).is_empty());
}

#[test]
fn counter_ticks_through_runner() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(dom.clone(), counter_view, Counter { count: 0 });

    // The initial tree: a <div> under the document root holding "0".
    let root = runner.root();
    {
        let dom = runner.dom();
        let dom = dom.borrow();
        assert_eq!(dom.kind(root), NodeKind::Element);
        assert_eq!(dom.element_name(root).unwrap().local.to_string(), "div");
        assert_eq!(text_child(&dom, root).as_deref(), Some("0"));
        // The root is attached under the document root.
        assert!(dom.dom_children(dom.document()).any(|c| c == root));
    }
    // The build recorded inserts (text under div, div under document) and
    // no removals.
    let build_muts = drain(&dom);
    assert!(
        build_muts
            .iter()
            .any(|m| matches!(m, DomMutation::Inserted { .. })),
        "build should record inserts: {build_muts:?}"
    );

    // Tick the counter a few times; each update diffs the new count into
    // the existing text node (a CharacterDataChanged, not a re-create).
    for expected in 1..=3u32 {
        runner.update(|s| s.count += 1);
        assert_eq!(runner.state().count, expected);

        {
            let dom = runner.dom();
            let dom = dom.borrow();
            assert_eq!(
                text_child(&dom, runner.root()).as_deref(),
                Some(expected.to_string().as_str()),
                "text under div should read the latest count"
            );
        }

        let muts = drain(&dom);
        // The only structural mutation a count change produces is the text
        // node's character-data update — no node churn.
        assert!(
            muts.iter()
                .any(|m| matches!(m, DomMutation::CharacterDataChanged { .. })),
            "tick to {expected} should record a CharacterDataChanged: {muts:?}"
        );
        assert!(
            !muts
                .iter()
                .any(|m| matches!(m, DomMutation::Inserted { .. })),
            "tick to {expected} should not insert nodes: {muts:?}"
        );
        assert!(
            !muts
                .iter()
                .any(|m| matches!(m, DomMutation::Removed { .. })),
            "tick to {expected} should not remove nodes: {muts:?}"
        );
    }
}

#[test]
fn hidden_and_parked_ancestors_are_not_admitted_or_focusable() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        |_count: &u32| {
            crate::button::<u32, (), (), _>(
                "increment",
                |count: &mut u32, _: crate::PointerClick| {
                    *count += 1;
                },
            )
        },
        0_u32,
    );
    let root = runner.root();
    runner.set_focus(Some(root));
    assert_eq!(runner.focus(), Some(root));

    dom.borrow_mut()
        .set_attribute(root, crate::attr_qual("inert"), "");

    assert!(
        runner
            .dispatch_key(crate::KeyEvent::new(crate::Key::Named(
                crate::NamedKey::Enter
            )))
            .is_empty()
    );
    assert_eq!(runner.state(), &0, "hidden key activation must be rejected");
    assert!(
        runner
            .dispatch_click(root, crate::PointerClick::at((0.0, 0.0)))
            .is_empty()
    );
    assert!(runner.focusables().is_empty());
    assert_eq!(runner.focus(), None, "a hidden focused node is scrubbed");
    assert_eq!(
        runner.state(),
        &0,
        "hidden activation must not mutate app state"
    );
    runner.set_focus(Some(root));
    assert_eq!(
        runner.focus(),
        None,
        "programmatic focus cannot target hidden content"
    );
}

#[derive(Default)]
struct HideInteractionState {
    hidden: bool,
    focus: Vec<FocusPhase>,
    pointer: Vec<crate::PointerPhase>,
    keys: usize,
    clicks: usize,
    values: usize,
}

fn hide_interaction_view(
    state: &HideInteractionState,
) -> impl crate::View<HideInteractionState, (), crate::GenetCtx, Element = crate::GenetElement> + use<>
{
    let element = if state.hidden {
        el::<_, HideInteractionState, ()>("div", "interaction").attr("hidden", "")
    } else {
        el::<_, HideInteractionState, ()>("div", "interaction")
    };
    crate::focusable(crate::on_focus(
        crate::on_value(
            crate::on_key(
                crate::on_pointer(
                    crate::on_click(element, |state: &mut HideInteractionState, _| {
                        state.clicks += 1;
                    }),
                    |state: &mut HideInteractionState, event: crate::PointerEvent| {
                        state.pointer.push(event.phase);
                    },
                ),
                |state: &mut HideInteractionState, _: crate::KeyEvent| state.keys += 1,
            ),
            |state: &mut HideInteractionState, event: crate::ValueEvent| {
                state.values += event.value as usize;
            },
        ),
        |state: &mut HideInteractionState, event: crate::FocusEvent| {
            state.focus.push(event.phase);
        },
    ))
}

#[test]
fn normal_hidden_rebuild_publishes_focus_lost_and_pointer_cancel_once() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner =
        GenetAppRunner::new(dom, hide_interaction_view, HideInteractionState::default());
    let root = runner.root();
    runner.set_focus(Some(root));
    runner.dispatch_pointer_down(
        root,
        crate::PointerEvent::new(crate::PointerPhase::Down, (1.0, 1.0), (4.0, 4.0)),
    );
    assert_eq!(runner.pointer_capture(), Some(root));
    assert_eq!(runner.state().focus, [FocusPhase::Gained]);
    assert_eq!(runner.state().pointer, [crate::PointerPhase::Down]);

    runner.update(|state| state.hidden = true);

    assert_eq!(runner.focus(), None);
    assert_eq!(runner.pointer_capture(), None);
    assert_eq!(runner.state().focus, [FocusPhase::Gained, FocusPhase::Lost]);
    assert_eq!(
        runner.state().pointer,
        [crate::PointerPhase::Down, crate::PointerPhase::Cancel]
    );
    runner.dispatch_key(crate::KeyEvent::new(crate::Key::Named(
        crate::NamedKey::Enter,
    )));
    runner.dispatch_pointer_down(
        root,
        crate::PointerEvent::new(crate::PointerPhase::Down, (1.0, 1.0), (4.0, 4.0)),
    );
    runner.dispatch_click(root, crate::PointerClick::at((1.0, 1.0)));
    assert_eq!(runner.state().keys, 0);
    assert_eq!(runner.state().clicks, 0);
    assert_eq!(runner.state().focus, [FocusPhase::Gained, FocusPhase::Lost]);
    assert_eq!(
        runner.state().pointer,
        [crate::PointerPhase::Down, crate::PointerPhase::Cancel]
    );
}

#[test]
fn aria_hidden_keeps_physical_clicks_but_rejects_accessibility_actions() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        hide_interaction_view,
        HideInteractionState::default(),
    );
    let root = runner.root();
    dom.borrow_mut()
        .set_attribute(root, crate::attr_qual("aria-hidden"), "true");

    runner.dispatch_click(root, crate::PointerClick::at((1.0, 1.0)));
    assert_eq!(runner.state().clicks, 1);
    assert!(runner.focusables().is_empty());
    assert!(
        runner
            .dispatch_value(root, crate::ValueEvent { value: 5.0 })
            .is_empty()
    );
    assert_eq!(runner.state().values, 0);
}

#[derive(Default)]
struct FileCompletionState {
    hidden: bool,
    completed: usize,
}

fn file_completion_view(
    state: &FileCompletionState,
) -> impl crate::View<FileCompletionState, (), crate::GenetCtx, Element = crate::GenetElement> + use<>
{
    let element = if state.hidden {
        el::<_, FileCompletionState, ()>("button", "choose").attr("hidden", "")
    } else {
        el::<_, FileCompletionState, ()>("button", "choose")
    };
    crate::open_file(
        element,
        true,
        crate::FileFilter::default(),
        |state: &mut FileCompletionState, _: crate::FileEvent| state.completed += 1,
    )
}

#[test]
fn hidden_live_file_owner_receives_already_started_completion() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(dom, file_completion_view, FileCompletionState::default());
    let request = runner.take_file_request().expect("initial file request");
    runner.update(|state| state.hidden = true);

    runner.dispatch_file(request.node, crate::FileEvent::default());

    assert_eq!(runner.state().completed, 1);
}

#[test]
fn admission_tokens_are_exact_and_boolean_attributes_are_presence_based() {
    assert!(is_true_token(" true "));
    assert!(is_true_token("1"));
    assert!(!is_true_token("true-ish"));
    assert!(!is_true_token("false"));
}

#[test]
fn hidden_numeric_control_does_not_accept_accessibility_value() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        |_: &u32| {
            crate::on_value(
                crate::el::<_, u32, ()>("input", ""),
                |state: &mut u32, event: crate::ValueEvent| *state = event.value as u32,
            )
        },
        0_u32,
    );
    let root = runner.root();
    dom.borrow_mut()
        .set_attribute(root, crate::attr_qual("inert"), "");
    assert!(
        runner
            .dispatch_value(root, crate::ValueEvent { value: 7.0 })
            .is_empty()
    );
    assert_eq!(runner.state(), &0);
}

#[test]
fn hiding_captured_pointer_target_cancels_capture_before_next_move() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        |_: &u32| {
            crate::on_pointer(
                crate::el::<_, u32, ()>("div", "drag"),
                |count: &mut u32, _: crate::PointerEvent| *count += 1,
            )
        },
        0_u32,
    );
    let root = runner.root();
    runner.dispatch_pointer_down(
        root,
        crate::PointerEvent::new(crate::PointerPhase::Down, (1.0, 1.0), (10.0, 10.0)),
    );
    assert_eq!(runner.pointer_capture(), Some(root));
    dom.borrow_mut()
        .set_attribute(root, crate::attr_qual("hidden"), "");

    runner.dispatch_pointer_move(crate::PointerEvent::new(
        crate::PointerPhase::Move,
        (2.0, 2.0),
        (10.0, 10.0),
    ));
    assert_eq!(runner.pointer_capture(), None);
    assert_eq!(
        runner.state(),
        &1,
        "hidden move must not reach the captured handler"
    );
}

#[test]
fn prepare_move_routes_pointer_cancel_before_host_reparents() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom,
        |_: &Vec<crate::PointerPhase>| {
            crate::on_pointer(
                crate::el::<_, Vec<crate::PointerPhase>, ()>("div", "drag"),
                |phases: &mut Vec<crate::PointerPhase>, event: crate::PointerEvent| {
                    phases.push(event.phase);
                },
            )
        },
        Vec::new(),
    );
    let root = runner.root();
    runner.dispatch_pointer_down(
        root,
        crate::PointerEvent::new(crate::PointerPhase::Down, (1.0, 1.0), (10.0, 10.0)),
    );
    assert!(runner.subtree_move_has_active_interaction(root));

    runner.prepare_subtree_move(root);

    assert_eq!(
        runner.state(),
        &[crate::PointerPhase::Down, crate::PointerPhase::Cancel]
    );
    assert_eq!(runner.pointer_capture(), None);
    assert!(!runner.subtree_move_has_active_interaction(root));
}

#[test]
fn preparing_move_commits_active_preedit_once_before_relocation() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(dom, crate::text_field_typed, crate::TextInput::new(""));
    let root = runner.root();
    runner.set_focus(Some(root));
    runner.dispatch_key(crate::KeyEvent::new(crate::Key::Composition(
        CompositionEvent::Preedit {
            text: "漢".into(),
            selection: None,
        },
    )));
    assert_eq!(runner.state().preedit(), "漢");
    assert!(runner.subtree_move_has_active_interaction(root));

    runner.prepare_subtree_move(root);
    assert!(!runner.subtree_move_has_active_interaction(root));
    assert_eq!(runner.state().text(), "漢");
    assert_eq!(runner.state().preedit(), "");

    runner.dispatch_key(crate::KeyEvent::new(crate::Key::Composition(
        CompositionEvent::Commit("漢".into()),
    )));
    assert_eq!(
        runner.state().text(),
        "漢",
        "matching late commit is deduplicated"
    );
}

#[test]
fn prepared_ime_commit_expires_on_lifecycle_or_ordinary_input() {
    let resets = [
        (crate::Key::Composition(CompositionEvent::Enabled), "漢漢"),
        (crate::Key::Composition(CompositionEvent::Disabled), "漢漢"),
        (
            crate::Key::Composition(CompositionEvent::Preedit {
                text: "漢".into(),
                selection: None,
            }),
            "漢漢",
        ),
        (crate::Key::Character("q".into()), "漢q漢"),
    ];
    for (reset, expected) in resets {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner =
            GenetAppRunner::new(dom, crate::text_field_typed, crate::TextInput::new(""));
        let root = runner.root();
        runner.set_focus(Some(root));
        runner.dispatch_key(crate::KeyEvent::new(crate::Key::Composition(
            CompositionEvent::Preedit {
                text: "漢".into(),
                selection: None,
            },
        )));
        runner.prepare_subtree_move(root);

        runner.dispatch_key(crate::KeyEvent::new(reset));
        runner.dispatch_key(crate::KeyEvent::new(crate::Key::Composition(
            CompositionEvent::Commit("漢".into()),
        )));

        assert_eq!(runner.state().text(), expected);
    }
}
