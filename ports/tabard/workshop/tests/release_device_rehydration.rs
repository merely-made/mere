// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Ignored headless GPU gate: upload CPU production pixels, drop every owned
//! GPU handle, boot a second core and explicitly rehydrate a fresh session.
//! This is orderly device recreation, not forced driver loss or swapchain
//! presentation. Caller must also impose a finite process timeout for boot.

#![cfg(all(not(target_arch = "wasm32"), feature = "native-host"))]

use genet_render_host::{PendingRgbaReadback, RenderCore, RgbaFrame};
use netrender::vello_backends::{CpuSession, SparseSessionError};
use netrender::{ColorLoad, ImageData, NetrenderOptions, Scene, SceneOp};
use std::time::{Duration, Instant};
use tabard_workshop::WorkshopState;

const WIDTH: u32 = 160;
const HEIGHT: u32 = 152;
const PREVIEW_HEIGHT: u32 = 120;
const PREVIEW_KEY: u64 = 0x7461_6272_6573_0001;
const ALPHA_KEY: u64 = PREVIEW_KEY + 1;

/// CPU sparse pixels are premultiplied encoded-sRGB RGBA8. ImageData consumed
/// by Netrender is straight-alpha RGBA8; dividing RGB by coverage is required.
/// Fully transparent RGB is canonical zero. All arithmetic fits in u32.
fn straight_rgba(premultiplied: &[u8]) -> Vec<u8> {
    assert_eq!(premultiplied.len() % 4, 0);
    premultiplied
        .chunks_exact(4)
        .flat_map(|p| {
            let a = u32::from(p[3]);
            assert!(p[..3].iter().all(|&c| u32::from(c) <= a));
            let channel = |c: u8| {
                if a == 0 {
                    0
                } else {
                    ((u32::from(c) * 255 + a / 2) / a).min(255) as u8
                }
            };
            [channel(p[0]), channel(p[1]), channel(p[2]), p[3]]
        })
        .collect()
}

fn collect(mut pending: PendingRgbaReadback) -> RgbaFrame {
    let start = Instant::now();
    loop {
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "host copy exceeded five seconds"
        );
        let before = Instant::now();
        let result = pending.poll();
        assert!(
            before.elapsed() < Duration::from_millis(250),
            "nonblocking copy poll blocked"
        );
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "host copy missed original deadline"
        );
        if let Some(result) = result {
            return result.expect("host copy/map succeeds");
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn pixel(frame: &RgbaFrame, x: u32, y: u32) -> &[u8] {
    let offset = ((y * frame.width + x) * 4) as usize;
    &frame.rgba[offset..offset + 4]
}

fn semantic_pixels(frame: &RgbaFrame) {
    assert_eq!((frame.width, frame.height), (WIDTH, HEIGHT));
    assert!(!frame.is_blank());
    // Exact interior samples avoid glyph/edge antialiasing and distinguish
    // correct straight-alpha upload from silently reusing premultiplied bytes.
    for (x, expected) in [
        (4, [128u8, 0, 127, 255]),
        (20, [96, 64, 95, 255]),
        (48, [0, 0, 255, 255]),
    ] {
        for (actual, expected) in pixel(frame, x, PREVIEW_HEIGHT + 4).iter().zip(expected) {
            assert!(
                actual.abs_diff(expected) <= 1,
                "alpha upload semantic sample at x={x}: {:?}",
                pixel(frame, x, PREVIEW_HEIGHT + 4)
            );
        }
    }
    let colors: std::collections::BTreeSet<_> = frame.rgba[..(WIDTH * PREVIEW_HEIGHT * 4) as usize]
        .chunks_exact(4)
        .collect();
    assert!(
        colors.len() > 12,
        "actual production preview foreground was lost"
    );
}

#[test]
#[ignore = "real-adapter headless upload/recreate gate; explicit GPU handoff and finite process timeout"]
fn release_headless_cpu_upload_survives_orderly_new_device_and_session_rehydration() {
    let state = WorkshopState::in_memory();
    let draft = state.draft_theme().clone();
    let active = state.registry().active_theme();
    let reader = state.reader_preview();
    let source = reader.borrow_mut().frame(WIDTH, PREVIEW_HEIGHT);
    assert!(
        source
            .ops
            .iter()
            .any(|op| matches!(op, SceneOp::GlyphRun(run) if !run.glyphs.is_empty()))
    );
    let mut cpu = CpuSession::new(Default::default());
    let preview = cpu
        .render(&source)
        .expect("actual production reader CPU render");
    let preview_data = ImageData::from_bytes(
        WIDTH,
        PREVIEW_HEIGHT,
        straight_rgba(preview.data_as_u8_slice()),
    );

    // An explicit translucent source-over fixture gives upload alpha its own
    // acceptance control, independent of cross-renderer image comparisons.
    let mut alpha = Scene::new(32, 32);
    alpha.push_image(
        0.0,
        0.0,
        32.0,
        32.0,
        1,
        ImageData::from_bytes(32, 32, [255, 0, 0, 128].repeat(32 * 32)),
    );
    alpha.push_image(
        16.0,
        0.0,
        32.0,
        32.0,
        2,
        ImageData::from_bytes(16, 32, [0, 255, 0, 64].repeat(16 * 32)),
    );
    let coverage = cpu.render(&alpha).expect("transparent source-over fixture");
    let alpha_data = ImageData::from_bytes(32, 32, straight_rgba(coverage.data_as_u8_slice()));
    let mut upload = Scene::new(WIDTH, HEIGHT);
    upload.push_rect(
        0.0,
        PREVIEW_HEIGHT as f32,
        WIDTH as f32,
        HEIGHT as f32,
        [0.0, 0.0, 1.0, 1.0],
    );
    upload.push_image(
        0.0,
        0.0,
        WIDTH as f32,
        PREVIEW_HEIGHT as f32,
        PREVIEW_KEY,
        preview_data,
    );
    upload.push_image(
        0.0,
        PREVIEW_HEIGHT as f32,
        32.0,
        HEIGHT as f32,
        ALPHA_KEY,
        alpha_data,
    );
    let source_bytes: Vec<_> = upload
        .image_sources
        .values()
        .map(|data| data.data.data().to_vec())
        .collect();
    cpu.render(&upload).expect("hydrate owned upload sources");
    let mut stale = upload.clone();
    stale.image_sources.clear();
    cpu.render(&stale)
        .expect("existing session owns these sources");
    drop(cpu);

    // Only CPU-owned source bytes and application state cross this boundary.
    // Pending readback owns a device clone, so it is consumed inside the scope.
    let first = {
        let core = RenderCore::boot(NetrenderOptions {
            enable_vello: true,
            tile_cache_size: Some(32),
            ..Default::default()
        })
        .expect("first real host device");
        let (texture, view) = core.rasterize_for(
            PREVIEW_KEY,
            &upload,
            WIDTH,
            HEIGHT,
            ColorLoad::Clear(Default::default()),
        );
        let frame = collect(
            core.start_rgba8_readback(&texture, WIDTH, HEIGHT)
                .expect("arm first host copy"),
        );
        semantic_pixels(&frame);
        drop(view);
        drop(texture);
        drop(core);
        frame
    };

    let mut restored = CpuSession::new(Default::default());
    let before = restored.stats();
    assert!(matches!(
        restored.render(&stale),
        Err(SparseSessionError::MissingImage { .. })
    ));
    assert_eq!(
        restored.stats(),
        before,
        "missing source refused before cache mutation"
    );
    for (&key, data) in &upload.image_sources {
        restored
            .set_image_source(key, data.clone())
            .expect("explicit owned-source rehydration");
    }
    restored
        .render(&stale)
        .expect("fresh session after resource rehydration");
    assert_eq!(
        upload
            .image_sources
            .values()
            .map(|data| data.data.data().to_vec())
            .collect::<Vec<_>>(),
        source_bytes
    );
    let second = {
        let core = RenderCore::boot(NetrenderOptions {
            enable_vello: true,
            tile_cache_size: Some(32),
            ..Default::default()
        })
        .expect("second real host device after teardown");
        let (texture, view) = core.rasterize_for(
            PREVIEW_KEY,
            &upload,
            WIDTH,
            HEIGHT,
            ColorLoad::Clear(Default::default()),
        );
        let frame = collect(
            core.start_rgba8_readback(&texture, WIDTH, HEIGHT)
                .expect("arm second host copy"),
        );
        semantic_pixels(&frame);
        drop(view);
        drop(texture);
        drop(core);
        frame
    };
    assert_eq!(
        first.rgba, second.rgba,
        "every uploaded pixel must survive orderly device recreation"
    );
    assert_eq!(state.draft_theme(), &draft);
    assert_eq!(state.registry().active_theme(), active);
    assert!(!state.has_changes());
    assert!(state.library_path().is_none());
    if let Some(output) = std::env::var_os("TABARD_RELEASE_DEVICE_RECEIPT") {
        let output = std::path::PathBuf::from(output);
        std::fs::create_dir_all(&output).unwrap();
        std::fs::write(output.join("first-device.rgba"), &first.rgba).unwrap();
        std::fs::write(output.join("second-device.rgba"), &second.rgba).unwrap();
        std::fs::write(output.join("receipt.json"), format!(
            "{{\"scope\":\"headless host CPU upload and orderly fresh-device/session rehydration\",\"width\":{WIDTH},\"height\":{HEIGHT},\"successful_sequential_core_boots\":2,\"copy_deadline_seconds\":5,\"source_alpha\":\"CPU premultiplied RGBA8 explicitly converted to straight ImageData\",\"exact_pixels_across_recreate\":true,\"missing_source_refused_atomically\":true,\"application_authority_preserved\":true,\"forced_driver_loss\":false,\"swapchain_presentation\":false,\"first_digest_fnv1a64\":\"{:016x}\",\"second_digest_fnv1a64\":\"{:016x}\"}}\n",
            first.digest(), second.digest())).unwrap();
    }
}
