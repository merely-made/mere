// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

#![cfg(feature = "inference-model")]

#[path = "../../../crates/intel/esp/tests/support/decoder.rs"]
mod fixture;

use distillery::inference::{AskRequestV1, RunStateV1, open_library_lane};
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
        let model = import_layered_model(
            &mut store,
            directory.path(),
            "fixture/layered",
            "MPL-2.0",
            StreamingOptions::default(),
        )
        .await
        .unwrap();
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
    let lane = open_library_lane(store.clone(), row, 8);
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
