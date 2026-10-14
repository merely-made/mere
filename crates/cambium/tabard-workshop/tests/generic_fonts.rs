// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Explicit host resources through actual retained Workshop sources.
use netrender::{Scene, SceneOp};
use tabard::theme::registry::Mode;
use tabard_workshop::{ReaderSpecimen, StylesheetSpecimen, WorkshopState};

fn ledger() -> Vec<(String, Vec<u8>)> {
    ["sans-serif", "system-ui", "serif", "monospace"]
        .into_iter()
        .map(|name| {
            (
                name.into(),
                include_bytes!("../../examples/genet_web_smoke/assets/Roboto-Regular.ttf").to_vec(),
            )
        })
        .collect()
}
fn actual_fonts(scene: &Scene) {
    let bytes = include_bytes!("../../examples/genet_web_smoke/assets/Roboto-Regular.ttf");
    assert!(
        scene
            .ops
            .iter()
            .any(|op| matches!(op, SceneOp::GlyphRun(run) if !run.glyphs.is_empty()))
    );
    let fonts: Vec<_> = scene
        .fonts
        .iter()
        .filter(|face| !face.data.as_ref().is_empty())
        .collect();
    assert!(!fonts.is_empty());
    assert!(fonts.iter().all(|face| face.data.as_ref() == bytes));
}
#[test]
fn explicit_generic_resources_survive_actual_reader_and_stylesheet_reconstruction() {
    let mut state = WorkshopState::in_memory();
    let mut reader = ReaderSpecimen::new(ReaderSpecimen::palette_for_state(&state));
    let mut stylesheet = StylesheetSpecimen::default();
    let source = reader.source_document();
    reader.set_generic_fonts(ledger()).unwrap();
    stylesheet.set_generic_fonts(ledger()).unwrap();
    let revisions = (reader.revision(), stylesheet.revision());
    assert!(!reader.set_generic_fonts(ledger()).unwrap());
    assert!(!stylesheet.set_generic_fonts(ledger()).unwrap());
    assert_eq!(revisions, (reader.revision(), stylesheet.revision()));
    for mode in [Mode::Light, Mode::Dark, Mode::HcLight, Mode::HcDark] {
        state.set_mode(mode.clone());
        reader.set_palette(ReaderSpecimen::palette_for_state(&state));
        stylesheet.set_appearance(&mode, &[]);
        actual_fonts(&reader.frame(477, 215));
        actual_fonts(&stylesheet.frame(980, 328));
        assert!(std::sync::Arc::ptr_eq(&source, &reader.source_document()));
    }
    let revisions = (reader.revision(), stylesheet.revision());
    assert!(
        reader
            .set_generic_fonts(vec![("sans-serif".into(), vec![0])])
            .is_err()
    );
    assert!(
        stylesheet
            .set_generic_fonts(vec![("sans-serif".into(), vec![0])])
            .is_err()
    );
    assert_eq!(revisions, (reader.revision(), stylesheet.revision()));
    actual_fonts(&reader.frame(420, 240));
    actual_fonts(&stylesheet.frame(640, 360));
    let mut typography = tabard::Typography::default();
    typography
        .set(
            tabard::FontRole::Reading,
            tabard::ThemeFontStyle {
                family: Some("serif".into()),
                size_px: Some(19.0),
                line_height: Some(1.4),
            },
        )
        .unwrap();
    reader.set_typography(typography.clone()).unwrap();
    stylesheet.set_typography(&typography).unwrap();
    actual_fonts(&reader.frame(477, 215));
    actual_fonts(&stylesheet.frame(980, 328));
}
