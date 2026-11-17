// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Chrome surface identifiers shared with host UX events.
//!
//! Hosts own accessibility instrumentation for their rendered surfaces.

use serde::{Deserialize, Serialize};

/// Identifier for a chrome surface that emits UX events. Adding a new
/// surface (e.g., a future "Inspector" pane) is one new variant here
/// and one new emission site at the surface's open/dismiss seams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SurfaceId {
    Omnibar,
    CommandPalette,
    NodeFinder,
    ContextMenu,
    ConfirmDialog,
    /// Modal that prompts the user for a URL when an action like
    /// `NodeNew` fires from a non-omnibar surface (palette, context
    /// menu, programmatic).
    NodeCreate,
    /// Modal that prompts the user for a new label when the
    /// `FrameRename` action fires.
    FrameRename,
    StatusBar,
    TreeSpine,
    NavigatorHost,
    /// A tile pane (tile-tabs + body). The variant is the granular
    /// surface; per-pane discrimination (PaneId) is host-side state
    /// and not carried in the `UxEvent` payload today — extending
    /// `UxEvent::SurfaceOpened` with a target field is queued as a
    /// future enrichment when downstream observers need it.
    TilePane,
    /// A canvas pane. Same per-pane discrimination caveat as
    /// [`Self::TilePane`].
    CanvasPane,
    /// The canvas base layer (empty Frame fallback).
    BaseLayer,
    /// The graph roster pane: node manifest/list for the current graph.
    RosterPane,
    /// The inspector pane: selected object provenance, trust, parse diagnostics,
    /// document structure, local cache state, and lineage.
    InspectorPane,
    /// The steward pane: live async operations and actionable actor/job state.
    StewardPane,
    /// The gloss pane: outline, swatch, and subgraph navigation surface.
    GlossPane,
    /// The apparatus pane: system diagnostics, settings, probes, and a11y status.
    ApparatusPane,
    /// The comms pane: mail/cabal conversations.
    CommsPane,
    /// The tiled workbench pane.
    WorkbenchPane,
}
