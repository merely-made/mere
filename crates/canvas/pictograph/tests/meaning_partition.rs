// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! F50 (dynamics grammar plan, G2): Louvain's resolution tuned against the
//! arXiv fixture and F34's bar (F ≥ 0.9), on MiniLM; then the other models
//! already on disk under the same partition grid; then the lexical fallback
//! re-swept with the resolution (F51). Records, asserts nothing about the
//! bar: the tuning is Mark's to adopt. Each sentence model is embedded once
//! on the host's GPU (greedy boot, F31), and every tuning replays those
//! vectors through the canvas's own Meaning path.
//!
//! None of the three other models' directories records its pooling or a
//! prompt prefix (no `1_Pooling`, no model card), so each is measured with
//! mean and with CLS pooling, and the e5 models also with the `query: `
//! prefix their family is known for. MiniLM's own pooling config (mean) is on
//! disk, and it alone is measured that way.
//!
//! `ESP_MODELS_DIR=<repo>/models cargo test --release -p pictograph
//! --features meaning-gpu --test meaning_partition -- --ignored --nocapture
//! --test-threads=1`

#![cfg(feature = "meaning-gpu")]

mod meaning_common;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use esp::embed::EmbeddingProvider;
use esp::embed::bert::{BertEmbeddingProvider, Pooling};
use meaning_common::meaning_topics::{arxiv_graph, f_measure, inverse_purity, purity, shuffled};
use meaning_common::{Fixed, arxiv_titles, print_confusion, row, snapshot_on};
use pictograph::canvas::meaning_device::{check_meaning_device, host_meaning_device};
use pictograph::canvas::{
    Embedded, LexicalMeaning, MeaningBackend, MeaningParams, MeaningSnapshot, physics_device_for,
};

const TOP_K: [usize; 4] = [4, 8, 16, 32];
const RESOLUTIONS: [f32; 11] = [0.25, 0.5, 0.75, 0.9, 1.0, 1.1, 1.25, 1.5, 2.0, 3.0, 4.0];
const FLOOR: f32 = 0.15;
/// A tuning reads as meeting the bar only at its edge when it clears 0.9 by
/// less than this (about nine titles of 900; *Reading*, the lane's line).
const EDGE: f64 = 0.01;

struct Variant {
    model: &'static str,
    pooling: Pooling,
    prefix: &'static str,
}

impl Variant {
    fn name(&self) -> String {
        format!(
            "{} {}{}",
            self.model,
            match self.pooling {
                Pooling::Cls => "cls",
                _ => "mean",
            },
            if self.prefix.is_empty() {
                String::new()
            } else {
                format!(" prefix {:?}", self.prefix)
            }
        )
    }
}

/// One cell of a sweep.
struct Cell {
    params: MeaningParams,
    f: f64,
    control: f64,
    snapshot: MeaningSnapshot,
}

fn sweep(
    name: &str,
    vectors: &Embedded,
    backend: MeaningBackend,
    topics: &std::collections::HashMap<kernel::graph::NodeKey, usize>,
    shuffled: &std::collections::HashMap<kernel::graph::NodeKey, usize>,
) -> Vec<Cell> {
    let mut cells = Vec::new();
    for top_k in TOP_K {
        for resolution in RESOLUTIONS {
            let params = MeaningParams {
                top_k,
                min_similarity: FLOOR,
                resolution,
            };
            let (snapshot, _) = snapshot_on(Arc::new(Fixed {
                vectors: vectors.clone(),
                params,
                backend,
            }));
            let (f, control) = row(
                &format!("{name} sweep top_k {top_k} floor {FLOOR} resolution {resolution}"),
                &snapshot.groups,
                topics,
                shuffled,
            );
            cells.push(Cell {
                params,
                f,
                control,
                snapshot,
            });
        }
    }
    cells
}

fn best(cells: &[Cell]) -> &Cell {
    cells
        .iter()
        .max_by(|a, b| a.f.total_cmp(&b.f))
        .expect("a sweep has cells")
}

fn verdict(f: f64) -> &'static str {
    if f < 0.9 {
        "misses the bar"
    } else if f - 0.9 < EDGE {
        "meets the bar at its edge"
    } else {
        "meets the bar"
    }
}

#[test]
#[ignore = "requires the local models (ESP_MODELS_DIR) and a wgpu adapter; a release receipt"]
fn the_partitions_resolution_and_the_local_models_on_the_arxiv_bar() {
    let models = std::env::var_os("ESP_MODELS_DIR")
        .map(PathBuf::from)
        .expect("ESP_MODELS_DIR must point at the local models directory");
    let plain = netrender::boot().expect("a wgpu adapter");
    let backend = plain.adapter.get_info().backend;
    drop(plain);
    let needs = netrender::TenantNeeds {
        greedy: true,
        ..Default::default()
    };
    let handles =
        netrender::boot_shared(backend.into(), None, &needs).expect("a greedy boot (F31)");
    let device = physics_device_for(&handles);
    check_meaning_device(&device).expect("the host's device carries a model");
    println!("adapter: {}", handles.adapter.get_info().name);

    let (_, keys, topics) = arxiv_graph();
    let shuffled = shuffled(&keys, &topics);
    let titles = arxiv_titles();

    let variants = [
        Variant {
            model: "all-MiniLM-L6-v2",
            pooling: Pooling::Mean,
            prefix: "",
        },
        Variant {
            model: "bge-micro-v2",
            pooling: Pooling::Mean,
            prefix: "",
        },
        Variant {
            model: "bge-micro-v2",
            pooling: Pooling::Cls,
            prefix: "",
        },
        Variant {
            model: "e5-small-v2",
            pooling: Pooling::Mean,
            prefix: "",
        },
        Variant {
            model: "e5-small-v2",
            pooling: Pooling::Mean,
            prefix: "query: ",
        },
        Variant {
            model: "e5-small-v2",
            pooling: Pooling::Cls,
            prefix: "",
        },
        Variant {
            model: "e5-base-v2",
            pooling: Pooling::Mean,
            prefix: "",
        },
        Variant {
            model: "e5-base-v2",
            pooling: Pooling::Mean,
            prefix: "query: ",
        },
        Variant {
            model: "e5-base-v2",
            pooling: Pooling::Cls,
            prefix: "",
        },
    ];
    let mut summary: Vec<(String, MeaningParams, f64, f64, MeaningSnapshot, usize)> = Vec::new();
    for (index, variant) in variants.iter().enumerate() {
        let name = variant.name();
        let started = Instant::now();
        let provider =
            BertEmbeddingProvider::load(models.join(variant.model), host_meaning_device(&device))
                .unwrap_or_else(|error| panic!("{name}: {error}"))
                .with_pooling(variant.pooling);
        let texts: Vec<String> = titles
            .iter()
            .map(|t| format!("{}{t}", variant.prefix))
            .collect();
        let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
        let vectors = Embedded::Dense(provider.embed(&refs).expect("the model embeds the corpus"));
        println!(
            "{name}: loaded and embedded 900 titles on the GPU in {:?}",
            started.elapsed()
        );
        drop(provider);
        let cells = sweep(
            &name,
            &vectors,
            MeaningBackend::ModelGpu,
            &topics,
            &shuffled,
        );
        let at_recorded = cells
            .iter()
            .find(|c| c.params.top_k == 4 && c.params.resolution == 1.0)
            .expect("the recorded tuning is in the grid");
        let top = best(&cells);
        assert!(
            top.control < top.f,
            "{name}: the best beats shuffled topics"
        );
        println!(
            "{name} summary: at the recorded tuning (k 4, resolution 1) F {:.3}; best F {:.3} at k {} resolution {} \
             (purity {:.3}, inverse purity {:.3}, {} groups, shuffled {:.3}): {}",
            at_recorded.f,
            top.f,
            top.params.top_k,
            top.params.resolution,
            purity(&top.snapshot.groups, &topics),
            inverse_purity(&top.snapshot.groups, &topics),
            meaning_common::meaning_topics::partition(&top.snapshot.groups).len(),
            top.control,
            verdict(top.f),
        );
        let meeting: Vec<String> = cells
            .iter()
            .filter(|c| c.f >= 0.9)
            .map(|c| {
                format!(
                    "k {} r {} F {:.3}",
                    c.params.top_k, c.params.resolution, c.f
                )
            })
            .collect();
        println!("{name}: cells at or above the bar: {meeting:?}");
        summary.push((
            name,
            top.params,
            top.f,
            top.control,
            top.snapshot.clone(),
            index,
        ));
    }

    summary.sort_by(|a, b| b.2.total_cmp(&a.2));
    println!("ranking by best F:");
    for (name, params, f, control, _, _) in &summary {
        println!(
            "  {name}: F {f:.3} at k {} resolution {} ({}), shuffled {control:.3}",
            params.top_k,
            params.resolution,
            verdict(*f)
        );
    }
    for (name, params, _, _, snapshot, _) in summary.iter().take(2) {
        print_confusion(
            &format!(
                "{name} at k {} resolution {}",
                params.top_k, params.resolution
            ),
            &snapshot.groups,
            &topics,
        );
    }

    // The best variant on the CPU, at its best tuning: the same partition?
    let (name, params, f, _, gpu, index) = &summary[0];
    let variant = &variants[*index];
    let started = Instant::now();
    let provider = BertEmbeddingProvider::load(
        models.join(variant.model),
        esp::embed::bert::Device::ndarray(),
    )
    .expect("the model loads on the CPU")
    .with_pooling(variant.pooling);
    let texts: Vec<String> = titles
        .iter()
        .map(|t| format!("{}{t}", variant.prefix))
        .collect();
    let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
    let cpu_vectors = Embedded::Dense(provider.embed(&refs).expect("the CPU embeds the corpus"));
    let (cpu, _) = snapshot_on(Arc::new(Fixed {
        vectors: cpu_vectors,
        params: *params,
        backend: MeaningBackend::ModelCpu,
    }));
    let cpu_f = f_measure(&cpu.groups, &topics);
    println!(
        "{name} on the CPU at its best tuning: F {cpu_f:.3} against the GPU's {f:.3}; the same partition: {} ({:?})",
        meaning_common::meaning_topics::partition(&cpu.groups)
            == meaning_common::meaning_topics::partition(&gpu.groups),
        started.elapsed()
    );
}

/// F51: the lexical fallback re-swept with the resolution, on the CPU.
#[test]
#[ignore = "a release receipt"]
fn the_lexical_fallback_re_swept_with_the_resolution() {
    let (_, keys, topics) = arxiv_graph();
    let shuffled = shuffled(&keys, &topics);
    let mut cells = Vec::new();
    for top_k in [4, 8, 16] {
        for min_similarity in [0.1, 0.15] {
            for resolution in RESOLUTIONS {
                let params = MeaningParams {
                    top_k,
                    min_similarity,
                    resolution,
                };
                let (snapshot, _) =
                    snapshot_on(Arc::new(LexicalMeaning::new().with_params(params)));
                let (f, control) = row(
                    &format!(
                        "lexical sweep top_k {top_k} floor {min_similarity} resolution {resolution}"
                    ),
                    &snapshot.groups,
                    &topics,
                    &shuffled,
                );
                cells.push((params, f, control));
            }
        }
    }
    let (params, f, control) = cells
        .iter()
        .copied()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap();
    println!("lexical re-sweep best: F {f:.3} at {params:?}, shuffled {control:.3}");
}

/// What each leading model costs on the host's device: its weights' bytes on
/// the host's client, and the 900 titles embedded twice (the second warm).
#[test]
#[ignore = "requires the local models (ESP_MODELS_DIR) and a wgpu adapter; a release receipt"]
fn the_leading_models_cost_on_the_host_device() {
    let models = std::env::var_os("ESP_MODELS_DIR")
        .map(PathBuf::from)
        .expect("ESP_MODELS_DIR must point at the local models directory");
    let plain = netrender::boot().expect("a wgpu adapter");
    let backend = plain.adapter.get_info().backend;
    drop(plain);
    let needs = netrender::TenantNeeds {
        greedy: true,
        ..Default::default()
    };
    let handles =
        netrender::boot_shared(backend.into(), None, &needs).expect("a greedy boot (F31)");
    let device = physics_device_for(&handles);
    let in_use = || {
        device.client().bytes_in_use()
    };
    let titles = arxiv_titles();
    let refs: Vec<&str> = titles.iter().map(String::as_str).collect();
    for model in ["all-MiniLM-L6-v2", "e5-small-v2", "e5-base-v2"] {
        let before = in_use();
        let started = Instant::now();
        let provider =
            BertEmbeddingProvider::load(models.join(model), host_meaning_device(&device))
                .expect("the model loads");
        let load = started.elapsed();
        let loaded = in_use();
        let started = Instant::now();
        let first = provider.embed(&refs).expect("the model embeds the corpus");
        let cold = started.elapsed();
        let started = Instant::now();
        let second = provider.embed(&refs).expect("the model embeds the corpus");
        let warm = started.elapsed();
        assert_eq!(first, second, "{model}: one embedding twice");
        println!(
            "{model}: weights {:.1} MiB on the host's client, load {load:?}, 900 titles {cold:?} then {warm:?} warm, width {}",
            loaded.saturating_sub(before) as f64 / (1024.0 * 1024.0),
            first[0].len()
        );
        drop(provider);
    }
}
