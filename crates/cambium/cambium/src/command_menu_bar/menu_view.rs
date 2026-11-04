/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

pub(super) fn menu_view(
    state: &CommandMenuBarState,
    items: &[CommandItem],
    compact: bool,
    path: &[usize],
) -> BarView {
    let Some(menu) = menu_items(items, path) else {
        return Box::new(el::<_, CommandMenuBarState, CommandEvent>("div", ()));
    };
    let depth = path.len() - usize::from(!compact);
    let selected = state
        .selected_path
        .get(depth)
        .copied()
        .unwrap_or(0)
        .min(menu.len().saturating_sub(1));
    let mut rows: Vec<BarView> = Vec::new();
    // Compact menus navigate one level at the same anchor. Sideways flyouts
    // would escape a narrow viewport after the first disclosure.
    if compact && !path.is_empty() {
        rows.push(Box::new(on_click(
            el::<_, CommandMenuBarState, CommandEvent>("div", "‹ Back")
                .attr("class", "command-menu-bar-row command-menu-bar-back")
                .attr("role", "menuitem")
                .attr("aria-label", "Back to parent menu")
                .attr("tabindex", "-1"),
            |state: &mut CommandMenuBarState, click: PointerClick| {
                click.stop_propagation();
                state.open_path.pop();
                state.selected_path.pop();
                state.focus_active = true;
            },
        )));
    }
    for (index, item) in menu.iter().enumerate() {
        let mut item_path = path.to_vec();
        item_path.push(index);
        let submenu = (!compact
            && state.open_path.len() > path.len()
            && state.open_path.starts_with(&item_path))
        .then(|| menu_view(state, items, compact, &item_path));
        let reason = item.disabled_reason.as_ref().map(|reason| {
            el::<_, CommandMenuBarState, CommandEvent>("span", reason.clone())
                .attr("class", "command-menu-bar-reason")
                .attr("aria-hidden", "true")
        });
        let shortcut = item.shortcut.as_ref().map(|shortcut| {
            el::<_, CommandMenuBarState, CommandEvent>("span", shortcut.clone())
                .attr("class", "command-menu-bar-shortcut")
                .attr("aria-hidden", "true")
        });
        let mark = (!item.children.is_empty()).then(|| {
            el::<_, CommandMenuBarState, CommandEvent>("span", "›").attr("aria-hidden", "true")
        });
        let mut row = el::<_, CommandMenuBarState, CommandEvent>(
            "div",
            (
                el::<_, CommandMenuBarState, CommandEvent>("span", item.label.clone()),
                reason,
                shortcut,
                mark,
                submenu,
            ),
        )
        .attr("id", row_id(state, &item_path))
        .attr("data-key", item.id.clone())
        .attr(
            "class",
            if index == selected {
                "command-menu-bar-row selected"
            } else {
                "command-menu-bar-row"
            },
        )
        .attr("role", "menuitem")
        .attr("aria-label", item.label.clone())
        .attr("tabindex", "-1")
        .attr(
            "aria-disabled",
            if item.disabled { "true" } else { "false" },
        );
        if let Some(reason) = &item.disabled_reason {
            row = row.attr("aria-description", reason.clone());
        }
        if !item.children.is_empty() {
            row = row.attr("aria-haspopup", "menu").attr(
                "aria-expanded",
                if state.open_path.starts_with(&item_path) && state.open_path.len() > path.len() {
                    "true"
                } else {
                    "false"
                },
            );
        }
        let disabled = item.disabled;
        let has_children = !item.children.is_empty();
        rows.push(Box::new(on_click(
            row,
            move |state: &mut CommandMenuBarState, click: PointerClick| {
                click.stop_propagation();
                if disabled {
                    return None;
                }
                if let Some(slot) = state.selected_path.get_mut(depth) {
                    *slot = index;
                }
                state.selected_path.truncate(depth + 1);
                if has_children {
                    state.open_path = item_path.clone();
                    state.selected_path.push(0);
                    None
                } else {
                    state.close();
                    Some(CommandEvent::Activate(item_path.clone()))
                }
            },
        )));
    }
    let selected_path = {
        let mut p = path.to_vec();
        p.push(selected);
        p
    };
    let current = state.open_path == path;
    let menu = el::<_, CommandMenuBarState, CommandEvent>("div", rows)
        .attr(
            "class",
            if compact {
                "command-menu-bar-menu command-menu-bar-compact"
            } else if depth == 0 {
                "command-menu-bar-menu"
            } else {
                "command-menu-bar-menu command-menu-bar-submenu"
            },
        )
        .attr("role", "menu")
        .attr("tabindex", if current { "0" } else { "-1" })
        .attr(
            "aria-activedescendant",
            if menu.is_empty() {
                String::new()
            } else {
                row_id(state, &selected_path)
            },
        );
    let path_for_focus = path.to_vec();
    Box::new(request_focus(
        focusable_if(
            on_focus(menu, move |state: &mut CommandMenuBarState, focus| {
                if focus.phase == FocusPhase::Lost
                    && state.open
                    && state.open_path == path_for_focus
                {
                    state.leave();
                }
            }),
            current,
        ),
        state.open && current && state.focus_active,
    ))
}
