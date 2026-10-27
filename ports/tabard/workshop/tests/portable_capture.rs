// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

#![cfg(all(feature = "native-host", not(target_arch = "wasm32")))]

use cambium_genet_winit_host::{Harness, WindowCommands};
use paint_list_render::bundle::{
    PortablePaintCapture, ResourceAlpha, ResourceEncoding, scene_digest, translate_capture,
    validate_capture,
};
use tabard::theme::registry::Mode;
use tabard_workshop::{
    READER_LEAF_KEY, STYLESHEET_LEAF_KEY, WorkshopState,
    native_host::{WorkshopLogic, native_hooks, native_init},
};

type WorkshopHarness = Harness<WorkshopState, WorkshopLogic, tabard_workshop::WorkshopView>;

fn state_receipt(host: &WorkshopHarness) -> serde_json::Value {
    serde_json::json!({
        "authored_theme": serde_json::from_str::<serde_json::Value>(&tabard::portable::theme_json(host.state().draft_theme()).unwrap()).unwrap(),
        "preview_mode": host.state().mode_key(),
        "dirty": host.state().is_dirty(),
    })
}

fn capture_case(host: &mut WorkshopHarness, width: f32, case: &str) -> PortablePaintCapture {
    let before = state_receipt(host);
    let capture = host
        .capture_portable()
        .expect("real production portable capture");
    let after = state_receipt(host);
    assert_eq!(
        before, after,
        "capture preserves application authoring and mode state"
    );
    if let Some(directory) = std::env::var_os("TABARD_PORTABLE_CAPTURE_OUTPUT") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).unwrap();
        let stem = format!("workshop-{}-{case}", width as u32);
        let bytes = postcard::to_allocvec(&capture).unwrap();
        std::fs::write(directory.join(format!("{stem}.postcard")), &bytes).unwrap();
        let translated = translate_capture(&capture).unwrap();
        let producers: Vec<_> = capture.producers.iter().map(|source| serde_json::json!({
            "texture_key": source.texture_key, "content_generation": source.content_generation,
            "source_identity": source.source_identity, "source_revision": source.source_revision,
            "logical_size": source.logical_size, "physical_size": source.physical_size,
            "layout_scale": source.layout_scale, "alpha": source.alpha, "encoding": source.encoding,
            "scene_ops": source.scene.ops.len(), "scene_fonts": source.scene.fonts.len(),
            "scene_digest": scene_digest(&source.scene).unwrap(),
        })).collect();
        let receipt = serde_json::json!({
            "schema": "tabard-production-portable-capture-v1", "case": case,
            "authored_window_width": width, "viewport": [capture.envelope.viewport.width, capture.envelope.viewport.height],
            "commands": capture.envelope.commands.len(), "fonts": capture.envelope.fonts.len(),
            "images": capture.envelope.images.len(), "producers": producers,
            "external_draws": translated.external_textures.len(), "shadow_masks": translated.box_shadow_masks.len(),
            "capture_bytes": bytes.len(), "capture_digest": paint_list_render::bundle::capture_digest(&capture).unwrap(),
            "state_before": before, "state_after": after, "state_unchanged": true,
            "qualification": "CPU source capture and admission only",
        });
        std::fs::write(
            directory.join(format!("{stem}.json")),
            serde_json::to_vec_pretty(&receipt).unwrap(),
        )
        .unwrap();
    }
    capture
}

fn assert_complete(capture: &PortablePaintCapture) {
    validate_capture(capture).expect("intact full workshop source ledger");
    let translated = translate_capture(capture).expect("production authored commands translate");
    assert_eq!(translated.external_textures.len(), 2);
    assert!(
        !capture.envelope.fonts.is_empty(),
        "retain real host font resources"
    );
    assert!(
        capture.envelope.commands.len() > 100,
        "capture the entire workshop"
    );
    assert_eq!(capture.producers.len(), 2);
    for source in &capture.producers {
        assert!(matches!(
            source.texture_key,
            READER_LEAF_KEY | STYLESHEET_LEAF_KEY
        ));
        assert_ne!(source.source_identity, 0);
        assert_ne!(source.content_generation, 0);
        assert_eq!(source.alpha, ResourceAlpha::Straight);
        assert_eq!(source.encoding, ResourceEncoding::Srgb);
        assert_eq!(
            source.physical_size,
            source
                .logical_size
                .map(|value| (f64::from(value) * f64::from(source.layout_scale)).ceil() as u32)
        );
        assert_eq!(
            [source.scene.viewport_width, source.scene.viewport_height],
            source
                .logical_size
                .map(|value| value.round().max(1.0) as u32)
        );
        assert!(!source.scene.ops.is_empty(), "real bound preview callback");
        assert!(
            translated
                .external_textures
                .iter()
                .any(|draw| draw.texture_key == source.texture_key
                    && draw.content_generation == Some(source.content_generation))
        );
    }
}

#[test]
fn production_workshop_portable_capture_tracks_authored_lifecycle_at_wide_and_narrow_sizes() {
    for width in [1280.0, 420.0] {
        let mut state = WorkshopState::in_memory();
        let mut authored = state.draft_theme().clone();
        authored.name = "Portable capture authored appearance".into();
        authored.seeds.primary.a = 73;
        state.edit_definition(&authored, Some(Mode::Light)).unwrap();
        let mut host = Harness::with_hooks(
            native_init(state, &WindowCommands::new()),
            native_hooks(|_| panic!("capture cannot export")),
        );
        host.layout_at(width, 1100.0);
        let cold = capture_case(&mut host, width, "cold");
        assert_complete(&cold);
        let repeat = capture_case(&mut host, width, "repeat");
        assert_complete(&repeat);
        for (before, after) in cold.producers.iter().zip(&repeat.producers) {
            assert_eq!(before.content_generation, after.content_generation);
            assert_eq!(before.source_identity, after.source_identity);
            assert_eq!(
                scene_digest(&before.scene).unwrap(),
                scene_digest(&after.scene).unwrap()
            );
        }

        host.update(|state| {
            *state.text_field_mut("seed-hex").unwrap() = cambium::TextInput::new("#123456");
            assert!(state.apply_hex());
            assert_eq!(
                state.draft_theme().seeds.primary.a,
                73,
                "RGB editing retains authored coverage"
            );
        });
        let edited = capture_case(&mut host, width, "edited");
        assert_complete(&edited);
        assert!(
            edited
                .producers
                .iter()
                .zip(&repeat.producers)
                .any(|(after, before)| after.content_generation > before.content_generation)
        );
        let authored_after_edit = host.state().draft_theme().clone();
        host.update(|state| state.set_mode(Mode::Dark));
        let mode = capture_case(&mut host, width, "mode");
        assert_complete(&mode);
        assert_eq!(
            host.state().draft_theme(),
            &authored_after_edit,
            "preview mode preserves authoring authority"
        );
        assert!(
            mode.producers
                .iter()
                .zip(&edited.producers)
                .any(|(after, before)| after.source_revision != before.source_revision)
        );

        host.layout_at(width + 100.0, 960.0);
        let resized = capture_case(&mut host, width, "resized");
        assert_complete(&resized);
        assert!(
            resized
                .producers
                .iter()
                .zip(&mode.producers)
                .any(
                    |(after, before)| after.physical_size != before.physical_size
                        && after.content_generation > before.content_generation
                )
        );

        // Replace the actual application value under retained PreviewBindings:
        // matching revision counters cannot retain an old source identity.
        host.update(|state| *state = WorkshopState::in_memory());
        let rebound = capture_case(&mut host, width, "rebound");
        assert_complete(&rebound);
        for (before, after) in resized.producers.iter().zip(&rebound.producers) {
            assert_ne!(before.source_identity, after.source_identity);
            assert!(after.content_generation > before.content_generation);
        }
    }
}
