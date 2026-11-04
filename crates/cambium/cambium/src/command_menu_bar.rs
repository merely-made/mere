/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A desktop menubar over the same `CommandItem` tree as the palette and menus.
//! The caller chooses compact presentation from its measured titlebar width.

use meristem::AnyView;

use crate::{
    CommandEvent, CommandItem, FocusPhase, GenetCtx, GenetElement, Key, KeyEvent, NamedKey,
    PointerClick, View, el, focusable_if, on_click, on_focus, on_key, request_focus,
};

type BarView = Box<dyn AnyView<CommandMenuBarState, CommandEvent, GenetCtx, GenetElement>>;

/// Styles for a bar placed inside a client-drawn titlebar. The bar and every
/// menu control carve a `no-drag` region out of a draggable titlebar.
pub const COMMAND_MENU_BAR_CSS: &str = r#"
.command-menu-bar { --app-region: no-drag; display: inline-flex; align-items: stretch; position: relative; }
.command-menu-bar-item { --app-region: no-drag; position: relative; padding: 5px 10px; cursor: default; }
.command-menu-bar-item:focus-visible, .command-menu-bar-item.open { outline: 2px solid currentColor; outline-offset: -2px; }
.command-menu-bar-menu { --app-region: no-drag; position: absolute; z-index: 100; left: 0; top: 100%; min-width: 190px; padding: 4px; background: var(--menu-background, #242833); color: var(--menu-foreground, #f5f5f5); box-shadow: 0 8px 24px #0006; }
.command-menu-bar-row { --app-region: no-drag; position: relative; display: flex; gap: 12px; align-items: center; min-height: 28px; padding: 2px 8px; white-space: nowrap; cursor: default; }
.command-menu-bar-row.selected { background: var(--menu-selection, #40546f); }
.command-menu-bar-row[aria-disabled="true"] { opacity: .62; }
.command-menu-bar-shortcut { margin-left: auto; opacity: .72; }
.command-menu-bar-reason { font-size: .85em; opacity: .72; }
.command-menu-bar-submenu { left: 100%; top: 0; }
.command-menu-bar-compact { box-sizing: border-box; width: 198px; min-width: 0; max-width: calc(100vw - 24px); max-height: 70vh; overflow: auto; }
.command-menu-bar-compact .command-menu-bar-row { white-space: normal; }
"#;

/// Retained focus and disclosure state. `open_path` addresses the parent whose
/// children form the deepest open menu; `selected_path` holds each level's
/// selected row. Paths always index the caller's original `CommandItem` tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandMenuBarState {
    pub active: usize,
    pub open: bool,
    pub open_path: Vec<usize>,
    pub selected_path: Vec<usize>,
    pub id: String,
    pub label: String,
    /// Set by [`Self::enter`] to move focus to the active top-level item.
    pub focus_active: bool,
    compact: bool,
}

impl Default for CommandMenuBarState {
    fn default() -> Self {
        Self {
            active: 0,
            open: false,
            open_path: Vec::new(),
            selected_path: Vec::new(),
            id: "cambium-command-menu-bar".into(),
            label: "Application commands".into(),
            focus_active: false,
            compact: false,
        }
    }
}

impl CommandMenuBarState {
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Focus the bar without opening a menu. Call this from the application's
    /// window-level Alt/F10 handler when focus currently belongs elsewhere.
    pub fn enter(&mut self) {
        self.open = false;
        self.open_path.clear();
        self.selected_path.clear();
        self.focus_active = true;
    }

    /// Recognize a platform-neutral entry key. The host should route this
    /// only on platforms where the client-drawn menubar is shown. In
    /// particular, macOS Option remains available to text input.
    pub fn handle_entry_key(&mut self, event: &KeyEvent) -> bool {
        let plain_f10 = matches!(event.key, Key::Named(NamedKey::F10))
            && !event.mods.shift
            && !event.mods.ctrl
            && !event.mods.alt
            && !event.mods.meta;
        let plain_alt = matches!(event.key, Key::Named(NamedKey::Alt))
            && !event.mods.shift
            && !event.mods.ctrl
            && !event.mods.meta;
        if plain_f10 || plain_alt {
            self.enter();
            event.prevent_default();
            true
        } else {
            false
        }
    }

    fn close(&mut self) {
        self.open = false;
        self.open_path.clear();
        self.selected_path.clear();
        self.focus_active = true;
    }

    fn leave(&mut self) {
        self.open = false;
        self.open_path.clear();
        self.selected_path.clear();
        self.focus_active = false;
    }

    fn show(&mut self, items: &[CommandItem], compact: bool) {
        self.compact = compact;
        if !compact {
            self.active = effective_active(items, self.active);
        }
        self.open = true;
        self.open_path = if compact {
            Vec::new()
        } else {
            vec![self.active]
        };
        self.selected_path = vec![0];
        self.focus_active = true;
    }

    fn sync_mode(&mut self, items: &[CommandItem], compact: bool) {
        if self.compact != compact {
            self.close();
            self.compact = compact;
        }
        if !compact {
            self.active = effective_active(items, self.active);
        }
        let base = usize::from(!compact);
        if self.open
            && (self.open_path.len() < base
                || menu_items(items, &self.open_path).is_none()
                || self.selected_path.len() <= self.open_path.len() - base)
        {
            self.close();
        }
    }
}

/// Render a menubar from the same command tree used by `command_palette` and
/// `command_menu`. `compact` folds every group under one Menu button; the
/// caller owns the width threshold. An activation emits
/// `CommandEvent::Activate` with a path from the original root.
pub fn command_menu_bar(
    state: &CommandMenuBarState,
    items: &[CommandItem],
    compact: bool,
) -> impl View<CommandMenuBarState, CommandEvent, GenetCtx, Element = GenetElement> + use<> {
    let mut normalized = state.clone();
    normalized.sync_mode(items, compact);
    let state = &normalized;
    let mut top: Vec<BarView> = Vec::new();
    let count = if compact { 1 } else { items.len() };
    for index in 0..count {
        let item = (!compact).then(|| &items[index]);
        let label = item.map_or("Menu", |item| item.label.as_str());
        let disabled = item.is_some_and(|item| item.disabled);
        let reason = item.and_then(|item| item.disabled_reason.as_deref());
        let active = if compact {
            true
        } else {
            index == effective_active(items, state.active)
        };
        let open = state.open && active;
        let popup_path = if compact {
            state.open_path.clone()
        } else {
            vec![index]
        };
        let popup = open.then(|| menu_view(state, items, compact, &popup_path));
        let mut top_item =
            el::<_, CommandMenuBarState, CommandEvent>("div", (label.to_owned(), popup))
                .attr("id", format!("{}-top-{index}", state.id))
                .attr(
                    "class",
                    if open {
                        "command-menu-bar-item open"
                    } else {
                        "command-menu-bar-item"
                    },
                )
                .attr("role", "menuitem")
                .attr("tabindex", if active { "0" } else { "-1" })
                .attr("aria-haspopup", "menu")
                .attr("aria-expanded", if open { "true" } else { "false" })
                .attr("aria-disabled", if disabled { "true" } else { "false" })
                .attr(
                    "data-key",
                    item.map_or("menu", |item| item.id.as_str()).to_owned(),
                );
        if let Some(reason) = reason {
            top_item = top_item.attr("aria-description", reason.to_owned());
        }
        let items_for_click = items.to_vec();
        top.push(Box::new(request_focus(
            focusable_if(
                on_focus(
                    on_click(
                        top_item,
                        move |state: &mut CommandMenuBarState, click: PointerClick| {
                            click.stop_propagation();
                            state.sync_mode(&items_for_click, compact);
                            if disabled {
                                return;
                            }
                            if state.open && (compact || state.active == index) {
                                state.close();
                            } else {
                                state.active = index;
                                state.show(&items_for_click, compact);
                            }
                        },
                    ),
                    |state: &mut CommandMenuBarState, focus| {
                        if focus.phase == FocusPhase::Lost && !state.open {
                            state.leave();
                        }
                    },
                ),
                active,
            ),
            state.focus_active && active && !state.open,
        )));
    }
    let root = el::<_, CommandMenuBarState, CommandEvent>("div", top)
        .attr("id", state.id.clone())
        .attr("class", "command-menu-bar")
        .attr("role", "menubar")
        .attr("aria-label", state.label.clone())
        .attr("aria-orientation", "horizontal");
    let items = items.to_vec();
    on_key(root, move |state: &mut CommandMenuBarState, event| {
        handle_key(state, &items, compact, &event)
    })
}

fn row_id(state: &CommandMenuBarState, path: &[usize]) -> String {
    format!(
        "{}-item-{}",
        state.id,
        path.iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join("-")
    )
}

fn menu_items<'a>(items: &'a [CommandItem], path: &[usize]) -> Option<&'a [CommandItem]> {
    let mut current = items;
    for index in path {
        current = &current.get(*index)?.children;
    }
    Some(current)
}

fn handle_key(
    state: &mut CommandMenuBarState,
    items: &[CommandItem],
    compact: bool,
    event: &KeyEvent,
) -> Option<CommandEvent> {
    let top_len = if compact { 1 } else { items.len() };
    if top_len == 0 {
        return None;
    }
    state.sync_mode(items, compact);
    let mut output = None;
    let mut handled = true;
    let base = usize::from(!compact);
    if !state.open {
        match event.key {
            Key::Named(NamedKey::ArrowLeft) => {
                state.active = next_top(items, state.active, false);
                state.focus_active = true;
            },
            Key::Named(NamedKey::ArrowRight) => {
                state.active = next_top(items, state.active, true);
                state.focus_active = true;
            },
            Key::Named(NamedKey::Home) => {
                state.active = first_top(items);
                state.focus_active = true;
            },
            Key::Named(NamedKey::End) => {
                state.active = last_top(items);
                state.focus_active = true;
            },
            Key::Named(NamedKey::ArrowDown | NamedKey::Enter | NamedKey::Space) => {
                if compact || !items[state.active.min(items.len() - 1)].disabled {
                    state.show(items, compact);
                }
            },
            Key::Named(NamedKey::ArrowUp) => {
                if compact || !items[state.active.min(items.len() - 1)].disabled {
                    state.show(items, compact);
                    if let Some(menu) = menu_items(items, &state.open_path) {
                        state.selected_path[0] = menu.len().saturating_sub(1);
                    }
                }
            },
            Key::Named(NamedKey::Escape) => {
                state.focus_active = false;
                output = Some(CommandEvent::Dismiss);
            },
            _ => handled = false,
        }
    } else {
        let Some(menu) = menu_items(items, &state.open_path) else {
            state.close();
            return None;
        };
        let depth = state.open_path.len() - base;
        let selected = state
            .selected_path
            .get(depth)
            .copied()
            .unwrap_or(0)
            .min(menu.len().saturating_sub(1));
        match event.key {
            Key::Named(NamedKey::ArrowDown) => {
                if !menu.is_empty() {
                    state.selected_path[depth] = (selected + 1) % menu.len();
                }
            },
            Key::Named(NamedKey::ArrowUp) => {
                if !menu.is_empty() {
                    state.selected_path[depth] = (selected + menu.len() - 1) % menu.len();
                }
            },
            Key::Named(NamedKey::Home) => {
                if !menu.is_empty() {
                    state.selected_path[depth] = 0;
                }
            },
            Key::Named(NamedKey::End) => {
                if !menu.is_empty() {
                    state.selected_path[depth] = menu.len() - 1;
                }
            },
            Key::Named(NamedKey::ArrowRight | NamedKey::Enter | NamedKey::Space) => {
                if let Some(item) = menu.get(selected)
                    && !item.disabled
                {
                    let mut path = state.open_path.clone();
                    path.push(selected);
                    if !item.children.is_empty() {
                        state.open_path = path;
                        state.selected_path.push(0);
                    } else if !matches!(event.key, Key::Named(NamedKey::ArrowRight)) {
                        state.close();
                        output = Some(CommandEvent::Activate(path));
                    } else if !compact && depth == 0 {
                        state.active = next_top(items, state.active, true);
                        state.show(items, false);
                    }
                } else if !compact
                    && depth == 0
                    && matches!(event.key, Key::Named(NamedKey::ArrowRight))
                {
                    state.active = next_top(items, state.active, true);
                    state.show(items, false);
                }
            },
            Key::Named(NamedKey::ArrowLeft) => {
                if state.open_path.len() > base {
                    state.open_path.pop();
                    state.selected_path.pop();
                } else if compact {
                    state.close();
                } else {
                    state.active = next_top(items, state.active, false);
                    state.show(items, false);
                }
            },
            Key::Named(NamedKey::Escape) => {
                if state.open_path.len() > base {
                    state.open_path.pop();
                    state.selected_path.pop();
                } else {
                    state.close();
                    output = Some(CommandEvent::Dismiss);
                }
            },
            Key::Named(NamedKey::Tab) => {
                state.close();
                output = Some(CommandEvent::Dismiss);
                handled = false;
            },
            _ => handled = false,
        }
    }
    if handled {
        event.prevent_default();
    }
    output
}

fn next_top(items: &[CommandItem], current: usize, forward: bool) -> usize {
    if items.is_empty() {
        return 0;
    }
    for step in 1..=items.len() {
        let index = if forward {
            (current + step) % items.len()
        } else {
            (current + items.len() - step % items.len()) % items.len()
        };
        if !items[index].disabled {
            return index;
        }
    }
    current.min(items.len() - 1)
}

fn effective_active(items: &[CommandItem], active: usize) -> usize {
    if items.get(active).is_some_and(|item| !item.disabled) {
        return active;
    }
    items.iter().position(|item| !item.disabled).unwrap_or(0)
}

fn first_top(items: &[CommandItem]) -> usize {
    items.iter().position(|item| !item.disabled).unwrap_or(0)
}

fn last_top(items: &[CommandItem]) -> usize {
    items.iter().rposition(|item| !item.disabled).unwrap_or(0)
}

#[cfg(test)]
#[path = "command_menu_bar/tests.rs"]
mod tests;

#[path = "command_menu_bar/menu_view.rs"]
mod menu_view;

use menu_view::menu_view;
