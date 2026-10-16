// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

#![cfg(feature = "inference-model")]

#[path = "../../../crates/intel/esp/tests/support/decoder.rs"]
mod fixture;

use distillery::inference::{AskRequestV1, RunStateV1, open_library_lane_with_journal};
use eidetic::typed::save_typed;
use eidetic::{PrivacyClass, ProvenanceRecord, Store, Timestamp, TrustEnvelope};
use esp::infer::decoder::streaming::StreamingOptions;
use esp::infer::layer_library::import_layered_model;
use esp::infer::library::{InferenceModelRow, resolve_library_session};
use esp::infer::prompt::PromptTemplate;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[test]
fn resident_lane_reads_layers_from_its_owner_and_releases_the_library_on_close() {
    exercise_layer_owner(false);
}

#[test]
fn resident_lane_selects_flex_for_a_qualified_bf16_row() {
    exercise_layer_owner(true);
}

fn exercise_layer_owner(half: bool) {
    let directory = tempfile::tempdir().unwrap();
    let mut config: serde_json::Value = serde_json::from_str(fixture::CONFIG_JSON).unwrap();
    config["max_position_embeddings"] = serde_json::json!(256);
    std::fs::write(
        directory.path().join("config.json"),
        serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    std::fs::write(
        directory.path().join("tokenizer.json"),
        fixture::tokenizer_json(),
    )
    .unwrap();
    std::fs::write(
        directory.path().join("model.safetensors"),
        fixture::weights_bytes(),
    )
    .unwrap();
    let path = directory.path().join("library.redb");
    let mut store = muniment::RedbBackend::open(&path).unwrap();
    let (row, damaged_key) = pollster::block_on(async {
        let model = if half {
            esp::infer::layer_library::import_formatted_model(
                &mut store,
                directory.path(),
                "fixture/layered",
                "MPL-2.0",
                StreamingOptions::default(),
                esp::infer::decoder::WeightFormat::Bf16,
            )
            .await
            .unwrap()
        } else {
            import_layered_model(
                &mut store,
                directory.path(),
                "fixture/layered",
                "MPL-2.0",
                StreamingOptions::default(),
            )
            .await
            .unwrap()
        };
        let template = PromptTemplate::from_bytes(include_bytes!(
            "../../../crates/intel/esp/templates/smollm2-instruct-v1.json"
        ))
        .unwrap();
        let template = save_typed(
            &mut store,
            &template,
            vec![],
            PrivacyClass::LocalOnly,
            ProvenanceRecord::self_imported("local"),
            TrustEnvelope::self_asserted(),
            Timestamp::ZERO,
        )
        .await
        .unwrap();
        let row = save_typed(
            &mut store,
            &InferenceModelRow { model, template },
            vec![],
            PrivacyClass::LocalOnly,
            ProvenanceRecord::self_imported("local"),
            TrustEnvelope::self_asserted(),
            Timestamp::ZERO,
        )
        .await
        .unwrap();
        let row = if half {
            use esp::infer::layer_library::{
                FormatQualification, FormattedLayerWeightIndex, PromptDeviation,
                qualify_formatted_model,
            };
            use esp::infer::{GenerationRequest, InferenceProvider};
            let resolved = esp::infer::library::resolve_library_candidate(&mut store, row)
                .await
                .unwrap();
            let manifest = eidetic::ModelLibrary::load_model(&mut store, model)
                .await
                .unwrap()
                .unwrap();
            let index = eidetic::typed::load_typed::<FormattedLayerWeightIndex>(
                &mut store,
                &mut eidetic::NoFetcher,
                manifest.weight_blob,
            )
            .await
            .unwrap()
            .unwrap();
            let mut layers = Vec::new();
            for artifact in resolved.layer_artifacts().unwrap() {
                layers.push(
                    artifact
                        .read(&mut store, index.weights.options.max_layer_bytes)
                        .await
                        .unwrap(),
                );
            }
            let provider = resolved
                .load_with_layer_source(
                    &esp::infer::decoder::DecoderDevice::flex(),
                    "burn-flex",
                    8,
                    Some(Box::new(move |i: usize, _| Ok(layers[i].clone()))),
                )
                .unwrap();
            let start = Instant::now();
            let result = provider
                .generate_checkpointed(
                    &GenerationRequest {
                        prompt: "t3 t7".into(),
                        max_tokens: 4,
                        ..Default::default()
                    },
                    None,
                    &mut |_| Ok(()),
                    &mut || false,
                )
                .unwrap();
            let rendered = PromptTemplate::from_bytes(include_bytes!(
                "../../../crates/intel/esp/templates/smollm2-instruct-v1.json"
            ))
            .unwrap()
            .render("t3 t7", "")
            .unwrap();
            let qualification = FormatQualification {
                baseline_model: model,
                baseline_loader: provider.capability().loader.clone(),
                measured_loader: provider.capability().loader.clone(),
                prompts: vec![PromptDeviation {
                    rendered_prompt: rendered,
                    baseline_tokens: result.token_ids.clone(),
                    format_tokens: result.token_ids.clone(),
                    logit_rmse: 0.0,
                    logit_max_abs: 0.0,
                }],
                encoded_weight_bytes: index.weights.shared.byte_size
                    + index
                        .weights
                        .layers
                        .iter()
                        .map(|a| a.byte_size)
                        .sum::<u64>(),
                weight_payload_bytes: fixture::weights_bytes().len() as u64,
                streaming_resident_payload_bytes: index.weights.shared.byte_size
                    + index.weights.layers[0].byte_size,
                largest_layer_bytes: index
                    .weights
                    .layers
                    .iter()
                    .map(|a| a.byte_size)
                    .max()
                    .unwrap(),
                logical_stream_bytes: provider
                    .bound()
                    .provider()
                    .streaming_stats()
                    .unwrap()
                    .encoded_bytes_read,
                elapsed_seconds: start.elapsed().as_secs_f64(),
            };
            let model = qualify_formatted_model(&mut store, model, qualification)
                .await
                .unwrap();
            save_typed(
                &mut store,
                &InferenceModelRow { model, template },
                vec![],
                PrivacyClass::LocalOnly,
                ProvenanceRecord::self_imported("local"),
                TrustEnvelope::self_asserted(),
                Timestamp::ZERO,
            )
            .await
            .unwrap()
        } else {
            row
        };
        let resolved = resolve_library_session(&mut store, row).await.unwrap();
        let artifact = resolved.layer_artifacts().unwrap().remove(1);
        let manifest = eidetic::manifest::load_manifest(&mut store, artifact.blob)
            .await
            .unwrap()
            .unwrap();
        let key = manifest
            .sources
            .into_iter()
            .find_map(|source| match source {
                eidetic::BlobSource::Local { key } => Some(key),
                _ => None,
            })
            .unwrap();
        (row, key)
    });
    let store = Arc::new(tokio::sync::Mutex::new(store));
    let journal = Arc::new(distillery::inference::durable::PersonaRunJournal::new(
        muniment::MemoryBackend::new(),
        [42; 32],
        &personae::Ed25519Keypair::from_seed([13; 32]),
        "local".into(),
        [23; 32],
    ));
    let lane =
        open_library_lane_with_journal(store.clone(), row, 8, Some(journal.clone())).unwrap();
    let wait = |name: &str| {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let runs = lane.runs();
            if let Some(run) = runs
                .runs
                .into_iter()
                .find(|run| run.run == name && run.state.is_terminal())
            {
                break run;
            }
            assert!(Instant::now() < deadline, "resident ask did not finish");
            std::thread::sleep(Duration::from_millis(5));
        }
    };
    lane.ask(AskRequestV1 {
        run: "complete".into(),
        question: "t3 t7".into(),
    })
    .unwrap();
    let run = wait("complete");
    assert_eq!(run.state, RunStateV1::Finished);
    // This non-journaled composition reports text; durable token counters are
    // exercised by inference_durable and the provider's checkpoint fixtures.
    assert!(!run.text.is_empty());
    assert!(run.session.is_some());
    assert_eq!(
        journal
            .retained_record("complete")
            .unwrap()
            .weight_format
            .as_deref(),
        Some(if half { "bf16" } else { "f32" })
    );
    // Corrupt the second layer after provider construction. A subsequent ask
    // must read it again and fail before publishing its first generated token.
    pollster::block_on(store.blocking_lock().put(&damaged_key, b"corrupt")).unwrap();
    lane.ask(AskRequestV1 {
        run: "missing".into(),
        question: "t3 t7".into(),
    })
    .unwrap();
    let run = wait("missing");
    assert_eq!(run.state, RunStateV1::Failed);
    assert_eq!(run.tokens, 0);
    assert!(run.text.is_empty());
    lane.close().unwrap();
    assert_eq!(Arc::strong_count(&store), 1);
    drop(store);
    drop(muniment::RedbBackend::open(&path).unwrap());
}
