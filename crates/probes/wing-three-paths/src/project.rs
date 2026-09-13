// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The shared camera, face extraction, and the CPU depth oracle.
//!
//! Every path projects the same world through the same `clip_from_world`;
//! the oracle rasterizes the same faces with a z-buffer so ordering errors
//! show up as pixels, not opinions.

use glam::{Mat4, Vec3, Vec4};
use mesocosm_mesh::{PartMesh, mesh_volume};
use mesocosm_render::geometry::{face_shade, material_colour};
use mesocosm_render::Camera;

use crate::world::{Body, Chunk, Part, World, CHUNK, WORLD_Y};

pub const BACKGROUND: [f32; 3] = [0.06, 0.06, 0.08];

pub struct View {
    pub w: u32,
    pub h: u32,
    pub camera: Camera,
    pub clip_from_world: Mat4,
}

impl View {
    pub fn new(w: u32, h: u32) -> Self {
        let camera = Camera {
            target: [16.0, 3.0, 16.0],
            extent: 20.0,
            yaw: std::f32::consts::FRAC_PI_4,
            pitch: 0.615_479_7,
            aspect: w as f32 / h as f32,
        };
        Self {
            w,
            h,
            camera,
            clip_from_world: camera.view_proj(),
        }
    }

    /// World point to (screen x, screen y, ndc depth).
    pub fn project(&self, p: Vec3) -> Vec3 {
        let c = self.clip_from_world * Vec4::new(p.x, p.y, p.z, 1.0);
        Vec3::new(
            (c.x + 1.0) * 0.5 * self.w as f32,
            (1.0 - c.y) * 0.5 * self.h as f32,
            c.z,
        )
    }

    pub fn to_camera(&self) -> Vec3 {
        let c = self.camera;
        Vec3::new(
            c.yaw.cos() * c.pitch.cos(),
            c.pitch.sin(),
            c.yaw.sin() * c.pitch.cos(),
        )
    }
}

/// One face in world space with its flat colour.
#[derive(Clone, Copy, Debug)]
pub struct Face {
    pub corners: [Vec3; 4],
    pub color: [f32; 3],
    /// Depth key of the owning cell (ground) or body (parts).
    pub key: i32,
}

/// The same face on screen: x, y, ndc depth per corner.
#[derive(Clone, Copy, Debug)]
pub struct ScreenFace {
    pub p: [Vec3; 4],
    pub color: [f32; 3],
}

/// Display value of a face: Mesocosm's shader encodes its linear vertex
/// colour to sRGB for an Unorm target, and Vello stores the bytes it is
/// given, so the flat paths and the oracle encode the same way here.
pub fn shaded(material: u8, axis: u8, positive: bool) -> [f32; 3] {
    let b = material_colour(material);
    let s = face_shade(axis, positive);
    [srgb(b[0] * s), srgb(b[1] * s), srgb(b[2] * s)]
}

pub fn srgb(l: f32) -> f32 {
    if l <= 0.003_130_8 { l * 12.92 } else { 1.055 * l.max(0.0).powf(1.0 / 2.4) - 0.055 }
}

/// Visible iff the face's outward normal points toward the camera. The
/// normal comes from the face's axis and sign, mapped through the same
/// placement as its corners; the mesher's corner winding is not relied on
/// (it is reversed for y faces, which a cull_mode-None pipeline never sees).
fn front_facing(view: &View, normal: Vec3) -> bool {
    normal.dot(view.to_camera()) > 0.0
}

fn axis_normal(axis: u8, positive: bool) -> [i32; 3] {
    let mut n = [0i32; 3];
    n[axis as usize] = if positive { 1 } else { -1 };
    n
}

/// Greedy-meshed faces of a part, placed by `to_world`, back faces culled.
pub fn part_faces(view: &View, body: &Body, part: &Part, mesh: &PartMesh) -> Vec<Face> {
    let key = body.key();
    let mut out = Vec::with_capacity(mesh.quads.len());
    for q in &mesh.quads {
        let corners = q.corners().map(|c| body.to_world(part, c));
        let n = axis_normal(q.axis, q.positive);
        let o = q.origin;
        let normal = body.to_world(part, [o[0] + n[0], o[1] + n[1], o[2] + n[2]]) - body.to_world(part, o);
        if !front_facing(view, normal) {
            continue;
        }
        out.push(Face {
            corners,
            color: shaded(q.material, q.axis, q.positive),
            key,
        });
    }
    out
}

pub fn mesh_part(part: &Part) -> PartMesh {
    mesh_volume(&part.volume)
}

/// Per-cell visible faces of a chunk, keyed by the cell's x+y+z. Neighbours
/// are looked up across chunk seams through `solid`, so seam faces between
/// two solid cells are not emitted.
pub fn chunk_cell_faces(
    view: &View,
    chunk: &Chunk,
    solid: &dyn Fn(i32, i32, i32) -> bool,
) -> Vec<Face> {
    let o = chunk.origin();
    let mut out = Vec::new();
    for z in 0..CHUNK {
        for y in 0..WORLD_Y {
            for x in 0..CHUNK {
                let m = chunk.volume.get(x, y, z);
                if m == 0 {
                    continue;
                }
                let (wx, wy, wz) = (o[0] + x as i32, o[1] + y as i32, o[2] + z as i32);
                let key = wx + wy + wz;
                let base = Vec3::new(wx as f32, wy as f32, wz as f32);
                for (axis, positive) in [(0u8, true), (1, true), (2, true), (0, false), (1, false), (2, false)] {
                    let n = axis_normal(axis, positive);
                    if solid(wx + n[0], wy + n[1], wz + n[2]) {
                        continue;
                    }
                    if !front_facing(view, Vec3::new(n[0] as f32, n[1] as f32, n[2] as f32)) {
                        continue;
                    }
                    let corners = unit_face(base, axis, positive);
                    out.push(Face {
                        corners,
                        color: shaded(m, axis, positive),
                        key,
                    });
                }
            }
        }
    }
    out
}

/// A unit cube face at `base`, corners counter-clockwise seen from outside,
/// matching `Quad::corners` for size 1.
fn unit_face(base: Vec3, axis: u8, positive: bool) -> [Vec3; 4] {
    let (u, v) = match axis {
        0 => (Vec3::Y, Vec3::Z),
        1 => (Vec3::X, Vec3::Z),
        _ => (Vec3::X, Vec3::Y),
    };
    let mut o = base;
    if positive {
        o[axis as usize] += 1.0;
    }
    let (a, b, c, d) = (o, o + u, o + u + v, o + v);
    if positive { [a, b, c, d] } else { [a, d, c, b] }
}

/// Greedy faces of a whole chunk placed in the world (for the oracle and the
/// sprite bake), back faces culled.
pub fn chunk_greedy_faces(view: &View, chunk: &Chunk) -> Vec<Face> {
    let mesh = mesh_volume(&chunk.volume);
    let o = chunk.origin();
    let ov = Vec3::new(o[0] as f32, o[1] as f32, o[2] as f32);
    let mut out = Vec::with_capacity(mesh.quads.len());
    for q in &mesh.quads {
        let corners = q
            .corners()
            .map(|c| ov + Vec3::new(c[0] as f32, c[1] as f32, c[2] as f32));
        let n = axis_normal(q.axis, q.positive);
        if !front_facing(view, Vec3::new(n[0] as f32, n[1] as f32, n[2] as f32)) {
            continue;
        }
        out.push(Face {
            corners,
            color: shaded(q.material, q.axis, q.positive),
            key: 0,
        });
    }
    out
}

pub fn world_solid(world: &World) -> impl Fn(i32, i32, i32) -> bool + '_ {
    move |x, y, z| {
        if x < 0 || y < 0 || z < 0 {
            return false;
        }
        let (cx, cz) = (x as u32 / CHUNK, z as u32 / CHUNK);
        world
            .chunks
            .iter()
            .find(|c| c.cx == cx && c.cz == cz)
            .is_some_and(|c| c.volume.get(x as u32 % CHUNK, y as u32, z as u32 % CHUNK) != 0)
    }
}

pub fn to_screen(view: &View, f: &Face) -> ScreenFace {
    ScreenFace {
        p: f.corners.map(|c| view.project(c)),
        color: f.color,
    }
}

/// Every visible face of the world, for the oracle.
pub fn world_faces(view: &View, world: &World) -> Vec<Face> {
    let mut out = Vec::new();
    for c in &world.chunks {
        out.extend(chunk_greedy_faces(view, c));
    }
    for b in &world.bodies {
        for p in &b.parts {
            let mesh = mesh_part(p);
            out.extend(part_faces(view, b, p, &mesh));
        }
    }
    out
}

pub struct Raster {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
    pub depth: Vec<f32>,
}

impl Raster {
    pub fn new(w: u32, h: u32, background: Option<[f32; 3]>) -> Self {
        let n = (w * h) as usize;
        let mut rgba = vec![0u8; n * 4];
        if let Some(bg) = background {
            for px in rgba.chunks_exact_mut(4) {
                px[0] = to_u8(bg[0]);
                px[1] = to_u8(bg[1]);
                px[2] = to_u8(bg[2]);
                px[3] = 255;
            }
        }
        Self {
            w,
            h,
            rgba,
            depth: vec![f32::INFINITY; n],
        }
    }

    /// Fill a screen-space parallelogram with per-pixel depth, nearer wins.
    pub fn face(&mut self, f: &ScreenFace, offset: (f32, f32)) {
        let p0 = f.p[0] - Vec3::new(offset.0, offset.1, 0.0);
        let e1 = f.p[1] - f.p[0];
        let e3 = f.p[3] - f.p[0];
        let det = e1.x * e3.y - e3.x * e1.y;
        if det.abs() < 1e-6 {
            return;
        }
        let inv = 1.0 / det;
        let xs = [p0.x, p0.x + e1.x, p0.x + e1.x + e3.x, p0.x + e3.x];
        let ys = [p0.y, p0.y + e1.y, p0.y + e1.y + e3.y, p0.y + e3.y];
        let x0 = xs.iter().cloned().fold(f32::INFINITY, f32::min).floor().max(0.0) as i64;
        let x1 = (xs.iter().cloned().fold(f32::NEG_INFINITY, f32::max).ceil() as i64).min(self.w as i64);
        let y0 = ys.iter().cloned().fold(f32::INFINITY, f32::min).floor().max(0.0) as i64;
        let y1 = (ys.iter().cloned().fold(f32::NEG_INFINITY, f32::max).ceil() as i64).min(self.h as i64);
        let col = [to_u8(f.color[0]), to_u8(f.color[1]), to_u8(f.color[2]), 255];
        for y in y0..y1 {
            for x in x0..x1 {
                let cx = x as f32 + 0.5 - p0.x;
                let cy = y as f32 + 0.5 - p0.y;
                let u = (cx * e3.y - e3.x * cy) * inv;
                let v = (e1.x * cy - cx * e1.y) * inv;
                if !(0.0..1.0).contains(&u) || !(0.0..1.0).contains(&v) {
                    continue;
                }
                let z = p0.z + u * e1.z + v * e3.z;
                let i = (y as u32 * self.w + x as u32) as usize;
                if z < self.depth[i] {
                    self.depth[i] = z;
                    self.rgba[i * 4..i * 4 + 4].copy_from_slice(&col);
                }
            }
        }
    }
}

pub fn to_u8(x: f32) -> u8 {
    (x.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

/// Oracle frame: everything, z-buffered, background filled, outside an
/// optional clip painted background.
pub fn oracle(view: &View, world: &World, clip: Option<[f32; 4]>) -> Raster {
    let mut r = Raster::new(view.w, view.h, Some(BACKGROUND));
    for f in world_faces(view, world) {
        r.face(&to_screen(view, &f), (0.0, 0.0));
    }
    if let Some(c) = clip {
        let bg = [to_u8(BACKGROUND[0]), to_u8(BACKGROUND[1]), to_u8(BACKGROUND[2]), 255];
        for y in 0..view.h {
            for x in 0..view.w {
                let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
                if fx < c[0] || fx >= c[2] || fy < c[1] || fy >= c[3] {
                    let i = (y * view.w + x) as usize * 4;
                    r.rgba[i..i + 4].copy_from_slice(&bg);
                }
            }
        }
    }
    r
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Compare {
    pub pixels: usize,
    pub mismatch: usize,
    pub interior: usize,
    pub interior_mismatch: usize,
    pub mean_abs: f64,
}

/// Compare a path's frame to the oracle. A pixel mismatches when any channel
/// differs by more than 32/255. Interior pixels are those whose four oracle
/// neighbours share its colour, so antialiased edges are excluded from the
/// ordering verdict.
pub fn compare(oracle: &Raster, got: &[u8]) -> Compare {
    let (w, h) = (oracle.w as usize, oracle.h as usize);
    let mut c = Compare {
        pixels: w * h,
        ..Default::default()
    };
    let mut sum = 0.0f64;
    let same = |a: &[u8], b: &[u8]| (0..3).all(|k| a[k].abs_diff(b[k]) <= 2);
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 4;
            let o = &oracle.rgba[i..i + 4];
            let g = &got[i..i + 4];
            let d = (0..3).map(|k| o[k].abs_diff(g[k])).max().unwrap();
            sum += (0..3).map(|k| o[k].abs_diff(g[k]) as f64).sum::<f64>() / 3.0;
            let bad = d > 32;
            if bad {
                c.mismatch += 1;
            }
            let interior = x > 0
                && y > 0
                && x + 1 < w
                && y + 1 < h
                && same(o, &oracle.rgba[i - 4..i])
                && same(o, &oracle.rgba[i + 4..i + 8])
                && same(o, &oracle.rgba[i - w * 4..i - w * 4 + 4])
                && same(o, &oracle.rgba[i + w * 4..i + w * 4 + 4]);
            if interior {
                c.interior += 1;
                if bad {
                    c.interior_mismatch += 1;
                }
            }
        }
    }
    c.mean_abs = sum / (w * h) as f64;
    c
}
