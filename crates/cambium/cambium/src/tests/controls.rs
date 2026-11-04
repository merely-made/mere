/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};

use crate::{
    AnyView, DomHandle, GenetAppRunner, GenetCtx, GenetElement, Key, KeyEvent, Modifiers, NamedKey,
    PointerClick, PointerEvent, PointerPhase, RadioGroup, RadioOption, SelectState, Slider,
    TextInput, View, WheelEvent, button, checkbox, checkbox_disabled, el, focusable, lens,
    on_click, on_pointer, on_wheel, overlay_at, overlay_rect, radio_group, radio_options, select,
    slider, text_field,
};

/// The text data of the single text child under `node`, if any.
fn text_child(dom: &ScriptedDom, node: NodeId) -> Option<String> {
    dom.dom_children(node)
        .find(|&c| dom.kind(c) == NodeKind::Text)
        .and_then(|c| dom.text(c).map(str::to_string))
}

/// All text in `node`'s subtree concatenated in document order. The field
/// renders its content as `(before, preedit-span, after)`, so its rendered
/// text is the concatenation, not a single child.
fn field_text(dom: &ScriptedDom, node: NodeId) -> String {
    fn go(dom: &ScriptedDom, node: NodeId, out: &mut String) {
        if dom.kind(node) == NodeKind::Text
            && let Some(t) = dom.text(node)
        {
            out.push_str(t);
        }
        for c in dom.dom_children(node) {
            go(dom, c, out);
        }
    }
    let mut out = String::new();
    go(dom, node, &mut out);
    out
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

/// The app-owned text control container, identified by its semantic role.
fn find_textbox(dom: &ScriptedDom, node: NodeId) -> Option<NodeId> {
    if dom.kind(node) == NodeKind::Element
        && dom.attribute(node, &Namespace::from(""), &LocalName::from("role")) == Some("textbox")
    {
        return Some(node);
    }
    dom.dom_children(node).find_map(|c| find_textbox(dom, c))
}

fn ch(s: &str) -> KeyEvent {
    KeyEvent::new(Key::Character(s.to_string()))
}

fn named(k: NamedKey) -> KeyEvent {
    KeyEvent::new(Key::Named(k))
}

/// Type the canonical sequence into a focused field; the buffer progresses:
///   "h", "i", Space, "y", "o", Backspace
///     → "h", "hi", "hi ", "hi y", "hi yo", "hi y"
/// Space goes through `Named(Space)` (the Stage 3b convention); the caret
/// stays at the end throughout, so each char inserts there and Backspace
/// removes the trailing "o". Callers assert the resulting buffer / DOM text.
fn type_hi_y(runner: &mut impl FieldRunner) {
    runner.key(ch("h"));
    runner.key(ch("i"));
    runner.key(named(NamedKey::Space));
    runner.key(ch("y"));
    runner.key(ch("o"));
    runner.key(named(NamedKey::Backspace));
}

/// Minimal shim so `type_hi_y` drives either runner shape (the bare-`String`
/// runner and the `lens`-composed struct runner) through one `dispatch_key`.
trait FieldRunner {
    fn key(&mut self, ev: KeyEvent);
}

// --- bare String state ----------------------------------------------------

impl<Logic, V> FieldRunner for GenetAppRunner<TextInput, Logic, V, ()>
where
    Logic: FnMut(&TextInput) -> V,
    V: View<TextInput, (), GenetCtx, Element = GenetElement>,
{
    fn key(&mut self, ev: KeyEvent) {
        self.dispatch_key(ev);
    }
}

// --- lens-composed struct field -------------------------------------------

/// A larger app with an independently-edited text field plus a sibling field
/// the edits must *not* touch — proving the `text_field` composes via `lens`.
struct Form {
    name: TextInput,
    other: String,
}

impl<Logic, V> FieldRunner for GenetAppRunner<Form, Logic, V, ()>
where
    Logic: FnMut(&Form) -> V,
    V: View<Form, (), GenetCtx, Element = GenetElement>,
{
    fn key(&mut self, ev: KeyEvent) {
        self.dispatch_key(ev);
    }
}

/// Read a null-namespace attribute of `node`.
fn attr(dom: &ScriptedDom, node: NodeId, name: &str) -> Option<String> {
    dom.attribute(node, &Namespace::from(""), &LocalName::from(name))
        .map(str::to_string)
}

/// A `button(label, handler)`: renders a `<button>` with the label, and a
/// click runs the handler.
#[test]
fn button_runs_its_handler() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let inc: fn(&mut u32, PointerClick) = |n, _| *n += 1;
    let mut runner =
        GenetAppRunner::<_, _, _, ()>::new(dom.clone(), move |_: &u32| button("inc", inc), 0u32);
    let btn = runner.root();

    assert_eq!(
        AsRef::<str>::as_ref(&dom.borrow().element_name(btn).unwrap().local),
        "button"
    );
    assert_eq!(text_child(&dom.borrow(), btn).as_deref(), Some("inc"));

    runner.dispatch_click(btn, PointerClick::at((0.0, 0.0)));
    assert_eq!(runner.state(), &1);
    runner.set_focus(Some(btn));
    runner.dispatch_key(named(NamedKey::Enter));
    runner.dispatch_key(named(NamedKey::Space));
    assert_eq!(runner.state(), &3, "Enter and Space activate a button");
}

/// `button(label, h).attr("class", ..)`: the fluent `OnClick::attr` forwards
/// to the wrapped `<button>`, so a class (or any attribute) can be set after
/// wrapping. The handler still fires.
#[test]
fn button_attr_stamps_class_and_keeps_handler() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let inc: fn(&mut u32, PointerClick) = |n, _| *n += 1;
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        move |_: &u32| button("inc", inc).attr("class", "primary"),
        0u32,
    );
    let btn = runner.root();

    assert_eq!(
        AsRef::<str>::as_ref(&dom.borrow().element_name(btn).unwrap().local),
        "button"
    );
    assert_eq!(
        attr(&dom.borrow(), btn, "class").as_deref(),
        Some("primary")
    );

    runner.dispatch_click(btn, PointerClick::at((0.0, 0.0)));
    assert_eq!(runner.state(), &1);
}

/// `overlay_at(x, y, content)`: a `<div>` carrying its position in an inline
/// `style` (`position: absolute` + the insets), wrapping the content. The
/// inline style is what Genet's cascade reads to place the overlay.
#[test]
fn overlay_at_carries_inline_position() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |_: &()| overlay_at::<_, (), ()>(30.0, 15.0, "menu"),
        (),
    );
    let ov = runner.root();

    assert_eq!(
        AsRef::<str>::as_ref(&dom.borrow().element_name(ov).unwrap().local),
        "div"
    );
    assert_eq!(text_child(&dom.borrow(), ov).as_deref(), Some("menu"));
    assert_eq!(
        attr(&dom.borrow(), ov, "style").as_deref(),
        Some("position: absolute; left: 30px; top: 15px;"),
    );
}

/// `overlay_rect(x, y, w, h, content)`: a positioned `<div>` that also carries
/// `width`/`height` in its inline `style`, so the host can size a surface (a card,
/// the comms pane) from the view rather than re-stamping geometry each frame.
#[test]
fn overlay_rect_carries_inline_geometry() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |_: &()| overlay_rect::<_, (), ()>(30.0, 15.0, 200.0, 120.0, "card"),
        (),
    );
    let ov = runner.root();

    assert_eq!(
        AsRef::<str>::as_ref(&dom.borrow().element_name(ov).unwrap().local),
        "div"
    );
    assert_eq!(text_child(&dom.borrow(), ov).as_deref(), Some("card"));
    assert_eq!(
        attr(&dom.borrow(), ov, "style").as_deref(),
        Some("position: absolute; left: 30px; top: 15px; width: 200px; height: 120px;"),
    );
}

/// `Box<dyn AnyView>` whose inner view changes *type* across a rebuild
/// (`<div>` → `<span>`): the element node is swapped in place via
/// `AnyElement::replace_inner`, staying attached under the document. Proves
/// erased/dynamic views work on Genet's uniform element type.
#[test]
fn any_view_swaps_node_on_type_change() {
    // The two branches are *different concrete View types* (their children
    // sequences differ: a single text vs a two-text tuple → `El<&str,…>` vs
    // `El<(&str,&str),…>`), so `AnyView` sees a type change and must take the
    // `replace_inner` path — not a same-type rebuild (which wouldn't even
    // re-tag the element).
    fn view(on: &bool) -> Box<dyn AnyView<bool, (), GenetCtx, GenetElement>> {
        if *on {
            Box::new(el::<_, bool, ()>("span", ("on", "!")))
        } else {
            Box::new(el::<_, bool, ()>("div", "off"))
        }
    }
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        view as fn(&bool) -> Box<dyn AnyView<bool, (), GenetCtx, GenetElement>>,
        false,
    );

    let root0 = runner.root();
    {
        let dom = dom.borrow();
        assert_eq!(
            AsRef::<str>::as_ref(&dom.element_name(root0).unwrap().local),
            "div"
        );
        assert_eq!(text_child(&dom, root0).as_deref(), Some("off"));
        assert!(dom.dom_children(dom.document()).any(|c| c == root0));
    }

    // Flip → the boxed view switches <div> → <span>: a type change, so the
    // node is replaced in place under the document.
    runner.update(|s| *s = true);
    let root1 = runner.root();
    assert_ne!(root0, root1, "a type change swaps the node");
    {
        let dom = dom.borrow();
        assert_eq!(
            AsRef::<str>::as_ref(&dom.element_name(root1).unwrap().local),
            "span"
        );
        assert_eq!(
            dom.dom_children(root1).count(),
            2,
            "span has its two text children"
        );
        assert!(
            dom.dom_children(dom.document()).any(|c| c == root1),
            "new node attached"
        );
        assert!(
            !dom.dom_children(dom.document()).any(|c| c == root0),
            "old node detached"
        );
    }
}

// --- selection (model + keyboard) -----------------------------------------

/// A `Shift`-held key event (extends the selection).
fn shift_named(k: NamedKey) -> KeyEvent {
    KeyEvent::with_mods(
        Key::Named(k),
        Modifiers {
            shift: true,
            ..Default::default()
        },
    )
}

#[path = "controls/text.rs"]
mod text;

#[path = "controls/choices.rs"]
mod choices;

#[path = "controls/pointer.rs"]
mod pointer;
