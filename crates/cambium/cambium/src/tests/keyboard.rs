/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, NodeKind};

use crate::{
    DomHandle, El, FocusExit, GenetAppRunner, GenetCtx, GenetElement, Key, KeyEvent, Modifiers,
    NamedKey, OnClick, OnKey, PointerClick, View, el, focusable, on_click, on_key,
};

/// The app state: a text buffer a key handler edits.
struct Editor {
    text: String,
}

/// An activation counter — the state a plain button's click handler bumps.
struct Clicks {
    count: u32,
}

/// Append the typed character / apply the editing key to `text`. A free
/// function (not a closure) so the view type stays a nameable `fn` pointer.
fn edit(s: &mut Editor, ev: KeyEvent) {
    match ev.key {
        Key::Character(c) => s.text.push_str(&c),
        Key::Named(NamedKey::Backspace) => {
            s.text.pop();
        },
        Key::Named(_) | Key::Composition(_) => {},
    }
}

/// The first element in `node`'s subtree (pre-order) named `name`.
fn find_element_by_name(dom: &ScriptedDom, node: NodeId, name: &str) -> Option<NodeId> {
    if dom.kind(node) == NodeKind::Element
        && dom
            .element_name(node)
            .is_some_and(|q| AsRef::<str>::as_ref(&q.local) == name)
    {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|c| find_element_by_name(dom, c, name))
}

fn ch(s: &str) -> KeyEvent {
    KeyEvent::new(Key::Character(s.to_string()))
}

fn named(k: NamedKey) -> KeyEvent {
    KeyEvent::new(Key::Named(k))
}

/// A `Tab` key event, optionally with `Shift` (for reverse traversal).
fn tab(shift: bool) -> KeyEvent {
    KeyEvent::with_mods(
        Key::Named(NamedKey::Tab),
        Modifiers {
            shift,
            ..Default::default()
        },
    )
}

/// A key listener on a composite ancestor can observe Escape bubbling from
/// a focused child without creating an extra Tab stop of its own.
#[test]
fn passive_ancestor_key_listener_is_not_a_tab_stop() {
    #[derive(Default)]
    struct State {
        dismissed: bool,
    }

    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |_: &State| {
            on_key(
                el::<_, State, ()>(
                    "section",
                    focusable(on_click(
                        el::<_, State, ()>("button", "inside"),
                        |_state: &mut State, _| {},
                    )),
                ),
                |state: &mut State, event: KeyEvent| {
                    if matches!(event.key, Key::Named(NamedKey::Escape)) {
                        state.dismissed = true;
                    }
                },
            )
            .focusable(false)
        },
        State::default(),
    );
    let root = runner.root();
    let button = find_element_by_name(&dom.borrow(), root, "button").expect("button");

    runner.dispatch_key(tab(false));
    assert_eq!(
        runner.focus(),
        Some(button),
        "the ancestor is skipped by Tab"
    );
    runner.dispatch_key(named(NamedKey::Escape));
    assert!(runner.state().dismissed, "Escape bubbled to the ancestor");
}

// --- focus routing --------------------------------------------------------

/// `<div><input on_key=edit/><span/></div>`: a focusable `<input>` (carries a
/// key handler) beside a non-focusable `<span>`. Concrete `fn`-pointer
/// handler so the type is nameable.
type FocusView = El<
    (
        OnKey<El<&'static str, Editor, ()>, Editor, (), fn(&mut Editor, KeyEvent)>,
        El<&'static str, Editor, ()>,
    ),
    Editor,
    (),
>;

fn focus_view(_s: &Editor) -> FocusView {
    let handler: fn(&mut Editor, KeyEvent) = edit;
    el::<_, Editor, ()>(
        "div",
        (
            on_key(el::<_, Editor, ()>("input", ""), handler),
            el::<_, Editor, ()>("span", "x"),
        ),
    )
}

/// Focus the `<input>`, type "h" then "i" (state == "hi"), then Backspace
/// (state == "h"). The key handler routes to the focused element.
#[test]
fn typed_keys_reach_focused_element() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        focus_view,
        Editor {
            text: String::new(),
        },
    );
    let root = runner.root();

    let input = {
        let d = dom.borrow();
        find_element_by_name(&d, root, "input").expect("an <input> must exist")
    };

    runner.set_focus(Some(input));
    assert_eq!(runner.focus(), Some(input));

    runner.dispatch_key(ch("h"));
    runner.dispatch_key(ch("i"));
    assert_eq!(
        runner.state().text,
        "hi",
        "typed chars append to the buffer"
    );

    runner.dispatch_key(named(NamedKey::Backspace));
    assert_eq!(runner.state().text, "h", "Backspace deletes the last char");
}

/// With no focus (never focused), `dispatch_key` is a no-op: it returns empty
/// and leaves state unchanged. Also covered: explicitly clearing focus.
#[test]
fn no_focus_is_a_noop() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        focus_view,
        Editor {
            text: String::new(),
        },
    );

    assert_eq!(runner.focus(), None, "nothing focused initially");
    let out = runner.dispatch_key(ch("h"));
    assert!(out.is_empty(), "no focus → empty action vec");
    assert_eq!(runner.state().text, "", "no focus → state unchanged");

    // Explicitly clearing focus is the same no-op.
    runner.set_focus(None);
    let out = runner.dispatch_key(ch("z"));
    assert!(out.is_empty());
    assert_eq!(runner.state().text, "");
}

// --- click sets focus -----------------------------------------------------

/// `<div on_key=edit><label/></div>`: the focusable `<div>` carries the key
/// handler and contains a non-focusable `<label>` child. The div also has no
/// click handler — focus does not depend on `on_click`.
type ClickFocusView =
    OnKey<El<El<&'static str, Editor, ()>, Editor, ()>, Editor, (), fn(&mut Editor, KeyEvent)>;

fn click_focus_view(_s: &Editor) -> ClickFocusView {
    let handler: fn(&mut Editor, KeyEvent) = edit;
    on_key(
        el::<_, Editor, ()>("div", el::<_, Editor, ()>("label", "L")),
        handler,
    )
}

/// Clicking the focusable div (via its child) focuses the div; a subsequent
/// `dispatch_key` routes there. Then clicking a non-focusable node clears
/// focus to `None`.
#[test]
fn click_sets_and_clears_focus() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        click_focus_view,
        Editor {
            text: String::new(),
        },
    );
    let div = runner.root(); // the OnKey wraps the root <div>

    let label = {
        let d = dom.borrow();
        find_element_by_name(&d, div, "label").expect("a <label>")
    };

    assert_eq!(runner.focus(), None);

    // Click the label (a child of the focusable div) → focus the div, since
    // the nearest focusable ancestor of the label is the div.
    runner.dispatch_click(label, PointerClick::at((0.0, 0.0)));
    assert_eq!(
        runner.focus(),
        Some(div),
        "clicking inside the focusable div focuses the div"
    );

    // A key now reaches the div's handler.
    runner.dispatch_key(ch("a"));
    assert_eq!(runner.state().text, "a", "key routes to the focused div");

    // Clicking a node whose ancestor chain has no key handler clears focus.
    // The document root is such a node (above the div, no handler).
    let doc = dom.borrow().document();
    runner.dispatch_click(doc, PointerClick::at((0.0, 0.0)));
    assert_eq!(
        runner.focus(),
        None,
        "clicking outside any focusable element clears focus"
    );
}

// --- key bubbling ---------------------------------------------------------

/// A key handler on a *parent* fires when the focused child has none.
/// `<div on_key=edit><button on_click=noop/></div>`: focus the button (no key
/// handler) and dispatch — the key bubbles to the div's handler. The button
/// carries only a click handler, so it is *not* focusable; we `set_focus`
/// it directly to model "a non-focusable node happened to be focused" and
/// confirm the bubble still reaches the parent.
type BubbleKeyView = OnKey<
    El<
        OnClick<El<&'static str, Editor, ()>, Editor, (), fn(&mut Editor, PointerClick)>,
        Editor,
        (),
    >,
    Editor,
    (),
    fn(&mut Editor, KeyEvent),
>;

fn bubble_key_view(_s: &Editor) -> BubbleKeyView {
    let key_handler: fn(&mut Editor, KeyEvent) = edit;
    let click_noop: fn(&mut Editor, PointerClick) = |_s, _ev| {};
    on_key(
        el::<_, Editor, ()>(
            "div",
            on_click(el::<_, Editor, ()>("button", "+"), click_noop),
        ),
        key_handler,
    )
}

#[test]
fn key_bubbles_from_focused_child_to_parent_handler() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        bubble_key_view,
        Editor {
            text: String::new(),
        },
    );
    let div = runner.root();

    let button = {
        let d = dom.borrow();
        find_element_by_name(&d, div, "button").expect("a <button>")
    };
    assert_ne!(
        button, div,
        "button is a descendant of the handler-bearing div"
    );

    // The button has only a click handler, so it is not focusable; aim focus
    // at it directly to exercise the bubble (focus on a child with no key
    // handler → the parent's handler fires).
    runner.set_focus(Some(button));
    runner.dispatch_key(ch("z"));
    assert_eq!(
        runner.state().text,
        "z",
        "key on the focused child bubbles to the parent div's handler"
    );
}

// --- focusable() marker + Enter/Space activation (G2.3) --------------------

/// `focusable(on_click(button))`: a plain button (a click handler, no key
/// handler) made keyboard-operable by `focusable`. Without the marker it would
/// be tab-unreachable and un-activatable.
fn focusable_button_view(
    _s: &Clicks,
) -> impl View<Clicks, (), GenetCtx, Element = GenetElement> + use<> {
    focusable(on_click(
        el::<_, Clicks, ()>("button", "+"),
        |s: &mut Clicks, _ev: PointerClick| s.count += 1,
    ))
}

/// `focusable` puts the plain button in the Tab order, and Enter/Space activate
/// it by synthesizing a click; a character key does not.
#[test]
fn enter_and_space_activate_a_focusable_button() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner =
        GenetAppRunner::<_, _, _, ()>::new(dom.clone(), focusable_button_view, Clicks { count: 0 });
    let button = runner.root(); // `focusable` wraps the button, passing it through

    // Tab reaches it even though it carries no key handler — `focusable` put it
    // in the focusable set.
    assert_eq!(runner.focus(), None);
    runner.dispatch_key(tab(false));
    assert_eq!(
        runner.focus(),
        Some(button),
        "focusable() puts the plain button in the Tab order"
    );

    // Enter and Space each activate it (a synthesized click runs the on_click).
    runner.dispatch_key(named(NamedKey::Enter));
    assert_eq!(
        runner.state().count,
        1,
        "Enter activates the focused button"
    );
    runner.dispatch_key(named(NamedKey::Space));
    assert_eq!(
        runner.state().count,
        2,
        "Space activates the focused button"
    );

    // A character key is not an activation.
    runner.dispatch_key(ch("x"));
    assert_eq!(
        runner.state().count,
        2,
        "a character key does not activate the button"
    );
}

/// `clickable(child, handler)` is `focusable(on_click(..))` in one combinator: the div is
/// Tab-reachable, Enter activates it (the synthesized click), and a pointer click does too.
#[test]
fn clickable_is_focusable_and_activatable() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |_s: &Clicks| {
            crate::clickable(
                el::<_, Clicks, ()>("div", "row"),
                |s: &mut Clicks, _ev: PointerClick| s.count += 1,
            )
        },
        Clicks { count: 0 },
    );
    let row = runner.root();

    runner.dispatch_key(tab(false));
    assert_eq!(
        runner.focus(),
        Some(row),
        "clickable() puts the div in the Tab order"
    );
    runner.dispatch_key(named(NamedKey::Enter));
    assert_eq!(runner.state().count, 1, "Enter activates the clickable");
    runner.dispatch_click(row, PointerClick::at((0.0, 0.0)));
    assert_eq!(runner.state().count, 2, "a pointer click activates it too");
}

// --- Tab is overridable per-view (G2.3) -----------------------------------

#[path = "keyboard/traversal.rs"]
mod traversal;
