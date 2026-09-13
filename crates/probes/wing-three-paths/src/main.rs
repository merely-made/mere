// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! L0c — the three-path comparison.
//!
//! One voxel scene, one orthographic camera, six cases, presented three ways
//! into netrender at 1920x1080 with no genet DOM. Each path is checked at
//! three frames against a CPU depth oracle and timed over the run.
//!
//! Env: L0C_BACKEND=vulkan|dx12|all, L0C_ONLY=<case>, L0C_PATHS=abc,
//! L0C_OUT=<json path>, L0C_DUMP=1 writes PPM frames beside the JSON.
//! L0C_BODIES=<positive count>, L0C_SHAPES=1..65536 and L0C_SEED=<u64>
//! select swarm_v2. L0C_CHECK_SWARM=1 enables three sampled oracle checks
//! for swarms of at most32 bodies. L0C_MESH_CACHE=<positive count> defaults64.

mod gpu;
mod measurement;
mod path_a;
mod path_b;
mod path_c;
mod project;
mod world;

use std::time::Instant;

use netrender::boot_with;

use crate::measurement::Workload;
use crate::project::{Compare, View, compare, oracle};
use crate::world::{Case, FRAMES};

const W: u32 = 1920;
const H: u32 = 1080;
const WARMUP: usize = 3;
const CHECK_FRAMES: [usize; 3] = [0, 30, FRAMES - 1];
const MAX_CHECK_SWARM: usize = 32;

struct Config {
    swarm: Option<world::Swarm>,
    check_swarm: bool,
    mesh_cache: usize,
    paths: Vec<char>,
    cases: Vec<Case>,
}

impl Config {
    fn from_env() -> Result<Self, String> {
        let bodies = env_positive("L0C_BODIES")?;
        let shapes = env_positive("L0C_SHAPES")?;
        if shapes.is_some() && bodies.is_none() {
            return Err(
                "L0C_SHAPES requires L0C_BODIES; the default check fixtures are unchanged".into(),
            );
        }
        if shapes.is_some_and(|n| n > world::MAX_SHAPES) {
            return Err(format!(
                "L0C_SHAPES must be at most {} for {}",
                world::MAX_SHAPES,
                world::SHAPE_GENERATOR
            ));
        }
        let seed = match std::env::var("L0C_SEED") {
            Ok(value) => {
                let parsed = if let Some(hex) = value
                    .strip_prefix("0x")
                    .or_else(|| value.strip_prefix("0X"))
                {
                    u64::from_str_radix(hex, 16)
                } else {
                    value.parse()
                };
                parsed.map_err(|_| {
                    "L0C_SEED must be a u64 in decimal or 0x-prefixed hex".to_owned()
                })?
            },
            Err(std::env::VarError::NotPresent) => world::DEFAULT_SEED,
            Err(_) => return Err("L0C_SEED must be valid Unicode".into()),
        };
        let check_swarm = match std::env::var("L0C_CHECK_SWARM").as_deref() {
            Ok("1" | "true") => true,
            Ok("0" | "false") | Err(std::env::VarError::NotPresent) => false,
            _ => return Err("L0C_CHECK_SWARM must be 0 or 1".into()),
        };
        if check_swarm && bodies.is_some_and(|n| n > MAX_CHECK_SWARM) {
            return Err(format!(
                "L0C_CHECK_SWARM is limited to {MAX_CHECK_SWARM} bodies; use a smaller correctness swarm and a separate unchecked cost run"
            ));
        }
        let paths: Vec<_> = std::env::var("L0C_PATHS")
            .unwrap_or_else(|_| "abc".into())
            .chars()
            .collect();
        if paths.is_empty() || paths.iter().any(|p| !matches!(p, 'a' | 'b' | 'c')) {
            return Err("L0C_PATHS must contain only a, b and/or c".into());
        }
        let only = std::env::var("L0C_ONLY").ok();
        let cases: Vec<_> = Case::ALL
            .into_iter()
            .filter(|case| only.as_deref().is_none_or(|name| name == case.name()))
            .collect();
        if cases.is_empty() {
            return Err(format!(
                "unknown L0C_ONLY case: {}",
                only.unwrap_or_default()
            ));
        }
        Ok(Self {
            swarm: bodies.map(|bodies| world::Swarm {
                bodies,
                seed,
                shapes,
            }),
            check_swarm,
            mesh_cache: env_positive("L0C_MESH_CACHE")?.unwrap_or(64),
            paths,
            cases,
        })
    }
}

fn env_positive(name: &str) -> Result<Option<usize>, String> {
    match std::env::var(name) {
        Ok(value) => value
            .parse::<usize>()
            .ok()
            .filter(|&n| n > 0)
            .map(Some)
            .ok_or_else(|| format!("{name} must be a positive integer")),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(_) => Err(format!("{name} must be valid Unicode")),
    }
}

struct Run {
    case: Case,
    path: char,
    frames: usize,
    build_ms: f64,
    encode_ms: f64,
    poll_ms: f64,
    frame_ms: f64,
    frame_p95_ms: f64,
    frame_max_ms: f64,
    first_draw_ms: [f64; 3],
    checks: Vec<(usize, Compare)>,
    invalidation: serde_json::Value,
    rss_mb: f64,
    notes: String,
    workload: serde_json::Value,
}

fn dump_ppm(path: &str, w: u32, h: u32, rgba: &[u8]) {
    let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
    for px in rgba.chunks_exact(4) {
        out.extend_from_slice(&px[..3]);
    }
    std::fs::write(path, out).expect("write ppm");
}

fn run(
    handles: &netrender::WgpuHandles,
    case: Case,
    path: char,
    dump: Option<&str>,
    config: &Config,
    workload: &Workload,
) -> Run {
    let view = View::new(W, H);
    let (target, target_view) = gpu::make_target(&handles.device, W, H);
    let renderer = gpu::new_renderer(handles);
    let clip = case.clip_rect(W, H);

    let mut a = (path == 'a').then(|| path_a::PathA::new(renderer));
    let mut b = (path == 'b').then(|| path_b::PathB::new(gpu::new_renderer(handles)));
    let mut c = (path == 'c').then(|| {
        path_c::PathC::new(
            gpu::new_renderer(handles),
            &handles.device,
            W,
            H,
            config.mesh_cache,
        )
    });

    let mut world = workload.initial.clone();
    let check = config.swarm.is_none() || config.check_swarm;
    let mut builds = Vec::new();
    let mut encodes = Vec::new();
    let mut polls = Vec::new();
    let mut totals = Vec::new();
    let mut checks = Vec::new();
    let mut notes = String::new();
    let mut first_draw_ms = [0.0; 3];
    let mut warm_c = path_c::Invalidation::default();
    let mut warm_b = (0, 0);

    // Warmup: render frame 0 a few times so pipelines and caches exist.
    for i in 0..WARMUP + FRAMES {
        let frame = i.saturating_sub(WARMUP);
        let first = i == 0;
        if i == WARMUP {
            if let Some(c) = c.as_ref() {
                warm_c = c.inv;
            }
            if let Some(b) = b.as_ref() {
                warm_b = (b.inv.images, b.inv.bytes);
            }
        }
        let changes = if i < WARMUP {
            world::Changes::default()
        } else {
            world::step(&mut world, case, frame, config.swarm.is_some())
        };
        let (bm, em, pm) = match path {
            'a' => a.as_mut().unwrap().frame(
                &view,
                &world,
                &changes,
                first,
                clip,
                &target_view,
                &handles.device,
            ),
            'b' => b.as_mut().unwrap().frame(
                &view,
                &world,
                &changes,
                first,
                clip,
                &target_view,
                &handles.device,
            ),
            _ => c.as_mut().unwrap().frame(
                &view,
                &world,
                &changes,
                first,
                clip,
                &target_view,
                &handles.device,
                &handles.queue,
            ),
        };
        if first {
            first_draw_ms = [bm, em, pm];
        }
        if i >= WARMUP {
            builds.push(bm);
            encodes.push(em);
            polls.push(pm);
            totals.push(bm + em + pm);
            if check && CHECK_FRAMES.contains(&frame) {
                let t = Instant::now();
                let got = gpu::read_back(&handles.device, &handles.queue, &target, W, H);
                let orc = oracle(&view, &world, clip);
                let cmp = compare(&orc, &got);
                if let Some(dir) = dump {
                    dump_ppm(
                        &format!("{dir}/l0c_{}_{}_f{frame}.ppm", case.name(), path),
                        W,
                        H,
                        &got,
                    );
                    dump_ppm(
                        &format!("{dir}/l0c_{}_oracle_f{frame}.ppm", case.name()),
                        W,
                        H,
                        &orc.rgba,
                    );
                }
                eprintln!(
                    "    check f{frame}: mismatch {:.2}% interior {:.3}% mean_abs {:.1} ({} ms)",
                    100.0 * cmp.mismatch as f64 / cmp.pixels as f64,
                    100.0 * cmp.interior_mismatch as f64 / cmp.interior.max(1) as f64,
                    cmp.mean_abs,
                    gpu::ms(t.elapsed()) as u64
                );
                checks.push((frame, cmp));
            }
        }
    }

    let invalidation = match path {
        'a' => {
            let a = a.as_ref().unwrap();
            notes.push_str(&format!(
                "fragment_lower_count={} master_hits={} rects_total={}; ",
                a.renderer.fragment_lower_count().unwrap_or(0),
                a.renderer.fragment_master_hits().unwrap_or(0),
                a.rects_total
            ));
            serde_json::json!({
                "registered_fragments": a.inv.registered,
                "registered_rects": a.inv.registered_rects,
                "relower_calls": a.inv.relower_calls,
                "relowered_rects": a.inv.relowered_rects,
                "rects_resident": a.rects_total,
            })
        },
        'b' => {
            let b = b.as_ref().unwrap();
            serde_json::json!({
                "images_uploaded": b.inv.images, "image_bytes": b.inv.bytes,
                "cache_hits": b.inv.cache_hits, "cache_misses": b.inv.cache_misses,
                "part_cache_entries": b.inv.part_cache_entries,
                "measured_images_uploaded": b.inv.images - warm_b.0,
                "measured_image_bytes": b.inv.bytes - warm_b.1,
            })
        },
        _ => {
            let c = c.as_ref().unwrap();
            serde_json::json!({
                "mesh_uploads": c.inv.mesh_uploads,
                "mesh_upload_bytes": c.inv.mesh_upload_bytes,
                "instance_upload_bytes": c.inv.instance_upload_bytes,
                "evictions": c.inv.evictions,
                "draws_total": c.inv.draws,
                "cached_meshes_end": c.last.cached_meshes,
                "instances_per_frame": c.last.instances,
                "cache_capacity": config.mesh_cache,
                "measured_mesh_uploads": c.inv.mesh_uploads - warm_c.mesh_uploads,
                "measured_mesh_upload_bytes": c.inv.mesh_upload_bytes - warm_c.mesh_upload_bytes,
                "measured_instance_upload_bytes": c.inv.instance_upload_bytes - warm_c.instance_upload_bytes,
                "measured_evictions": c.inv.evictions - warm_c.evictions,
            })
        },
    };
    let (rss, _peak) = gpu::rss_mb();
    Run {
        case,
        path,
        frames: totals.len(),
        build_ms: gpu::median(&mut builds),
        encode_ms: gpu::median(&mut encodes),
        poll_ms: gpu::median(&mut polls),
        frame_max_ms: gpu::max(&totals),
        frame_p95_ms: measurement::percentile(&totals, 95),
        frame_ms: gpu::median(&mut totals),
        first_draw_ms,
        checks,
        invalidation,
        rss_mb: rss,
        notes,
        workload: serde_json::json!({
            "version": if config.swarm.is_some() { world::SWARM_VERSION } else { "check_fixtures_v1" },
            "shape_generator": config.swarm.as_ref().and_then(|s| s.shapes).map(|_| world::SHAPE_GENERATOR),
            "requested_shapes": config.swarm.as_ref().and_then(|s| s.shapes),
            "seed": config.swarm.as_ref().map(|s| s.seed),
            "digest": workload.digest,
            "digest_scope": "initial world and all scripted world frames; excludes path-specific pose quantization",
            "initial_geometry": workload.initial_geometry.json(),
            "final_geometry": workload.final_geometry.json(),
            "max_live_unique_volumes": workload.max_live_unique_volumes,
            "unique_volumes_over_run": workload.unique_volumes_over_run,
            "oracle_checked": check,
        }),
    }
}

fn main() -> Result<(), String> {
    let config = Config::from_env()?;
    // Build and validate every workload before creating GPU paths. A too-small
    // C cache is a configuration refusal, not an adapter panic mid-run.
    let workloads: Vec<_> = config
        .cases
        .iter()
        .map(|&case| {
            let workload = Workload::prepare(case, config.swarm.as_ref());
            (case, workload)
        })
        .collect();
    for (case, workload) in &workloads {
        if config.paths.contains(&'c') {
            workload
                .check_cache_capacity(config.mesh_cache)
                .map_err(|error| format!("{}: {error}", case.name()))?;
        }
        eprintln!(
            "workload {} {}: bodies={} requested_shapes={:?} actual_shapes={} live_mesh_keys={} all_mesh_keys={} digest={}",
            if config.swarm.is_some() {
                world::SWARM_VERSION
            } else {
                "check_fixtures_v1"
            },
            case.name(),
            workload.initial_geometry.bodies,
            config.swarm.as_ref().and_then(|s| s.shapes),
            workload.initial_geometry.body_topologies,
            workload.max_live_unique_volumes,
            workload.unique_volumes_over_run,
            workload.digest,
        );
    }
    if config.swarm.is_some() {
        eprintln!(
            "swarm_v2 uses corrected [0,1) RNG coverage; its timings are not the historical half-range swarm workload"
        );
    }
    let backend_env = std::env::var("L0C_BACKEND").unwrap_or_else(|_| "vulkan".into());
    let backends = match backend_env.as_str() {
        "vulkan" => wgpu::Backends::VULKAN,
        "dx12" => wgpu::Backends::DX12,
        _ => wgpu::Backends::all(),
    };
    let handles = boot_with(backends).unwrap_or_else(|e| {
        eprintln!("boot_with({backend_env}) failed: {e}; default boot()");
        netrender::boot().expect("wgpu boot")
    });
    let info = handles.adapter.get_info();
    eprintln!(
        "adapter: {} ({:?}, {:?}) driver={} {}",
        info.name, info.device_type, info.backend, info.driver, info.driver_info
    );

    let out = std::env::var("L0C_OUT")
        .unwrap_or_else(|_| "C:/Users/mark_/Code/testing/wing/l0c_raw.json".into());
    let dump_dir = std::env::var("L0C_DUMP").ok().map(|_| {
        let d = std::path::Path::new(&out)
            .parent()
            .unwrap()
            .join("l0c_frames");
        std::fs::create_dir_all(&d).ok();
        d.to_string_lossy().into_owned()
    });

    let mut runs = Vec::new();
    for (case, workload) in &workloads {
        for &p in &config.paths {
            eprintln!("running {} path {p} ...", case.name());
            let r = run(&handles, *case, p, dump_dir.as_deref(), &config, workload);
            eprintln!(
                "  build {:.2} encode {:.2} poll {:.2} frame {:.2} (p95 {:.2}, max {:.2}) {}",
                r.build_ms,
                r.encode_ms,
                r.poll_ms,
                r.frame_ms,
                r.frame_p95_ms,
                r.frame_max_ms,
                r.notes
            );
            runs.push(r);
        }
    }

    let json = serde_json::json!({
        "probe": "L0c three paths",
        "date": "2026-09-13",
        "schema_version": 2,
        "workload_version": if config.swarm.is_some() { world::SWARM_VERSION } else { "check_fixtures_v1" },
        "swarm_rng": config.swarm.as_ref().map(|_| "lcg_upper24_div_2pow24; uniform fraction [0,1); corrected from legacy half-range"),
        "shape_generator": config.swarm.as_ref().and_then(|s| s.shapes).map(|_| world::SHAPE_GENERATOR),
        "requested_shapes": config.swarm.as_ref().and_then(|s| s.shapes),
        "seed": config.swarm.as_ref().map(|s| s.seed),
        "mesh_cache_capacity": config.mesh_cache,
        "viewport": [W, H],
        "frames": FRAMES,
        "bodies": config.swarm.as_ref().map(|s| s.bodies),
        "warmup": WARMUP,
        "check_frames": CHECK_FRAMES,
        "oracle_checked": config.swarm.is_none() || config.check_swarm,
        "swarm_check_limit": MAX_CHECK_SWARM,
        "invalidation_scope": "warmup plus measured frames, except measured_* fields",
        "percentile_method": "nearest rank over finite measured frame samples",
        "adapter": {
            "name": info.name, "backend": format!("{:?}", info.backend),
            "device_type": format!("{:?}", info.device_type),
            "driver": info.driver, "driver_info": info.driver_info,
        },
        "runs": runs.iter().map(|r| serde_json::json!({
            "case": r.case.name(), "path": r.path.to_string(), "frames": r.frames,
            "build_ms": gpu::jf(r.build_ms), "encode_ms": gpu::jf(r.encode_ms), "poll_ms": gpu::jf(r.poll_ms),
            "frame_ms": gpu::jf(r.frame_ms), "frame_max_ms": gpu::jf(r.frame_max_ms),
            "frame_p95_ms": gpu::jf(r.frame_p95_ms),
            "pose_policy": if r.path == 'b' { "quarter_turn_sprites" } else { "continuous_body_yaw" },
            "first_draw": {
                "build_ms": gpu::jf(r.first_draw_ms[0]),
                "encode_ms": gpu::jf(r.first_draw_ms[1]),
                "poll_ms": gpu::jf(r.first_draw_ms[2]),
                "frame_ms": gpu::jf(r.first_draw_ms.iter().sum()),
                "scope": "first path.frame call; excludes GPU bootstrap and path construction",
            },
            "workload": r.workload,
            "checks": r.checks.iter().map(|(f, c)| serde_json::json!({
                "frame": f, "pixels": c.pixels, "mismatch": c.mismatch,
                "interior": c.interior, "interior_mismatch": c.interior_mismatch,
                "mean_abs": gpu::jf(c.mean_abs),
            })).collect::<Vec<_>>(),
            "invalidation": r.invalidation, "rss_mb": gpu::jf(r.rss_mb), "notes": r.notes,
        })).collect::<Vec<_>>(),
    });
    std::fs::write(&out, serde_json::to_string_pretty(&json).unwrap()).expect("write json");
    eprintln!("wrote {out}");
    Ok(())
}
