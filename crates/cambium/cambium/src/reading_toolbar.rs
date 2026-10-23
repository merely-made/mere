// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Reading controls shared by native and Web hosts. Persistence, font intent,
//! document search and the editable field belong to their existing owners.
use crate::{
    OptionalAction, ReadEvent, ReadFind, ShellControl, ShellSlot, el, find_field, find_row,
    is_find_shortcut, on_key, shell_control,
};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadCommand {
    Appearance,
    Back,
    Forward,
    ShowGraph,
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
    let find_open = find.is_some_and(|find| find.open && find.available);
    let appearance_open = find.is_some_and(|ui| ui.appearance_open);
    let mut header = Vec::new();
    for (label, name, command, disabled, expanded) in [
        ("Aa", "Reading appearance", ReadCommand::Appearance, !text_controls, appearance_open),
        ("Find", "Find in reading", ReadCommand::Find, find.is_none_or(|find| !find.available), find_open),
    ] {
        let handler = handler.clone();
        header.push(shell_control(label, name, ShellControl { disabled, expanded: Some(expanded), ..Default::default() },
            move |state, _| handler(state, ReadEvent::Command(command))));
    }
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
        el("div", header)
            .attr("class", "read-header")
            .attr("role", "toolbar")
            .attr("aria-label", "Read"),
    )];
    if appearance_open && text_controls {
        rows.push(Box::new(el("div", controls).attr("class", "read-controls").attr("role", "group").attr("aria-label", "Reading appearance")));
    }
    if let Some(find) = find.filter(|find| find.open && find.available) {
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

/// A temporary, explicitly activated relationship preview. Hosts admit actions
/// against current graph truth and provide only text they actually retain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationshipPreviewCommand { Read, ShowGraph, Close }
pub fn relationship_preview<State:'static, Action:'static, OA:OptionalAction<Action>+'static>(
    title:String, relationship:String, address:String, excerpt:Option<String>,
    handler:impl Fn(&mut State, RelationshipPreviewCommand)->OA+'static,
)->ShellSlot<State,Action> {
    let handler=Rc::new(handler);
    let mut children:Vec<ShellSlot<State,Action>>=vec![
        Box::new(el("p", relationship).attr("class","relationship-kind")),
        Box::new(el("strong",title)), Box::new(el("p",address).attr("class","relationship-address")),
        Box::new(el("p",excerpt.unwrap_or_else(||"No retained passage available for preview.".into())).attr("class","relationship-excerpt")),
    ];
    let mut controls=Vec::new();
    for (label,name,command) in [("Read", "Read previewed document", RelationshipPreviewCommand::Read), ("Graph", "Show previewed document on graph", RelationshipPreviewCommand::ShowGraph), ("Close", "Close relationship preview", RelationshipPreviewCommand::Close)] {
        let handler=handler.clone();
        controls.push(shell_control(label,name,ShellControl::default(),move |state,_|handler(state,command)));
    }
    children.push(Box::new(el("div",controls).attr("class","read-header")));
    Box::new(el("div",children).attr("class","cambium-relationship-preview").attr("role","group").attr("aria-label","Relationship preview"))
}
