// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The Meaning channel's sentence model on the host's device (dynamics
//! grammar plan, G2, P7's Meaning condition): on the arXiv fixture (900
//! titles, six categories; F42, F43), cluster purity is recorded on the native
//! GPU, on the CPU with the same model, and on the lexical fallback, F34's bar
//! is asserted, and the GPU path shares the host's device.
//!
//! "A single device" is asserted three ways: the engine's device is the one
//! the host's `PhysicsDevice` registered; loading the model raises that
//! client's own memory in use by the model's size; and a control model on a
//! freshly booted device leaves the host's client untouched, so the
//! instrument can tell two devices apart.
//!
//! The host boots its device the way netrender boots one for a JIT compute
//! tenant (`TenantNeeds::greedy`), holding every feature the adapter has.
//! CubeCL picks its kernel timing from the adapter's features, so on a
//! default-booted device, which lacks `TIMESTAMP_QUERY`, Burn's first timed
//! launch fails validation (the first run of this receipt, kept in the lane's
//! `meaning-device.log`).
//!
//! The model is the pinned one (`MeaningModel::pinned`, F56 and F58:
//! e5-base-v2 at its matched revision, mean-pooled, with the `query: `
//! prefix, at top-k 16 and classical modularity). The purity receipt is
//! ignored by default: it needs the local models directory and an adapter.
//! The boot receipt needs an adapter only.
//!
//! `ESP_MODELS_DIR=<repo>/models cargo test --release -p pictograph
//! --features meaning-gpu --test meaning_device -- --ignored --nocapture
//! --test-threads=1`

#![cfg(feature = "meaning-gpu")]

mod meaning_common;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use esp::embed::bert::{Device, DeviceKind};
use meaning_common::meaning_topics::{arxiv_graph, partition, shuffled};
use meaning_common::{print_confusion, row, snapshot_on};
use pictograph::canvas::meaning_device::{
    DeviceMeaning, check_meaning_device, host_meaning_device,
};
use pictograph::canvas::meaning_model::MeaningModel;
use pictograph::canvas::{
    Canvas, LexicalMeaning, MeaningBackend, MeaningEngine, MeaningParams, PhysicsChoice,
    PhysicsKindSource, PhysicsLaw, physics_device_for,
};

const MEBIBYTE: u64 = 1 << 20;

fn models_dir() -> PathBuf {
    std::env::var_os("ESP_MODELS_DIR")
        .map(PathBuf::from)
        .expect("ESP_MODELS_DIR must point at the local models directory")
}

#[test]
#[ignore = "requires the local models directory (ESP_MODELS_DIR) and a wgpu adapter"]
fn the_model_on_the_host_device_shares_it_and_records_purity() {
    let models = models_dir();
    let pinned = MeaningModel::pinned();
    println!(
        "model {} at {} ({}, {} pooling, prefix {:?}), tuning {:?}",
        pinned.model_id,
        pinned.revision,
        pinned.license,
        pinned.pooling,
        pinned.prefix,
        MeaningParams::MODEL
    );
    let plain = netrender::boot().expect("a wgpu adapter");
    let info = plain.adapter.get_info();
    println!(
        "adapter: {} ({:?}); a default boot's device lacks {:?}",
        info.name,
        info.backend,
        plain.adapter.features() - plain.device.features()
    );
    drop(plain);
    let needs = netrender::TenantNeeds {
        greedy: true,
        label: Some("meaning receipt host"),
        ..Default::default()
    };
    let handles = netrender::boot_shared(info.backend.into(), None, &needs)
        .expect("a device for a JIT compute tenant");
    println!(
        "a tenant boot's device lacks {:?}",
        handles.adapter.features() - handles.device.features()
    );
    let device = physics_device_for(&handles);
    let in_use = || {
        device.client().bytes_in_use()
    };

    // One device: the engine's is the one the physics device registered.
    let before = in_use();
    let started = Instant::now();
    let engine = DeviceMeaning::load_pinned(&models, &device)
        .expect("the model loads on the host's device")
        .with_pair_threshold(0);
    let load = started.elapsed();
    assert_eq!(engine.device(), &host_meaning_device(&device));
    println!("the engine's device: {:?}", engine.device());
    let loaded = in_use();
    println!(
        "host client bytes in use: {before} before the model, {loaded} after ({:.1} MiB more), load {load:?}",
        (loaded.saturating_sub(before)) as f64 / MEBIBYTE as f64
    );
    let weights = pinned.artifacts.weights.bytes;
    assert!(
        loaded.saturating_sub(before) >= weights * 9 / 10,
        "the model's weights ({weights} bytes) live on the host's client"
    );

    // The control: a model on a freshly booted device leaves the host's
    // client where it was.
    let control =
        DeviceMeaning::load_model_on(pinned, &models, Device::wgpu(DeviceKind::DefaultDevice))
            .expect("the control model loads on a device of its own");
    let after_control = in_use();
    println!(
        "control device {:?}: host client bytes in use {after_control} ({} bytes moved)",
        control.device(),
        after_control as i64 - loaded as i64
    );
    assert_ne!(
        control.device(),
        engine.device(),
        "the control is another device"
    );
    assert!(
        after_control.abs_diff(loaded) < MEBIBYTE,
        "a second device does not show on the host's client"
    );
    drop(control);

    let (_, keys, topics) = arxiv_graph();
    let shuffled = shuffled(&keys, &topics);

    // Off-path: on an offloaded canvas the run happens on the Meaning actor,
    // and no frame waits for it.
    let (graph, _, _) = arxiv_graph();
    let mut canvas = Canvas::with_graph(graph);
    canvas.offload_physics(Arc::new(|| {}));
    canvas.set_physics_device(Some(device.clone()));
    let engine: Arc<dyn MeaningEngine> = Arc::new(engine);
    canvas.set_meaning_engine(engine.clone());
    // Warm the canvas first: its first frame over 900 nodes builds the node
    // pool and is slow, and a run that lands inside it would hide whether
    // frames go on while a run is in flight.
    let mut warm = Vec::new();
    for _ in 0..3 {
        let frame = Instant::now();
        canvas.frame(1024, 600);
        warm.push(frame.elapsed());
    }
    println!("warm-up frames before the dispatch: {warm:?}");
    // Kinds by meaning, set through the canvas's spec and its flat view
    // (dynamics grammar plan, F162).
    let mut spec = canvas.dynamics_spec().expect("the record reads");
    PhysicsChoice {
        law: PhysicsLaw::Kinds,
        kind: PhysicsKindSource::Meaning,
        ..PhysicsChoice::live(&canvas)
    }
    .write_into(&mut spec, &PhysicsChoice::live(&canvas));
    canvas.set_dynamics_spec(&spec).expect("not refused");
    // The build dispatched the run; frames go on while it is in flight.
    let dispatched = Instant::now();
    let mut in_flight: Vec<Duration> = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(600);
    while canvas.meaning().is_none() {
        assert!(Instant::now() < deadline, "the actor's run landed");
        let frame = Instant::now();
        canvas.frame(1024, 600);
        if canvas.meaning().is_none() {
            in_flight.push(frame.elapsed());
        }
        std::thread::sleep(Duration::from_millis(4));
    }
    let landed = dispatched.elapsed();
    for _ in 0..30 {
        canvas.frame(1024, 600);
    }
    let gpu = canvas.meaning().unwrap().clone();
    assert_eq!(canvas.meaning_runs(), 1, "one run for one content");
    in_flight.sort();
    println!(
        "offloaded: {} run(s), landed {landed:?} after dispatch; {} frames rendered while it was \
         in flight, median {:?}, slowest {:?}",
        canvas.meaning_runs(),
        in_flight.len(),
        in_flight
            .get(in_flight.len() / 2)
            .copied()
            .unwrap_or_default(),
        in_flight.last().copied().unwrap_or_default(),
    );
    assert!(
        in_flight.len() >= 2,
        "frames went on while the run was in flight"
    );

    // A second content, on warm kernels: one more run, off-path again.
    canvas.ingest_graph(|g| {
        use kernel::graph::fixtures::GraphFixtures;
        g.set_node_title(
            keys[3],
            "Watching meteor showers from a dark site".to_string(),
        )
    });
    let edited = Instant::now();
    let mut frames = 0;
    while canvas.meaning().is_some_and(|s| s.run == 1) {
        assert!(
            edited.elapsed() < Duration::from_secs(600),
            "the second run landed"
        );
        canvas.frame(1024, 600);
        frames += 1;
        std::thread::sleep(Duration::from_millis(4));
    }
    println!(
        "second content: run {} landed {:?} after the edit, {frames} frames meanwhile",
        canvas.meaning().unwrap().run,
        edited.elapsed()
    );
    assert_eq!(
        canvas.meaning_runs(),
        2,
        "one more run for one more content"
    );
    assert_eq!(gpu.backend, MeaningBackend::ModelGpu);
    let gpu_row = row("model on the host's GPU", &gpu.groups, &topics, &shuffled);

    // The same model on the CPU, and the lexical fallback.
    let started = Instant::now();
    let cpu_engine = pinned
        .load_cpu(&models)
        .expect("the model loads on the CPU");
    let (cpu, cpu_runs) = snapshot_on(Arc::new(cpu_engine));
    println!("model on the CPU: load and run {:?}", started.elapsed());
    assert_eq!(cpu_runs, 1);
    let cpu_row = row("model on the CPU", &cpu.groups, &topics, &shuffled);
    let (lexical, _) = snapshot_on(Arc::new(LexicalMeaning::new()));
    assert_eq!(lexical.backend, MeaningBackend::Lexical);
    let lexical_row = row(
        "lexical fallback on the CPU",
        &lexical.groups,
        &topics,
        &shuffled,
    );
    println!(
        "pairs: GPU {}, CPU {}, lexical {}; GPU and CPU partitions equal: {}",
        gpu.pairs.len(),
        cpu.pairs.len(),
        lexical.pairs.len(),
        partition(&gpu.groups) == partition(&cpu.groups)
    );

    print_confusion("model on the host's GPU", &gpu.groups, &topics);
    print_confusion("lexical fallback", &lexical.groups, &topics);

    // The measure says no to shuffled topics on every backend, and the model
    // knows more than the lexical fallback.
    for (name, (f, control)) in [("GPU", gpu_row), ("CPU", cpu_row), ("lexical", lexical_row)] {
        assert!(control < f, "{name}: {f:.3} against shuffled {control:.3}");
    }
    assert!(
        gpu_row.0 > lexical_row.0,
        "{gpu_row:?} against {lexical_row:?}"
    );
    assert!(
        (gpu_row.0 - cpu_row.0).abs() < 0.05,
        "{gpu_row:?} against {cpu_row:?}"
    );
    // F34's bar: the model at F >= 0.9 on GPU and CPU.
    assert!(gpu_row.0 >= 0.9, "the model on the GPU: F {:.3}", gpu_row.0);
    assert!(cpu_row.0 >= 0.9, "the model on the CPU: F {:.3}", cpu_row.0);
}

/// F31: a host enabling the model boots its device greedy. A device booted
/// the default way, on an adapter that offers timestamp queries, is refused
/// with the reason before any model is read (the model path here does not
/// exist); the greedy boot passes the same check. Skips without an adapter;
/// says so when the adapter has no timestamps to miss.
#[test]
fn a_default_boot_is_refused_with_its_reason_and_a_greedy_boot_passes() {
    let Ok(plain) = netrender::boot() else {
        eprintln!("no wgpu adapter: skipping the boot receipt");
        return;
    };
    let info = plain.adapter.get_info();
    let default_device = physics_device_for(&plain);
    let features = default_device
        .features()
        .expect("built from the host's handles");
    println!(
        "adapter {} ({:?}): default boot lacks the adapter's timestamps: {}",
        info.name,
        info.backend,
        features.lacks_adapter_timestamps()
    );
    if !features.lacks_adapter_timestamps() {
        eprintln!("this adapter offers no timestamp queries: nothing for a default boot to lack");
        return;
    }
    let refused = DeviceMeaning::load("no/such/model", &default_device)
        .err()
        .expect("a default boot is refused");
    println!("refused: {refused}");
    assert!(refused.to_string().contains("TIMESTAMP_QUERY"), "{refused}");
    assert!(refused.to_string().contains("greedy"), "{refused}");
    drop(default_device);
    drop(plain);

    let needs = netrender::TenantNeeds {
        greedy: true,
        ..Default::default()
    };
    let greedy = netrender::boot_shared(info.backend.into(), None, &needs).expect("a greedy boot");
    let device = physics_device_for(&greedy);
    assert!(!device.features().unwrap().lacks_adapter_timestamps());
    check_meaning_device(&device).expect("a greedy boot carries the model");
    // Past the check, a missing model is a load error, not the boot refusal.
    let missing = DeviceMeaning::load("no/such/model", &device)
        .err()
        .expect("no model at that path");
    assert!(
        !missing.to_string().contains("TIMESTAMP_QUERY"),
        "{missing}"
    );
}

/// F58's manifest: it parses, names the pinned revision, pooling and prefix,
/// and the load check refuses a model directory that is missing or holds an
/// artifact at another size. Needs no model and no adapter.
#[test]
fn the_pinned_manifest_parses_and_the_load_check_refuses_a_wrong_size() {
    let pinned = MeaningModel::pinned();
    assert_eq!(pinned.model_id, "intfloat/e5-base-v2");
    assert_eq!(pinned.revision, "f52bf8ec8c7124536f0efb74aca902b2995e5bcd");
    assert_eq!(pinned.license, "MIT");
    assert_eq!(pinned.prefix, "query: ");
    assert_eq!(
        pinned.pooling().expect("a known pooling"),
        esp::embed::bert::Pooling::Mean
    );
    let root = std::env::temp_dir().join(format!("meaning-model-check-{}", std::process::id()));
    let missing = pinned.check(&root).expect_err("no model directory");
    assert!(missing.to_string().contains(&pinned.model_id), "{missing}");
    let dir = root.join(&pinned.directory);
    std::fs::create_dir_all(&dir).unwrap();
    for artifact in pinned.artifacts() {
        std::fs::write(dir.join(&artifact.file), b"not the pinned bytes").unwrap();
    }
    let wrong = pinned.check(&root).expect_err("artifacts at other sizes");
    println!("refused: {wrong}");
    assert!(wrong.to_string().contains("pinned at"), "{wrong}");
    std::fs::remove_dir_all(&root).unwrap();
}
