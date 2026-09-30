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
        let popup_path = if compact { Vec::new() } else { vec![index] };
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

fn menu_view(
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
    for (index, item) in menu.iter().enumerate() {
        let mut item_path = path.to_vec();
        item_path.push(index);
        let submenu = (state.open_path.len() > path.len()
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
            if depth == 0 {
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
mod tests {
    use super::*;
    use crate::{DomHandle, GenetAppRunner};
    use genet_scripted_dom::{NodeId, ScriptedDom};
    use layout_dom_api::{LayoutDom, LocalName, Namespace};
    use std::{cell::RefCell, rc::Rc};

    fn items() -> Vec<CommandItem> {
        vec![
            CommandItem::new("File").with_id("file").with_children([
                CommandItem::new("Open")
                    .with_id("open")
                    .with_shortcut("Ctrl+O"),
                CommandItem::new("Export").with_id("export").with_children([
                    CommandItem::new("Plain text").with_id("plain"),
                    CommandItem::new("PDF")
                        .with_id("pdf")
                        .disabled_because("PDF unavailable"),
                ]),
                CommandItem::new("Close")
                    .with_id("close")
                    .disabled_because("Keep one document open"),
            ]),
            CommandItem::new("Edit")
                .with_id("edit")
                .with_children([CommandItem::new("Undo").with_id("undo")]),
        ]
    }

    fn attr<'a>(dom: &'a ScriptedDom, node: NodeId, name: &str) -> Option<&'a str> {
        dom.attribute(node, &Namespace::from(""), &LocalName::from(name))
    }

    fn find(dom: &ScriptedDom, root: NodeId, name: &str, value: &str) -> Option<NodeId> {
        if attr(dom, root, name) == Some(value) {
            return Some(root);
        }
        dom.dom_children(root)
            .find_map(|child| find(dom, child, name, value))
    }

    #[test]
    fn top_roving_and_command_activation_keep_original_path() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::new(
            dom.clone(),
            |state: &CommandMenuBarState| command_menu_bar(state, &items(), false),
            CommandMenuBarState::default(),
        );
        assert_eq!(attr(&dom.borrow(), runner.root(), "role"), Some("menubar"));
        runner.set_focus(Some(runner.root()));
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowRight)));
        assert_eq!(runner.state().active, 1);
        let edit = find(&dom.borrow(), runner.root(), "data-key", "edit").expect("Edit item");
        assert_eq!(attr(&dom.borrow(), edit, "tabindex"), Some("0"));
        assert_eq!(runner.focus(), Some(edit));
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Home)));
        assert_eq!(runner.state().active, 0);
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowDown)));
        let row = find(&dom.borrow(), runner.root(), "data-key", "open").expect("open row");
        assert_eq!(attr(&dom.borrow(), row, "role"), Some("menuitem"));
        assert_eq!(
            runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter))),
            [CommandEvent::Activate(vec![0, 0])]
        );
    }

    #[test]
    fn submenu_disabled_item_is_described_and_inert() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::new(
            dom.clone(),
            |state: &CommandMenuBarState| command_menu_bar(state, &items(), false),
            CommandMenuBarState::default(),
        );
        runner.set_focus(Some(runner.root()));
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowDown)));
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowDown)));
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowRight)));
        assert_eq!(runner.state().open_path, vec![0, 1]);
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowDown)));
        let row = find(&dom.borrow(), runner.root(), "data-key", "pdf").expect("PDF row");
        assert_eq!(
            attr(&dom.borrow(), row, "aria-description"),
            Some("PDF unavailable")
        );
        assert!(
            runner
                .dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter)))
                .is_empty()
        );
        assert!(
            runner
                .dispatch_click(row, PointerClick::at((1.0, 1.0)))
                .is_empty()
        );
        assert!(runner.state().open);
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Home)));
        assert_eq!(
            runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter))),
            [CommandEvent::Activate(vec![0, 1, 0])]
        );
    }

    #[test]
    fn compact_overflow_is_one_button_and_keeps_paths() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::new(
            dom.clone(),
            |state: &CommandMenuBarState| command_menu_bar(state, &items(), true),
            CommandMenuBarState::default(),
        );
        let button = find(&dom.borrow(), runner.root(), "data-key", "menu").expect("Menu button");
        assert!(find(&dom.borrow(), runner.root(), "data-key", "file").is_none());
        runner.dispatch_click(button, PointerClick::at((1.0, 1.0)));
        assert!(find(&dom.borrow(), runner.root(), "data-key", "file").is_some());
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter)));
        assert_eq!(runner.state().open_path, vec![0]);
        assert_eq!(
            runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter))),
            [CommandEvent::Activate(vec![0, 0])]
        );
    }

    #[test]
    fn entry_keys_and_escape() {
        let mut state = CommandMenuBarState::default();
        assert!(state.handle_entry_key(&KeyEvent::new(Key::Named(NamedKey::F10))));
        assert!(state.handle_entry_key(&KeyEvent::new(Key::Named(NamedKey::Alt))));
        assert!(!state.handle_entry_key(&KeyEvent::new(Key::Character("a".into()))));
        assert!(!state.handle_entry_key(&KeyEvent::with_mods(
            Key::Named(NamedKey::F10),
            crate::Modifiers {
                shift: true,
                ..Default::default()
            }
        )));
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::new(
            dom,
            |state: &CommandMenuBarState| command_menu_bar(state, &items(), false),
            state,
        );
        runner.set_focus(Some(runner.root()));
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowDown)));
        assert_eq!(
            runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Escape))),
            [CommandEvent::Dismiss]
        );
        assert!(!runner.state().open);
    }

    #[test]
    fn entry_request_rearms_after_focus_leaves_the_bar() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::new(
            dom.clone(),
            |state: &CommandMenuBarState| command_menu_bar(state, &items(), false),
            CommandMenuBarState::default(),
        );
        let file = find(&dom.borrow(), runner.root(), "data-key", "file").expect("File item");
        runner.update(|state| state.enter());
        assert_eq!(runner.focus(), Some(file));
        runner.set_focus(None);
        assert!(
            !runner.state().focus_active,
            "blur resets the retained edge"
        );
        runner.update(|state| state.enter());
        assert_eq!(runner.focus(), Some(file), "F10 can enter again");

        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowDown)));
        assert!(runner.state().open);
        runner.set_focus(None);
        assert!(!runner.state().open, "blur dismisses the open popup");
        assert!(!runner.state().focus_active);
        runner.update(|state| state.enter());
        assert_eq!(runner.focus(), Some(file));
    }

    #[test]
    fn width_mode_change_normalizes_an_open_menu() {
        let items = items();
        let mut state = CommandMenuBarState::default();
        state.show(&items, true);
        assert!(state.open_path.is_empty());
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::new(
            dom.clone(),
            move |state: &CommandMenuBarState| command_menu_bar(state, &items, false),
            state,
        );
        // The wide view projects the stale compact disclosure closed. The
        // next interaction commits wide mode before opening a group.
        assert!(find(&dom.borrow(), runner.root(), "data-key", "open").is_none());
        runner.set_focus(Some(runner.root()));
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowDown)));
        assert_eq!(runner.state().open_path, vec![0]);
        assert!(find(&dom.borrow(), runner.root(), "data-key", "open").is_some());
    }

    #[test]
    fn disabled_top_items_do_not_take_home_end_tabstop() {
        let items = vec![
            CommandItem::new("File").disabled_because("Unavailable"),
            CommandItem::new("Edit").with_children([CommandItem::new("Undo")]),
            CommandItem::new("View").disabled(true),
        ];
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let mut runner = GenetAppRunner::new(
            dom.clone(),
            move |state: &CommandMenuBarState| command_menu_bar(state, &items, false),
            CommandMenuBarState::default(),
        );
        let edit = find(&dom.borrow(), runner.root(), "data-key", "Edit").expect("Edit");
        assert_eq!(attr(&dom.borrow(), edit, "tabindex"), Some("0"));
        runner.set_focus(Some(runner.root()));
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Home)));
        assert_eq!(runner.state().active, 1);
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::End)));
        assert_eq!(runner.state().active, 1);
        assert_eq!(runner.focus(), Some(edit));
    }
}
