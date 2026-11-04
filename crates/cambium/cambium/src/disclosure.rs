/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Shared disclosure behavior, plus accordion and recursive tree compositions.

use std::hash::Hash;

use meristem::{AnyView, ViewSequence};

use crate::{
    El, GenetCtx, GenetElement, Key, Keyed, NamedKey, View, el, focusable_if, on_click, on_key,
    request_focus,
};

/// Controlled state for a single disclosure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisclosureState {
    pub id: String,
    pub label: String,
    pub expanded: bool,
    pub disabled: bool,
}

impl Default for DisclosureState {
    fn default() -> Self {
        Self::new("cambium-disclosure", "Details")
    }
}

/// Render a labelled disclosure button and its controlled content.
///
/// Explicit inputs in, caller-state handler out (the grid's house style): the
/// [`DisclosureState`] is an input the caller owns and stores wherever it
/// likes, and the toggle is reported back through `on_toggle`. The panel's
/// content is therefore a view over the *application's* state, so a row inside
/// the panel calls the application's own verbs. Where the application state is
/// the disclosure state itself, the handler is [`DisclosureState::toggle`].
pub fn disclosure<State, Action, Content, Toggle>(
    state: &DisclosureState,
    content: Content,
    on_toggle: Toggle,
) -> impl View<State, Action, GenetCtx, Element = GenetElement> + use<State, Action, Content, Toggle>
where
    State: 'static,
    Action: 'static,
    Content: ViewSequence<State, Action, GenetCtx, GenetElement>,
    Toggle: Fn(&mut State) + 'static,
{
    disclosure_with(state, state.label.clone(), content, on_toggle)
}

/// Render a disclosure with application-defined trigger content.
pub fn disclosure_with<State, Action, Trigger, Content, Toggle>(
    state: &DisclosureState,
    trigger: Trigger,
    content: Content,
    on_toggle: Toggle,
) -> impl View<State, Action, GenetCtx, Element = GenetElement>
+ use<State, Action, Trigger, Content, Toggle>
where
    State: 'static,
    Action: 'static,
    Trigger: ViewSequence<State, Action, GenetCtx, GenetElement>,
    Content: ViewSequence<State, Action, GenetCtx, GenetElement>,
    Toggle: Fn(&mut State) + 'static,
{
    let trigger_id = format!("{}-trigger", state.id);
    let panel_id = format!("{}-panel", state.id);
    let control = el::<_, State, Action>("button", trigger)
        .attr("id", trigger_id.clone())
        .attr("class", "disclosure-trigger")
        .attr("type", "button");
    // `disabled` is read off the state as passed, so the widget still refuses
    // the toggle while disabled no matter what the caller's handler does.
    let control = disclosure_control(
        control,
        Some(panel_id.clone()),
        state.expanded,
        state.disabled,
        true,
        false,
        on_toggle,
    );
    let mut panel = el::<_, State, Action>("div", content)
        .attr("id", panel_id)
        .attr("class", "disclosure-panel")
        .attr("aria-labelledby", trigger_id);
    if !state.expanded {
        panel = panel.attr("hidden", "true");
    }
    el::<_, State, Action>("div", (control, panel))
        .attr("id", state.id.clone())
        .attr("class", "disclosure")
}

/// A disclosure's trigger alone, for a host that keeps the disclosed content
/// elsewhere in its own tree (a toolbar button for a region further down, or
/// content it builds only while open). The button announces `aria-expanded` and
/// toggles through `on_toggle`; the label may change with the state ("Show
/// …" / "Hide …"). Where the content follows the trigger directly, prefer
/// [`disclosure`].
pub fn disclosure_trigger<State, Action, Label, Toggle>(
    label: Label,
    expanded: bool,
    on_toggle: Toggle,
) -> impl View<State, Action, GenetCtx, Element = GenetElement> + use<State, Action, Label, Toggle>
where
    State: 'static,
    Action: 'static,
    Label: ViewSequence<State, Action, GenetCtx, GenetElement>,
    Toggle: Fn(&mut State) + 'static,
{
    let control = el::<_, State, Action>("button", label)
        .attr("class", "disclosure-trigger")
        .attr("type", "button")
        .attr("aria-expanded", if expanded { "true" } else { "false" });
    disclosure_control(control, None, expanded, false, true, false, on_toggle)
}

/// Whether an accordion permits one or many expanded panels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AccordionMode {
    Single,
    #[default]
    Multiple,
}

/// Semantic and behavior settings for an [`accordion`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccordionConfig {
    pub heading_level: u8,
    pub panel_regions: bool,
}

impl Default for AccordionConfig {
    fn default() -> Self {
        Self {
            heading_level: 3,
            panel_regions: true,
        }
    }
}

/// Application-owned accordion state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccordionState<Id> {
    pub id: String,
    pub label: String,
    pub expanded: Vec<Id>,
    pub mode: AccordionMode,
    pub collapsible: bool,
}

impl<Id> Default for AccordionState<Id> {
    fn default() -> Self {
        Self::new()
    }
}

/// One accordion header and its default text body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccordionItem<Id> {
    pub id: Id,
    pub dom_id: String,
    pub label: String,
    pub body: String,
}

/// Render an accordion with text panel bodies.
///
/// Same shape as [`disclosure`]: the [`AccordionState`] is an input and each
/// header reports its item's identity through `on_toggle`. Where the
/// application state is the accordion state itself, the handler is
/// [`AccordionState::toggle`].
pub fn accordion<State, Action, Id, Toggle>(
    state: &AccordionState<Id>,
    items: &[AccordionItem<Id>],
    config: AccordionConfig,
    on_toggle: Toggle,
) -> impl View<State, Action, GenetCtx, Element = GenetElement> + use<State, Action, Id, Toggle>
where
    State: 'static,
    Action: 'static,
    Id: Clone + Eq + Hash + 'static,
    Toggle: Fn(&mut State, Id) + Clone + 'static,
{
    accordion_with(state, items, config, |item| item.body.clone(), on_toggle)
}

/// Render an accordion with application-defined panel content.
pub fn accordion_with<State, Action, Id, Content, Render, Toggle>(
    state: &AccordionState<Id>,
    items: &[AccordionItem<Id>],
    config: AccordionConfig,
    render: Render,
    on_toggle: Toggle,
) -> impl View<State, Action, GenetCtx, Element = GenetElement>
+ use<State, Action, Id, Content, Render, Toggle>
where
    State: 'static,
    Action: 'static,
    Id: Clone + Eq + Hash + 'static,
    Content: ViewSequence<State, Action, GenetCtx, GenetElement>,
    Render: Fn(&AccordionItem<Id>) -> Content,
    Toggle: Fn(&mut State, Id) + Clone + 'static,
{
    let heading_level = config.heading_level.clamp(1, 6).to_string();
    let children: Vec<_> = items
        .iter()
        .cloned()
        .map(|item| {
            let expanded = state.is_expanded(&item.id);
            let trigger_id = format!("{}-item-{}-trigger", state.id, sanitize(&item.dom_id));
            let panel_id = format!("{}-item-{}-panel", state.id, sanitize(&item.dom_id));
            let disabled = expanded && state.mode == AccordionMode::Single && !state.collapsible;
            let control = el::<_, State, Action>("button", item.label.clone())
                .attr("id", trigger_id.clone())
                .attr("class", "accordion-trigger")
                .attr("type", "button");
            let toggle_id = item.id.clone();
            let on_toggle = on_toggle.clone();
            let control = disclosure_control(
                control,
                Some(panel_id.clone()),
                expanded,
                disabled,
                true,
                false,
                move |state: &mut State| on_toggle(state, toggle_id.clone()),
            );
            let heading = el::<_, State, Action>("div", control)
                .attr("class", "accordion-heading")
                .attr("role", "heading")
                .attr("aria-level", heading_level.clone());
            let mut panel = el::<_, State, Action>("div", render(&item))
                .attr("id", panel_id)
                .attr("class", "accordion-panel")
                .attr("aria-labelledby", trigger_id);
            if config.panel_regions {
                panel = panel.attr("role", "region");
            }
            if !expanded {
                panel = panel.attr("hidden", "true");
            }
            let section =
                el::<_, State, Action>("section", (heading, panel)).attr("class", "accordion-item");
            (item.id, section)
        })
        .collect();

    el::<_, State, Action>("div", Keyed::new(children))
        .attr("id", state.id.clone())
        .attr("class", "accordion")
        .attr("role", "group")
        .attr("aria-label", state.label.clone())
}

/// One node in a [`tree_view`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeItem<Id> {
    pub id: Id,
    pub dom_id: String,
    pub label: String,
    pub children: Vec<TreeItem<Id>>,
}

/// Whether moving tree focus also changes selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TreeSelectionMode {
    Explicit,
    #[default]
    FollowsFocus,
}

/// Roving focus, expansion, and selection state for a recursive tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeState<Id> {
    pub id: String,
    pub label: String,
    pub expanded: Vec<Id>,
    pub selection_mode: TreeSelectionMode,
    active: Option<Id>,
    selected: Option<Id>,
    focus_active: bool,
}

impl<Id> Default for TreeState<Id> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
struct VisibleNode<Id> {
    id: Id,
    label: String,
    parent: Option<Id>,
    has_children: bool,
    expanded: bool,
}

type TreeNodeView<Id> = Box<dyn AnyView<TreeState<Id>, (), GenetCtx, GenetElement>>;

/// Render a recursive, single-select tree with selection following focus.
pub fn tree_view<Id>(
    state: &mut TreeState<Id>,
    items: &[TreeItem<Id>],
) -> impl View<TreeState<Id>, (), GenetCtx, Element = GenetElement> + use<Id>
where
    Id: Clone + Eq + Hash + 'static,
{
    state.reconcile(items);
    let visible = visible_nodes(items, &state.expanded);
    let nodes = tree_nodes(items, state, 1);
    let root = el::<_, TreeState<Id>, ()>("ul", nodes)
        .attr("id", state.id.clone())
        .attr("class", "tree-view")
        .attr("role", "tree")
        .attr("aria-label", state.label.clone());

    on_key(root, move |state: &mut TreeState<Id>, event| {
        let Some(current) = state
            .active
            .as_ref()
            .and_then(|active| visible.iter().position(|node| &node.id == active))
        else {
            return;
        };
        let node = &visible[current];
        let destination = match &event.key {
            Key::Named(NamedKey::ArrowDown) => Some((current + 1).min(visible.len() - 1)),
            Key::Named(NamedKey::ArrowUp) => Some(current.saturating_sub(1)),
            Key::Named(NamedKey::Home) => Some(0),
            Key::Named(NamedKey::End) => Some(visible.len() - 1),
            Key::Named(NamedKey::ArrowRight) if node.has_children && !node.expanded => {
                state.toggle(node.id.clone());
                None
            },
            Key::Named(NamedKey::ArrowRight) if node.has_children => visible
                .get(current + 1)
                .filter(|child| child.parent.as_ref() == Some(&node.id))
                .map(|_| current + 1),
            Key::Named(NamedKey::ArrowLeft) if node.has_children && node.expanded => {
                state.toggle(node.id.clone());
                None
            },
            Key::Named(NamedKey::ArrowLeft) => node
                .parent
                .as_ref()
                .and_then(|parent| visible.iter().position(|candidate| &candidate.id == parent)),
            Key::Named(NamedKey::Enter) if node.has_children => {
                state.activate(node.id.clone());
                state.toggle(node.id.clone());
                None
            },
            Key::Named(NamedKey::Enter | NamedKey::Space) => {
                state.activate(node.id.clone());
                None
            },
            Key::Character(text) if !text.is_empty() => {
                let query = text.to_lowercase();
                (1..=visible.len())
                    .map(|offset| (current + offset) % visible.len())
                    .find(|index| visible[*index].label.to_lowercase().starts_with(&query))
            },
            _ => return,
        };
        if let Some(destination) = destination {
            state.move_active(visible[destination].id.clone());
        }
        event.prevent_default();
    })
    .focusable(false)
}

fn tree_nodes<Id>(
    items: &[TreeItem<Id>],
    state: &TreeState<Id>,
    level: usize,
) -> Vec<TreeNodeView<Id>>
where
    Id: Clone + Eq + Hash + 'static,
{
    let set_size = items.len();
    items
        .iter()
        .cloned()
        .enumerate()
        .map(|(position, item)| {
            let has_children = !item.children.is_empty();
            let expanded = has_children && state.is_expanded(&item.id);
            let active = state.active.as_ref() == Some(&item.id);
            let selected = state.selected.as_ref() == Some(&item.id);
            let item_id = format!("{}-item-{}", state.id, sanitize(&item.dom_id));
            let group_id = format!("{item_id}-group");
            let children = tree_nodes(&item.children, state, level + 1);
            let group = has_children.then(|| {
                let mut group = el::<_, TreeState<Id>, ()>("ul", children)
                    .attr("id", group_id.clone())
                    .attr("class", "tree-group")
                    .attr("role", "group");
                if !expanded {
                    group = group.attr("hidden", "true");
                }
                group
            });
            let control = el::<_, TreeState<Id>, ()>(
                "li",
                (
                    el::<_, TreeState<Id>, ()>("span", item.label.clone())
                        .attr("class", "tree-label"),
                    group,
                ),
            )
            .attr("id", item_id)
            .attr(
                "class",
                if active {
                    "tree-item active"
                } else {
                    "tree-item"
                },
            )
            .attr("role", "treeitem")
            .attr("tabindex", if active { "0" } else { "-1" })
            .attr("aria-level", level.to_string())
            .attr("aria-posinset", (position + 1).to_string())
            .attr("aria-setsize", set_size.to_string())
            .attr("aria-selected", if selected { "true" } else { "false" });
            let toggle_id = item.id.clone();
            let control = disclosure_control(
                control,
                has_children.then_some(group_id),
                expanded,
                false,
                active,
                state.focus_active && active,
                move |state: &mut TreeState<Id>| {
                    state.activate(toggle_id.clone());
                    if has_children {
                        state.toggle(toggle_id.clone());
                    }
                },
            );
            Box::new(control) as TreeNodeView<Id>
        })
        .collect()
}

fn disclosure_control<State, Action, Seq, Toggle>(
    mut control: El<Seq, State, Action>,
    panel_id: Option<String>,
    expanded: bool,
    disabled: bool,
    is_focusable: bool,
    focus_requested: bool,
    on_toggle: Toggle,
) -> impl View<State, Action, GenetCtx, Element = GenetElement> + use<State, Action, Seq, Toggle>
where
    State: 'static,
    Action: 'static,
    Seq: ViewSequence<State, Action, GenetCtx, GenetElement>,
    Toggle: Fn(&mut State) + 'static,
{
    control = control.attr("data-disclosure-control", "true");
    if let Some(panel_id) = panel_id {
        control = control
            .attr("aria-controls", panel_id)
            .attr("aria-expanded", if expanded { "true" } else { "false" });
    }
    if disabled {
        control = control.attr("aria-disabled", "true");
    }
    request_focus(
        focusable_if(
            on_click(control, move |state, event| {
                event.stop_propagation();
                if !disabled {
                    on_toggle(state);
                }
            }),
            is_focusable,
        ),
        focus_requested,
    )
}

fn collect_all<Id: Clone>(items: &[TreeItem<Id>], out: &mut Vec<Id>) {
    for item in items {
        out.push(item.id.clone());
        collect_all(&item.children, out);
    }
}

fn visible_nodes<Id>(items: &[TreeItem<Id>], expanded: &[Id]) -> Vec<VisibleNode<Id>>
where
    Id: Clone + Eq,
{
    fn collect<Id>(
        items: &[TreeItem<Id>],
        expanded: &[Id],
        parent: Option<&Id>,
        out: &mut Vec<VisibleNode<Id>>,
    ) where
        Id: Clone + Eq,
    {
        for item in items {
            let is_expanded = expanded.contains(&item.id);
            out.push(VisibleNode {
                id: item.id.clone(),
                label: item.label.clone(),
                parent: parent.cloned(),
                has_children: !item.children.is_empty(),
                expanded: is_expanded,
            });
            if is_expanded {
                collect(&item.children, expanded, Some(&item.id), out);
            }
        }
    }

    let mut out = Vec::new();
    collect(items, expanded, None, &mut out);
    out
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "disclosure/tests.rs"]
mod tests;

#[path = "disclosure/state_ops.rs"]
mod state_ops;
