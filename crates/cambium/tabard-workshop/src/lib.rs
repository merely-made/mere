// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Tabard's appearance workshop: one retained product model and Cambium
//! surface for standalone and embedding hosts.

mod graph;
mod interchange;
mod reader;
mod state;
mod stylesheet;
mod surface;

pub use interchange::{ExportArtifact, ExportFormat};
pub use reader::{READER_LEAF_KEY, ReaderSpecimen};
pub use state::{SeedRole, WorkshopState};
pub use stylesheet::{APPLICATION_SOURCE, PreviewScene, STYLESHEET_LEAF_KEY, StylesheetSpecimen};
pub use surface::{
    WORKSHOP_CODE_SAMPLE, WORKSHOP_CSS, WorkshopView, workshop_view, workshop_view_with_captions,
};

/// The shared component sheets and the workshop frame, in cascade order.
pub fn workshop_stylesheet() -> String {
    format!(
        "{}\n{}\n{}\n{}",
        cambium::TITLE_BAR_CSS,
        cambium::GRAPH_CANVAS_SWATCH_CSS,
        cambium::SYNTAX_HIGHLIGHT_CSS,
        WORKSHOP_CSS
    )
}

pub const GRAPH_LEAF_KEY: u64 = graph::GRAPH_LEAF_KEY;
