// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

struct CpuSource;
impl TextureProducer for CpuSource {
    fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
        None
    }
}

fn draw(key: u64, stamp: u64) -> PaintCmd {
    PaintCmd::DrawExternalTexture(ExternalTextureItem {
        placement: CommonPlacement::new(LayoutRect::new(
            LayoutPoint::new(0.0, 0.0),
            LayoutPoint::new(8.0, 6.0),
        )),
        texture_key: key,
        content_generation: Some(stamp),
        opacity: 0.73,
    })
}

fn translated(commands: &[PaintCmd]) -> paint_list_render::TranslatedDisplayList {
    paint_list_render::translate_paint_cmd_stream(
        paint_list_api::DeviceIntSize::new(8, 6),
        commands,
        &[],
        &[],
    )
    .expect("producer fixture translation")
}

fn keys(display: &paint_list_render::TranslatedDisplayList) -> Vec<u64> {
    display
        .scene
        .ops
        .iter()
        .filter_map(|op| match op {
            netrender::SceneOp::Image(image) => Some(image.key),
            netrender::SceneOp::Pattern(pattern) => Some(pattern.tile),
            _ => None,
        })
        .collect()
}

#[test]
fn physical_identity_is_distinct_stable_and_never_recycled_for_same_local_key() {
    let mut first = ProducerRegistry::new();
    let mut second = ProducerRegistry::new();
    first.register(7, CpuSource, &[]).unwrap();
    second.register(7, CpuSource, &[]).unwrap();
    let first_key = first.entries[&7].renderer_key;
    let second_key = second.entries[&7].renderer_key;
    assert_ne!(first_key, second_key);
    assert!(first_key >= (netrender::EXTERNAL_IMAGE_KEY_BASE | PHYSICAL_PRODUCER_BASE));
    first.suspend_all(None);
    assert_eq!(first.entries[&7].renderer_key, first_key);
    assert_eq!(second.entries[&7].renderer_key, second_key);
    assert!(first.remove(7));
    assert_eq!(first.pending_retire, [first_key]);
    first.register(7, CpuSource, &[]).unwrap();
    let rebound_key = first.entries[&7].renderer_key;
    assert_ne!(rebound_key, first_key);
    assert_ne!(rebound_key, second_key);
    // Dropping the first registry cannot retire or replace a registration
    // belonging to the second; actual GPU cleanup is qualified separately.
    drop(first);
    assert!(second.contains(7));
    assert_eq!(second.entries[&7].renderer_key, second_key);
}

#[test]
fn physical_identity_exhaustion_is_typed_and_never_wraps() {
    let next = AtomicU64::new(PHYSICAL_PRODUCER_END - 1);
    assert_eq!(
        allocate_renderer_key(&next),
        Ok(SHADOW_MASK_IMAGE_KEY_BASE - 1)
    );
    for _ in 0..2 {
        assert_eq!(
            allocate_renderer_key(&next),
            Err(ProducerRegistrationError::IdentityExhausted)
        );
        assert_eq!(next.load(Ordering::Relaxed), PHYSICAL_PRODUCER_END);
    }
}

#[test]
fn equal_local_keys_stamps_and_extents_remap_to_distinct_physical_resources() {
    let mut first = ProducerRegistry::new();
    let mut second = ProducerRegistry::new();
    first.register(7, CpuSource, &[]).unwrap();
    second.register(7, CpuSource, &[]).unwrap();
    let mut first_display = translated(&[draw(7, 1)]);
    let mut second_display = translated(&[draw(7, 1)]);
    remap_gpu_display_list(&mut first_display, |_| {
        Ok(GpuBinding {
            renderer_key: first.entries[&7].renderer_key,
            stamp: 1,
        })
    })
    .unwrap();
    remap_gpu_display_list(&mut second_display, |_| {
        Ok(GpuBinding {
            renderer_key: second.entries[&7].renderer_key,
            stamp: 1,
        })
    })
    .unwrap();
    assert_ne!(keys(&first_display), keys(&second_display));
    assert_eq!(first_display.external_textures[0].texture_key, 7);
    assert_eq!(second_display.external_textures[0].texture_key, 7);
    assert_eq!(
        first_display.external_textures[0].content_generation,
        Some(1)
    );
    assert_eq!(
        second_display.external_textures[0].content_generation,
        Some(1)
    );
    assert_eq!(
        (
            first_display.scene.viewport_width,
            first_display.scene.viewport_height
        ),
        (
            second_display.scene.viewport_width,
            second_display.scene.viewport_height
        )
    );
}

#[test]
fn authored_high_local_key_is_remapped_with_local_command_identity_preserved() {
    let local = LOCAL_PRODUCER_END - 1;
    let mut registry = ProducerRegistry::new();
    registry.register(local, CpuSource, &[]).unwrap();
    let physical = registry.entries[&local].renderer_key;
    let commands = [draw(local, 1)];
    let mut display = translated(&commands);
    display.scene.push_pattern(
        netrender::external_image_key(local),
        [0.0, 0.0, 8.0, 6.0],
        [1.0, 1.0],
    );
    assert_eq!(keys(&display), [u64::MAX, u64::MAX]);
    // Pure production remapping consumes copied binding metadata; this CPU
    // control does not claim an actual GPU view was staged.
    remap_gpu_display_list(&mut display, |key| {
        assert_eq!(key, local);
        Ok(GpuBinding {
            renderer_key: physical,
            stamp: 1,
        })
    })
    .unwrap();
    assert_eq!(keys(&display), [physical, physical]);
    assert_ne!(physical, u64::MAX);
    assert_eq!(display.external_textures[0].texture_key, local);
    assert_eq!(display.external_textures[0].content_generation, Some(1));
    assert_eq!(display.external_textures[0].image_key, physical);
    assert!(
        matches!(&commands[0], PaintCmd::DrawExternalTexture(item) if item.texture_key == local)
    );
    assert_eq!(display.scene.ops.len(), 2);
}

#[test]
fn missing_late_binding_refuses_without_partial_scene_or_sidechannel_rewrite() {
    let mut display = translated(&[draw(7, 1), draw(8, 1)]);
    let before = keys(&display);
    let mut registry = ProducerRegistry::new();
    registry.register(7, CpuSource, &[]).unwrap();
    let physical = registry.entries[&7].renderer_key;
    let result = remap_gpu_display_list(&mut display, |key| {
        if key == 7 {
            Ok(GpuBinding {
                renderer_key: physical,
                stamp: 1,
            })
        } else {
            Err(ProducerGpuRemapError::MissingProducer { local_key: key })
        }
    });
    assert_eq!(
        result,
        Err(ProducerGpuRemapError::MissingProducer { local_key: 8 })
    );
    assert_eq!(keys(&display), before);
    assert_eq!(
        display
            .external_textures
            .iter()
            .map(|draw| draw.image_key)
            .collect::<Vec<_>>(),
        before
    );
    assert!(registry.pending_retire.is_empty());
    assert_eq!(registry.next_stamp, 0);
}

#[test]
fn actual_registry_refuses_unstaged_and_unknown_local_sources() {
    let mut registry = ProducerRegistry::new();
    registry.register(7, CpuSource, &[]).unwrap();
    let mut known = translated(&[draw(7, 1)]);
    assert_eq!(
        registry.remap_gpu_display_list(&mut known),
        Err(ProducerGpuRemapError::UnstagedProducer { local_key: 7 })
    );
    let mut missing = translated(&[draw(8, 1)]);
    assert_eq!(
        registry.remap_gpu_display_list(&mut missing),
        Err(ProducerGpuRemapError::MissingProducer { local_key: 8 })
    );
    assert_eq!(keys(&known), [netrender::external_image_key(7)]);
    assert_eq!(keys(&missing), [netrender::external_image_key(8)]);
}

#[test]
fn stale_generation_and_foreign_inline_payload_are_typed_refusals() {
    let physical = allocate_renderer_key(&AtomicU64::new(PHYSICAL_PRODUCER_BASE)).unwrap();
    let mut stale = translated(&[draw(7, 3)]);
    assert_eq!(
        remap_gpu_display_list(&mut stale, |_| Ok(GpuBinding {
            renderer_key: physical,
            stamp: 4
        })),
        Err(ProducerGpuRemapError::StaleGeneration {
            local_key: 7,
            expected: 4,
            actual: Some(3)
        })
    );
    assert_eq!(keys(&stale), [netrender::external_image_key(7)]);
    let mut foreign = translated(&[]);
    foreign.scene.image_sources.insert(
        physical,
        netrender::ImageData::from_bytes(1, 1, vec![0, 0, 0, 0]),
    );
    assert_eq!(
        remap_gpu_display_list(&mut foreign, |_| unreachable!(
            "reserved inline data must refuse before lookup"
        )),
        Err(ProducerGpuRemapError::ForeignInlineImage {
            renderer_key: physical
        })
    );
}
