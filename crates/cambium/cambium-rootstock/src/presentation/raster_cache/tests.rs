// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! CPU qualification of retained output selection; these tokens are not GPU images.

use super::QualifiedRasterCache;
use crate::presentation::{ContextGeneration, PreparedFrame, PreparedPaint};
use crate::{PortablePaintCapture, ProducerImageIdentity};
use layout_dom_api::LayoutDom;
use paint_list_api::{
    ColorF, CommonPlacement, DeviceIntSize, EngineId, ExternalTextureItem, FontInstanceKey,
    FontResource, IdNamespace, ImageKey, ImageResource, LayoutPoint, LayoutRect, PaintCmd,
    PaintEnvelope, RectItem, RetainedFragmentRef,
};
use std::sync::Arc;

fn rect(red: f32) -> PaintCmd {
    PaintCmd::DrawRect(RectItem {
        placement: CommonPlacement::new(LayoutRect::new(
            LayoutPoint::new(1., 2.),
            LayoutPoint::new(12., 14.),
        )),
        color: ColorF {
            r: red,
            g: 0.,
            b: 0.,
            a: 1.,
        },
    })
}

fn envelope() -> PaintEnvelope {
    PaintEnvelope {
        engine: EngineId::GENET,
        viewport: DeviceIntSize::new(80, 60),
        generation: 5,
        commands: vec![
            rect(0.25),
            rect(0.75),
            PaintCmd::DrawExternalTexture(ExternalTextureItem {
                placement: CommonPlacement::new(LayoutRect::new(
                    LayoutPoint::new(20., 20.),
                    LayoutPoint::new(40., 40.),
                )),
                texture_key: 7,
                opacity: 1.,
                content_generation: Some(11),
            }),
        ],
        // CPU key comparison treats font bytes as opaque; it does not decode a font.
        fonts: vec![FontResource {
            key: FontInstanceKey(IdNamespace(3), 9),
            data: Arc::new(vec![0, 1, 2, 3]),
            index: 0,
        }],
        images: vec![ImageResource {
            key: ImageKey(IdNamespace(3), 10),
            width: 1,
            height: 1,
            data: Arc::new(vec![20, 40, 60, 255]),
        }],
    }
}

fn frame() -> PreparedFrame {
    PreparedFrame {
        paint: PreparedPaint::Native(envelope()),
        native_scene: None,
        native_producer_identities: vec![ProducerImageIdentity {
            renderer_key: netrender::external_image_key(1 << 61),
            owner: None,
            generation: 4,
            stamp: 11,
            size: [20, 20],
        }],
        host: 23,
        sequence: 8,
        width: 160,
        height: 120,
        scale: 2.,
        generation: ContextGeneration(3),
        clear: [0., 0., 0., 1.],
        profile: Default::default(),
        started: crate::Instant::now(),
        animating: false,
    }
}

fn paint(frame: &mut PreparedFrame) -> &mut PaintEnvelope {
    match &mut frame.paint {
        PreparedPaint::Native(paint) => paint,
        PreparedPaint::Portable(_) => panic!("native fixture required"),
    }
}

fn retained_or_fresh(
    cache: &mut QualifiedRasterCache<u64, u64>,
    device: u64,
    frame: &PreparedFrame,
    fresh_output: u64,
) -> u64 {
    if let Some(output) = cache.lookup(&device, frame, true) {
        return *output;
    }
    cache.insert(device, frame, fresh_output);
    fresh_output
}

#[test]
fn cold_and_fresh_caches_select_their_own_outputs() {
    let original = frame();
    let mut warm = QualifiedRasterCache::<u64, u64>::default();
    assert_eq!(warm.lookup(&41, &original, true), None);
    assert_eq!(retained_or_fresh(&mut warm, 41, &original, 101), 101);
    assert_eq!(retained_or_fresh(&mut warm, 41, &original, 999), 101);

    let mut cold = QualifiedRasterCache::<u64, u64>::default();
    assert_eq!(cold.lookup(&41, &original, true), None);
    assert_eq!(retained_or_fresh(&mut cold, 41, &original, 202), 202);
    assert_eq!(warm.lookup(&41, &original, true), Some(&101));
    assert_eq!(cold.lookup(&41, &original, true), Some(&202));
}

#[test]
fn publication_sequence_and_paint_generation_do_not_force_a_new_output() {
    let mut cache = QualifiedRasterCache::<u64, u64>::default();
    let original = frame();
    cache.insert(41, &original, 101);

    let mut republished = frame();
    republished.sequence += 1;
    paint(&mut republished).generation += 1;
    assert_eq!(retained_or_fresh(&mut cache, 41, &republished, 202), 101);
}

#[test]
fn equal_resource_bytes_in_new_allocations_reuse_the_output() {
    let original = frame();
    let fresh = frame();
    let (PreparedPaint::Native(a), PreparedPaint::Native(b)) = (&original.paint, &fresh.paint)
    else {
        unreachable!()
    };
    assert!(!Arc::ptr_eq(&a.fonts[0].data, &b.fonts[0].data));
    assert!(!Arc::ptr_eq(&a.images[0].data, &b.images[0].data));

    let mut cache = QualifiedRasterCache::<u64, u64>::default();
    cache.insert(41, &original, 101);
    assert_eq!(retained_or_fresh(&mut cache, 41, &fresh, 202), 101);
}

#[test]
fn host_device_context_extent_scale_and_clear_changes_select_new_outputs() {
    let changes: &[(&str, fn(&mut PreparedFrame))] = &[
        ("host", |f| f.host += 1),
        ("context generation", |f| f.generation.0 += 1),
        ("width", |f| f.width += 1),
        ("height", |f| f.height += 1),
        ("scale", |f| f.scale = 1.5),
        ("clear", |f| f.clear[0] = 0.5),
    ];
    for (name, change) in changes {
        let original = frame();
        let mut changed = frame();
        change(&mut changed);
        let mut cache = QualifiedRasterCache::<u64, u64>::default();
        cache.insert(41, &original, 101);
        assert_eq!(cache.lookup(&41, &changed, true), None, "{name}");
        assert_eq!(
            retained_or_fresh(&mut cache, 41, &changed, 202),
            202,
            "{name}"
        );
    }

    let mut cache = QualifiedRasterCache::<u64, u64>::default();
    let original = frame();
    cache.insert(41, &original, 101);
    assert_eq!(cache.lookup(&42, &original, true), None);
    assert_eq!(retained_or_fresh(&mut cache, 42, &original, 303), 303);
}

#[test]
fn paint_content_changes_miss_even_at_the_same_generation() {
    let changes: &[(&str, fn(&mut PaintEnvelope))] = &[
        ("engine", |p| p.engine = EngineId::NEMATIC),
        ("paint viewport", |p| {
            p.viewport = DeviceIntSize::new(81, 60)
        }),
        ("color", |p| p.commands[0] = rect(0.5)),
        ("paint order", |p| p.commands.swap(0, 1)),
        ("command removal", |p| {
            p.commands.pop();
        }),
        ("placement", |p| {
            let PaintCmd::DrawRect(rect) = &mut p.commands[0] else {
                unreachable!()
            };
            rect.placement.bounds =
                LayoutRect::new(LayoutPoint::new(3., 2.), LayoutPoint::new(14., 14.));
        }),
        ("external opacity", |p| {
            let PaintCmd::DrawExternalTexture(image) = &mut p.commands[2] else {
                unreachable!()
            };
            image.opacity = 0.5;
        }),
        ("external local key", |p| {
            let PaintCmd::DrawExternalTexture(image) = &mut p.commands[2] else {
                unreachable!()
            };
            image.texture_key += 1;
        }),
        ("external content generation", |p| {
            let PaintCmd::DrawExternalTexture(image) = &mut p.commands[2] else {
                unreachable!()
            };
            image.content_generation = Some(12);
        }),
    ];
    for (name, change) in changes {
        let original = frame();
        let mut changed = frame();
        change(paint(&mut changed));
        let mut cache = QualifiedRasterCache::<u64, u64>::default();
        cache.insert(41, &original, 101);
        assert_eq!(cache.lookup(&41, &changed, true), None, "{name}");
        assert_eq!(
            retained_or_fresh(&mut cache, 41, &changed, 202),
            202,
            "{name}"
        );
    }
}

#[test]
fn font_and_image_payloads_and_resource_metadata_participate_in_the_key() {
    let changes: &[(&str, fn(&mut PaintEnvelope))] = &[
        ("font bytes", |p| {
            Arc::make_mut(&mut p.fonts[0].data)[2] = 99
        }),
        ("font key", |p| p.fonts[0].key.1 += 1),
        ("font collection index", |p| p.fonts[0].index += 1),
        ("font removal", |p| p.fonts.clear()),
        ("image bytes", |p| {
            Arc::make_mut(&mut p.images[0].data)[0] = 99
        }),
        ("image key", |p| p.images[0].key.1 += 1),
        ("image dimensions", |p| {
            p.images[0].width = 2;
            p.images[0].data = Arc::new(vec![20, 40, 60, 255, 20, 40, 60, 255]);
        }),
        ("image removal", |p| p.images.clear()),
    ];
    for (name, change) in changes {
        let original = frame();
        let mut changed = frame();
        change(paint(&mut changed));
        let mut cache = QualifiedRasterCache::<u64, u64>::default();
        cache.insert(41, &original, 101);
        assert_eq!(cache.lookup(&41, &changed, true), None, "{name}");
    }
}

#[test]
fn every_physical_producer_identity_field_invalidates_reuse() {
    let changes: &[(&str, fn(&mut ProducerImageIdentity))] = &[
        ("registration", |p| p.renderer_key += 1),
        ("owner", |p| {
            p.owner = Some(crate::ScriptedDom::new().document())
        }),
        ("device generation", |p| p.generation += 1),
        ("content stamp", |p| p.stamp += 1),
        ("width", |p| p.size[0] += 1),
        ("height", |p| p.size[1] += 1),
    ];
    for (name, change) in changes {
        let original = frame();
        let mut changed = frame();
        change(&mut changed.native_producer_identities[0]);
        let mut cache = QualifiedRasterCache::<u64, u64>::default();
        cache.insert(41, &original, 101);
        assert_eq!(cache.lookup(&41, &changed, true), None, "{name}");
        assert_eq!(
            retained_or_fresh(&mut cache, 41, &changed, 202),
            202,
            "{name}"
        );
    }
}

#[test]
fn the_complete_producer_vector_is_required_with_unchanged_local_commands() {
    let mut original = frame();
    let mut second = original.native_producer_identities[0];
    second.renderer_key += 1;
    original.native_producer_identities.push(second);
    let mut cache = QualifiedRasterCache::<u64, u64>::default();
    cache.insert(41, &original, 101);

    let mut changed = frame();
    changed.native_producer_identities = original.native_producer_identities.clone();
    changed.native_producer_identities[1].stamp += 1;
    assert_eq!(cache.lookup(&41, &changed, true), None);
    changed.native_producer_identities.pop();
    assert_eq!(cache.lookup(&41, &changed, true), None);
    changed.native_producer_identities.clear();
    assert_eq!(cache.lookup(&41, &changed, true), None);
}

#[test]
fn policy_refusal_does_not_return_or_discard_a_qualified_output() {
    let original = frame();
    let mut cache = QualifiedRasterCache::<u64, u64>::default();
    cache.insert(41, &original, 101);
    assert_eq!(cache.lookup(&41, &original, false), None);
    assert_eq!(cache.lookup(&41, &original, true), Some(&101));
}

#[test]
fn clear_prevents_reuse_until_a_new_output_is_inserted() {
    let original = frame();
    let mut cache = QualifiedRasterCache::<u64, u64>::default();
    cache.insert(41, &original, 101);
    cache.clear();
    assert_eq!(cache.lookup(&41, &original, true), None);
    assert_eq!(retained_or_fresh(&mut cache, 41, &original, 202), 202);
}

#[test]
fn portable_packets_never_select_a_native_retained_output() {
    let original = frame();
    let mut portable = frame();
    portable.paint = PreparedPaint::Portable(PortablePaintCapture {
        envelope: envelope(),
        producers: vec![],
    });
    portable.native_producer_identities.clear();
    let mut cache = QualifiedRasterCache::<u64, u64>::default();
    cache.insert(41, &original, 101);
    assert_eq!(cache.lookup(&41, &portable, true), None);
    cache.insert(41, &portable, 202);
    assert_eq!(cache.lookup(&41, &portable, true), None);
}

#[test]
fn retained_fragments_require_fresh_outputs_despite_equal_envelopes() {
    let packet = || {
        let mut frame = frame();
        paint(&mut frame)
            .commands
            .push(PaintCmd::PlaceRetainedFragment(RetainedFragmentRef {
                id: 71,
                origin: LayoutPoint::new(4., 5.),
            }));
        frame
    };
    let original = packet();
    let repeated = packet();
    let (PreparedPaint::Native(a), PreparedPaint::Native(b)) = (&original.paint, &repeated.paint)
    else {
        unreachable!()
    };
    assert!(a.same_content(b));

    let mut cache = QualifiedRasterCache::<u64, u64>::default();
    cache.insert(41, &original, 101);
    assert_eq!(cache.lookup(&41, &original, true), None);
    assert_eq!(retained_or_fresh(&mut cache, 41, &repeated, 202), 202);
    assert_eq!(retained_or_fresh(&mut cache, 41, &repeated, 303), 303);
}

#[test]
fn nan_target_scale_or_clear_never_reuses_an_output() {
    let changes: &[(&str, fn(&mut PreparedFrame))] = &[
        ("scale", |f| f.scale = f32::NAN),
        ("clear", |f| f.clear[0] = f32::NAN),
    ];
    for (name, change) in changes {
        let original = frame();
        let mut invalid = frame();
        change(&mut invalid);
        let mut cache = QualifiedRasterCache::<u64, u64>::default();
        cache.insert(41, &original, 101);
        assert_eq!(cache.lookup(&41, &invalid, true), None, "{name}");
        cache.insert(41, &invalid, 202);
        assert_eq!(cache.lookup(&41, &invalid, true), None, "{name}");
    }
}
