// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Path B: cached sprites. One RGBA image per ground chunk and one per
//! part per quarter-turn facing, baked by the probe's own rasterizer under
//! the shared camera so all three paths share one projection. A content
//! change or a new facing is a new image key, uploaded once by netrender.
//! This path still quantizes body yaw; it is not continuous-yaw equivalent.

use std::collections::BTreeMap;
use std::time::Instant;

use glam::{Mat4, Vec3, Vec4};
use mesocosm_core::Yaw;
use netrender::{ColorLoad, ImageData, Renderer, Scene, SceneClip, SceneLayer};

use crate::project::{
    BACKGROUND, Face, Raster, View, chunk_greedy_faces, mesh_part, shaded, to_screen,
};
use crate::world::{Body, Changes, Part, World};

#[derive(Default, Debug, Clone, Copy)]
pub struct Invalidation {
    pub images: usize,
    pub bytes: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub part_cache_entries: usize,
}

#[cfg(test)]
#[path = "path_b_tests.rs"]
mod tests;

/// The linear projection and culling direction affect baked pixels. Camera
/// translation only changes placement; baking always uses the same pixel phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ProjectionKey {
    size: [u32; 2],
    linear: [u32; 12],
    to_camera: [u32; 3],
}

impl ProjectionKey {
    fn of(view: &View) -> Self {
        let matrix = view.clip_from_world.to_cols_array();
        Self {
            size: [view.w, view.h],
            linear: std::array::from_fn(|i| matrix[i].to_bits()),
            to_camera: view.to_camera().to_array().map(f32::to_bits),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PartKey {
    /// The volume address includes occupancy and per-voxel material IDs.
    content: [u8; 32],
    /// Keep rasterized dimensions explicit alongside the dimension-aware hash.
    size: [u32; 3],
    facing: u8,
    projection: ProjectionKey,
}

impl PartKey {
    fn of(view: &View, body: &Body, part: &Part) -> Self {
        Self {
            content: part.reference.0,
            size: part.volume.size,
            facing: facing_of(body, part),
            projection: ProjectionKey::of(view),
        }
    }
}

#[derive(Default)]
struct PartCache {
    sprites: BTreeMap<PartKey, Sprite>,
}

impl PartCache {
    fn ensure(
        &mut self,
        view: &View,
        body: &Body,
        part: &Part,
        next_key: &mut u64,
        inv: &mut Invalidation,
    ) -> PartKey {
        let id = PartKey::of(view, body, part);
        if self.sprites.contains_key(&id) {
            inv.cache_hits += 1;
        } else {
            let view = canonical_view(view);
            let faces = canonical_part_faces(&view, part, body.quantized_yaw(part));
            let sprite = bake(&view, &faces, Vec3::ZERO, *next_key);
            *next_key += 1;
            inv.images += 1;
            inv.bytes += (sprite.w * sprite.h * 4) as usize;
            inv.cache_misses += 1;
            self.sprites.insert(id, sprite);
            inv.part_cache_entries = self.sprites.len();
        }
        id
    }
}

struct Sprite {
    key: u64,
    data: ImageData,
    /// Screen offset of the image's top-left from the placement point.
    dx: f32,
    dy: f32,
    w: u32,
    h: u32,
}

pub struct PathB {
    pub renderer: Renderer,
    chunks: BTreeMap<usize, Sprite>,
    parts: PartCache,
    chunk_projection: Option<(ProjectionKey, [u32; 4])>,
    next_key: u64,
    pub inv: Invalidation,
}

fn facing_of(body: &Body, part: &Part) -> u8 {
    match body.quantized_yaw(part) {
        Yaw::Zero => 0,
        Yaw::Quarter => 1,
        Yaw::Half => 2,
        Yaw::ThreeQuarter => 3,
    }
}

fn whole_body_yaw(body: &Body) -> Yaw {
    match ((body.yaw_deg / 90.0).round() as i32).rem_euclid(4) {
        0 => Yaw::Zero,
        1 => Yaw::Quarter,
        2 => Yaw::Half,
        _ => Yaw::ThreeQuarter,
    }
}

/// Canonical local zero lands exactly on screen pixel zero. In particular,
/// a body's subpixel origin never changes the cached image's raster phase.
fn canonical_view(view: &View) -> View {
    let matrix = view.clip_from_world;
    View {
        w: view.w,
        h: view.h,
        camera: view.camera,
        clip_from_world: Mat4::from_cols(
            matrix.x_axis,
            matrix.y_axis,
            matrix.z_axis,
            Vec4::new(-1.0, 1.0, 0.0, 1.0),
        ),
    }
}

/// Geometry and the probe's fixed material/shading function are the only
/// appearance inputs. Pivots and attachment positions are draw-time placement.
fn canonical_part_faces(view: &View, part: &Part, yaw: Yaw) -> Vec<Face> {
    mesh_part(part)
        .quads
        .iter()
        .filter_map(|q| {
            let mut normal = [0; 3];
            normal[q.axis as usize] = if q.positive { 1 } else { -1 };
            let normal = Vec3::from_array(yaw.rotate(normal).map(|v| v as f32));
            (normal.dot(view.to_camera()) > 0.0).then(|| Face {
                corners: q
                    .corners()
                    .map(|p| Vec3::from_array(yaw.rotate(p).map(|v| v as f32))),
                color: shaded(q.material, q.axis, q.positive),
                key: 0,
            })
        })
        .collect()
}

/// R_body * (R_part * (local - pivot) + pivot_at) + body.origin.
/// The cached image supplies R_body * R_part * local; rotate the complete
/// attachment offset here so a body turn moves its off-axis parts together.
fn part_origin(body: &Body, part: &Part) -> Vec3 {
    let pivot = part.yaw.rotate(part.pivot);
    let offset = std::array::from_fn(|i| part.pivot_at[i] - pivot[i]);
    body.origin + Vec3::from_array(whole_body_yaw(body).rotate(offset).map(|v| v as f32))
}

fn chunk_projection(view: &View) -> (ProjectionKey, [u32; 4]) {
    (
        ProjectionKey::of(view),
        view.clip_from_world.w_axis.to_array().map(f32::to_bits),
    )
}

/// Bake faces into an image tight to their screen bounds; `origin` is the
/// screen point the sprite is later placed at.
fn bake(view: &View, faces: &[Face], origin: Vec3, key: u64) -> Sprite {
    let screen: Vec<_> = faces.iter().map(|f| to_screen(view, f)).collect();
    let mut x0 = f32::INFINITY;
    let mut y0 = f32::INFINITY;
    let mut x1 = f32::NEG_INFINITY;
    let mut y1 = f32::NEG_INFINITY;
    for s in &screen {
        for p in &s.p {
            x0 = x0.min(p.x);
            y0 = y0.min(p.y);
            x1 = x1.max(p.x);
            y1 = y1.max(p.y);
        }
    }
    if screen.is_empty() {
        return Sprite {
            key,
            data: ImageData::from_bytes(1, 1, vec![0; 4]),
            dx: 0.0,
            dy: 0.0,
            w: 1,
            h: 1,
        };
    }
    let (ox, oy) = (x0.floor(), y0.floor());
    let w = ((x1 - ox).ceil() as u32).max(1);
    let h = ((y1 - oy).ceil() as u32).max(1);
    let mut r = Raster::new(w, h, None);
    for s in &screen {
        r.face(s, (ox, oy));
    }
    Sprite {
        key,
        data: ImageData::from_bytes(w, h, r.rgba),
        dx: ox - origin.x,
        dy: oy - origin.y,
        w,
        h,
    }
}

impl PathB {
    pub fn new(renderer: Renderer) -> Self {
        Self {
            renderer,
            chunks: BTreeMap::new(),
            parts: PartCache::default(),
            chunk_projection: None,
            next_key: 1,
            inv: Invalidation::default(),
        }
    }

    fn key(&mut self) -> u64 {
        let k = self.next_key;
        self.next_key += 1;
        k
    }

    fn bake_chunk(&mut self, view: &View, world: &World, ci: usize) {
        let faces = chunk_greedy_faces(view, &world.chunks[ci]);
        let key = self.key();
        let s = bake(view, &faces, Vec3::ZERO, key);
        self.inv.images += 1;
        self.inv.bytes += (s.w * s.h * 4) as usize;
        self.chunks.insert(ci, s);
    }

    pub fn frame(
        &mut self,
        view: &View,
        world: &World,
        changes: &Changes,
        first: bool,
        clip: Option<[f32; 4]>,
        target: &wgpu::TextureView,
        device: &wgpu::Device,
    ) -> (f64, f64, f64) {
        let t0 = Instant::now();
        let projection = chunk_projection(view);
        if first || self.chunk_projection != Some(projection) {
            for ci in 0..world.chunks.len() {
                self.bake_chunk(view, world, ci);
            }
            self.chunk_projection = Some(projection);
        } else {
            for &ci in &changes.chunks {
                self.bake_chunk(view, world, ci);
            }
        }
        let mut placements: Vec<(i32, u8, usize, usize, Option<PartKey>)> = Vec::new();
        for ci in 0..world.chunks.len() {
            placements.push((i32::MIN, 0, ci, 0, None));
        }
        for bi in 0..world.bodies.len() {
            for pi in 0..world.bodies[bi].parts.len() {
                let body = &world.bodies[bi];
                let id = self.parts.ensure(
                    view,
                    body,
                    &body.parts[pi],
                    &mut self.next_key,
                    &mut self.inv,
                );
                placements.push((world.bodies[bi].key(), 1, bi, pi, Some(id)));
            }
        }
        // Chunk sprites first (a whole chunk is one picture, so it cannot
        // interleave with bodies at all), then parts by key.
        placements.sort_by_key(|p| (p.0, p.1));

        let mut scene = Scene::new(view.w, view.h);
        scene.push_rect(
            0.0,
            0.0,
            view.w as f32,
            view.h as f32,
            [BACKGROUND[0], BACKGROUND[1], BACKGROUND[2], 1.0],
        );
        if let Some(c) = clip {
            scene.push_layer(SceneLayer::clip(SceneClip::Rect {
                rect: c,
                radii: [0.0; 4],
            }));
        }
        for (_, kind, a, b, id) in placements {
            let (s, origin) = if kind == 0 {
                (&self.chunks[&a], Vec3::ZERO)
            } else {
                let id = id.unwrap();
                let body = &world.bodies[a];
                (
                    &self.parts.sprites[&id],
                    view.project(part_origin(body, &body.parts[b])),
                )
            };
            let x0 = (origin.x + s.dx).round();
            let y0 = (origin.y + s.dy).round();
            scene.push_image(
                x0,
                y0,
                x0 + s.w as f32,
                y0 + s.h as f32,
                s.key,
                s.data.clone(),
            );
        }
        if clip.is_some() {
            scene.pop_layer();
        }
        let build = crate::gpu::ms(t0.elapsed());
        let t1 = Instant::now();
        self.renderer
            .render_vello(&scene, target, ColorLoad::Clear(wgpu::Color::BLACK));
        let encode = crate::gpu::ms(t1.elapsed());
        let t2 = Instant::now();
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("poll");
        (build, encode, crate::gpu::ms(t2.elapsed()))
    }
}
