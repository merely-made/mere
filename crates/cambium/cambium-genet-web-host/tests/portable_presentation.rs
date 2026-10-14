// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
#![cfg(feature = "webgl")]
use cambium_genet_web_host::portable::{PortablePresentationError, materialize};
use netrender::vello_backends::{CpuSession, SparseResourceLimits};
use netrender::{ImageData, NO_CLIP, Scene};
use paint_list_api::*;
use paint_list_render::bundle::*;
fn capture() -> PortablePaintCapture {
    let mut commands = Vec::new();
    let mut producers = Vec::new();
    for (key, x, rgba, opacity) in [
        (7, 1., [255, 0, 0, 255], 1.),
        (8, 2., [0, 255, 0, 255], 0.5),
    ] {
        commands.push(PaintCmd::DrawExternalTexture(ExternalTextureItem {
            placement: CommonPlacement::new(LayoutRect::new(
                LayoutPoint::new(x, 1.),
                LayoutPoint::new(x + 2., 3.),
            )),
            texture_key: key,
            opacity,
            content_generation: Some(3),
        }));
        let mut scene = Scene::new(2, 2);
        // Identical local image key, different owners and pixels.
        scene
            .image_sources
            .insert(71, ImageData::from_bytes(1, 1, rgba.to_vec()));
        scene.push_image_full(0., 0., 3., 3., [0., 0., 1., 1.], [1.; 4], 71, 0, NO_CLIP);
        producers.push(ProducerSource {
            texture_key: key,
            content_generation: 3,
            source_identity: key,
            source_revision: 0,
            logical_size: [2., 2.],
            physical_size: [4, 4],
            layout_scale: 2.,
            alpha: ResourceAlpha::Straight,
            encoding: ResourceEncoding::Srgb,
            scene,
        });
    }
    PortablePaintCapture {
        envelope: PaintEnvelope {
            engine: EngineId::GENET,
            viewport: DeviceIntSize::new(8, 4),
            generation: 1,
            commands,
            fonts: vec![],
            images: vec![],
        },
        producers,
    }
}
#[test]
fn producer_order_opacity_local_keys_and_device_scale_survive_composition() {
    let scene = materialize(&capture(), 16, 8, 2., SparseResourceLimits::default()).unwrap();
    let mut cpu = CpuSession::new(SparseResourceLimits::default());
    let frame = cpu.render(&scene).unwrap();
    let bytes = frame.data_as_u8_slice();
    let pixel = |x: usize, y: usize| &bytes[(y * 16 + x) * 4..][..4];
    for (x, y, expected) in [
        (3, 3, [255u8, 0, 0, 255]),
        (5, 3, [128, 128, 0, 255]),
        (7, 3, [0, 128, 0, 128]),
        (9, 3, [0, 0, 0, 0]),
    ] {
        let actual = pixel(x, y);
        assert!(
            actual.iter().zip(expected).all(|(a, b)| a.abs_diff(b) <= 2),
            "{x},{y}: {actual:?}, expected {expected:?}"
        );
    }
}
#[test]
fn missing_producer_and_wrong_content_stamp_refuse_complete_capture() {
    let mut missing = capture();
    missing.producers.remove(0);
    assert!(materialize(&missing, 16, 8, 2., SparseResourceLimits::default()).is_err());
    let mut stale = capture();
    stale.producers[0].content_generation = 4;
    assert!(materialize(&stale, 16, 8, 2., SparseResourceLimits::default()).is_err());
}
#[test]
fn unsupported_producer_is_attributed_before_any_backend_draw() {
    let mut capture = capture();
    let mut layer = netrender::SceneLayer::alpha(1.);
    layer.filters.push(netrender::SceneFilter::Blur(2.));
    capture.producers[0]
        .scene
        .ops
        .push(netrender::SceneOp::PushLayer(layer));
    capture.producers[0]
        .scene
        .ops
        .push(netrender::SceneOp::PopLayer);
    let PortablePresentationError { source_key, .. } =
        materialize(&capture, 16, 8, 2., SparseResourceLimits::default()).unwrap_err();
    assert_eq!(source_key, Some(7));
}

#[test]
fn producer_ledger_order_does_not_change_private_resource_identity() {
    let capture = capture();
    let first = materialize(&capture, 16, 8, 2., SparseResourceLimits::default()).unwrap();
    let mut reversed = capture;
    reversed.producers.reverse();
    let second = materialize(&reversed, 16, 8, 2., SparseResourceLimits::default()).unwrap();
    assert_eq!(
        scene_digest(&first).unwrap(),
        scene_digest(&second).unwrap()
    );
}
