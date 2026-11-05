// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::time::{Duration, Instant};

fn collect<T>(
    pending: &mut genet_render_host::PendingNativeFrame<T>,
) -> genet_render_host::AcceptedNativeFrame<T> {
    let deadline = Instant::now() + Duration::from_secs(6);
    loop {
        if let Some(result) = pending.poll() {
            return result.unwrap();
        }
        assert!(
            Instant::now() < deadline,
            "native raster candidate did not complete"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn native_encode_under_active_admission_does_not_accept_a_cold_candidate() {
    let surface = surface();
    let packet = frame(&surface, ColorF::new(1., 0., 0., 1.));
    let presenter = WgpuPresentation::new(&surface);
    let mut cache = RasterCache::default();
    let mut profile = FrameProfile::default();
    let batch = surface.0.try_begin_native_frame().unwrap();
    let output = presenter
        .encode_native(&packet, &batch, &mut profile, &mut cache)
        .unwrap();
    assert!(!profile.raster_reused);
    assert!(cache.lookup(surface.device(), &packet, true).is_none());
    let mut pending = batch.seal(output);
    let output = collect(&mut pending)
        .publish(|output| {
            assert!(
                !presenter
                    .present_output(&packet, &output, &mut profile)
                    .unwrap()
            );
            output
        })
        .unwrap();
    assert!(cache.lookup(surface.device(), &packet, true).is_none());
    assert_eq!(&pixels(&surface, &output)[..4], &[255, 0, 0, 255]);
}

#[test]
fn native_cache_hit_clones_inside_its_batch_without_nested_checked_admission() {
    let surface = surface();
    let packet = frame(&surface, ColorF::WHITE);
    let mut cache = RasterCache::default();
    let (accepted, _) = raster(&surface, &packet, &mut cache);
    let before = surface.core().renderer().vello_curve_quality_status();
    let batch = surface.0.try_begin_native_frame().unwrap();
    let mut profile = FrameProfile {
        raster_us: 999,
        raster_total_us: 999,
        dirty_tiles: 999,
        ..Default::default()
    };
    let reused = WgpuPresentation::new(&surface)
        .encode_native(&packet, &batch, &mut profile, &mut cache)
        .unwrap();
    assert!(profile.raster_reused);
    assert_eq!(reused._texture, accepted._texture);
    assert_eq!(profile.raster_us, 0);
    assert_eq!(profile.raster_total_us, 0);
    assert_eq!(profile.dirty_tiles, 0);
    assert_eq!(
        surface.core().renderer().vello_curve_quality_status(),
        before
    );
    let mut pending = batch.seal(reused);
    let reused = collect(&mut pending).publish(|output| output).unwrap();
    assert_eq!(pixels(&surface, &reused), pixels(&surface, &accepted));
}

#[test]
fn failed_native_presentation_preserves_the_previous_qualified_cache_entry() {
    let surface = surface();
    let old_packet = frame(&surface, ColorF::new(1., 0., 0., 1.));
    let new_packet = frame(&surface, ColorF::new(0., 1., 0., 1.));
    let mut cache = RasterCache::default();
    let (accepted, _) = raster(&surface, &old_packet, &mut cache);
    let presenter = WgpuPresentation::new(&surface);
    let batch = surface.0.try_begin_native_frame().unwrap();
    let mut profile = FrameProfile::default();
    let output = presenter
        .encode_native(&new_packet, &batch, &mut profile, &mut cache)
        .unwrap();
    assert!(!profile.raster_reused);
    assert_ne!(output._texture, accepted._texture);
    assert!(cache.lookup(surface.device(), &new_packet, true).is_none());
    assert_eq!(
        cache
            .lookup(surface.device(), &old_packet, true)
            .unwrap()
            ._texture,
        accepted._texture
    );
    let mut pending = batch.seal(output);
    let output = collect(&mut pending)
        .publish(|output| {
            assert!(
                !presenter
                    .present_output(&new_packet, &output, &mut profile)
                    .unwrap()
            );
            output
        })
        .unwrap();
    assert!(cache.lookup(surface.device(), &new_packet, true).is_none());
    assert_eq!(
        cache
            .lookup(surface.device(), &old_packet, true)
            .unwrap()
            ._texture,
        accepted._texture
    );
    assert_eq!(&pixels(&surface, &output)[..4], &[0, 255, 0, 255]);
}
