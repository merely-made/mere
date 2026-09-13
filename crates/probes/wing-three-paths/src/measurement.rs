// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Workload identity and geometry counts, prepared outside timed frames.

use std::collections::{BTreeMap, BTreeSet};

use mesocosm_mesh::{Volume, mesh_volume};

use crate::world::{self, Case, FRAMES, Swarm, World};

#[derive(Debug, PartialEq, Eq)]
pub struct GeometryCounts {
    pub bodies: usize,
    pub parts: usize,
    pub chunks: usize,
    pub body_topologies: usize,
    pub part_topologies: usize,
    pub all_topologies: usize,
    pub part_volumes: usize,
    pub chunk_volumes: usize,
    pub all_volumes: usize,
    pub voxel_cells_instances: usize,
    pub solid_voxels_instances: usize,
    pub mesh_quads_instances: usize,
    pub mesh_quads_unique_volumes: usize,
}

impl GeometryCounts {
    pub fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "body_instances": self.bodies,
            "part_instances": self.parts,
            "chunk_instances": self.chunks,
            "draw_instances": self.parts + self.chunks,
            "unique_body_topologies": self.body_topologies,
            "unique_part_topologies": self.part_topologies,
            "unique_topologies": self.all_topologies,
            "unique_part_volumes": self.part_volumes,
            "unique_chunk_volumes": self.chunk_volumes,
            "unique_volumes": self.all_volumes,
            "voxel_cells_instances": self.voxel_cells_instances,
            "solid_voxels_instances": self.solid_voxels_instances,
            "mesh_quads_instances": self.mesh_quads_instances,
            "mesh_quads_unique_volumes": self.mesh_quads_unique_volumes,
        })
    }
}

fn volumes(world: &World) -> impl Iterator<Item = ([u8; 32], &Volume)> {
    world
        .chunks
        .iter()
        .map(|c| (c.reference.0, &c.volume))
        .chain(
            world
                .bodies
                .iter()
                .flat_map(|b| b.parts.iter().map(|p| (p.reference.0, &p.volume))),
        )
}

pub fn topology_key(volume: &Volume) -> [u8; 32] {
    let occupancy: Vec<u8> = volume
        .clone()
        .into_voxels()
        .into_iter()
        .map(|m| u8::from(m != 0))
        .collect();
    world::reference_of(volume.size, &occupancy).0
}

pub fn geometry_counts(world: &World) -> GeometryCounts {
    let mut unique: BTreeMap<[u8; 32], (&Volume, usize)> = BTreeMap::new();
    for (key, volume) in volumes(world) {
        unique.entry(key).or_insert((volume, 0)).1 += 1;
    }
    let topologies: BTreeMap<_, _> = unique
        .iter()
        .map(|(&key, &(v, _))| (key, topology_key(v)))
        .collect();
    let mut body_topologies = BTreeSet::new();
    let mut part_topologies = BTreeSet::new();
    let mut part_volumes = BTreeSet::new();
    for body in &world.bodies {
        let mut hash = blake3::Hasher::new();
        hash.update(b"wing-body-topology-v1\0");
        for part in &body.parts {
            let topology = topologies[&part.reference.0];
            hash.update(&part.id.0.to_le_bytes());
            hash.update(&topology);
            for coordinate in part.pivot.into_iter().chain(part.pivot_at) {
                hash.update(&coordinate.to_le_bytes());
            }
            // Pose and material are not topology.
            part_topologies.insert(topology);
            part_volumes.insert(part.reference.0);
        }
        body_topologies.insert(*hash.finalize().as_bytes());
    }
    let mut cells = 0;
    let mut solids = 0;
    let mut quads_instances = 0;
    let mut quads_unique = 0;
    for &(volume, count) in unique.values() {
        cells += count * volume.voxel_count();
        solids += count * volume.solid_count();
        let quads = mesh_volume(volume).quads.len();
        quads_instances += count * quads;
        quads_unique += quads;
    }
    GeometryCounts {
        bodies: world.bodies.len(),
        parts: world.bodies.iter().map(|b| b.parts.len()).sum(),
        chunks: world.chunks.len(),
        body_topologies: body_topologies.len(),
        part_topologies: part_topologies.len(),
        all_topologies: topologies.values().collect::<BTreeSet<_>>().len(),
        part_volumes: part_volumes.len(),
        chunk_volumes: world
            .chunks
            .iter()
            .map(|c| c.reference.0)
            .collect::<BTreeSet<_>>()
            .len(),
        all_volumes: unique.len(),
        voxel_cells_instances: cells,
        solid_voxels_instances: solids,
        mesh_quads_instances: quads_instances,
        mesh_quads_unique_volumes: quads_unique,
    }
}

/// Shared world input for every selected path. The digest covers dimensions,
/// volume content addresses, attachments and all scripted measured-frame poses;
/// it describes the supplied world, not a path's quantized rendered pose.
pub struct Workload {
    pub initial: World,
    pub initial_geometry: GeometryCounts,
    pub final_geometry: GeometryCounts,
    pub max_live_unique_volumes: usize,
    pub unique_volumes_over_run: usize,
    pub digest: String,
}

impl Workload {
    pub fn prepare(case: Case, swarm: Option<&Swarm>) -> Self {
        let initial = swarm.map_or_else(|| world::build(case), |s| world::build_swarm(case, s));
        let initial_geometry = geometry_counts(&initial);
        let mut scripted = initial.clone();
        let mut seen = BTreeSet::new();
        let mut max_live = 0;
        let mut hash = blake3::Hasher::new();
        hash.update(b"wing-workload-v2\0");
        hash.update(case.name().as_bytes());
        hash.update(&(FRAMES as u64).to_le_bytes());
        record_world(&mut hash, &initial);
        for frame in 0..FRAMES {
            world::step(&mut scripted, case, frame, swarm.is_some());
            let live: BTreeSet<_> = volumes(&scripted).map(|(key, _)| key).collect();
            max_live = max_live.max(live.len());
            seen.extend(live);
            record_world(&mut hash, &scripted);
        }
        let final_geometry = geometry_counts(&scripted);
        Self {
            initial,
            initial_geometry,
            final_geometry,
            max_live_unique_volumes: max_live,
            unique_volumes_over_run: seen.len(),
            digest: hash.finalize().to_hex().to_string(),
        }
    }

    pub fn check_cache_capacity(&self, capacity: usize) -> Result<(), String> {
        if capacity < self.max_live_unique_volumes {
            Err(format!(
                "path C needs {} simultaneously live mesh keys including terrain, but L0C_MESH_CACHE={capacity}; increase capacity ({} unique keys occur over the whole run)",
                self.max_live_unique_volumes, self.unique_volumes_over_run
            ))
        } else {
            Ok(())
        }
    }
}

fn record_world(hash: &mut blake3::Hasher, world: &World) {
    hash.update(&(world.chunks.len() as u64).to_le_bytes());
    for chunk in &world.chunks {
        hash.update(&chunk.reference.0);
        for coordinate in chunk.origin() {
            hash.update(&coordinate.to_le_bytes());
        }
    }
    hash.update(&(world.bodies.len() as u64).to_le_bytes());
    for body in &world.bodies {
        for value in body.origin.to_array().into_iter().chain([body.yaw_deg]) {
            hash.update(&value.to_bits().to_le_bytes());
        }
        hash.update(&(body.parts.len() as u64).to_le_bytes());
        for part in &body.parts {
            hash.update(&part.reference.0);
            hash.update(&part.id.0.to_le_bytes());
            for coordinate in part.pivot.into_iter().chain(part.pivot_at) {
                hash.update(&coordinate.to_le_bytes());
            }
            let yaw = match part.yaw {
                mesocosm_core::Yaw::Zero => 0,
                mesocosm_core::Yaw::Quarter => 1,
                mesocosm_core::Yaw::Half => 2,
                mesocosm_core::Yaw::ThreeQuarter => 3,
            };
            hash.update(&[yaw]);
        }
    }
}

/// Nearest-rank percentile over finite samples. Reports a measured sample,
/// rather than interpolating a frame that did not occur.
pub fn percentile(samples: &[f64], percentile: usize) -> f64 {
    assert!((1..=100).contains(&percentile));
    let mut values: Vec<_> = samples.iter().copied().filter(|x| x.is_finite()).collect();
    if values.is_empty() {
        return f64::NAN;
    }
    values.sort_by(f64::total_cmp);
    values[(values.len() * percentile).div_ceil(100) - 1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_change_preserves_shapes_and_prices_old_and_new_keys_separately() {
        let workload = Workload::prepare(
            Case::MaterialChange,
            Some(&Swarm {
                bodies: 8,
                seed: world::DEFAULT_SEED,
                shapes: Some(4),
            }),
        );
        assert_eq!(workload.initial_geometry.body_topologies, 4);
        assert_eq!(workload.final_geometry.body_topologies, 4);
        assert_eq!(
            workload.initial_geometry.all_volumes,
            workload.final_geometry.all_volumes
        );
        assert_eq!(
            workload.max_live_unique_volumes,
            workload.initial_geometry.all_volumes
        );
        assert_eq!(
            workload.unique_volumes_over_run,
            workload.max_live_unique_volumes + 4
        );
        assert!(
            workload
                .check_cache_capacity(workload.max_live_unique_volumes)
                .is_ok()
        );
        assert!(
            workload
                .check_cache_capacity(workload.max_live_unique_volumes - 1)
                .is_err()
        );
    }

    #[test]
    fn requested_shapes_are_bounded_by_body_count_and_repeat_by_content() {
        let sparse = world::build_swarm(
            Case::Crossing,
            &Swarm {
                bodies: 3,
                seed: world::DEFAULT_SEED,
                shapes: Some(16),
            },
        );
        assert_eq!(geometry_counts(&sparse).body_topologies, 3);
        let shared = world::build_swarm(
            Case::Crossing,
            &Swarm {
                bodies: 256,
                seed: world::DEFAULT_SEED,
                shapes: Some(128),
            },
        );
        let counts = geometry_counts(&shared);
        assert_eq!(counts.body_topologies, 128);
        assert_eq!(counts.part_volumes, 129); // distinct torsos + one head
        assert!(counts.mesh_quads_instances > counts.mesh_quads_unique_volumes);
    }

    #[test]
    fn digest_is_repeatable_and_changes_with_seed_or_topology() {
        let swarm = Swarm {
            bodies: 4,
            seed: world::DEFAULT_SEED,
            shapes: Some(2),
        };
        let a = Workload::prepare(Case::ContinuousYaw, Some(&swarm));
        let again = Workload::prepare(Case::ContinuousYaw, Some(&swarm));
        assert_eq!(a.digest, again.digest);
        let seed = Workload::prepare(Case::ContinuousYaw, Some(&Swarm { seed: 7, ..swarm }));
        let shape = Workload::prepare(
            Case::ContinuousYaw,
            Some(&Swarm {
                shapes: Some(3),
                ..swarm
            }),
        );
        assert_ne!(a.digest, seed.digest);
        assert_ne!(a.digest, shape.digest);
    }

    #[test]
    fn nearest_rank_p95_separates_the_tail_from_the_maximum() {
        let samples: Vec<_> = (1..=60).map(f64::from).collect();
        assert_eq!(percentile(&samples, 95), 57.0);
        assert_eq!(percentile(&[3.0, f64::NAN, 1.0], 95), 3.0);
        assert!(percentile(&[], 95).is_nan());
    }
}
