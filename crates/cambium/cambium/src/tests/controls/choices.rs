/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

/// A `checkbox` over a `bool`: clicking toggles the bool and reflects it as
/// `aria-checked` + the `checked` class (for a11y + styling).
#[test]
fn checkbox_toggles_and_reflects_state() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner =
        GenetAppRunner::<_, _, _, ()>::new(dom.clone(), |c: &bool| checkbox(*c), false);
    let cb = runner.root(); // the checkbox is the whole view → its root element

    assert_eq!(runner.state(), &false);
    assert_eq!(
        attr(&dom.borrow(), cb, "aria-checked").as_deref(),
        Some("false")
    );
    assert_eq!(
        attr(&dom.borrow(), cb, "class").as_deref(),
        Some("checkbox")
    );

    // Click → checked.
    runner.dispatch_click(cb, PointerClick::at((0.0, 0.0)));
    assert_eq!(runner.state(), &true);
    assert_eq!(
        attr(&dom.borrow(), cb, "aria-checked").as_deref(),
        Some("true")
    );
    assert_eq!(
        attr(&dom.borrow(), cb, "class").as_deref(),
        Some("checkbox checked")
    );

    // Click again → back to unchecked.
    runner.dispatch_click(cb, PointerClick::at((0.0, 0.0)));
    assert_eq!(runner.state(), &false);

    runner.set_focus(Some(cb));
    runner.dispatch_key(named(NamedKey::Enter));
    assert_eq!(runner.state(), &false, "Enter does not toggle a checkbox");
    runner.dispatch_key(named(NamedKey::Space));
    assert!(*runner.state(), "Space toggles a checkbox");
}

/// `select`: clicking the box opens the option list (an absolute `top: 100%`
/// child appears); clicking an option sets `selected` and closes the list.
#[test]
fn select_opens_picks_and_closes() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let opts = ["red", "green", "blue"];
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        move |s: &SelectState| select(s, &opts),
        SelectState::new(0),
    );
    let root = runner.root();

    // Closed: selected = 0, the root has only the box child (no list).
    assert_eq!(runner.state().selected, 0);
    assert!(!runner.state().open);
    assert_eq!(
        dom.borrow().dom_children(root).count(),
        1,
        "closed: box only"
    );

    // Click the box (root's first child) → the list opens.
    let box_node = dom.borrow().dom_children(root).next().expect("box");
    runner.dispatch_click(box_node, PointerClick::at((0.0, 0.0)));
    assert!(runner.state().open, "clicking the box opens the list");
    assert_eq!(
        dom.borrow().dom_children(root).count(),
        2,
        "open: box + list"
    );

    // Click the second option (index 1) → selected = 1, list closes.
    let opt1 = {
        let dom = dom.borrow();
        let list = dom.dom_children(root).nth(1).expect("list");
        dom.dom_children(list).nth(1).expect("second option")
    };
    runner.dispatch_click(opt1, PointerClick::at((0.0, 0.0)));
    assert_eq!(runner.state().selected, 1, "picking option 1 selects it");
    assert!(!runner.state().open, "picking an option closes the list");
    assert_eq!(
        dom.borrow().dom_children(root).count(),
        1,
        "closed again: box only"
    );

    // The box now shows the new selection.
    let box_node = dom.borrow().dom_children(root).next().expect("box");
    assert_eq!(
        text_child(&dom.borrow(), box_node).as_deref(),
        Some("green")
    );

    runner.set_focus(Some(root));
    runner.dispatch_key(named(NamedKey::ArrowDown));
    assert_eq!(runner.state().selected, 2);
    assert!(runner.state().open, "ArrowDown opens the list");
    runner.dispatch_key(named(NamedKey::Escape));
    assert!(!runner.state().open, "Escape closes the list");
}

/// `radio_group`: clicking an option selects it (and only it), reflected in
/// `selected` and `aria-checked`.
#[test]
fn radio_group_selects_one() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let opts = ["a", "b", "c"];
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        move |s: &RadioGroup| radio_group(s, &opts),
        RadioGroup::new(0),
    );
    let root = runner.root();
    assert_eq!(runner.state().selected, 0);

    // Click the third option (index 2).
    let opt2 = {
        dom.borrow()
            .dom_children(root)
            .nth(2)
            .expect("third option")
    };
    runner.dispatch_click(opt2, PointerClick::at((0.0, 0.0)));
    assert_eq!(runner.state().selected, 2, "clicking option 2 selects it");

    // aria-checked reflects the selection: option 2 true, option 0 false.
    let opt0 = dom
        .borrow()
        .dom_children(root)
        .next()
        .expect("first option");
    let opt2 = dom
        .borrow()
        .dom_children(root)
        .nth(2)
        .expect("third option");
    assert_eq!(
        attr(&dom.borrow(), opt2, "aria-checked").as_deref(),
        Some("true")
    );
    assert_eq!(
        attr(&dom.borrow(), opt0, "aria-checked").as_deref(),
        Some("false")
    );

    assert_eq!(attr(&dom.borrow(), opt2, "tabindex").as_deref(), Some("0"));
    assert_eq!(attr(&dom.borrow(), opt0, "tabindex").as_deref(), Some("-1"));
    runner.set_focus(Some(opt2));
    runner.dispatch_key(named(NamedKey::ArrowRight));
    assert_eq!(runner.state().selected, 0, "radio arrows wrap and select");
    assert_eq!(runner.focus(), Some(opt0));
    assert!(
        runner.default_prevented(),
        "the radio component consumes navigation"
    );

    runner.dispatch_key(named(NamedKey::End));
    assert_eq!(
        runner.state().selected,
        2,
        "End remains a Cambium extension"
    );
    assert_eq!(runner.focus(), Some(opt2));
    runner.dispatch_key(named(NamedKey::Space));
    assert_eq!(runner.state().selected, 2, "Space checks the focused radio");
    assert!(
        runner.default_prevented(),
        "the radio component consumes Space"
    );

    // The focused radio is the command origin even if selection was changed
    // independently, as can happen through programmatic or AT focus.
    let opt1 = dom
        .borrow()
        .dom_children(root)
        .nth(1)
        .expect("second option");
    runner.set_focus(Some(opt1));
    runner.dispatch_key(named(NamedKey::Space));
    assert_eq!(
        runner.state().selected,
        1,
        "Space checks the focused, previously unselected radio"
    );
    runner.set_focus(Some(opt0));
    runner.dispatch_key(named(NamedKey::ArrowLeft));
    assert_eq!(
        runner.state().selected,
        2,
        "Arrow navigation starts from the focused radio"
    );
    assert_eq!(runner.focus(), Some(opt2));

    // A retained request is an edge for its exact target. An unrelated
    // rebuild after focus leaves must not steal focus back.
    runner.set_focus(None);
    runner.update(|state| state.selected = 1);
    assert_eq!(runner.state().selected, 1);
    assert_eq!(runner.focus(), None);
}

/// Disabled radio options keep their place but cannot be clicked, focused
/// or reached by arrows, `Home` or `End`; the indicator is its own element
/// beside the label text, so the option's text is the label alone.
#[test]
fn radio_options_skip_disabled_and_draw_a_separate_indicator() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |s: &RadioGroup| {
            radio_options(
                s,
                &[
                    RadioOption::new("Auto"),
                    RadioOption::new("Reader")
                        .disabled(true)
                        .with_attr("data-id", "reader"),
                    RadioOption::new("Genet"),
                    RadioOption::new("Servo").disabled(true),
                ],
            )
        },
        RadioGroup::new(0),
    );
    let root = runner.root();
    let option = |i: usize| dom.borrow().dom_children(root).nth(i).expect("option");
    let (auto, reader, genet, servo) = (option(0), option(1), option(2), option(3));

    assert_eq!(
        attr(&dom.borrow(), reader, "aria-disabled").as_deref(),
        Some("true")
    );
    assert_eq!(
        attr(&dom.borrow(), reader, "class").as_deref(),
        Some("radio disabled")
    );
    assert_eq!(
        attr(&dom.borrow(), reader, "data-id").as_deref(),
        Some("reader")
    );
    assert_eq!(
        attr(&dom.borrow(), reader, "aria-label").as_deref(),
        Some("Reader")
    );
    assert_eq!(
        attr(&dom.borrow(), auto, "class").as_deref(),
        Some("radio selected")
    );
    assert_eq!(
        attr(&dom.borrow(), reader, "tabindex").as_deref(),
        Some("-1")
    );
    assert_eq!(attr(&dom.borrow(), auto, "tabindex").as_deref(), Some("0"));

    let parts: Vec<_> = dom.borrow().dom_children(genet).collect();
    assert_eq!(parts.len(), 2, "indicator and label are separate nodes");
    assert_eq!(
        attr(&dom.borrow(), parts[0], "class").as_deref(),
        Some("radio-indicator")
    );
    let label = dom
        .borrow()
        .dom_children(parts[1])
        .next()
        .expect("label text");
    assert_eq!(dom.borrow().text(label).as_deref(), Some("Genet"));

    runner.dispatch_click(reader, PointerClick::at((0.0, 0.0)));
    runner.dispatch_click(servo, PointerClick::at((0.0, 0.0)));
    assert_eq!(runner.state().selected, 0, "disabled options ignore clicks");

    runner.set_focus(Some(auto));
    runner.dispatch_key(named(NamedKey::ArrowRight));
    assert_eq!(runner.state().selected, 2, "arrows skip disabled options");
    assert_eq!(runner.focus(), Some(genet));
    runner.dispatch_key(named(NamedKey::End));
    assert_eq!(
        runner.state().selected,
        2,
        "End stops at the last enabled option"
    );
    runner.dispatch_key(named(NamedKey::Home));
    assert_eq!(
        runner.state().selected,
        0,
        "Home goes to the first enabled option"
    );
    runner.set_focus(Some(reader));
    runner.dispatch_key(named(NamedKey::Space));
    assert_eq!(
        runner.state().selected,
        0,
        "a disabled option cannot be activated"
    );
}

/// A saved selection that has become disabled stays checked, and the first
/// enabled option becomes the tab stop.
#[test]
fn radio_options_keep_a_disabled_selection_but_move_the_tab_stop() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |s: &RadioGroup| {
            radio_options(
                s,
                &[RadioOption::new("a"), RadioOption::new("b").disabled(true)],
            )
        },
        RadioGroup::new(1),
    );
    let root = runner.root();
    let b = dom.borrow().dom_children(root).nth(1).expect("b");
    let a = dom.borrow().dom_children(root).next().expect("a");
    assert_eq!(
        attr(&dom.borrow(), b, "aria-checked").as_deref(),
        Some("true")
    );
    assert_eq!(attr(&dom.borrow(), b, "tabindex").as_deref(), Some("-1"));
    assert_eq!(attr(&dom.borrow(), a, "tabindex").as_deref(), Some("0"));
}

/// The checkbox draws through an empty `control-mark` child and a disabled
/// one is announced and inert.
#[test]
fn checkbox_mark_is_a_child_and_disabled_is_inert() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |c: &bool| checkbox_disabled::<bool, ()>(*c),
        true,
    );
    let cb = runner.root();
    assert_eq!(
        attr(&dom.borrow(), cb, "aria-disabled").as_deref(),
        Some("true")
    );
    assert_eq!(
        attr(&dom.borrow(), cb, "class").as_deref(),
        Some("checkbox checked disabled")
    );
    let mark = dom.borrow().dom_children(cb).next().expect("mark");
    assert_eq!(
        attr(&dom.borrow(), mark, "class").as_deref(),
        Some("control-mark")
    );
    assert!(
        dom.borrow().dom_children(mark).next().is_none(),
        "no glyph text"
    );
    assert_eq!(attr(&dom.borrow(), cb, "tabindex"), None);
}

/// ARIA roles expose semantics; they do not opt arbitrary nodes into
/// Cambium's authored-widget keyboard behavior.
#[test]
fn runner_does_not_invent_radio_behavior_from_roles() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |_: &usize| {
            let radios: Vec<_> = (0..2)
                .map(|index| {
                    focusable(on_click(
                        el::<_, usize, ()>("div", format!("radio {index}"))
                            .attr("role", "radio")
                            .attr("aria-checked", "false"),
                        move |count: &mut usize, _| *count += index + 1,
                    ))
                })
                .collect();
            el::<_, usize, ()>("div", radios).attr("role", "radiogroup")
        },
        0usize,
    );
    let root = runner.root();
    let first = dom
        .borrow()
        .dom_children(root)
        .next()
        .expect("first semantic radio");
    runner.set_focus(Some(first));

    runner.dispatch_key(named(NamedKey::ArrowRight));
    assert_eq!(runner.state(), &0, "Arrow does not synthesize a click");
    assert_eq!(runner.focus(), Some(first), "Arrow does not move focus");
    assert!(
        !runner.default_prevented(),
        "an unhandled semantic role does not consume Arrow"
    );

    runner.dispatch_key(named(NamedKey::Space));
    assert_eq!(runner.state(), &0, "Space does not synthesize a click");
    assert_eq!(runner.focus(), Some(first), "Space leaves focus unchanged");
    assert!(
        !runner.default_prevented(),
        "an unhandled semantic role does not consume Space"
    );
}
