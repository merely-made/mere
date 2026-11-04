/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

pub(super) fn handle_key(
    state: &mut CommandState,
    root: &[(usize, CommandItem)],
    kind: CommandSurfaceKind,
    key_event: &KeyEvent,
) -> KeyOutput {
    let root_positions = navigable_positions(root, kind);
    let selected = nearest_enabled(state.selected, &root_positions);
    let active_parent = state.submenu.and_then(|parent| {
        root.iter()
            .find(|(index, _)| *index == parent)
            .map(|(_, item)| item)
    });
    let child_items: Vec<(usize, CommandItem)> = active_parent
        .map(|item| item.children.iter().cloned().enumerate().collect())
        .unwrap_or_default();
    let child_positions = navigable_positions(&child_items, kind);
    let child_selected = nearest_enabled(state.submenu_selected, &child_positions);
    let in_submenu = active_parent.is_some();

    let mut output = KeyOutput {
        prevent_default: true,
        event: None,
    };
    match &key_event.key {
        Key::Named(NamedKey::ArrowDown) => {
            if in_submenu {
                state.submenu_selected = next_enabled(child_selected, &child_positions, true);
            } else {
                state.selected = next_enabled(selected, &root_positions, true);
            }
        },
        Key::Named(NamedKey::ArrowUp) => {
            if in_submenu {
                state.submenu_selected = next_enabled(child_selected, &child_positions, false);
            } else {
                state.selected = next_enabled(selected, &root_positions, false);
            }
        },
        Key::Named(NamedKey::Home) => {
            if in_submenu {
                state.submenu_selected = child_positions.first().copied().unwrap_or(0);
            } else {
                state.selected = root_positions.first().copied().unwrap_or(0);
            }
        },
        Key::Named(NamedKey::End) => {
            if in_submenu {
                state.submenu_selected = child_positions.last().copied().unwrap_or(0);
            } else {
                state.selected = root_positions.last().copied().unwrap_or(0);
            }
        },
        Key::Named(NamedKey::ArrowRight)
            if matches!(kind, CommandSurfaceKind::ContextMenu { .. }) && !in_submenu =>
        {
            if let Some(position) = selected
                && let Some((index, item)) = root.get(position)
                && !item.disabled
                && !item.children.is_empty()
            {
                state.submenu = Some(*index);
                state.submenu_selected = navigable_positions(
                    &item
                        .children
                        .iter()
                        .cloned()
                        .enumerate()
                        .collect::<Vec<_>>(),
                    kind,
                )
                .first()
                .copied()
                .unwrap_or(0);
            }
        },
        Key::Named(NamedKey::ArrowLeft)
            if matches!(kind, CommandSurfaceKind::ContextMenu { .. }) && in_submenu =>
        {
            state.submenu = None;
        },
        Key::Named(NamedKey::Enter) => {
            if in_submenu {
                if let (Some(parent), Some(position)) = (state.submenu, child_selected)
                    && let Some((child, item)) = child_items.get(position)
                    && !item.disabled
                {
                    output.event = Some(CommandEvent::Activate(vec![parent, *child]));
                }
            } else if let Some(position) = selected
                && let Some((index, item)) = root.get(position)
                && !item.disabled
            {
                if matches!(kind, CommandSurfaceKind::ContextMenu { .. })
                    && !item.children.is_empty()
                {
                    state.submenu = Some(*index);
                    state.submenu_selected = 0;
                } else {
                    output.event = Some(CommandEvent::Activate(vec![*index]));
                }
            }
        },
        Key::Named(NamedKey::Space) if kind == CommandSurfaceKind::Palette => {
            state.query.push(' ');
            state.selected = 0;
            state.submenu = None;
        },
        Key::Named(NamedKey::Space) => {
            if in_submenu {
                if let (Some(parent), Some(position)) = (state.submenu, child_selected)
                    && let Some((child, item)) = child_items.get(position)
                    && !item.disabled
                {
                    output.event = Some(CommandEvent::Activate(vec![parent, *child]));
                }
            } else if let Some(position) = selected
                && let Some((index, item)) = root.get(position)
                && !item.disabled
            {
                if matches!(kind, CommandSurfaceKind::ContextMenu { .. })
                    && !item.children.is_empty()
                {
                    state.submenu = Some(*index);
                    state.submenu_selected = 0;
                } else {
                    output.event = Some(CommandEvent::Activate(vec![*index]));
                }
            }
        },
        Key::Named(NamedKey::Escape) => {
            if in_submenu {
                state.submenu = None;
            } else {
                output.event = Some(CommandEvent::Dismiss);
            }
        },
        Key::Named(NamedKey::Backspace) if kind == CommandSurfaceKind::Palette => {
            state.query.pop();
            state.selected = 0;
            state.submenu = None;
        },
        Key::Character(text) if kind == CommandSurfaceKind::Palette => {
            if !key_event.mods.ctrl && !key_event.mods.alt && !key_event.mods.meta {
                state.query.push_str(text);
                state.selected = 0;
                state.submenu = None;
            } else {
                output.prevent_default = false;
            }
        },
        Key::Named(NamedKey::Tab) if matches!(kind, CommandSurfaceKind::ContextMenu { .. }) => {
            output.event = Some(CommandEvent::Dismiss);
            output.prevent_default = false;
        },
        _ => {
            output.prevent_default = false;
        },
    }
    output
}
