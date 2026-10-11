// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Explicit real-adapter gate for the two production workshop previews.
//! This is ignored during ordinary CPU testing: a boot failure is a failure,
//! and the caller must impose a process timeout around GPU/driver execution.
//! TABARD_PREVIEW_RENDER_OUTPUT optionally retains raw RGBA and JSON receipts.

#![cfg(not(target_arch = "wasm32"))]

use genet_render_host::{PendingRgbaReadback, RenderCore, RgbaFrame};
use netrender::{ColorLoad, NetrenderOptions, Scene, SceneOp};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
use tabard::theme::registry::Mode;
use tabard_workshop::WorkshopState;

// The same stable identities are reused across modes, authored edits and
// resizes. Distinct previews share one core without sharing retained tiles.
const READER_SURFACE: u64 = 0x7461_6261_7264_7201;
const CSS_SURFACE: u64 = READER_SURFACE + 1;
const COPY_DEADLINE: Duration = Duration::from_secs(5);
const AUTHORED_BG: [u8; 4] = [17, 43, 79, 255];
const AUTHORED_CSS: &str = ":root { --paper: rgb(17, 43, 79); } body { background-color: var(--paper); color: rgb(221, 231, 241); } #preview-heading { color: rgb(243, 211, 117); } #preview-button { background-color: rgb(61, 87, 113); color: rgb(251, 249, 245); }";

struct Copy {
    label: String,
    width: u32,
    height: u32,
    background: [u8; 4],
    glyph_runs: usize,
    glyph_colors: Vec<[u8; 4]>,
    armed: Instant,
    completed_ms: Option<u128>,
    pending: PendingRgbaReadback,
}

fn rgba(color: tinct::Srgb) -> [u8; 4] {
    [color.r, color.g, color.b, 255]
}

fn queue_scene(
    core: &RenderCore,
    surface: u64,
    scene: Scene,
    label: String,
    background: [u8; 4],
) -> Copy {
    let (width, height) = (scene.viewport_width, scene.viewport_height);
    let glyph_colors: Vec<_> = scene
        .ops
        .iter()
        .filter_map(|op| match op {
            SceneOp::GlyphRun(run) if !run.glyphs.is_empty() => Some(
                run.color
                    .map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u8),
            ),
            _ => None,
        })
        .collect();
    assert!(
        glyph_colors.len() >= 3,
        "{label}: real shaped text is required"
    );
    eprintln!(
        "rasterize {label}: {width}x{height}, {} glyph runs",
        glyph_colors.len()
    );
    // Load means transparent base on Vello. The actual document scene must
    // supply its selected background; a test-created clear cannot satisfy it.
    let (texture, _) = core.rasterize_for(surface, &scene, width, height, ColorLoad::Load);
    let armed = Instant::now();
    let pending = core
        .start_rgba8_readback(&texture, width, height)
        .unwrap_or_else(|error| panic!("{label}: cannot arm real copy: {error}"));
    Copy {
        label,
        width,
        height,
        background,
        glyph_runs: glyph_colors.len(),
        glyph_colors,
        armed,
        completed_ms: None,
        pending,
    }
}

fn queue_pair(
    core: &RenderCore,
    state: &WorkshopState,
    size: (u32, u32),
    label: &str,
    authored_bg: Option<[u8; 4]>,
) -> [Copy; 2] {
    let reader = state.reader_preview();
    let css = state.stylesheet_preview();
    assert!(
        reader
            .borrow()
            .accessible_name()
            .contains("The leaves hold yesterday's weather")
    );
    assert!(css.borrow().accessible_name().contains("Every path begins"));
    let reader_scene = reader.borrow_mut().frame(size.0, size.1);
    let css_scene = css.borrow_mut().frame(size.0, size.1);
    assert!(
        css.borrow().diagnostics().is_empty(),
        "{label}: actual CSS diagnostics"
    );
    assert_eq!(
        (reader_scene.viewport_width, reader_scene.viewport_height),
        size
    );
    assert_eq!((css_scene.viewport_width, css_scene.viewport_height), size);
    [
        queue_scene(
            core,
            READER_SURFACE,
            reader_scene,
            format!("reader-{label}"),
            rgba(state.preview_syntax().surface),
        ),
        queue_scene(
            core,
            CSS_SURFACE,
            css_scene,
            format!("css-{label}"),
            authored_bg.unwrap_or_else(|| rgba(state.preview_palette().bg)),
        ),
    ]
}

fn collect(copy: &mut Copy) -> RgbaFrame {
    loop {
        assert!(
            copy.armed.elapsed() < COPY_DEADLINE,
            "{}: copy exceeded five seconds from arm",
            copy.label
        );
        let before = Instant::now();
        let result = copy.pending.poll();
        assert!(
            before.elapsed() < Duration::from_millis(250),
            "{}: nonblocking poll blocked",
            copy.label
        );
        assert!(
            copy.armed.elapsed() < COPY_DEADLINE,
            "{}: completion missed original deadline",
            copy.label
        );
        if let Some(result) = result {
            copy.completed_ms = Some(copy.armed.elapsed().as_millis());
            let frame = result.unwrap_or_else(|error| panic!("{}: {error}", copy.label));
            assert!(
                matches!(copy.pending.poll(), Some(Err(_))),
                "an owned copy is collected once"
            );
            return frame;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn json_string(value: &str) -> String {
    let mut result = String::from("\"");
    for character in value.chars() {
        match character {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            character if character.is_control() => {
                result.push_str(&format!("\\u{:04x}", character as u32))
            },
            character => result.push(character),
        }
    }
    result.push('"');
    result
}

fn inspect(copy: &Copy, frame: &RgbaFrame, output: Option<&Path>) -> u64 {
    assert_eq!((frame.width, frame.height), (copy.width, copy.height));
    assert_eq!(frame.rgba.len(), (copy.width * copy.height * 4) as usize);
    let digest = frame.digest();
    let colors: BTreeSet<_> = frame
        .rgba
        .chunks_exact(4)
        .map(|pixel| [pixel[0], pixel[1], pixel[2], pixel[3]])
        .collect();
    let foreground_pixels = frame
        .rgba
        .chunks_exact(4)
        .filter(|pixel| {
            pixel[3] == 255
                && copy.glyph_colors.iter().any(|color| {
                    color[3] == 255
                        && (0..3)
                            .any(|channel| color[channel].abs_diff(copy.background[channel]) > 12)
                        && (0..3).all(|channel| pixel[channel].abs_diff(color[channel]) <= 2)
                })
        })
        .count();
    // Persist successful mapping before assertions so a bad rendered image is
    // available for diagnosis, rather than being replaced by a synthetic image.
    if let Some(output) = output {
        std::fs::write(output.join(format!("{}.rgba", copy.label)), &frame.rgba).unwrap();
        std::fs::write(output.join(format!("{}.json", copy.label)), format!(
            "{{\"label\":{},\"width\":{},\"height\":{},\"format\":\"RGBA8 tightly packed\",\"digest_fnv1a64\":\"{digest:016x}\",\"glyph_runs\":{},\"unique_colors\":{},\"foreground_pixels\":{},\"expected_background\":{:?},\"top_left\":{:?},\"copy_elapsed_ms\":{}}}\n",
            json_string(&copy.label), frame.width, frame.height, copy.glyph_runs, colors.len(), foreground_pixels, copy.background, &frame.rgba[..4], copy.completed_ms.expect("mapped copy timestamp")
        )).unwrap();
    }
    assert!(!frame.is_blank(), "{}: transparent preview", copy.label);
    // Both production documents have zero body margin and inset content; the
    // top corners lie on their actual selected background, outside all text.
    for x in [0, copy.width - 1] {
        let offset = (x * 4) as usize;
        assert_eq!(
            &frame.rgba[offset..offset + 4],
            copy.background,
            "{}: selected background at ({x},0)",
            copy.label
        );
    }
    assert!(
        colors.len() >= 12,
        "{}: flat/missing foreground ({} colors)",
        copy.label,
        colors.len()
    );
    assert!(
        foreground_pixels >= 32,
        "{}: selected glyph foreground is missing ({foreground_pixels} pixels)",
        copy.label
    );
    eprintln!(
        "mapped {}: {digest:016x}, {} colors, {foreground_pixels} foreground pixels",
        copy.label,
        colors.len()
    );
    digest
}

// Parallel glyph coverage can round an antialiased channel one byte
// differently. The first Metal repeat differed in one of 349,687 pixels.
// This allowance applies only to same-image comparisons: backgrounds,
// foreground presence, modes and capture deadlines retain their own gates.
fn compare_pixel_precision(first: &[u8], repeat: &[u8]) -> Result<usize, String> {
    if first.len() != repeat.len() || first.len() % 4 != 0 {
        return Err("RGBA image dimensions/lengths differ".into());
    }
    let mut changed = 0;
    for (index, (a, b)) in first
        .chunks_exact(4)
        .zip(repeat.chunks_exact(4))
        .enumerate()
    {
        if a == b {
            continue;
        }
        if a.iter().zip(b).any(|(a, b)| a.abs_diff(*b) > 1) {
            return Err(format!(
                "pixel {index} changed a channel by more than one byte"
            ));
        }
        changed += 1;
        if changed > 64 {
            return Err(format!("more than 64 pixels changed ({changed})"));
        }
    }
    Ok(changed)
}

fn render_case(
    core: &RenderCore,
    state: &mut WorkshopState,
    size: (u32, u32),
    label: &str,
    authored_bg: Option<[u8; 4]>,
    output: Option<&Path>,
) -> ([u64; 2], Vec<u8>) {
    let next_mode = if state.mode().dark() {
        Mode::Light
    } else {
        Mode::Dark
    };
    let mut copies = Vec::from(queue_pair(
        core,
        state,
        size,
        &format!("{label}-first"),
        authored_bg,
    ));
    copies.extend(queue_pair(
        core,
        state,
        size,
        &format!("{label}-repeat"),
        authored_bg,
    ));
    state.set_mode(next_mode);
    // These later production scenes reuse each stable raster identity before
    // any earlier copy is collected. Earlier owned bytes must remain original.
    copies.extend(queue_pair(
        core,
        state,
        size,
        &format!("{label}-changed"),
        None,
    ));
    // Collect every owned copy first. Artifact I/O and pixel analysis must not
    // consume another copy's GPU deadline or obscure its actual map latency.
    let mut frames: Vec<Option<RgbaFrame>> = (0..copies.len()).map(|_| None).collect();
    for index in (0..copies.len()).rev() {
        frames[index] = Some(collect(&mut copies[index]));
    }
    let mut digests = [0; 6];
    for index in 0..copies.len() {
        digests[index] = inspect(&copies[index], frames[index].as_ref().unwrap(), output);
    }
    for (first, repeat, kind) in [(0, 2, "reader"), (1, 3, "CSS")] {
        let changed = compare_pixel_precision(
            &frames[first].as_ref().unwrap().rgba,
            &frames[repeat].as_ref().unwrap().rgba,
        )
        .unwrap_or_else(|error| panic!("{label}: {kind} settled repeat: {error}"));
        eprintln!("settled {label} {kind}: {changed} pixels differ by at most one byte");
    }
    assert_ne!(
        digests[0], digests[4],
        "{label}: reader mode did not change actual pixels"
    );
    assert_ne!(
        digests[1], digests[5],
        "{label}: CSS mode did not change actual pixels"
    );
    ([digests[0], digests[1]], frames[0].take().unwrap().rgba)
}

#[test]
fn pixel_precision_allowance_rejects_larger_or_more_widespread_changes() {
    let original = [17, 43, 79, 255].repeat(65);
    assert_eq!(compare_pixel_precision(&original, &original), Ok(0));
    let mut allowed = original.clone();
    for pixel in allowed.chunks_exact_mut(4).take(64) {
        pixel[0] += 1;
    }
    assert_eq!(compare_pixel_precision(&original, &allowed), Ok(64));
    allowed[64 * 4] += 1;
    assert!(
        compare_pixel_precision(&original, &allowed).is_err(),
        "65 changed pixels must fail"
    );
    let mut larger = original.clone();
    larger[0] += 2;
    assert!(
        compare_pixel_precision(&original, &larger).is_err(),
        "a two-step channel difference must fail even in one pixel"
    );
    assert!(compare_pixel_precision(&original, &original[..original.len() - 4]).is_err());
}

#[test]
#[ignore = "requires a real GPU; run explicitly with an external process timeout"]
fn production_reader_and_css_previews_share_one_gpu_with_retained_original_copies() {
    let output = std::env::var_os("TABARD_PREVIEW_RENDER_OUTPUT").map(PathBuf::from);
    if let Some(output) = &output {
        std::fs::create_dir_all(output).unwrap();
    }
    eprintln!("boot production preview GPU: Vello enabled, tile_cache_size=32");
    let core = RenderCore::boot(NetrenderOptions {
        enable_vello: true,
        tile_cache_size: Some(32),
        ..Default::default()
    })
    .expect("real GPU boot is required; absent adapter is not a passed receipt");
    let info = core.renderer().wgpu_device.core.adapter.get_info();
    eprintln!(
        "preview adapter: {} ({:?}, {:?}); driver={} {}",
        info.name, info.backend, info.device_type, info.driver, info.driver_info
    );
    if let Some(output) = &output {
        std::fs::write(output.join("adapter.json"), format!("{{\"name\":{},\"backend\":{},\"device_type\":{},\"driver\":{},\"driver_info\":{},\"enable_vello\":true,\"tile_cache_size\":32}}\n", json_string(&info.name), json_string(&format!("{:?}", info.backend)), json_string(&format!("{:?}", info.device_type)), json_string(&info.driver), json_string(&info.driver_info))).unwrap();
    }
    let mut state = WorkshopState::in_memory();
    let source = state.reader_preview().borrow().source_document();
    assert_eq!(source.title.as_deref(), Some("The garden after rain"));
    let mut receipts = BTreeMap::new();
    for size in [(727, 481), (263, 337)] {
        for mode in [Mode::Light, Mode::Dark, Mode::HcLight, Mode::HcDark] {
            state.set_mode(mode.clone());
            let label = format!("{}-{}x{}", mode.as_key(), size.0, size.1);
            let digest = render_case(&core, &mut state, size, &label, None, output.as_deref());
            assert!(
                Arc::ptr_eq(&source, &state.reader_preview().borrow().source_document()),
                "appearance must retain the extracted Reader packet"
            );
            receipts.insert((mode.as_key(), size), digest);
        }
        for preview in 0..2 {
            let unique: BTreeSet<_> = ["light", "dark", "hc_light", "hc_dark"]
                .into_iter()
                .map(|mode| receipts[&(mode.to_owned(), size)].0[preview])
                .collect();
            assert_eq!(
                unique.len(),
                4,
                "four canonical modes must differ in actual preview pixels"
            );
        }
        state.set_mode(Mode::Light);
        *state
            .text_field_mut("mode-sheet")
            .expect("actual workshop CSS editor") = cambium::TextInput::new(AUTHORED_CSS);
        state.apply_stylesheet();
        assert_eq!(
            state.stylesheet_preview().borrow().rules(),
            &[AUTHORED_CSS.to_owned()]
        );
        let authored = render_case(
            &core,
            &mut state,
            size,
            &format!("authored-{}x{}", size.0, size.1),
            Some(AUTHORED_BG),
            output.as_deref(),
        );
        let changed =
            compare_pixel_precision(&authored.1, &receipts[&("light".to_owned(), size)].1)
                .unwrap_or_else(|error| {
                    panic!("authored-{size:?}: CSS changed the typed Reader pixels: {error}")
                });
        eprintln!(
            "authored Reader isolation {size:?}: {changed} pixels differ by at most one byte"
        );
        assert!(
            Arc::ptr_eq(&source, &state.reader_preview().borrow().source_document()),
            "authored CSS must retain the extracted Reader packet"
        );
        assert_ne!(
            authored.0[1],
            receipts[&("light".to_owned(), size)].0[1],
            "authored CSS must change the actual application preview"
        );
        state.set_mode(Mode::Light);
        state.clear_stylesheet();
    }
}
