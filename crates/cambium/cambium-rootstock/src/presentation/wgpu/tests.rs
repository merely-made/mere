// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Real checked GPU output, independent of window-system availability.
use super::*;
use crate::ContextGeneration;
use paint_list_api::{
    AlphaType, ColorF, CommonPlacement, DeviceIntSize, EngineId, ImageItem, ImageKey,
    ImageRendering, ImageResource, LayoutPoint, LayoutRect, PaintCmd, PaintEnvelope, RectItem,
};
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
mod native_tests;

struct Offscreen(Arc<genet_render_host::RenderCore>);
impl Surface for Offscreen {
    fn core(&self) -> &genet_render_host::RenderCore {
        &self.0
    }
    fn format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8Unorm
    }
    fn resize(&mut self, _: u32, _: u32) {}
    fn acquire(&self) -> Option<wgpu::SurfaceTexture> {
        None
    }
}
fn surface() -> Offscreen {
    Offscreen(Arc::new(
        genet_render_host::RenderCore::boot(netrender::NetrenderOptions {
            enable_vello: true,
            tile_cache_size: Some(16),
            ..Default::default()
        })
        .expect("qualified raster control requires an actual GPU adapter"),
    ))
}
fn frame(surface: &Offscreen, color: ColorF) -> PreparedFrame {
    let envelope = PaintEnvelope {
        engine: EngineId::GENET,
        viewport: DeviceIntSize::new(16, 16),
        generation: 1,
        commands: vec![PaintCmd::DrawRect(RectItem {
            placement: CommonPlacement::new(LayoutRect::new(
                LayoutPoint::new(0., 0.),
                LayoutPoint::new(16., 16.),
            )),
            color,
        })],
        fonts: vec![],
        images: vec![],
    };
    let translated = paint_list_render::translate_envelope_with_external_textures(&envelope)
        .expect("ordinary paint admission");
    PreparedFrame {
        paint: PreparedPaint::Native(envelope),
        native_scene: Some(translated),
        native_producer_identities: vec![],
        host: 4001,
        sequence: 1,
        width: 16,
        height: 16,
        scale: 1.,
        generation: ContextGeneration(surface.core().generation()),
        clear: [0., 0., 0., 1.],
        profile: Default::default(),
        started: crate::Instant::now(),
        animating: false,
    }
}
fn raster(
    surface: &Offscreen,
    packet: &PreparedFrame,
    cache: &mut RasterCache,
) -> (WgpuOutput, FrameProfile) {
    let translated = packet.native_scene.as_ref().unwrap();
    let mut profile = FrameProfile::default();
    let output = WgpuPresentation::new(surface)
        .rasterize_scene(
            packet,
            &translated.scene,
            &translated.box_shadow_masks,
            packet.scale,
            &mut profile,
            cache,
            None,
        )
        .expect("native checked raster");
    (output, profile)
}
fn pixels(surface: &Offscreen, output: &WgpuOutput) -> Vec<u8> {
    crate::read_frame(surface, &output.view, 16, 16)
        .expect("actual output readback")
        .rgba
}

#[test]
fn repeated_checked_frame_reuses_output_and_matches_an_independent_cold_raster() {
    let surface = surface();
    let mut packet = frame(&surface, ColorF::new(1., 0., 0., 1.));
    let mut cache = RasterCache::default();
    let (first, first_profile) = raster(&surface, &packet, &mut cache);
    assert!(!first_profile.raster_reused);
    let first_pixels = pixels(&surface, &first);
    assert_eq!(&first_pixels[0..4], &[255, 0, 0, 255]);
    packet.sequence += 1;
    if let PreparedPaint::Native(paint) = &mut packet.paint {
        paint.generation += 1;
    }
    let (second, profile) = raster(&surface, &packet, &mut cache);
    let (cold, cold_profile) = raster(&surface, &packet, &mut RasterCache::default());
    assert!(!cold_profile.raster_reused);
    assert_ne!(first._texture, cold._texture, "independent cold target");
    assert_eq!(pixels(&surface, &second), first_pixels);
    assert_eq!(pixels(&surface, &cold), first_pixels);
    assert!(
        profile.raster_reused,
        "unchanged checked output was rasterized again"
    );
    assert_eq!(first._texture, second._texture, "same retained GPU image");
    assert_eq!(profile.raster_us, 0);
    assert_eq!(profile.raster_total_us, 0);
    assert_eq!(profile.dirty_tiles, 0);
}

#[test]
fn changed_pixels_under_the_same_inline_resource_key_match_a_cold_reference() {
    let surface = surface();
    let mut packet = frame(&surface, ColorF::new(1., 0., 0., 1.));
    let key = ImageKey::new(paint_list_api::IdNamespace(7), 9);
    fn set_image(packet: &mut PreparedFrame, key: ImageKey, rgba: Vec<u8>) {
        let PreparedPaint::Native(paint) = &mut packet.paint else {
            panic!()
        };
        paint.commands = vec![PaintCmd::DrawImage(ImageItem {
            placement: CommonPlacement::new(LayoutRect::new(
                LayoutPoint::new(0., 0.),
                LayoutPoint::new(16., 16.),
            )),
            image_key: key,
            image_rendering: ImageRendering::Pixelated,
            alpha_type: AlphaType::Alpha,
            color: ColorF::WHITE,
        })];
        paint.images = vec![ImageResource {
            key,
            width: 1,
            height: 1,
            data: Arc::new(rgba),
        }];
        packet.native_scene =
            Some(paint_list_render::translate_envelope_with_external_textures(paint).unwrap());
    }
    set_image(&mut packet, key, vec![255, 0, 0, 255]);
    let mut cache = RasterCache::default();
    let (red, _) = raster(&surface, &packet, &mut cache);
    let red_pixels = pixels(&surface, &red);
    set_image(&mut packet, key, vec![0, 255, 0, 255]);
    let (green, profile) = raster(&surface, &packet, &mut cache);
    let (cold, _) = raster(&surface, &packet, &mut RasterCache::default());
    assert!(!profile.raster_reused);
    assert_ne!(red._texture, green._texture);
    let green_pixels = pixels(&surface, &green);
    assert_ne!(red_pixels, green_pixels);
    assert_eq!(&green_pixels[0..4], &[0, 255, 0, 255]);
    assert_eq!(green_pixels, pixels(&surface, &cold));
    let (retained, profile) = raster(&surface, &packet, &mut cache);
    assert!(profile.raster_reused);
    assert_eq!(retained._texture, green._texture);
}

#[test]
fn unavailable_surface_never_publishes_a_retained_raster() {
    let surface = surface();
    let packet = frame(&surface, ColorF::WHITE);
    let mut cache = RasterCache::default();
    let presenter = WgpuPresentation::new(&surface);
    let mut profile = FrameProfile::default();
    assert!(
        presenter
            .present(&packet, &mut profile, &mut cache, None)
            .unwrap()
            .is_none()
    );
    assert!(!profile.raster_reused);
    let mut profile = FrameProfile::default();
    assert!(
        presenter
            .present(&packet, &mut profile, &mut cache, None)
            .unwrap()
            .is_none()
    );
    assert!(profile.raster_reused);
}

#[test]
fn retained_output_never_bypasses_a_pending_native_batch() {
    let surface = surface();
    let packet = frame(&surface, ColorF::WHITE);
    let mut cache = RasterCache::default();
    let _ = raster(&surface, &packet, &mut cache);
    let _submission = surface
        .0
        .try_begin_native_frame()
        .expect("native attribution owner");
    let translated = packet.native_scene.as_ref().unwrap();
    let mut profile = FrameProfile::default();
    let result = WgpuPresentation::new(&surface).rasterize_scene(
        &packet,
        &translated.scene,
        &translated.box_shadow_masks,
        packet.scale,
        &mut profile,
        &mut cache,
        None,
    );
    match result {
        Err(PresentationFailure {
            native: Some(NativePresentationFailure::Rasterize(_)),
            ..
        }) => {},
        _ => panic!("cached raster escaped the current Core's native frame admission"),
    }
    assert!(!profile.raster_reused);
}

#[test]
fn unchanged_offscreen_workload_records_checked_and_retained_raster_costs() {
    let surface = surface();
    let mut packet = frame(&surface, ColorF::WHITE);
    packet.width = 1280;
    packet.height = 800;
    let PreparedPaint::Native(paint) = &mut packet.paint else {
        panic!()
    };
    paint.viewport = DeviceIntSize::new(1280, 800);
    paint.commands = (0..128)
        .map(|i| {
            let x = (i % 16) as f32 * 80.;
            let y = (i / 16) as f32 * 100.;
            PaintCmd::DrawRect(RectItem {
                placement: CommonPlacement::new(LayoutRect::new(
                    LayoutPoint::new(x, y),
                    LayoutPoint::new(x + 80., y + 100.),
                )),
                color: ColorF::new(i as f32 / 128., 0.25, 0.5, 1.),
            })
        })
        .collect();
    packet.native_scene =
        Some(paint_list_render::translate_envelope_with_external_textures(paint).unwrap());
    let mut cache = RasterCache::default();
    let _ = raster(&surface, &packet, &mut cache);
    let mut checked_us = Vec::new();
    let mut retained_us = Vec::new();
    // Alternating pairs keep GPU/host warmup and time drift shared. Every
    // checked member forces the old full-raster behavior on this same path.
    for _ in 0..32 {
        cache.clear();
        let started = crate::Instant::now();
        let (checked, profile) = raster(&surface, &packet, &mut cache);
        checked_us.push(elapsed_us(started.elapsed()));
        assert!(!profile.raster_reused);
        let started = crate::Instant::now();
        let (retained, profile) = raster(&surface, &packet, &mut cache);
        retained_us.push(elapsed_us(started.elapsed()));
        assert!(profile.raster_reused);
        assert_eq!(checked._texture, retained._texture);
    }
    // This reports raster-only host wall time, including policy/content
    // admission. It is not GPU duration, WSI presentation or a full app frame.
    eprintln!(
        "qualified-raster-workload {{\"fixture\":\"1280x800 128 ordinary rectangles\",\"pairs\":32,\"checked_us\":{checked_us:?},\"retained_us\":{retained_us:?}}}"
    );
}
