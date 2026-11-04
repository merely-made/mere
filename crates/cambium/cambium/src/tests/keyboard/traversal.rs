/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

/// Tab / Shift+Tab move focus across the focusable set in document order,
/// wrapping — the runner-level traversal default (a document engine has no
/// built-in tab order). Two focusable elements (each carries an `on_key`).
#[test]
fn tab_traverses_focusables() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let noop: fn(&mut (), KeyEvent) = |_, _| {};
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        move |_: &()| {
            el::<_, (), ()>(
                "div",
                (
                    on_key(el::<_, (), ()>("a", ()), noop),
                    on_key(el::<_, (), ()>("b", ()), noop),
                ),
            )
        },
        (),
    );
    let (a, b) = {
        let d = dom.borrow();
        let root = runner.root();
        (
            find_element_by_name(&d, root, "a").expect("<a>"),
            find_element_by_name(&d, root, "b").expect("<b>"),
        )
    };

    // Nothing focused → Tab focuses the first; Tab advances; Tab wraps.
    assert_eq!(runner.focus(), None);
    runner.dispatch_key(tab(false));
    assert_eq!(
        runner.focus(),
        Some(a),
        "Tab from nothing focuses the first"
    );
    runner.dispatch_key(tab(false));
    assert_eq!(runner.focus(), Some(b), "Tab advances");
    runner.dispatch_key(tab(false));
    assert_eq!(runner.focus(), Some(a), "Tab wraps to the first");

    // Shift+Tab goes backward (wrapping to the last).
    runner.dispatch_key(tab(true));
    assert_eq!(runner.focus(), Some(b), "Shift+Tab goes backward (wraps)");
}

/// With focus exits on, Tab past the last focusable and Shift+Tab before
/// the first leave the runner: focus clears and the exit waits for the
/// host, once. Inside the set, traversal moves as before. A runner with
/// nothing focusable exits at once. (App composition brief: a composed
/// session must hand focus back to its host.)
#[test]
fn tab_exits_at_the_edges_when_focus_exits_are_on() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let noop: fn(&mut (), KeyEvent) = |_, _| {};
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        move |_: &()| {
            el::<_, (), ()>(
                "div",
                (
                    on_key(el::<_, (), ()>("a", ()), noop),
                    on_key(el::<_, (), ()>("b", ()), noop),
                ),
            )
        },
        (),
    );
    let (a, b) = {
        let d = dom.borrow();
        let root = runner.root();
        (
            find_element_by_name(&d, root, "a").expect("<a>"),
            find_element_by_name(&d, root, "b").expect("<b>"),
        )
    };
    runner.set_focus_exits(true);

    runner.dispatch_key(tab(false));
    assert_eq!(runner.focus(), Some(a), "Tab from nothing still enters");
    runner.dispatch_key(tab(false));
    assert_eq!(runner.focus(), Some(b));
    assert_eq!(runner.take_focus_exit(), None, "inside the set, no exit");

    runner.dispatch_key(tab(false));
    assert_eq!(runner.focus(), None, "Tab past the last leaves");
    assert_eq!(runner.take_focus_exit(), Some(FocusExit::Forward));
    assert_eq!(runner.take_focus_exit(), None, "an exit is taken once");

    runner.set_focus(Some(a));
    runner.dispatch_key(tab(true));
    assert_eq!(runner.focus(), None, "Shift+Tab before the first leaves");
    assert_eq!(runner.take_focus_exit(), Some(FocusExit::Backward));

    // The host can enter from either end by traversing from nothing.
    runner.focus_traverse(false);
    assert_eq!(runner.focus(), Some(b));

    // Turning exits off restores wrapping and drops a pending exit.
    runner.focus_traverse(true);
    runner.set_focus_exits(false);
    assert_eq!(runner.take_focus_exit(), None);
    runner.set_focus(Some(b));
    runner.dispatch_key(tab(false));
    assert_eq!(runner.focus(), Some(a), "wrapping again");

    let empty_dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut empty =
        GenetAppRunner::<_, _, _, ()>::new(empty_dom, |_: &()| el::<_, (), ()>("div", ()), ());
    empty.set_focus_exits(true);
    empty.dispatch_key(tab(false));
    assert_eq!(empty.take_focus_exit(), Some(FocusExit::Forward));
}

/// A handler that consumes Tab at the last focusable keeps focus there:
/// the override wins over the exit, as it wins over wrapping.
#[test]
fn a_handled_tab_does_not_exit() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let trap: fn(&mut Editor, KeyEvent) = |s, ev| {
        if matches!(ev.key, Key::Named(NamedKey::Tab)) {
            s.text.push('\t');
            ev.prevent_default();
        }
    };
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        move |_: &Editor| {
            el::<_, Editor, ()>("div", on_key(el::<_, Editor, ()>("textarea", ""), trap))
        },
        Editor {
            text: String::new(),
        },
    );
    let textarea = {
        let d = dom.borrow();
        find_element_by_name(&d, runner.root(), "textarea").expect("<textarea>")
    };
    runner.set_focus_exits(true);
    runner.set_focus(Some(textarea));
    runner.dispatch_key(tab(false));
    assert_eq!(runner.focus(), Some(textarea));
    assert_eq!(runner.take_focus_exit(), None);
    assert_eq!(runner.state().text, "\t");
}

/// Tab is delivered to the focused element's `on_key` first; a handler that
/// inserts a tab character and calls `prevent_default` keeps focus put (the
/// textarea case), where an ordinary focusable lets the runner traverse.
#[test]
fn tab_is_overridable_by_a_handler() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    // The first field captures Tab: insert '\t' and prevent the traversal.
    let trap: fn(&mut Editor, KeyEvent) = |s, ev| {
        if matches!(ev.key, Key::Named(NamedKey::Tab)) {
            s.text.push('\t');
            ev.prevent_default();
        }
    };
    let plain: fn(&mut Editor, KeyEvent) = |_, _| {};
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        move |_: &Editor| {
            el::<_, Editor, ()>(
                "div",
                (
                    on_key(el::<_, Editor, ()>("input", ""), trap),
                    on_key(el::<_, Editor, ()>("textarea", ""), plain),
                ),
            )
        },
        Editor {
            text: String::new(),
        },
    );
    let (input, textarea) = {
        let d = dom.borrow();
        let root = runner.root();
        (
            find_element_by_name(&d, root, "input").expect("<input>"),
            find_element_by_name(&d, root, "textarea").expect("<textarea>"),
        )
    };

    // Focus the trap field, press Tab: the handler consumes it (a tab char) and
    // prevents the default, so focus stays — the override.
    runner.set_focus(Some(input));
    runner.dispatch_key(tab(false));
    assert_eq!(
        runner.focus(),
        Some(input),
        "a Tab handler that prevents default keeps focus put"
    );
    assert_eq!(
        runner.state().text,
        "\t",
        "the overriding handler inserted a tab character"
    );

    // The plain field does not prevent: Tab there still traverses (wrapping to
    // the first focusable), proving the default survives where unoverridden.
    runner.set_focus(Some(textarea));
    runner.dispatch_key(tab(false));
    assert_eq!(
        runner.focus(),
        Some(input),
        "an un-prevented Tab still traverses the focusable set"
    );
}
