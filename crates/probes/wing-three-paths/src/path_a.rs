// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Path A: retained planar fragments. Ground as depth strips (one fragment
//! per chunk and x+y+z key, per-cell faces), parts as fragments of greedy
//! faces relative to the projected body origin. Placement is the six-cell
//! affine; a turn or an edit is new content through `update_fragment`.

use std::collections::BTreeMap;
use std::time::Instant;

use glam::Vec3;
use netrender::scene::{SceneFragment, Transform};
use netrender::{ColorLoad, Renderer, Scene, SceneClip, SceneLayer};

use crate::project::{BACKGROUND, Face, View, chunk_cell_faces, mesh_part, part_faces, world_solid};
use crate::world::{Changes, World};

#[derive(Default, Debug, Clone, Copy)]
pub struct Invalidation {
    pub relower_calls: usize,
    pub relowered_rects: usize,
    pub registered: usize,
    pub registered_rects: usize,
}

const OUTSET_PX: f32 = 0.35;

pub struct PathA {
    pub renderer: Renderer,
    /// (chunk index, key) -> (fragment id, rect count)
    strips: BTreeMap<(usize, i32), (u64, usize)>,
    /// (body, part) -> (fragment id, rect count, generation, yaw_deg, yaw)
    parts: BTreeMap<(usize, usize), (u64, usize)>,
    pub inv: Invalidation,
    pub rects_total: usize,
}

/// Face-local rectangle under a per-face affine; `origin` is subtracted so
/// the fragment is relative to a placement point.
fn face_transform(view: &View, f: &Face, origin: Vec3) -> Transform {
    let p = f.corners.map(|c| view.project(c) - origin);
    let e1 = p[1] - p[0];
    let e3 = p[3] - p[0];
    let mut t = Transform::IDENTITY;
    t.m[0] = e1.x;
    t.m[1] = e1.y;
    t.m[4] = e3.x;
    t.m[5] = e3.y;
    t.m[12] = p[0].x;
    t.m[13] = p[0].y;
    t
}

fn fragment_of(view: &View, faces: &[Face], origin: Vec3) -> SceneFragment {
    let mut s = Scene::new(view.w, view.h);
    // Painter order inside the fragment: farther first.
    let mut order: Vec<&Face> = faces.iter().collect();
    order.sort_by(|a, b| {
        let da: f32 = a.corners.iter().map(|c| view.project(*c).z).sum();
        let db: f32 = b.corners.iter().map(|c| view.project(*c).z).sum();
        db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
    });
    for f in order {
        let t = face_transform(view, f, origin);
        // Outset each face by a third of a pixel so adjacent antialiased
        // edges do not leave a seam of background between two faces.
        let eu = OUTSET_PX / (t.m[0].hypot(t.m[1])).max(1e-3);
        let ev = OUTSET_PX / (t.m[4].hypot(t.m[5])).max(1e-3);
        let tid = s.push_transform(t);
        s.push_rect_transformed(-eu, -ev, 1.0 + eu, 1.0 + ev, [f.color[0], f.color[1], f.color[2], 1.0], tid);
    }
    SceneFragment::from_scene(s)
}

impl PathA {
    pub fn new(renderer: Renderer) -> Self {
        Self {
            renderer,
            strips: BTreeMap::new(),
            parts: BTreeMap::new(),
            inv: Invalidation::default(),
            rects_total: 0,
        }
    }

    fn build_chunk(&mut self, view: &View, world: &World, ci: usize) {
        let solid = world_solid(world);
        let faces = chunk_cell_faces(view, &world.chunks[ci], &solid);
        let mut by_key: BTreeMap<i32, Vec<Face>> = BTreeMap::new();
        for f in faces {
            by_key.entry(f.key).or_default().push(f);
        }
        for (key, faces) in by_key {
            let frag = fragment_of(view, &faces, Vec3::ZERO);
            let n = faces.len();
            match self.strips.get(&(ci, key)) {
                Some(&(id, old_n)) => {
                    self.renderer.update_fragment(id, frag);
                    self.inv.relower_calls += 1;
                    self.inv.relowered_rects += n;
                    self.rects_total = self.rects_total + n - old_n;
                    self.strips.insert((ci, key), (id, n));
                }
                None => {
                    let id = self.renderer.register_fragment(frag).expect("vello enabled");
                    self.inv.registered += 1;
                    self.inv.registered_rects += n;
                    self.rects_total += n;
                    self.strips.insert((ci, key), (id, n));
                }
            }
        }
    }

    fn build_part(&mut self, view: &View, world: &World, bi: usize, pi: usize) {
        let body = &world.bodies[bi];
        let part = &body.parts[pi];
        let mesh = mesh_part(part);
        let faces = part_faces(view, body, part, &mesh);
        let origin = view.project(body.origin);
        let frag = fragment_of(view, &faces, origin);
        let n = faces.len();
        match self.parts.get(&(bi, pi)) {
            Some(&(id, old_n)) => {
                self.renderer.update_fragment(id, frag);
                self.inv.relower_calls += 1;
                self.inv.relowered_rects += n;
                self.rects_total = self.rects_total + n - old_n;
                self.parts.insert((bi, pi), (id, n));
            }
            None => {
                let id = self.renderer.register_fragment(frag).expect("vello enabled");
                self.inv.registered += 1;
                self.inv.registered_rects += n;
                self.rects_total += n;
                self.parts.insert((bi, pi), (id, n));
            }
        }
    }

    /// Apply changes, build the frame's scene, render it. Returns
    /// (build ms, encode ms, poll ms).
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
        if first {
            for ci in 0..world.chunks.len() {
                self.build_chunk(view, world, ci);
            }
            for bi in 0..world.bodies.len() {
                for pi in 0..world.bodies[bi].parts.len() {
                    self.build_part(view, world, bi, pi);
                }
            }
        } else {
            for &ci in &changes.chunks {
                self.build_chunk(view, world, ci);
            }
            for &(bi, pi) in &changes.parts {
                self.build_part(view, world, bi, pi);
            }
            for &bi in &changes.turned {
                for pi in 0..world.bodies[bi].parts.len() {
                    self.build_part(view, world, bi, pi);
                }
            }
        }

        let mut scene = Scene::new(view.w, view.h);
        scene.push_rect(0.0, 0.0, view.w as f32, view.h as f32, [BACKGROUND[0], BACKGROUND[1], BACKGROUND[2], 1.0]);
        if let Some(c) = clip {
            scene.push_layer(SceneLayer::clip(SceneClip::Rect {
                rect: c,
                radii: [0.0; 4],
            }));
        }
        // Painter order across units: by key, strips before parts on a tie.
        let mut placements: Vec<(i32, u8, u64, Transform)> = Vec::new();
        for (&(_, key), &(id, _)) in &self.strips {
            placements.push((key, 0, id, Transform::IDENTITY));
        }
        for (&(bi, _), &(id, _)) in &self.parts {
            let body = &world.bodies[bi];
            let o = view.project(body.origin);
            placements.push((body.key(), 1, id, Transform::translate_2d(o.x, o.y)));
        }
        placements.sort_by_key(|p| (p.0, p.1));
        for (_, _, id, t) in placements {
            scene.place_fragment(id, t);
        }
        if clip.is_some() {
            scene.pop_layer();
        }
        let build = crate::gpu::ms(t0.elapsed());

        let t1 = Instant::now();
        self.renderer.render_vello(&scene, target, ColorLoad::Clear(wgpu::Color::BLACK));
        let encode = crate::gpu::ms(t1.elapsed());
        let t2 = Instant::now();
        device.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
        (build, encode, crate::gpu::ms(t2.elapsed()))
    }
}
