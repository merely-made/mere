/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The mere view: a mere's sessions and the graph a host shows, as one
//! Cambium component every application embeds (reservoir plan V2b).
//!
//! The host feeds it and acts for it. It fills a [`MereViewModel`], and the
//! view reports what the person asked for as [`MereViewRequest`]s: activate a
//! node, mint, switch, fork, trash or restore a session, a host action, another
//! layout. The host carries each out or declines it, and a declined request
//! comes back as the model's `notice`, shown as the host worded it. The view
//! never reads the host's state and never writes a store.
//!
//! What it shows:
//! - the sessions, each with its state in words and the steps the host offers;
//! - the graph through [`cambium::graph_canvas`]: every node a labelled native
//!   target with a stable `data-key`, its host state in words as well as
//!   paint, and every relation its own cell, marked with its provenance;
//! - the layout the host prefers, laid out through cartography's graph-only
//!   strategies, with positions keyed by node so a switch keeps identity;
//! - the host's action slot, and plain empty, unavailable and building states,
//!   each with the action the host offers.
//!
//! Like [`cambium::graph_canvas`], the graph paints through a custom leaf the
//! host registers under the view's leaf key: [`MereView::paint_leaf`] builds it
//! from the same numbers the view lays out with. [`MERE_VIEW_CSS`] is the
//! structural sheet; colour comes from the host's sheet and palette.

mod layout;
mod model;
mod view;

#[cfg(test)]
mod tests;

pub use layout::{DEFAULT_LAYOUT, lay_out};
pub use model::{
    GraphModel, HostAction, MereViewModel, NodeEntry, NodeState, Provenance, RelationEntry,
    SessionEntry, SessionStep, StatusNote, ViewStatus,
};
pub use view::{
    MERE_VIEW_CSS, MereView, MereViewElement, MereViewEvent, MereViewRequest, MereViewState,
    mere_view,
};
