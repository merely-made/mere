// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Presentation furniture for the existing host-owned workspace.

use crate::el;

/// Optional Instrument presentation over the existing workspace, pane, split
/// and title-bar contracts. Hosts provide Tabard color/font properties and
/// opt in with `instrument-workspace`; this sheet owns no arrangement or data.
/// Mark utility-control regions with `workspace-controls`. Title-bar and pane
/// actions are already admitted regions. Content buttons, including transparent
/// graph node/relation targets, keep their domain's presentation and geometry.
/// A font-limited host sets `--workspace-font-interface` to its admitted default
/// face. It is retained at the end of each family list even when an authored
/// family is unavailable; the host's existing font engine chooses the face.
pub const INSTRUMENT_CSS: &str = include_str!("instrument.css");

/// Independent host-reported pane states. Selection is attention to material;
/// active identifies the working pane. Neither one implies keyboard focus,
/// unsaved changes, or a loading operation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorkspacePaneStatus {
    pub active: bool,
    pub selected: bool,
    pub dirty: bool,
    pub loading: bool,
}

/// A shared pane header with a source/context caption and ordinary host-owned
/// actions. It does not open, select, retain or save anything. The caller keeps
/// the pane's content and its existing stable identity, including when a view
/// changes from a graph occurrence to a tile or a working document.
pub fn workspace_pane_header<State: 'static>(
    title: impl Into<String>,
    context: impl Into<String>,
    status: WorkspacePaneStatus,
    actions: crate::TitleBarSlot<State>,
) -> crate::TitleBarSlot<State> {
    let words: Vec<crate::TitleBarSlot<State>> = [
        (status.active, "active", "Active pane"),
        (status.selected, "selected", "Selected material"),
        (status.dirty, "dirty", "Unsaved changes"),
        (status.loading, "loading", "Loading"),
    ]
    .into_iter()
    .filter(|(present, _, _)| *present)
    .map(|(_, kind, label)| {
        Box::new(
            el("span", label)
                .attr("class", "workspace-pane-state")
                .attr("data-state", kind),
        ) as crate::TitleBarSlot<State>
    })
    .collect();
    Box::new(
        el(
            "header",
            (
                el(
                    "div",
                    (
                        el("h2", title.into()).attr("class", "workspace-pane-title"),
                        el("p", context.into()).attr("class", "workspace-pane-context"),
                    ),
                )
                .attr("class", "workspace-pane-heading"),
                el("div", words).attr("class", "workspace-pane-states"),
                el("div", actions).attr("class", "workspace-pane-actions"),
            ),
        )
        .attr("class", "workspace-pane-header")
        .attr("data-active", if status.active { "true" } else { "false" })
        .attr(
            "data-selected",
            if status.selected { "true" } else { "false" },
        )
        .attr("aria-busy", if status.loading { "true" } else { "false" }),
    )
}

// AUDIT tracked Rust control change
