// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use mesocosm_core::PartId;
use mesocosm_mesh::Volume;

fn specimen() -> Body {
    let mut world = crate::world::build(crate::world::Case::Crossing);
    let mut body = world.bodies.remove(0);
    body.parts.truncate(1);
    let part = &mut body.parts[0];
    let cells = vec![3; 64];
    part.reference = crate::world::reference_of([4, 4, 4], &cells);
    part.volume = Volume::new([4, 4, 4], cells).unwrap();
    part.size = part.volume.size;
    body
}

#[test]
fn equal_content_shares_images_across_bodies_origins_and_attachments() {
    let view = View::new(320, 180);
    let mut cache = PartCache::default();
    let mut inv = Invalidation::default();
    let mut next_key = 1;
    let mut first = None;
    for i in 0..100 {
        let mut body = specimen();
        body.origin = Vec3::new(i as f32 * 0.137, 2.0, i as f32 * -0.219);
        let part = &mut body.parts[0];
        part.id = PartId(i);
        part.generation = i as u64;
        part.pivot = [i as i32 % 3, 1, 2];
        part.pivot_at = [i as i32, 7, -3];
        let key = cache.ensure(&view, &body, &body.parts[0], &mut next_key, &mut inv);
        assert_eq!(*first.get_or_insert(key), key);
    }
    assert_eq!(inv.cache_hits, 99);
    assert_eq!(inv.cache_misses, 1);
    assert_eq!(inv.part_cache_entries, 1);
    assert_eq!(inv.images, 1);
    assert_eq!(next_key, 2);
    let sprite = &cache.sprites[&first.unwrap()];
    assert_eq!(inv.bytes, (sprite.w * sprite.h * 4) as usize);

    let mut body = specimen();
    body.parts[0].yaw = Yaw::Quarter;
    let quarter = cache.ensure(&view, &body, &body.parts[0], &mut next_key, &mut inv);
    assert_ne!(quarter, first.unwrap());
    assert_eq!(inv.cache_misses, 2);

    // Opposite whole-body and part turns compose to the existing zero-facing
    // image. Attachment placement still follows the whole-body turn.
    body.yaw_deg = -90.0;
    let zero = cache.ensure(&view, &body, &body.parts[0], &mut next_key, &mut inv);
    assert_eq!(zero, first.unwrap());
    assert_eq!(inv.cache_misses, 2);

    // Material IDs are part of content, and dimensions disambiguate equal
    // voxel byte strings interpreted as different geometry.
    let part = &mut body.parts[0];
    let cells = vec![8; 64];
    part.reference = crate::world::reference_of([4, 4, 4], &cells);
    part.volume = Volume::new([4, 4, 4], cells.clone()).unwrap();
    let changed = cache.ensure(&view, &body, &body.parts[0], &mut next_key, &mut inv);
    assert_ne!(changed, zero);
    body.parts[0].reference = crate::world::reference_of([2, 4, 8], &cells);
    body.parts[0].volume = Volume::new([2, 4, 8], cells).unwrap();
    body.parts[0].size = [2, 4, 8];
    let resized = cache.ensure(&view, &body, &body.parts[0], &mut next_key, &mut inv);
    assert_ne!(resized, changed);
    assert_eq!(inv.cache_misses, 4);
    assert_eq!(inv.images, 4);
    assert_eq!(inv.part_cache_entries, 4);
    assert_eq!(
        inv.bytes,
        cache
            .sprites
            .values()
            .map(|s| (s.w * s.h * 4) as usize)
            .sum::<usize>()
    );
}

#[test]
fn cached_local_geometry_and_rotated_attachment_match_quantized_world_pose() {
    let view = View::new(320, 180);
    let local_view = canonical_view(&view);
    assert_eq!(local_view.project(Vec3::ZERO), Vec3::ZERO);
    let mut body = specimen();
    body.origin = Vec3::new(3.125, 2.25, -7.375);
    body.parts[0].pivot = [2, 1, 3];
    body.parts[0].pivot_at = [8, 7, -5];
    for body_turn in [0.0, 90.0, 180.0, 270.0, -90.0] {
        body.yaw_deg = body_turn;
        for part_turn in [Yaw::Zero, Yaw::Quarter, Yaw::Half, Yaw::ThreeQuarter] {
            body.parts[0].yaw = part_turn;
            let part = &body.parts[0];
            for point in [[0, 0, 0], [1, 2, 3], [4, 4, 4]] {
                let rotated =
                    Vec3::from_array(body.quantized_yaw(part).rotate(point).map(|v| v as f32));
                let placed = rotated + part_origin(&body, part);
                let expected = body.to_world(part, point);
                assert!(placed.abs_diff_eq(expected, 0.000_01));
                let cached_screen =
                    local_view.project(rotated) + view.project(part_origin(&body, part));
                assert!(cached_screen.abs_diff_eq(view.project(expected), 0.000_1));
            }
        }
    }
}

#[test]
fn camera_translation_reuses_canonical_pixels_but_projection_changes_miss() {
    let mut view = View::new(320, 180);
    let body = specimen();
    let mut cache = PartCache::default();
    let mut inv = Invalidation::default();
    let mut next_key = 1;
    let first = cache.ensure(&view, &body, &body.parts[0], &mut next_key, &mut inv);
    view.clip_from_world.w_axis.x += 0.137;
    view.clip_from_world.w_axis.y -= 0.293;
    assert_eq!(
        cache.ensure(&view, &body, &body.parts[0], &mut next_key, &mut inv),
        first
    );
    view.clip_from_world.x_axis *= 1.25;
    assert_ne!(
        cache.ensure(&view, &body, &body.parts[0], &mut next_key, &mut inv),
        first
    );
    view.w += 20;
    let _ = cache.ensure(&view, &body, &body.parts[0], &mut next_key, &mut inv);
    assert_eq!(inv.cache_hits, 1);
    assert_eq!(inv.cache_misses, 3);
    assert_eq!(inv.part_cache_entries, 3);
}
