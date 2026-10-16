// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Focused producer-namespace controls for translator-owned blurred shadows.
use super::*;
use paint_list_api::{
    BorderRadius, BoxShadowClipMode, ClipKind, ClipSpec, ColorF, LayerSpec, LayoutVector2D,
    ShadowItem,
};

fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> LayoutRect {
    LayoutRect::new(LayoutPoint::new(x0, y0), LayoutPoint::new(x1, y1))
}

fn shadow() -> PaintCmd {
    PaintCmd::DrawShadow(ShadowItem {
        placement: CommonPlacement::new(rect(1.0, 2.0, 7.0, 8.0)),
        box_bounds: rect(1.0, 2.0, 7.0, 8.0),
        offset: LayoutVector2D::new(2.0, 3.0),
        color: ColorF {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 0.7,
        },
        blur_radius: 4.0,
        spread_radius: 1.0,
        border_radius: BorderRadius::zero(),
        clip_mode: BoxShadowClipMode::Outset,
    })
}

fn draw(key: u64) -> PaintCmd {
    PaintCmd::DrawExternalTexture(ExternalTextureItem {
        placement: CommonPlacement::new(rect(0.0, 0.0, 8.0, 6.0)),
        texture_key: key,
        content_generation: Some(9),
        opacity: 0.73,
    })
}

fn translated(commands: &[PaintCmd]) -> paint_list_render::TranslatedDisplayList {
    paint_list_render::translate_paint_cmd_stream(
        paint_list_api::DeviceIntSize::new(32, 24),
        commands,
        &[],
        &[],
    )
}

fn op_keys(display: &paint_list_render::TranslatedDisplayList) -> Vec<(char, u64)> {
    display
        .scene
        .ops
        .iter()
        .filter_map(|op| match op {
            netrender::SceneOp::Image(image) => Some(('i', image.key)),
            netrender::SceneOp::Pattern(pattern) => Some(('p', pattern.tile)),
            _ => None,
        })
        .collect()
}

fn image_geometry(
    display: &paint_list_render::TranslatedDisplayList,
) -> Vec<(f32, f32, f32, f32, [f32; 4], u32)> {
    display
        .scene
        .ops
        .iter()
        .filter_map(|op| match op {
            netrender::SceneOp::Image(image) => Some((
                image.x0,
                image.y0,
                image.x1,
                image.y1,
                image.uv,
                image.transform_id,
            )),
            _ => None,
        })
        .collect()
}

fn metadata(
    display: &paint_list_render::TranslatedDisplayList,
) -> Vec<(u64, Option<u64>, u64, usize)> {
    display
        .external_textures
        .iter()
        .map(|draw| {
            (
                draw.texture_key,
                draw.content_generation,
                draw.image_key,
                draw.scene_op_boundary,
            )
        })
        .collect()
}

fn binding(renderer_key: u64) -> impl Fn(u64) -> Result<GpuBinding, ProducerGpuRemapError> {
    move |local_key| {
        assert_eq!(local_key, 7);
        Ok(GpuBinding {
            renderer_key,
            stamp: 9,
        })
    }
}

#[test]
fn blurred_shadow_without_producers_is_accepted_and_mask_key_is_preserved() {
    let mut display = translated(&[shadow()]);
    assert_eq!(display.box_shadow_masks.len(), 1);
    let raw_key = display.box_shadow_masks[0].key;
    assert_eq!(raw_key, SHADOW_MASK_IMAGE_KEY_BASE);

    remap_gpu_display_list(&mut display, |_| {
        panic!("shadow-mask ownership must not trigger producer lookup")
    })
    .expect("a translator-owned shadow mask is not a foreign inline image");

    assert_eq!(display.box_shadow_masks[0].key, raw_key);
    assert_eq!(op_keys(&display), [('i', raw_key)]);
}

#[test]
fn shadow_and_producer_keep_clip_opacity_order_geometry_and_distinct_keys() {
    let commands = [
        PaintCmd::PushClip(ClipSpec {
            kind: ClipKind::Rect(rect(0.0, 0.0, 18.0, 16.0)),
        }),
        PaintCmd::PushLayer(LayerSpec {
            opacity: 0.42,
            ..LayerSpec::default()
        }),
        shadow(),
        draw(7),
        PaintCmd::PopLayer,
        PaintCmd::PopClip,
    ];
    let mut display = translated(&commands);
    assert_eq!(display.box_shadow_masks.len(), 1);
    assert_eq!(display.external_textures.len(), 1);
    let old_mask = display.box_shadow_masks[0].clone();
    let old_draw = display.external_textures[0].clone();
    let old_image_geometry = image_geometry(&display);
    let renderer_key = allocate_renderer_key(&AtomicU64::new(PHYSICAL_PRODUCER_BASE)).unwrap();

    remap_gpu_display_list(&mut display, binding(renderer_key)).unwrap();

    assert_eq!(display.box_shadow_masks[0].key, old_mask.key);
    assert_eq!(display.box_shadow_masks[0].dim, old_mask.dim);
    assert_eq!(display.box_shadow_masks[0].bounds, old_mask.bounds);
    assert_eq!(
        display.box_shadow_masks[0].blur_radius_px,
        old_mask.blur_radius_px
    );
    assert_eq!(display.box_shadow_masks[0].invert, old_mask.invert);
    assert_eq!(
        display.external_textures[0].texture_key,
        old_draw.texture_key
    );
    assert_eq!(
        display.external_textures[0].content_generation,
        old_draw.content_generation
    );
    assert_eq!(
        display.external_textures[0].placement.dest_rect,
        old_draw.placement.dest_rect
    );
    assert_eq!(
        display.external_textures[0].placement.uv,
        old_draw.placement.uv
    );
    assert_eq!(
        display.external_textures[0].placement.opacity,
        old_draw.placement.opacity
    );
    assert_eq!(
        display.external_textures[0].placement.alpha,
        old_draw.placement.alpha
    );
    assert_eq!(
        display.external_textures[0].scene_op_boundary,
        old_draw.scene_op_boundary
    );
    assert_eq!(display.external_textures[0].image_key, renderer_key);
    assert_eq!(image_geometry(&display), old_image_geometry);

    let mut layers = Vec::new();
    for op in &display.scene.ops {
        if let netrender::SceneOp::PushLayer(layer) = op {
            layers.push((layer.alpha, layer.clip.clone()));
        }
    }
    assert_eq!(layers.len(), 2);
    assert_eq!(layers[0].0, 1.0);
    assert!(matches!(
        &layers[0].1,
        netrender::SceneClip::Rect { rect, .. } if *rect == [0.0, 0.0, 18.0, 16.0]
    ));
    assert_eq!(layers[1].0, 0.42);
    assert!(matches!(
        display.scene.ops.first(),
        Some(netrender::SceneOp::PushLayer(_))
    ));
    assert!(matches!(
        display.scene.ops.last(),
        Some(netrender::SceneOp::PopLayer)
    ));

    let keys = op_keys(&display);
    assert_eq!(keys.len(), 2);
    let mask_key = display.box_shadow_masks[0].key;
    assert_eq!(keys[0], ('i', mask_key));
    assert_eq!(keys[1], ('i', renderer_key));
    assert!(
        renderer_key < mask_key,
        "producer image keys stay below the reserved shadow-mask range"
    );
    assert!(old_draw.scene_op_boundary < display.scene.ops.len());
}

#[test]
fn producer_key_equal_to_raw_mask_local_bits_remaps_image_and_pattern_separately() {
    let only_shadow = translated(&[shadow()]);
    let raw_mask_key = only_shadow.box_shadow_masks[0].key;
    let local_key = raw_mask_key & ((1_u64 << 62) - 1);
    let mut display = translated(&[shadow(), draw(local_key)]);
    assert_eq!(
        display.box_shadow_masks[0].key,
        netrender::external_image_key(local_key)
    );
    display.scene.push_pattern(
        netrender::external_image_key(local_key),
        [0.0, 0.0, 24.0, 12.0],
        [2.0, 3.0],
    );
    let renderer_key = allocate_renderer_key(&AtomicU64::new(PHYSICAL_PRODUCER_BASE)).unwrap();

    remap_gpu_display_list(&mut display, |key| {
        assert_eq!(key, local_key);
        Ok(GpuBinding {
            renderer_key,
            stamp: 9,
        })
    })
    .unwrap();

    let mask_key = display.box_shadow_masks[0].key;
    assert_eq!(mask_key, raw_mask_key);
    assert_ne!(mask_key, renderer_key);
    assert!(
        renderer_key < mask_key,
        "producer image must not capture the reserved shadow key"
    );
    assert_eq!(
        op_keys(&display),
        [('i', mask_key), ('i', renderer_key), ('p', renderer_key)]
    );
    assert_eq!(display.external_textures[0].texture_key, local_key);
    assert_eq!(display.external_textures[0].image_key, renderer_key);
    assert_eq!(display.external_textures[0].content_generation, Some(9));
}

#[test]
fn late_producer_binding_failure_leaves_shadow_masks_ops_and_metadata_unchanged() {
    let mut display = translated(&[shadow(), draw(7), draw(8)]);
    let before_mask_keys = display
        .box_shadow_masks
        .iter()
        .map(|mask| mask.key)
        .collect::<Vec<_>>();
    let before_ops = op_keys(&display);
    let before_metadata = metadata(&display);
    let before_mask_bounds = display
        .box_shadow_masks
        .iter()
        .map(|mask| mask.bounds)
        .collect::<Vec<_>>();
    let first_renderer_key =
        allocate_renderer_key(&AtomicU64::new(PHYSICAL_PRODUCER_BASE)).unwrap();

    let result = remap_gpu_display_list(&mut display, |key| match key {
        7 => Ok(GpuBinding {
            renderer_key: first_renderer_key,
            stamp: 9,
        }),
        8 => Err(ProducerGpuRemapError::MissingProducer { local_key: 8 }),
        _ => unreachable!("only the two producer draws should be looked up"),
    });

    assert_eq!(
        result,
        Err(ProducerGpuRemapError::MissingProducer { local_key: 8 })
    );
    assert_eq!(
        display
            .box_shadow_masks
            .iter()
            .map(|mask| mask.key)
            .collect::<Vec<_>>(),
        before_mask_keys
    );
    assert_eq!(op_keys(&display), before_ops);
    assert_eq!(metadata(&display), before_metadata);
    assert_eq!(
        display
            .box_shadow_masks
            .iter()
            .map(|mask| mask.bounds)
            .collect::<Vec<_>>(),
        before_mask_bounds
    );
}

#[test]
fn duplicate_or_inline_colliding_shadow_mask_is_rejected_without_rewrite() {
    let mut duplicate = translated(&[shadow()]);
    duplicate
        .box_shadow_masks
        .push(duplicate.box_shadow_masks[0].clone());
    let duplicate_before = op_keys(&duplicate);
    let duplicate_mask_keys = duplicate
        .box_shadow_masks
        .iter()
        .map(|mask| mask.key)
        .collect::<Vec<_>>();
    assert!(matches!(
        remap_gpu_display_list(&mut duplicate, |_| unreachable!()),
        Err(ProducerGpuRemapError::InconsistentShadowMask { .. })
    ));
    assert_eq!(op_keys(&duplicate), duplicate_before);
    assert_eq!(
        duplicate
            .box_shadow_masks
            .iter()
            .map(|mask| mask.key)
            .collect::<Vec<_>>(),
        duplicate_mask_keys
    );

    let mut inline_collision = translated(&[shadow()]);
    let colliding_key = inline_collision.box_shadow_masks[0].key;
    inline_collision.scene.image_sources.insert(
        colliding_key,
        netrender::ImageData::from_bytes(1, 1, vec![0, 0, 0, 0]),
    );
    let collision_before = op_keys(&inline_collision);
    let collision_masks = inline_collision
        .box_shadow_masks
        .iter()
        .map(|mask| mask.key)
        .collect::<Vec<_>>();
    assert!(matches!(
        remap_gpu_display_list(&mut inline_collision, |_| unreachable!()),
        Err(ProducerGpuRemapError::InconsistentShadowMask { .. })
    ));
    assert_eq!(op_keys(&inline_collision), collision_before);
    assert_eq!(
        inline_collision
            .box_shadow_masks
            .iter()
            .map(|mask| mask.key)
            .collect::<Vec<_>>(),
        collision_masks
    );
    assert_eq!(
        inline_collision
            .scene
            .image_sources
            .keys()
            .copied()
            .collect::<Vec<_>>(),
        [colliding_key]
    );
}

#[test]
fn shadow_mask_outside_reserved_translator_range_is_rejected_without_rewrite() {
    let mut display = translated(&[shadow()]);
    display.box_shadow_masks[0].key = netrender::EXTERNAL_IMAGE_KEY_BASE - 1;
    let before_masks = display
        .box_shadow_masks
        .iter()
        .map(|mask| mask.key)
        .collect::<Vec<_>>();
    let before_ops = op_keys(&display);

    assert!(matches!(
        remap_gpu_display_list(&mut display, |_| unreachable!()),
        Err(ProducerGpuRemapError::InconsistentShadowMask { .. })
    ));
    assert_eq!(
        display
            .box_shadow_masks
            .iter()
            .map(|mask| mask.key)
            .collect::<Vec<_>>(),
        before_masks
    );
    assert_eq!(op_keys(&display), before_ops);
}
