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
//! The purity receipt is ignored by default: it needs the local MiniLM
//! artifact and an adapter. The boot receipt needs an adapter only.
//!
//! `ESP_MINILM_DIR=<repo>/models/all-MiniLM-L6-v2 cargo test -p pictograph
//! --features meaning-gpu --test meaning_device -- --ignored --nocapture
//! --test-threads=1`

#![cfg(feature = "meaning-gpu")]

mod meaning_common;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use esp::embed::bert::{Device, DeviceKind};
use meaning_common::meaning_topics::{arxiv_graph, partition, shuffled};
use meaning_common::{Fixed, print_confusion, row, snapshot_on};
use pictograph::canvas::meaning_device::{
    DeviceMeaning, check_meaning_device, host_meaning_device,
};
use pictograph::canvas::{
    Canvas, LexicalMeaning, MeaningBackend, MeaningEngine, MeaningParams, PhysicsKindSource,
    PhysicsLaw, ProviderMeaning, physics_device_for,
};

const MEBIBYTE: u64 = 1 << 20;

fn model_dir() -> PathBuf {
    std::env::var_os("ESP_MINILM_DIR")
        .map(PathBuf::from)
        .expect("ESP_MINILM_DIR must point at the local all-MiniLM-L6-v2 artifact")
}

#[test]
#[ignore = "requires the local all-MiniLM-L6-v2 artifact (ESP_MINILM_DIR) and a wgpu adapter"]
fn the_model_on_the_host_device_shares_it_and_records_purity() {
    let model = model_dir();
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
        device
            .client()
            .compute_client()
            .memory_usage()
            .expect("the host client's memory usage")
            .bytes_in_use
    };

    // One device: the engine's is the one the physics device registered.
    let before = in_use();
    let started = Instant::now();
    let engine = DeviceMeaning::load(&model, &device)
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
    assert!(
        loaded.saturating_sub(before) >= 50 * MEBIBYTE,
        "the model's weights live on the host's client"
    );

    // The control: a model on a freshly booted device leaves the host's
    // client where it was.
    let control = DeviceMeaning::load_on(&model, Device::wgpu(DeviceKind::DefaultDevice))
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
    canvas.set_physics_kind_source(PhysicsKindSource::Meaning);
    canvas
        .set_physics_law(PhysicsLaw::Kinds)
        .expect("not refused");
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
    let cpu_engine = ProviderMeaning::new(
        esp::embed::bert::load_cpu(&model).expect("the model loads on the CPU"),
        MeaningBackend::ModelCpu,
    );
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

    // The model's tuning neighbourhood, over the GPU's vectors embedded once,
    // for the record (the search and partition run per tuning).
    let texts: Vec<String> = {
        let (graph, _, _) = arxiv_graph();
        keys.iter()
            .map(|k| graph.get_node(*k).unwrap().title.clone())
            .collect()
    };
    let text_refs: Vec<&str> = texts.iter().map(String::as_str).collect();
    let vectors = engine.embed(&text_refs).expect("the GPU embeds the corpus");
    let mut best = (0.0, MeaningParams::MODEL);
    for top_k in [2, 4, 8, 16, 32, 64] {
        for min_similarity in [0.1, 0.15, 0.2, 0.25, 0.3, 0.4] {
            let params = MeaningParams {
                top_k,
                min_similarity,
                resolution: 1.0,
            };
            let (swept, _) = snapshot_on(Arc::new(Fixed {
                vectors: vectors.clone(),
                params,
                backend: MeaningBackend::ModelGpu,
            }));
            let (f, _) = row(
                &format!("model sweep top_k {top_k} min {min_similarity}"),
                &swept.groups,
                &topics,
                &shuffled,
            );
            if f > best.0 {
                best = (f, params);
            }
        }
    }
    println!("model sweep best: F {:.3} at {:?}", best.0, best.1);

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
