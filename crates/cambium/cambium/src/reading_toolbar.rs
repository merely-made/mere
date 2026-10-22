// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Reading controls shared by native and Web hosts. Persistence, font intent,
//! document search and the editable field belong to their existing owners.
use crate::{
    ReadEvent, ReadFind, ShellControl, ShellSlot, el, find_field, find_row,
    is_find_shortcut, on_key, shell_control,
};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadCommand {
    Smaller,
    Larger,
    Narrower,
    Wider,
    Reset,
    Find,
    Previous,
    Next,
    CloseFind,
}

pub fn read_toolbar<State: 'static, Action: 'static>(
    text_size: u16,
    column_em: u16,
    text_controls: bool,
    find: Option<&ReadFind>,
    handler: impl Fn(&mut State, ReadEvent) + 'static,
) -> ShellSlot<State, Action> {
    let handler = Rc::new(handler);
    let find_open = find.is_some_and(|find| find.open);
    let mut controls: Vec<ShellSlot<State, Action>> = Vec::new();
    for (label, name, command, disabled) in [
        (
            "A−",
            "Smaller reading text",
            ReadCommand::Smaller,
            text_size <= 14,
        ),
        (
            "A+",
            "Larger reading text",
            ReadCommand::Larger,
            text_size >= 28,
        ),
        (
            "Narrow",
            "Narrower reading column",
            ReadCommand::Narrower,
            column_em <= 24,
        ),
        (
            "Wide",
            "Wider reading column",
            ReadCommand::Wider,
            column_em >= 48,
        ),
        (
            "Reset",
            "Reset reading preferences",
            ReadCommand::Reset,
            text_size == 18 && column_em == 36,
        ),
        ("Find", "Find in reading", ReadCommand::Find, find.is_none()),
    ] {
        if !text_controls && command != ReadCommand::Find {
            continue;
        }
        if command == ReadCommand::Narrower {
            controls.push(Box::new(
                el("span", format!("{text_size} px")).attr("class", "read-value"),
            ));
        }
        if command == ReadCommand::Reset {
            controls.push(Box::new(
                el("span", format!("{column_em} em")).attr("class", "read-value"),
            ));
        }
        let handler = handler.clone();
        controls.push(shell_control(
            label,
            name,
            ShellControl {
                disabled,
                expanded: (command == ReadCommand::Find).then_some(find_open),
                ..Default::default()
            },
            move |state, _| handler(state, ReadEvent::Command(command)),
        ));
    }
    let mut rows: Vec<ShellSlot<State, Action>> = vec![Box::new(
        el("div", controls)
            .attr("class", "read-controls")
            .attr("role", "toolbar")
            .attr("aria-label", "Read"),
    )];
    if let Some(find) = find.filter(|find| find.open) {
        let commands = handler.clone();
        rows.push(find_row(
            find_field(find, handler.clone()),
            Rc::new(move |state: &mut State, command| commands(state, ReadEvent::Command(command))),
        ));
    }
    let shortcut = handler;
    Box::new(on_key(
        el("div", rows).attr("class", "cambium-read-toolbar"),
        move |state: &mut State, event: crate::KeyEvent| {
            if is_find_shortcut(&event) {
                event.prevent_default();
                event.stop_propagation();
                shortcut(state, ReadEvent::Command(ReadCommand::Find));
            }
        },
    ))
}
