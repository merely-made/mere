// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The one scene every path presents: four ground chunks with a plateau, a
//! wall row and a pit; up to three two- or three-part bodies; a scripted
//! motion per case.

use glam::Vec3;
use mesocosm_core::{PartId, VolumeRef, Yaw};
use mesocosm_mesh::{Volume, place_point};

pub const CHUNK: u32 = 16;
pub const WORLD_Y: u32 = 8;
pub const CHUNKS_X: u32 = 2;
pub const CHUNKS_Z: u32 = 2;
pub const FRAMES: usize = 60;
/// The variety workload encodes sixteen independent exterior notches.
pub const MAX_SHAPES: usize = 1 << 16;
pub const DEFAULT_SEED: u64 = 0x5EED;
pub const SWARM_VERSION: &str = "swarm_v2";
pub const SHAPE_GENERATOR: &str = "surface_notches_v1";

pub const GROUND_LOW: u8 = 241;
pub const GROUND_HIGH: u8 = 244;
pub const MAT_TORSO: u8 = 3;
pub const MAT_TORSO_ALT: u8 = 8;
pub const MAT_HEAD: u8 = 5;
pub const MAT_TAIL: u8 = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
    Crossing,
    Articulated,
    TerrainEdit,
    Clipped,
    MaterialChange,
    ContinuousYaw,
}

impl Case {
    pub const ALL: [Case; 6] = [
        Case::Crossing,
        Case::Articulated,
        Case::TerrainEdit,
        Case::Clipped,
        Case::MaterialChange,
        Case::ContinuousYaw,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Case::Crossing => "crossing",
            Case::Articulated => "articulated",
            Case::TerrainEdit => "terrain_edit",
            Case::Clipped => "clipped",
            Case::MaterialChange => "material_change",
            Case::ContinuousYaw => "continuous_yaw",
        }
    }

    /// Viewport clip in target pixels, for the clipped case.
    pub fn clip_rect(self, w: u32, h: u32) -> Option<[f32; 4]> {
        match self {
            Case::Clipped => Some([
                w as f32 * 0.2,
                h as f32 * 0.2,
                w as f32 * 0.8,
                h as f32 * 0.8,
            ]),
            _ => None,
        }
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct Chunk {
    pub cx: u32,
    pub cz: u32,
    pub volume: Volume,
    pub reference: VolumeRef,
    pub generation: u64,
}

impl Chunk {
    pub fn origin(&self) -> [i32; 3] {
        [(self.cx * CHUNK) as i32, 0, (self.cz * CHUNK) as i32]
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct Part {
    pub id: PartId,
    pub size: [u32; 3],
    pub material: u8,
    pub volume: Volume,
    pub reference: VolumeRef,
    pub pivot: [i32; 3],
    pub pivot_at: [i32; 3],
    pub yaw: Yaw,
    pub generation: u64,
}

#[derive(Clone)]
pub struct Body {
    #[allow(dead_code)]
    pub name: &'static str,
    pub origin: Vec3,
    /// Continuous yaw about the body origin's vertical axis, degrees. Only the
    /// continuous-yaw case sets it; a path's rotation policy is measured separately.
    pub yaw_deg: f32,
    pub parts: Vec<Part>,
}

impl Body {
    /// Part-local voxel corner to world space, through the quarter-turn
    /// placement and then the body's continuous yaw.
    pub fn to_world(&self, part: &Part, local: [i32; 3]) -> Vec3 {
        let placed = place_point(local, part.yaw, part.pivot, part.pivot_at);
        let p = Vec3::new(placed[0] as f32, placed[1] as f32, placed[2] as f32);
        let p = if self.yaw_deg != 0.0 {
            let a = self.yaw_deg.to_radians();
            let (s, c) = a.sin_cos();
            Vec3::new(p.x * c + p.z * s, p.y, -p.x * s + p.z * c)
        } else {
            p
        };
        self.origin + p
    }

    /// Nearest quarter turn to the continuous yaw, composed onto a part's own.
    pub fn quantized_yaw(&self, part: &Part) -> Yaw {
        let steps = ((self.yaw_deg / 90.0).round() as i32).rem_euclid(4);
        let q = match steps {
            0 => Yaw::Zero,
            1 => Yaw::Quarter,
            2 => Yaw::Half,
            _ => Yaw::ThreeQuarter,
        };
        q.compose(part.yaw)
    }

    /// Depth key of the cell the body stands in, for painter placement.
    pub fn key(&self) -> i32 {
        self.origin.x.floor() as i32 + self.origin.y.floor() as i32 + self.origin.z.floor() as i32
    }
}

#[derive(Clone)]
pub struct World {
    pub chunks: Vec<Chunk>,
    pub bodies: Vec<Body>,
}

/// What a step changed, so each path invalidates only that.
#[derive(Default, Debug)]
pub struct Changes {
    pub chunks: Vec<usize>,
    /// (body, part) whose volume content changed.
    pub parts: Vec<(usize, usize)>,
    /// Bodies whose yaw or a part's yaw changed.
    pub turned: Vec<usize>,
}

pub fn reference_of(size: [u32; 3], voxels: &[u8]) -> VolumeRef {
    let mut hash = blake3::Hasher::new();
    hash.update(b"wing-volume-v2\0");
    for axis in size {
        hash.update(&axis.to_le_bytes());
    }
    hash.update(voxels);
    VolumeRef(*hash.finalize().as_bytes())
}

fn ground_height(x: i32, z: i32) -> i32 {
    let mut h = 2;
    if x >= 18 {
        h = 5;
    }
    // Wall pieces one row in front of body A's lane (torso z 14..20), with a
    // gap wide enough that body B (x 20..26) never touches them.
    if z == 20 && ((4..12).contains(&x) || (27..31).contains(&x)) {
        h = 6;
    }
    if (10..12).contains(&x) && (10..12).contains(&z) {
        h = 1;
    }
    h
}

fn ground_material(x: i32, y: i32, z: i32) -> u8 {
    if y >= 4 || (z == 20 && (4..12).contains(&x)) || (z == 20 && (27..31).contains(&x)) {
        GROUND_HIGH
    } else {
        GROUND_LOW
    }
}

fn chunk_voxels(cx: u32, cz: u32, carve: Option<([i32; 3], [i32; 3])>) -> Vec<u8> {
    let mut v = vec![0u8; (CHUNK * WORLD_Y * CHUNK) as usize];
    for z in 0..CHUNK as i32 {
        for x in 0..CHUNK as i32 {
            let wx = x + (cx * CHUNK) as i32;
            let wz = z + (cz * CHUNK) as i32;
            let h = ground_height(wx, wz);
            for y in 0..h.min(WORLD_Y as i32) {
                if let Some((lo, hi)) = carve
                    && wx >= lo[0]
                    && wx < hi[0]
                    && y >= lo[1]
                    && y < hi[1]
                    && wz >= lo[2]
                    && wz < hi[2]
                {
                    continue;
                }
                let i = x + y * CHUNK as i32 + z * (CHUNK * WORLD_Y) as i32;
                v[i as usize] = ground_material(wx, y, wz);
            }
        }
    }
    v
}

fn make_chunk(cx: u32, cz: u32, carve: Option<([i32; 3], [i32; 3])>, generation: u64) -> Chunk {
    let voxels = chunk_voxels(cx, cz, carve);
    let reference = reference_of([CHUNK, WORLD_Y, CHUNK], &voxels);
    Chunk {
        cx,
        cz,
        volume: Volume::new([CHUNK, WORLD_Y, CHUNK], voxels).expect("chunk size"),
        reference,
        generation,
    }
}

/// Two-material checker so the greedy mesher yields a mosaic of faces per
/// part (about 40 to 170 rectangles) rather than three quads per box.
fn checker(size: [u32; 3], material: u8) -> Vec<u8> {
    let mut v = Vec::with_capacity((size[0] * size[1] * size[2]) as usize);
    for z in 0..size[2] {
        for y in 0..size[1] {
            for x in 0..size[0] {
                v.push(if (x + y + z) % 2 == 0 {
                    material
                } else {
                    material.wrapping_add(1)
                });
            }
        }
    }
    v
}

fn make_part(
    id: u32,
    size: [u32; 3],
    material: u8,
    pivot: [i32; 3],
    pivot_at: [i32; 3],
    yaw: Yaw,
) -> Part {
    let voxels = checker(size, material);
    let reference = reference_of(size, &voxels);
    Part {
        id: PartId(id),
        size,
        material,
        volume: Volume::new(size, voxels).expect("part size"),
        reference,
        pivot,
        pivot_at,
        yaw,
        generation: 0,
    }
}

pub fn set_part_material(part: &mut Part, material: u8) {
    let mut voxels = checker(part.size, material);
    for (voxel, old) in voxels.iter_mut().zip(part.volume.clone().into_voxels()) {
        if old == 0 {
            *voxel = 0;
        }
    }
    part.reference = reference_of(part.size, &voxels);
    part.volume = Volume::new(part.size, voxels).expect("part size");
    part.material = material;
    part.generation += 1;
}

fn walker(name: &'static str, origin: Vec3) -> Body {
    Body {
        name,
        origin,
        yaw_deg: 0.0,
        parts: vec![
            make_part(0, [6, 8, 6], MAT_TORSO, [0, 0, 0], [0, 0, 0], Yaw::Zero),
            make_part(1, [4, 4, 4], MAT_HEAD, [2, 0, 2], [3, 8, 3], Yaw::Zero),
        ],
    }
}

fn tailed(name: &'static str, origin: Vec3) -> Body {
    let mut b = walker(name, origin);
    b.parts.push(make_part(
        2,
        [14, 2, 2],
        MAT_TAIL,
        [0, 1, 1],
        [6, 3, 3],
        Yaw::Zero,
    ));
    b
}

/// A walks +x along z 14..20 behind the wall piece; B (quarter-turned, so
/// its footprint is x 12..18, z origin-6..origin) walks -z across A's lane
/// before A reaches it. Footprints never overlap; at t = 0.5 they are close
/// diagonal neighbours.
fn crossing_origins(t: f32) -> (Vec3, Vec3) {
    (
        Vec3::new(8.0 * t, 2.0, 14.0),
        Vec3::new(12.0, 2.0, 30.0 - 30.0 * t),
    )
}

#[derive(Clone, Copy, Debug)]
pub struct Swarm {
    pub bodies: usize,
    pub seed: u64,
    /// Independent topology budget; unset preserves the legacy walker shape.
    /// Callers validate 1..=MAX_SHAPES. Actual designs cannot exceed bodies.
    pub shapes: Option<usize>,
}

fn lcg(s: &mut u64) -> f32 {
    *s = s
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    // Exactly representable 24-bit fraction in [0, 1). The old >>33/u32::MAX
    // expression only covered [0, 0.5); swarm_v2 is a different workload.
    (*s >> 40) as f32 / 16_777_216.0
}

/// Seed chooses the order of sixteen independent cells on the visible +x
/// face. Removing any subset leaves the torso's interior and attachment
/// planes intact. This deliberately bounded sampler prices content reuse;
/// it is not a general anatomy generator.
fn notch_order(seed: u64) -> [usize; 16] {
    let mut order = std::array::from_fn(|i| i);
    let mut state = seed ^ 0x4E4F_5443_4845_5331;
    for i in (1..order.len()).rev() {
        let other = (lcg(&mut state) * (i + 1) as f32) as usize;
        order.swap(i, other);
    }
    order
}

fn set_shape(body: &mut Body, shape: usize, order: &[usize; 16]) {
    assert!(shape < MAX_SHAPES, "shape exceeds bounded notch sampler");
    let torso = &mut body.parts[0];
    for (bit, &cell) in order.iter().enumerate() {
        if shape & (1 << bit) != 0 {
            torso
                .volume
                .set(5, 1 + (cell / 4) as u32, 1 + (cell % 4) as u32, 0);
        }
    }
    torso.reference = reference_of(torso.size, &torso.volume.clone().into_voxels());
}

/// The scale variant: `n` seeded bodies over the same ground, overlapping
/// freely. Sampled oracle checking is opt-in and body-count bounded; it
/// measures painter-path errors even when the geometry intersects.
/// Every case's rule then applies to every body.
pub fn build_swarm(case: Case, swarm: &Swarm) -> World {
    assert!(swarm.bodies > 0, "swarm needs a positive body count");
    assert!(swarm.shapes.is_none_or(|n| (1..=MAX_SHAPES).contains(&n)));
    let mut w = build(case);
    let mut s = swarm.seed;
    let order = notch_order(swarm.seed);
    let mut bodies = Vec::with_capacity(swarm.bodies);
    for i in 0..swarm.bodies {
        let x = lcg(&mut s) * 26.0;
        let z = lcg(&mut s) * 26.0;
        let mut b = if case == Case::Articulated {
            tailed("S", Vec3::new(x, 2.0, z))
        } else {
            walker("S", Vec3::new(x, 2.0, z))
        };
        if let Some(shapes) = swarm.shapes {
            set_shape(&mut b, i % shapes, &order);
        }
        let q = [Yaw::Zero, Yaw::Quarter, Yaw::Half, Yaw::ThreeQuarter][i % 4];
        b.parts.iter_mut().for_each(|p| p.yaw = p.yaw.compose(q));
        bodies.push(b);
    }
    w.bodies = bodies;
    w
}

pub fn build(case: Case) -> World {
    let chunks = (0..CHUNKS_Z)
        .flat_map(|cz| (0..CHUNKS_X).map(move |cx| make_chunk(cx, cz, None, 0)))
        .collect();
    let (a0, b0) = crossing_origins(0.0);
    let (ah, bh) = crossing_origins(0.5);
    let bodies = match case {
        Case::Crossing | Case::Clipped | Case::MaterialChange => {
            let mut b = walker("B", b0);
            b.parts.iter_mut().for_each(|p| p.yaw = Yaw::Quarter);
            vec![walker("A", a0), b]
        },
        Case::Articulated => vec![
            walker("A", ah),
            walker("B", bh),
            tailed("C", Vec3::new(20.0, 5.0, 8.0)),
        ],
        Case::TerrainEdit => vec![walker("A", ah), walker("B", bh)],
        Case::ContinuousYaw => vec![walker("A", Vec3::new(14.0, 2.0, 12.0))],
    };
    World { chunks, bodies }
}

/// Advance the world to `frame`'s state. Returns what changed since the
/// previous frame; frame 0 reports nothing (everything is fresh).
pub fn step(world: &mut World, case: Case, frame: usize, swarm: bool) -> Changes {
    if swarm {
        return step_swarm(world, case, frame);
    }
    let mut ch = Changes::default();
    let t = frame as f32 / (FRAMES - 1) as f32;
    match case {
        Case::Crossing | Case::Clipped | Case::MaterialChange => {
            let (a, b) = crossing_origins(t);
            world.bodies[0].origin = a;
            world.bodies[1].origin = b;
            if case == Case::MaterialChange && frame == 30 {
                set_part_material(&mut world.bodies[0].parts[0], MAT_TORSO_ALT);
                ch.parts.push((0, 0));
            }
        },
        Case::Articulated => {
            if frame > 0 && frame % 15 == 0 {
                let tail = &mut world.bodies[2].parts[2];
                tail.yaw = tail.yaw.compose(Yaw::Quarter);
                ch.turned.push(2);
            }
        },
        Case::TerrainEdit => {
            if frame == 30 {
                let carve = Some(([15, 0, 9], [17, 3, 11]));
                for (i, c) in world.chunks.iter_mut().enumerate() {
                    if c.cz == 0 {
                        *c = make_chunk(c.cx, c.cz, carve, 1);
                        ch.chunks.push(i);
                    }
                }
            }
        },
        Case::ContinuousYaw => {
            if frame > 0 {
                world.bodies[0].yaw_deg = 3.0 * frame as f32;
                ch.turned.push(0);
            }
        },
    }
    ch
}

fn step_swarm(world: &mut World, case: Case, frame: usize) -> Changes {
    let mut ch = Changes::default();
    let n = world.bodies.len();
    match case {
        Case::Crossing | Case::Clipped | Case::MaterialChange => {
            // Every body drifts along its own direction and wraps.
            for (i, b) in world.bodies.iter_mut().enumerate() {
                let a = (i % 8) as f32 * std::f32::consts::FRAC_PI_4;
                b.origin.x = (b.origin.x + 0.2 * a.cos()).rem_euclid(26.0);
                b.origin.z = (b.origin.z + 0.2 * a.sin()).rem_euclid(26.0);
            }
            if case == Case::MaterialChange && frame == 30 {
                for bi in 0..n {
                    set_part_material(&mut world.bodies[bi].parts[0], MAT_TORSO_ALT);
                    ch.parts.push((bi, 0));
                }
            }
        },
        Case::Articulated => {
            if frame > 0 && frame % 15 == 0 {
                for (bi, b) in world.bodies.iter_mut().enumerate() {
                    if let Some(tail) = b.parts.get_mut(2) {
                        tail.yaw = tail.yaw.compose(Yaw::Quarter);
                        ch.turned.push(bi);
                    }
                }
            }
        },
        Case::TerrainEdit => {
            if frame == 30 {
                let carve = Some(([15, 0, 9], [17, 3, 11]));
                for (i, c) in world.chunks.iter_mut().enumerate() {
                    if c.cz == 0 {
                        *c = make_chunk(c.cx, c.cz, carve, 1);
                        ch.chunks.push(i);
                    }
                }
            }
        },
        Case::ContinuousYaw => {
            if frame > 0 {
                for (bi, b) in world.bodies.iter_mut().enumerate() {
                    b.yaw_deg = 3.0 * frame as f32;
                    ch.turned.push(bi);
                }
            }
        },
    }
    ch
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_addresses_distinguish_equal_bytes_with_different_dimensions() {
        let voxels = [MAT_HEAD; 8];
        assert_ne!(
            reference_of([2, 2, 2], &voxels),
            reference_of([1, 2, 4], &voxels)
        );
        assert_eq!(
            reference_of([2, 2, 2], &voxels),
            reference_of([2, 2, 2], &voxels)
        );
    }

    #[test]
    fn swarm_rng_covers_both_halves_without_reaching_one() {
        let mut seed = DEFAULT_SEED;
        let values: Vec<_> = (0..10_000).map(|_| lcg(&mut seed)).collect();
        assert!(values.iter().all(|x| (0.0..1.0).contains(x)));
        assert!(values.iter().any(|&x| x > 0.9));
        assert!(values.iter().any(|&x| x < 0.1));
    }

    #[test]
    fn unset_shape_control_preserves_legacy_walker_geometry() {
        let legacy = walker("legacy", Vec3::ZERO);
        let swarm = Swarm {
            bodies: 4,
            seed: DEFAULT_SEED,
            shapes: None,
        };
        let repeated = build_swarm(Case::Crossing, &swarm);
        let explicit_one = build_swarm(
            Case::Crossing,
            &Swarm {
                shapes: Some(1),
                ..swarm
            },
        );
        for (body, explicit) in repeated.bodies.iter().zip(&explicit_one.bodies) {
            assert_eq!(body.origin, explicit.origin);
            for ((part, source), equivalent) in
                body.parts.iter().zip(&legacy.parts).zip(&explicit.parts)
            {
                assert_eq!(part.volume, source.volume);
                assert_eq!(part.volume, equivalent.volume);
                assert_eq!(part.reference, source.reference);
            }
        }
    }
}
