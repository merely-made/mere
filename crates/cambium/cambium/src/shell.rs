// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Compact application shell presentation. The host supplies actions, document
//! identity, menu contents and placement. A Map and a Read are projections of
//! host data; this module neither owns a graph nor creates reading objects.

use crate::{AnyView, GenetCtx, GenetElement, OptionalAction, PointerClick, clickable, el};

/// An application-owned shell slot with its ordinary state and action types.
pub type ShellSlot<State, Action = ()> = Box<dyn AnyView<State, Action, GenetCtx, GenetElement>>;

/// Responsive disclosure policy measured after native caption reservations.
/// Width and zoom must use the same coordinate space as the host's layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellDensity {
    Tiny,
    Compact,
    Regular,
    Wide,
}

impl ShellDensity {
    pub fn for_width(width: f32, zoom: f32) -> Self {
        let width = width / zoom.max(0.1);
        if width < 300.0 {
            Self::Tiny
        } else if width < 540.0 {
            Self::Compact
        } else if width <= 700.0 {
            Self::Regular
        } else {
            Self::Wide
        }
    }

    pub fn is_compact(self) -> bool {
        matches!(self, Self::Tiny | Self::Compact)
    }
}

/// Shared shell measurements, independent of application persistence or tiling.
pub struct ShellMetrics;

impl ShellMetrics {
    pub const ACTIVITY_RAIL_WIDTH: f32 = 56.0;
    pub const DOCUMENT_NAV_WIDTH: f32 = 144.0;
    pub const DOCUMENT_ROW_HEIGHT: f32 = 24.0;
    pub const DOCUMENT_RAIL_MAX_WIDTH: f32 = 120.0;

    pub fn toolbar_height(zoom: f32) -> f32 {
        24.0 + 16.0 * zoom
    }
}

/// Visual and accessibility state of one control. A disclosure uses
/// `expanded`; a persistent choice uses `pressed`. Disabled controls cannot
/// receive keyboard focus or invoke their handler, including synthetic clicks.
#[derive(Clone, Copy, Debug, Default)]
pub struct ShellControl {
    pub disabled: bool,
    pub pressed: Option<bool>,
    pub expanded: Option<bool>,
}

pub fn shell_control<State, Action, OA, F>(
    label: impl Into<String>,
    accessible_name: impl Into<String>,
    state: ShellControl,
    handler: F,
) -> ShellSlot<State, Action>
where
    State: 'static,
    Action: 'static,
    OA: OptionalAction<Action> + 'static,
    F: Fn(&mut State, PointerClick) -> OA + 'static,
{
    shell_button(
        Box::new(el("span", label.into())),
        accessible_name.into(),
        "shell-control",
        state,
        handler,
    )
}

fn shell_button<State, Action, OA, F>(
    content: ShellSlot<State, Action>,
    accessible_name: String,
    class: &'static str,
    state: ShellControl,
    handler: F,
) -> ShellSlot<State, Action>
where
    State: 'static,
    Action: 'static,
    OA: OptionalAction<Action> + 'static,
    F: Fn(&mut State, PointerClick) -> OA + 'static,
{
    let mut view = el::<_, State, Action>("button", content)
        .attr("class", class)
        .attr("aria-label", accessible_name)
        .attr(
            "aria-disabled",
            if state.disabled { "true" } else { "false" },
        );
    if let Some(pressed) = state.pressed {
        view = view.attr("aria-pressed", if pressed { "true" } else { "false" });
    }
    if let Some(expanded) = state.expanded {
        // Popup content may be a group of ordinary buttons; do not promise
        // menuitem/roving focus semantics that the host has not supplied.
        view = view.attr("aria-expanded", if expanded { "true" } else { "false" });
    }
    if state.disabled {
        // An inert element rather than just a painted disabled state. The
        // toolkit's focusable wrapper also checks this attribute.
        view = view.attr("disabled", "");
    }
    if state.disabled {
        Box::new(view)
    } else {
        Box::new(clickable(view, handler))
    }
}

/// A compact activity control with a visible caption and a decorative mark.
/// The host owns its action and selected/expanded state, just as in a toolbar.
pub fn shell_activity_control<State, Action, OA, F>(
    mark: impl Into<String>,
    label: impl Into<String>,
    accessible_name: impl Into<String>,
    state: ShellControl,
    handler: F,
) -> ShellSlot<State, Action>
where
    State: 'static,
    Action: 'static,
    OA: OptionalAction<Action> + 'static,
    F: Fn(&mut State, PointerClick) -> OA + 'static,
{
    shell_button(
        Box::new(el("span", (
            el("span", mark.into()).attr("class", "shell-activity-mark").attr("aria-hidden", "true"),
            el("span", label.into()).attr("class", "shell-activity-caption"),
        )).attr("class", "shell-activity-content")),
        accessible_name.into(),
        "shell-control shell-activity-control",
        state,
        handler,
    )
}

/// A vertical shell navigator. Projections and tools have independent states;
/// the rail does not turn multiple visible panes into one selected tab.
pub fn shell_activity_rail<State: 'static, Action: 'static>(
    projections: Vec<ShellSlot<State, Action>>,
    tools: Vec<ShellSlot<State, Action>>,
    utilities: Vec<ShellSlot<State, Action>>,
) -> ShellSlot<State, Action> {
    Box::new(el("nav", (
        el("div", projections).attr("class", "shell-activity-projections"),
        el("div", tools).attr("class", "shell-activity-tools"),
        el("div", utilities).attr("class", "shell-activity-utilities"),
    )).attr("class", "cambium-activity-rail").attr("aria-label", "Views and tools"))
}

/// Documents grouped by their existing tile stacks. Each group can have its
/// own active document. Ordinary buttons preserve native keyboard activation;
/// the host activates the existing tile rather than flattening its layout.
pub fn shell_document_rail<State: 'static, Action: 'static>(
    groups: Vec<(String, Vec<ShellSlot<State, Action>>)>,
) -> ShellSlot<State, Action> {
    let groups = groups.into_iter().map(|(label, tabs)| {
        el("div", (el("div", label.clone()).attr("class", "shell-document-group-title"), tabs))
            .attr("class", "shell-document-group")
            .attr("role", "group").attr("aria-label", label)
    }).collect::<Vec<_>>();
    Box::new(el("nav", groups).attr("class", "cambium-shell-documents")
        .attr("aria-label", "Reading tiles"))
}

/// One row with a flexible identity/address slot between application-owned
/// controls. Caption reservations and top/bottom placement remain host-owned.
pub fn shell_toolbar<State: 'static, Action: 'static>(
    label: impl Into<String>,
    leading: Vec<ShellSlot<State, Action>>,
    location: ShellSlot<State, Action>,
    trailing: Vec<ShellSlot<State, Action>>,
) -> ShellSlot<State, Action> {
    Box::new(
        el(
            "div",
            (
                el("div", leading).attr("class", "shell-leading"),
                el("div", location).attr("class", "shell-location"),
                el("div", trailing).attr("class", "shell-trailing"),
            ),
        )
        .attr("class", "cambium-shell-toolbar")
        .attr("role", "group")
        .attr("aria-label", label.into()),
    )
}

/// Popup contents with ordinary keyboard-focusable controls. Hosts own the
/// disclosure state, dismissal policy and viewport-relative anchor. A popup
/// uses group semantics; a true ARIA menu should use `command_menu_bar`.
pub fn shell_popup<State: 'static, Action: 'static>(
    label: impl Into<String>,
    rows: Vec<ShellSlot<State, Action>>,
) -> ShellSlot<State, Action> {
    Box::new(
        el("div", rows)
            .attr("class", "shell-popup")
            .attr("role", "group")
            .attr("aria-label", label.into()),
    )
}

/// Render Map/Read as two persistent projection choices, or one alternate
/// projection button in a constrained toolbar. Hosts decide what switching
/// means and supply the two actions; no graph selection is mirrored here.
pub fn shell_projection_switch<State, Action, OA, F>(
    map_active: bool,
    compact: bool,
    activate: F,
) -> ShellSlot<State, Action>
where
    State: 'static,
    Action: 'static,
    OA: OptionalAction<Action> + 'static,
    F: Fn(&mut State, bool) -> OA + Clone + 'static,
{
    let mut controls = Vec::new();
    for (label, map) in [("Map", true), ("Read", false)] {
        if compact && map == map_active {
            continue;
        }
        let activate = activate.clone();
        controls.push(shell_control(
            label,
            label,
            ShellControl {
                pressed: (!compact).then_some(map_active == map),
                ..Default::default()
            },
            move |state, _| activate(state, map),
        ));
    }
    Box::new(
        el("div", controls)
            .attr("class", "shell-projections")
            .attr("role", "group")
            .attr("aria-label", "Document projection"),
    )
}

/// Presentation only. Hosts set theme variables and can scale the row through
/// `--shell-control-size`, `--shell-row-height` and `--shell-font-size`.
pub const SHELL_CSS: &str = include_str!("shell.css");

#[cfg(test)]
mod tests;
