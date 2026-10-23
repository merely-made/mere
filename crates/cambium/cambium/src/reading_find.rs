// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The find field every host mounts: one key map, one status wording, one
//! row. Document search itself stays with the host's engine; this owns what
//! the person sees and which command a key means. A host whose keys arrive
//! as winit events converts them to `KeyEvent`s and dispatches them into the
//! mounted field; it keeps no key map of its own.
use crate::{
    Key, KeyEvent, NamedKey, OptionalAction, ReadCommand, SINGLE_LINE_FIELD_STYLE, ShellControl,
    ShellSlot, TextFieldMode, TextInput, caret_field_children, el, on_key, request_focus,
    shell_control,
};
use std::rc::Rc;

/// What the toolbar and its find field report to the host.
#[derive(Clone, Debug, PartialEq)]
pub enum ReadEvent {
    Command(ReadCommand),
    /// A key edited the field; carries the edited input, whether or not the
    /// text changed.
    Query(TextInput),
}

/// Retained find field state; one per reading appearance.
#[derive(Clone, Debug, PartialEq)]
pub struct ReadFind {
    /// The host has a current document-search supplier. Appearance controls remain independent.
    pub available: bool,
    pub appearance_open: bool,
    pub open: bool,
    pub input: TextInput,
    pub status: String,
}

impl Default for ReadFind {
    fn default() -> Self { Self { available: true, appearance_open: false, open: false, input: Default::default(), status: String::new() } }
}

/// The one status wording. `searching` is for engines that answer later.
pub fn find_status(
    query_empty: bool,
    searching: bool,
    count: usize,
    current: Option<usize>,
) -> String {
    if query_empty {
        return "Type to find".into();
    }
    match (count, current) {
        (0, _) if searching => "Searching".into(),
        (0, _) => "No matches".into(),
        (count, Some(index)) => format!("{} of {count}", index + 1),
        (count, None) => format!("{count} matches"),
    }
}

/// An engine's refusal, worded the same everywhere.
pub fn find_failed(error: &str) -> String {
    format!("Find failed: {error}")
}

/// Enter steps next, Shift+Enter previous, Escape closes; anything else edits.
pub fn find_key(input: &TextInput, event: &KeyEvent) -> ReadEvent {
    match event.key {
        Key::Named(NamedKey::Enter) => ReadEvent::Command(if event.mods.shift {
            ReadCommand::Previous
        } else {
            ReadCommand::Next
        }),
        Key::Named(NamedKey::Escape) => ReadEvent::Command(ReadCommand::CloseFind),
        _ => {
            let mut edited = input.clone();
            edited.apply_key(event, TextFieldMode::SingleLine);
            ReadEvent::Query(edited)
        },
    }
}

/// Ctrl/Cmd+F opens find.
pub fn is_find_shortcut(event: &KeyEvent) -> bool {
    (event.mods.ctrl || event.mods.meta)
        && matches!(&event.key, Key::Character(key) if key.eq_ignore_ascii_case("f"))
}

fn field_body<State: 'static, Action: 'static>(input: &TextInput) -> crate::El<Vec<crate::FieldChild<State, Action>>, State, Action> {
    el("div", caret_field_children::<State, Action>(input, &[]))
        .attr("role", "textbox")
        .attr("aria-label", "Find text in reading")
        .attr("class", "read-query")
        .attr("data-cambium-text-value", input.text())
        .attr("style", SINGLE_LINE_FIELD_STYLE)
}

fn status_view<State: 'static, Action: 'static>(status: &str) -> ShellSlot<State, Action> {
    Box::new(
        el("span", status.to_owned())
            .attr("class", "read-result")
            .attr("role", "status")
            .attr("aria-live", "polite"),
    )
}

/// The field with the shared key map; focus follows `find.open`. Every host
/// mounts this one field: the handler may mutate state, return an action, or
/// both.
pub fn find_field<State: 'static, Action: 'static, OA: OptionalAction<Action> + 'static>(
    find: &ReadFind,
    handler: Rc<impl Fn(&mut State, ReadEvent) -> OA + 'static>,
) -> ShellSlot<State, Action> {
    let input = find.input.clone();
    let query = request_focus(
        on_key(field_body::<State, Action>(&find.input), move |state: &mut State, event: KeyEvent| {
            event.prevent_default();
            event.stop_propagation();
            handler(state, find_key(&input, &event))
        }),
        find.open,
    );
    Box::new(el("div", (query, status_view::<State, Action>(&find.status))).attr("class", "read-find-input"))
}

/// The find row: the field with previous, next and close.
pub fn find_row<State: 'static, Action: 'static, OA: OptionalAction<Action> + 'static>(
    field: ShellSlot<State, Action>,
    handler: Rc<impl Fn(&mut State, ReadCommand) -> OA + 'static>,
) -> ShellSlot<State, Action> {
    let mut controls = vec![field];
    for (label, name, command) in [
        ("↑", "Previous reading match", ReadCommand::Previous),
        ("↓", "Next reading match", ReadCommand::Next),
        ("×", "Close reading Find", ReadCommand::CloseFind),
    ] {
        let handler = handler.clone();
        controls.push(shell_control(label, name, ShellControl::default(), move |s, _| {
            handler(s, command)
        }));
    }
    Box::new(
        el("div", controls)
            .attr("class", "read-find")
            .attr("role", "search")
            .attr("aria-label", "Find in reading"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Modifiers;

    fn key(key: Key, shift: bool) -> KeyEvent {
        let mut event = KeyEvent::new(key);
        event.mods = Modifiers { shift, ..Default::default() };
        event
    }

    #[test]
    fn status_wording_is_one_vocabulary() {
        assert_eq!(find_status(true, false, 0, None), "Type to find");
        assert_eq!(find_status(false, true, 0, None), "Searching");
        assert_eq!(find_status(false, false, 0, None), "No matches");
        assert_eq!(find_status(false, false, 4, Some(1)), "2 of 4");
        assert_eq!(find_status(false, false, 4, None), "4 matches");
    }

    #[test]
    fn keys_map_to_step_close_or_edit() {
        let input = TextInput::new("ab");
        assert_eq!(
            find_key(&input, &key(Key::Named(NamedKey::Enter), false)),
            ReadEvent::Command(ReadCommand::Next)
        );
        assert_eq!(
            find_key(&input, &key(Key::Named(NamedKey::Enter), true)),
            ReadEvent::Command(ReadCommand::Previous)
        );
        assert_eq!(
            find_key(&input, &key(Key::Named(NamedKey::Escape), false)),
            ReadEvent::Command(ReadCommand::CloseFind)
        );
        let ReadEvent::Query(edited) = find_key(&input, &key(Key::Character("c".into()), false)) else {
            panic!("a character edits the query");
        };
        assert_eq!(edited.text(), "abc");
        assert_eq!(input.text(), "ab");
    }

    #[derive(Default)]
    struct Host {
        find: ReadFind,
        seen: Vec<ReadEvent>,
    }

    fn host_view(host: &Host) -> ShellSlot<Host, ()> {
        find_field(
            &host.find,
            Rc::new(|host: &mut Host, event: ReadEvent| {
                if let ReadEvent::Query(input) = &event {
                    host.find.input = input.clone();
                }
                host.seen.push(event);
            }),
        )
    }

    fn mounted() -> crate::GenetAppRunner<
        Host,
        fn(&Host) -> ShellSlot<Host, ()>,
        ShellSlot<Host, ()>,
        (),
    > {
        let dom: crate::DomHandle = Rc::new(std::cell::RefCell::new(genet_scripted_dom::ScriptedDom::new()));
        let mut host = Host::default();
        host.find.open = true;
        crate::GenetAppRunner::new(dom, host_view as fn(&Host) -> ShellSlot<Host, ()>, host)
    }

    #[test]
    fn the_mounted_field_applies_the_shared_map_with_a_working_caret() {
        let mut runner = mounted();
        let press = |runner: &mut crate::GenetAppRunner<_, _, _, ()>, key: Key, shift: bool| {
            runner.dispatch_key(self::key(key, shift));
        };
        for text in ["a", "b", "d"] {
            press(&mut runner, Key::Character(text.into()), false);
        }
        press(&mut runner, Key::Named(NamedKey::ArrowLeft), false);
        press(&mut runner, Key::Character("c".into()), false);
        assert_eq!(runner.state().find.input.text(), "abcd", "mid-string insert");
        press(&mut runner, Key::Named(NamedKey::Home), false);
        press(&mut runner, Key::Named(NamedKey::Delete), false);
        assert_eq!(runner.state().find.input.text(), "bcd", "Home then forward delete");
        press(&mut runner, Key::Named(NamedKey::End), false);
        press(&mut runner, Key::Named(NamedKey::ArrowLeft), true);
        press(&mut runner, Key::Named(NamedKey::ArrowLeft), true);
        assert_eq!(runner.state().find.input.selected_text(), "cd");
        press(&mut runner, Key::Character("x".into()), false);
        assert_eq!(runner.state().find.input.text(), "bx", "typing replaces the selection");
        press(&mut runner, Key::Named(NamedKey::Backspace), false);
        assert_eq!(runner.state().find.input.text(), "b");
    }

    #[test]
    fn the_mounted_field_reports_what_find_key_maps() {
        let mut runner = mounted();
        runner.update(|host| host.find.input = TextInput::new("ab"));
        for (key_event, expected) in [
            (key(Key::Named(NamedKey::Enter), false), ReadEvent::Command(ReadCommand::Next)),
            (key(Key::Named(NamedKey::Enter), true), ReadEvent::Command(ReadCommand::Previous)),
            (key(Key::Named(NamedKey::Escape), false), ReadEvent::Command(ReadCommand::CloseFind)),
        ] {
            assert_eq!(find_key(&runner.state().find.input, &key_event), expected);
            let before = runner.state().seen.len();
            runner.dispatch_key(key_event);
            assert_eq!(runner.state().seen[before..], [expected]);
        }
        assert_eq!(runner.state().find.input.text(), "ab", "commands leave the text alone");
    }

    impl crate::optional_action::Action for ReadEvent {}

    #[test]
    fn a_host_handler_may_return_the_action() {
        let dom: crate::DomHandle = Rc::new(std::cell::RefCell::new(genet_scripted_dom::ScriptedDom::new()));
        let mut find = ReadFind::default();
        find.open = true;
        let mut runner = crate::GenetAppRunner::<ReadFind, _, _, ReadEvent>::new(
            dom,
            |find: &ReadFind| -> ShellSlot<ReadFind, ReadEvent> {
                find_field(find, Rc::new(|_: &mut ReadFind, event: ReadEvent| Some(event)))
            },
            find,
        );
        let actions = runner.dispatch_key(key(Key::Named(NamedKey::Enter), true));
        assert_eq!(actions, [ReadEvent::Command(ReadCommand::Previous)]);
    }

    #[test]
    fn ctrl_or_cmd_f_opens_find() {
        let mut event = key(Key::Character("F".into()), false);
        assert!(!is_find_shortcut(&event));
        event.mods.ctrl = true;
        assert!(is_find_shortcut(&event));
        event.key = Key::Character("g".into());
        assert!(!is_find_shortcut(&event));
    }
}
