// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Reading controls shared by native and Web hosts. Persistence, font intent,
//! document search and the editable field belong to their existing owners.
use crate::{el, shell_control, ShellControl, ShellSlot, OptionalAction};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadCommand { Smaller, Larger, Narrower, Wider, Reset, Find, Previous, Next, CloseFind }

pub fn read_toolbar<State: 'static, Action: 'static, OA: OptionalAction<Action> + 'static>(
    text_size: u16, column_em: u16, find_open: bool,
    find: Option<ShellSlot<State, Action>>,
    handler: impl Fn(&mut State, ReadCommand) -> OA + 'static,
) -> ShellSlot<State, Action> {
    let handler = Rc::new(handler);
    let mut controls: Vec<ShellSlot<State, Action>> = Vec::new();
    for (label, name, command, disabled) in [
        ("A−", "Smaller reading text", ReadCommand::Smaller, text_size <= 14),
        ("A+", "Larger reading text", ReadCommand::Larger, text_size >= 28),
        ("↤", "Narrower reading column", ReadCommand::Narrower, column_em <= 24),
        ("↦", "Wider reading column", ReadCommand::Wider, column_em >= 48),
        ("Reset", "Reset reading preferences", ReadCommand::Reset, text_size == 18 && column_em == 36),
        ("Find", "Find in reading", ReadCommand::Find, false),
    ] {
        if command == ReadCommand::Narrower {
            controls.push(Box::new(el("span", format!("{text_size} px")).attr("class", "read-value")));
        }
        if command == ReadCommand::Reset {
            controls.push(Box::new(el("span", format!("{column_em} em")).attr("class", "read-value")));
        }
        let handler = handler.clone();
        controls.push(shell_control(label, name, ShellControl {
            disabled, expanded: (command == ReadCommand::Find).then_some(find_open), ..Default::default()
        }, move |state, _| handler(state, command)));
    }
    let mut rows: Vec<ShellSlot<State, Action>> = vec![Box::new(
        el("div", controls).attr("class", "read-controls").attr("role", "toolbar").attr("aria-label", "Read")
    )];
    if let Some(find) = find.filter(|_| find_open) {
        let mut controls = vec![find];
        for (label, name, command) in [
            ("↑", "Previous reading match", ReadCommand::Previous),
            ("↓", "Next reading match", ReadCommand::Next),
            ("×", "Close reading Find", ReadCommand::CloseFind),
        ] {
            let handler = handler.clone();
            controls.push(shell_control(label, name, ShellControl::default(), move |s, _| handler(s, command)));
        }
        rows.push(Box::new(el("div", controls).attr("class", "read-find").attr("role", "search").attr("aria-label", "Find in reading")));
    }
    Box::new(el("div", rows).attr("class", "cambium-read-toolbar"))
}
