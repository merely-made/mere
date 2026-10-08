// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The reader specimen uses Fleece extraction and the shared document canvas.
//! It is a read-only appearance, with no network loading or link activation.

use std::sync::Arc;

use inker::{EngineDocument, inline_text};
use mere_document_lanes::reader::{StaticArticleOutcome, extract_static_article};
use mere_document_lanes::{SmolwebDocument, SmolwebPalette, SmolwebTheme, lower_article};
use netrender::Scene;
use tinct::color_to_hex;

use crate::WorkshopState;

pub const READER_LEAF_KEY: u64 = 0x7461_6272;
pub const READER_SOURCE: &str = include_str!("../fixtures/reader.html");
pub const READER_ADDRESS: &str = "https://garden.example/after-rain";

/// One independently laid-out appearance of the fixed, genuinely extracted
/// article. A palette change preserves the original portable source packet.
pub struct ReaderSpecimen {
    source: Arc<EngineDocument>,
    document: SmolwebDocument,
    palette: SmolwebPalette,
    accessible_name: String,
    revision: u64,
}

impl Default for ReaderSpecimen {
    fn default() -> Self {
        Self::new(SmolwebPalette::for_theme(
            &SmolwebTheme::Light,
            READER_ADDRESS,
        ))
    }
}

impl ReaderSpecimen {
    pub fn new(palette: SmolwebPalette) -> Self {
        let article = match extract_static_article(READER_SOURCE) {
            StaticArticleOutcome::Article(article) => article,
            _ => panic!("the checked-in reader specimen must extract as an article"),
        };
        let source = Arc::new(lower_article(READER_ADDRESS, &article));
        // Reuse Inker's recursive inline traversal and accessible text
        // flattening instead of maintaining a second copy of the prose.
        let text = source
            .walk_inline_spans()
            .into_iter()
            .map(|span| inline_text(std::slice::from_ref(span)))
            .filter(|text| !text.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        let accessible_name = format!("Read-only reader preview. {text}");
        let document = SmolwebDocument::from_shared_document_with_theme(
            source.clone(),
            SmolwebTheme::App(palette.clone()),
        );
        Self {
            source,
            document,
            palette,
            accessible_name,
            revision: 1,
        }
    }

    /// Supply the existing reader palette seam with the actual selected
    /// profile. Links and quotes use the exact surface against which Tinct
    /// derived their contrast; document-lanes owns all role/style mapping.
    pub fn palette_for_state(state: &WorkshopState) -> SmolwebPalette {
        let palette = state.preview_palette();
        let syntax = state.preview_syntax();
        SmolwebPalette {
            bg: color_to_hex(syntax.surface),
            fg: color_to_hex(syntax.emphasis),
            link: color_to_hex(syntax.link),
            quote: color_to_hex(syntax.quote),
            pre_bg: color_to_hex(palette.surface_2),
        }
    }

    /// Invalidate retained layout only when appearance changes. The fixed
    /// source and its semantic text remain stable across every edit/profile.
    pub fn set_palette(&mut self, palette: SmolwebPalette) -> bool {
        if self.palette == palette {
            return false;
        }
        self.document = SmolwebDocument::from_shared_document_with_theme(
            self.source.clone(),
            SmolwebTheme::App(palette.clone()),
        );
        self.palette = palette;
        self.revision = self
            .revision
            .checked_add(1)
            .expect("reader revision exhausted");
        true
    }

    pub fn source_document(&self) -> Arc<EngineDocument> {
        self.source.clone()
    }

    pub fn palette(&self) -> &SmolwebPalette {
        &self.palette
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn accessible_name(&self) -> &str {
        &self.accessible_name
    }

    /// Document-canvas supplies typography, reflow, visible-band clipping,
    /// shaped fonts and scene lowering, exactly as the real reader lane.
    pub fn frame(&mut self, width: u32, height: u32) -> Scene {
        self.document.frame(width.max(1), height.max(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tabard::theme::registry::Mode;

    #[test]
    fn specimen_is_extracted_and_uses_real_shaped_reader_output() {
        let mut reader = ReaderSpecimen::default();
        let source = reader.source_document();
        assert_eq!(source.title.as_deref(), Some("The garden after rain"));
        assert_eq!(
            source.provenance.source_kind.as_deref(),
            Some("genet.reader")
        );
        assert_eq!(
            source.outgoing_links(),
            vec!["https://garden.example/fieldnotes"]
        );
        assert!(
            reader
                .accessible_name()
                .contains("The leaves hold yesterday's weather")
        );
        assert!(!reader.accessible_name().contains("Garden journal"));
        let scene = reader.frame(350, 240);
        assert_eq!((scene.viewport_width, scene.viewport_height), (350, 240));
        assert!(
            scene
                .ops
                .iter()
                .any(|op| matches!(op, netrender::SceneOp::GlyphRun(_)))
        );
    }

    #[test]
    fn mode_changes_preserve_source_and_refresh_the_shared_reader_palette() {
        let mut state = WorkshopState::in_memory();
        let mut reader = ReaderSpecimen::new(ReaderSpecimen::palette_for_state(&state));
        let source = reader.source_document();
        let name = reader.accessible_name().to_owned();
        let first = reader.palette().clone();
        let revision = reader.revision();
        assert!(!reader.set_palette(first.clone()));
        assert_eq!(reader.revision(), revision);
        state.set_mode(Mode::Dark);
        assert!(reader.set_palette(ReaderSpecimen::palette_for_state(&state)));
        assert_ne!(reader.palette().bg, first.bg);
        assert_ne!(reader.palette().fg, first.fg);
        assert_eq!(
            reader.palette().fg,
            color_to_hex(state.preview_syntax().emphasis)
        );
        assert_eq!(
            reader.palette().bg,
            color_to_hex(state.preview_syntax().surface)
        );
        assert!(Arc::ptr_eq(&source, &reader.source_document()));
        assert_eq!(source.as_ref(), reader.source_document().as_ref());
        assert_eq!(reader.accessible_name(), name);
        assert_eq!(reader.revision(), revision + 1);
        assert!(
            reader
                .frame(250, 180)
                .ops
                .iter()
                .any(|op| matches!(op, netrender::SceneOp::GlyphRun(_)))
        );
    }

    #[test]
    fn all_profiles_paint_actual_reader_surface_and_foreground() {
        let mut state = WorkshopState::in_memory();
        let mut reader = ReaderSpecimen::default();
        let channels = |color: tinct::Srgb| {
            [
                f32::from(color.r) / 255.0,
                f32::from(color.g) / 255.0,
                f32::from(color.b) / 255.0,
                1.0,
            ]
        };
        for mode in [Mode::Light, Mode::Dark, Mode::HcLight, Mode::HcDark] {
            state.set_mode(mode);
            reader.set_palette(ReaderSpecimen::palette_for_state(&state));
            let syntax = state.preview_syntax();
            let scene = reader.frame(420, 480);
            assert!(
                matches!(&scene.ops[0], netrender::SceneOp::Rect(rect) if rect.color == channels(syntax.surface))
            );
            assert!(scene.ops.iter().any(|op| matches!(op, netrender::SceneOp::GlyphRun(run) if run.color == channels(syntax.emphasis))));
            assert!(scene.ops.iter().any(|op| matches!(op, netrender::SceneOp::GlyphRun(run) if run.color == channels(syntax.link))));
        }
    }
}
