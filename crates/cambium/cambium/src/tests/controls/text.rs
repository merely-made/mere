/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

/// A `text_field` over bare `TextInput` state: focus it, type the sequence,
/// and assert the buffer reads `"hi y"` with the caret at the end, and the
/// textbox publishes the clean buffer in app-owned metadata.
#[test]
fn text_field_edits_its_own_buffer() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        // The field's state IS the whole app state here: a `TextInput`.
        |s: &TextInput| text_field(s),
        TextInput::default(),
    );
    let root = runner.root();

    // The app-owned textbox is the focusable element.
    let input = {
        let d = dom.borrow();
        find_textbox(&d, root).expect("the field renders a textbox")
    };

    runner.set_focus(Some(input));
    type_hi_y(&mut runner);

    assert_eq!(runner.state().text(), "hi y", "the field edited its buffer");
    assert_eq!(
        runner.state().caret(),
        4,
        "caret sits at the end after typing"
    );
    assert_eq!(
        field_text(&dom.borrow(), runner.root()),
        "hi y",
        "the rendered children match the clean buffer"
    );
    assert_eq!(
        dom.borrow().attribute(
            input,
            &Namespace::from(""),
            &LocalName::from("data-cambium-text-value")
        ),
        Some("hi y"),
        "the semantic value contains committed text"
    );
}

/// Caret movement + insert/delete at the caret: type, move left, insert in the
/// middle, then delete on both sides. Proves the field is an insertion-point
/// editor (not append-only), tracking the caret index as it moves. The DOM
/// text is the clean buffer (the caret is painted on screen, not in the text).
#[test]
fn text_field_caret_moves_and_edits() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |s: &TextInput| text_field(s),
        TextInput::default(),
    );
    let input = {
        let d = dom.borrow();
        find_textbox(&d, runner.root()).expect("a textbox")
    };
    runner.set_focus(Some(input));

    runner.key(ch("h"));
    runner.key(ch("i")); // "hi", caret 2
    runner.key(named(NamedKey::ArrowLeft)); // caret 1: "h|i"
    runner.key(ch("X")); // insert at 1: "hXi", caret 2
    assert_eq!(runner.state().text(), "hXi");
    assert_eq!(runner.state().caret(), 2, "caret after the inserted X");
    assert_eq!(
        field_text(&dom.borrow(), runner.root()),
        "hXi",
        "DOM text is the clean buffer; the caret is painted, not in the text"
    );

    runner.key(named(NamedKey::ArrowLeft));
    runner.key(named(NamedKey::ArrowLeft)); // caret 0: "|hXi"
    runner.key(named(NamedKey::Delete)); // delete after caret (h): "Xi", caret 0
    assert_eq!(runner.state().text(), "Xi");
    assert_eq!(runner.state().caret(), 0);

    runner.key(named(NamedKey::ArrowRight)); // caret 1
    runner.key(named(NamedKey::Backspace)); // delete before caret (X): "i", caret 0
    assert_eq!(runner.state().text(), "i");
    assert_eq!(runner.state().caret(), 0, "caret back at the start");
    assert_eq!(
        field_text(&dom.borrow(), runner.root()),
        "i",
        "DOM text is the clean buffer"
    );
}

/// `TextInput`'s edits are character-correct across a multi-byte char (`é` is
/// two UTF-8 bytes), exercised directly on the model (no runner): a byte-index
/// bug would split `é` and panic or corrupt the buffer.
#[test]
fn text_input_edits_are_char_correct() {
    let mut t = TextInput::new("aé"); // 2 chars, caret at 2
    assert_eq!(t.caret(), 2);
    t.move_left(false); // caret 1 (between 'a' and 'é')
    t.insert_str("X"); // "aXé", caret 2
    assert_eq!(t.text(), "aXé");
    assert_eq!(t.caret(), 2);
    t.backspace(); // remove 'X' before caret: "aé", caret 1
    assert_eq!(t.text(), "aé");
    t.delete(); // remove 'é' after caret: "a", caret 1
    assert_eq!(t.text(), "a");
    assert_eq!(t.caret(), 1);
    assert_eq!(t.display(), "a|");
}

/// `lens(text_field, |f| &mut f.name)` under a `<div>`: the field edits only
/// `Form::name`, leaving `Form::other` untouched. The same key sequence as
/// the bare-`String` test, proving the field is a drop-in `lens` component.
#[test]
fn text_field_composes_under_lens() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |_f: &Form| {
            // `text_field` takes `&TextInput` (the field renders from a
            // shared read), so a thin `|s: &mut TextInput| text_field(s)`
            // adapter bridges it to the `Fn(&mut ChildState) -> View` shape
            // `lens` wants — the same adapter the Stage 3a `bump_button` test
            // uses.
            el::<_, Form, ()>(
                "div",
                lens(
                    |s: &mut TextInput| text_field(s),
                    |f: &mut Form| &mut f.name,
                ),
            )
        },
        Form {
            name: TextInput::default(),
            other: "untouched".to_string(),
        },
    );
    let root = runner.root();

    let input = {
        let d = dom.borrow();
        find_textbox(&d, root).expect("the field renders a textbox")
    };

    runner.set_focus(Some(input));
    type_hi_y(&mut runner);

    assert_eq!(
        runner.state().name.text(),
        "hi y",
        "only the lensed field changed"
    );
    assert_eq!(
        runner.state().other,
        "untouched",
        "the sibling field must be untouched"
    );
    assert_eq!(
        text_child(&dom.borrow(), input).as_deref(),
        Some("hi y"),
        "the rebuild reflected edits into the lensed field's rendered text"
    );
}

/// Home / End jump the caret to the line ends (the new named keys), driven
/// through the full key-dispatch + edit path.
#[test]
fn text_field_home_end_move_caret() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |s: &TextInput| text_field(s),
        TextInput::default(),
    );
    let input = {
        let d = dom.borrow();
        find_textbox(&d, runner.root()).expect("a textbox")
    };
    runner.set_focus(Some(input));

    runner.key(ch("a"));
    runner.key(ch("b"));
    runner.key(ch("c")); // "abc", caret 3
    runner.key(named(NamedKey::Home));
    assert_eq!(runner.state().caret(), 0, "Home jumps to the start");
    assert_eq!(
        field_text(&dom.borrow(), runner.root()),
        "abc",
        "Home moves the caret, not the buffer text"
    );

    runner.key(named(NamedKey::End));
    assert_eq!(runner.state().caret(), 3, "End jumps to the end");
    assert_eq!(
        field_text(&dom.borrow(), runner.root()),
        "abc",
        "End moves the caret, not the buffer text"
    );
}

/// `TextInput` multi-line navigation (the textarea model): up/down move between
/// `\n`-delimited lines keeping a **sticky goal column** (Tier 2), clamped to each
/// line's length but restored on a longer line; home/end scope to the current line.
#[test]
fn textinput_line_navigation() {
    use crate::TextInput;
    // "abc\nde\nfghi": chars a0 b1 c2 \n3 d4 e5 \n6 f7 g8 h9 i10 (len 11).
    let mut t = TextInput::new("abc\nde\nfghi");
    assert_eq!(t.caret(), 11, "new() puts the caret at the end");
    t.move_up(false);
    assert_eq!(
        t.caret(),
        6,
        "up: goal col 4 clamps to end of 'de' (offset 6)"
    );
    t.move_up(false);
    // Tier 2: the goal column stays 4 (the original), so it clamps to the end of the
    // 3-char 'abc' (offset 3) — it does *not* drift to the clamped col 2 of 'de'.
    assert_eq!(
        t.caret(),
        3,
        "up: sticky goal col 4 clamps to end of 'abc' (offset 3)"
    );
    t.move_down(false);
    assert_eq!(
        t.caret(),
        6,
        "down: sticky goal col 4 clamps to end of 'de' (offset 6)"
    );
    t.home_line(false);
    assert_eq!(
        t.caret(),
        4,
        "home: start of 'de' (offset 4); resets the goal column"
    );
    t.end_line(false);
    assert_eq!(t.caret(), 6, "end: end of 'de' (offset 6)");
}

/// IME preedit (T2): the composing text renders spliced at the caret but
/// stays out of the committed buffer, and the caret sits after it.
#[test]
fn textinput_preedit_renders_inline() {
    use crate::TextInput;
    let mut t = TextInput::new("ab"); // caret at end (2)
    t.move_left(false); // caret between 'a' and 'b' (1)
    t.set_preedit("XY");
    assert_eq!(t.render_text(), "aXYb", "preedit spliced at the caret");
    assert_eq!(t.text(), "ab", "buffer stays clean (preedit not committed)");
    // In "aXYb", the caret sits after "XY": byte_of(1)=1 + preedit "XY"=2 → 3.
    assert_eq!(t.caret_byte_in_render(), 3);

    t.clear_preedit();
    assert_eq!(t.render_text(), "ab", "cleared preedit → plain buffer");
    assert_eq!(t.caret_byte_in_render(), 1);
}

/// Ghost autocomplete: the suffix is shown but stays out of the committed
/// buffer and the caret geometry, so submitting evaluates only the typed text.
/// `accept_ghost` (the host's → / Tab) is the only path that commits it.
#[test]
fn textinput_ghost_is_uncommitted_until_accepted() {
    use crate::TextInput;
    let mut t = TextInput::new(">ros"); // caret at end (4)
    t.set_ghost("ter");
    // The ghost is visible via ghost(), but never via the buffer or render.
    assert_eq!(t.ghost(), "ter");
    assert_eq!(t.text(), ">ros", "ghost is not in the committed buffer");
    assert_eq!(
        t.render_text(),
        ">ros",
        "ghost is not in the rendered (caret) text"
    );
    assert_eq!(
        t.caret_byte_in_render(),
        4,
        "the caret sits before the ghost"
    );

    // Accepting splices it in, moves the caret to the end, and clears it.
    t.accept_ghost();
    assert_eq!(t.text(), ">roster");
    assert_eq!(t.ghost(), "", "ghost cleared after accept");
    assert_eq!(t.caret(), 7, "caret moved to the new end");

    // Accepting with no ghost is a no-op.
    t.accept_ghost();
    assert_eq!(t.text(), ">roster");
}

/// Ctrl / Cmd + A selects the whole buffer (anchor at start, caret at end).
#[test]
fn textinput_select_all_covers_the_buffer() {
    use crate::TextInput;
    let mut t = TextInput::new("hello");
    t.move_left(false); // a collapsed caret, no selection
    assert!(!t.has_selection());
    t.select_all();
    assert!(t.has_selection());
    assert_eq!(t.selection(), (0, 5));
    assert_eq!(t.selected_text(), "hello");
}

/// `TextInput` selection: extending, replacing, deleting, and collapsing —
/// exercised directly on the model.
#[test]
fn text_input_selection_model() {
    // Select all (Home, then Shift+End) and replace it.
    let mut t = TextInput::new("hello");
    assert!(!t.has_selection());
    assert_eq!(t.selected_text(), "", "no selection → empty");
    t.home(false);
    t.end(true);
    assert!(t.has_selection());
    assert_eq!(t.selection(), (0, 5));
    assert_eq!(t.selected_text(), "hello"); // copy/cut source
    t.insert_str("X"); // replaces the whole selection
    assert_eq!(t.text(), "X");
    assert_eq!(t.caret(), 1);
    assert!(!t.has_selection());

    // Select "bc" in "abcd" (Shift+→ from index 1) and backspace it away.
    let mut t = TextInput::new("abcd");
    t.home(false);
    t.move_right(false); // caret 1
    t.move_right(true); // sel 1..2
    t.move_right(true); // sel 1..3 = "bc"
    assert_eq!(t.selection(), (1, 3));
    assert_eq!(t.selected_text(), "bc");
    t.backspace();
    assert_eq!(t.text(), "ad");
    assert_eq!(t.caret(), 1);
    assert!(!t.has_selection());

    // Shift+← extends; a plain ← collapses to the selection's left edge.
    let mut t = TextInput::new("abc");
    t.move_left(true);
    t.move_left(true); // sel 1..3
    assert_eq!(t.selection(), (1, 3));
    t.move_left(false); // collapse to the left edge, no extra move
    assert!(!t.has_selection());
    assert_eq!(t.caret(), 1);
}

/// Keyboard selection through the field: type, `Shift+←` to select, then type
/// to replace — proving the edit handler reads `ev.mods.shift`.
#[test]
fn text_field_keyboard_selection_replaces() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |s: &TextInput| text_field(s),
        TextInput::default(),
    );
    let input = {
        let d = dom.borrow();
        find_textbox(&d, runner.root()).expect("a textbox")
    };
    runner.set_focus(Some(input));

    runner.key(ch("a"));
    runner.key(ch("b"));
    runner.key(ch("c")); // "abc", caret 3
    runner.key(shift_named(NamedKey::ArrowLeft)); // sel 2..3
    runner.key(shift_named(NamedKey::ArrowLeft)); // sel 1..3 = "bc"
    assert!(runner.state().has_selection());
    assert_eq!(runner.state().selection(), (1, 3));

    runner.key(ch("X")); // replaces the selection
    assert_eq!(runner.state().text(), "aX");
    assert!(!runner.state().has_selection());
    assert_eq!(
        field_text(&dom.borrow(), runner.root()),
        "aX",
        "DOM text is the clean buffer after the replace"
    );
}
