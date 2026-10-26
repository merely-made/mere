// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Explicit separate-process qualification of the real resident recall route.
#![cfg(feature = "inference-embedding")]
mod common;
#[path = "fixtures/esp/decoder.rs"]
mod decoder_fixture;
use common::inference_door::{endpoint_name, intent, keeper_host, open, read_runs};
use common::{lane, lending, open_vault, owner, profile, unlock, write_device_settings};
use distillery::inference::recall::*;
use distillery::inference::{AskRequestV1, RunStateV1};
use djinn::door::app_broker::{AppEndpointCatalog, serve_app_broker};
use djinn::{
    resident::DjinnResident,
    settings::{InferenceLaneSettings, InferenceModelSettings, OwnerSettings},
};
use graphshell_endpoint::door::app_admission::{AllowedAppRoutes, AppId, AppRouteGrants};
use graphshell_host::endpoint_catalog::ResidentEndpointCatalog;
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

const CORPUS: &str = include_str!("fixtures/embedding/retained-paraphrases-v1.json");
fn settings(model: &str) -> EmbeddingRecallSettings {
    EmbeddingRecallSettings {
        provider: EmbeddingRecallProvider::Library {
            model: model.into(),
        },
        max_index_bytes: 8 * 1024 * 1024,
        lexical_weight: 1,
        vector_weight: 1,
        rrf_k: 60,
    }
}
async fn seed_model(root: &Path, data: &Path) -> eidetic::ManifestId {
    let source =
        PathBuf::from(std::env::var_os("ESP_MINILM_DIR").expect("pinned MiniLM directory"));
    let path = djinn::resident_distillery::model_library_path(data, &profile().0, None);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut store = muniment::RedbBackend::open(path).unwrap();
    let model = eidetic::ModelLibrary::save_model_with_components(
        &mut store,
        "sentence-transformers/all-MiniLM-L6-v2",
        "bert",
        "Apache-2.0",
        serde_json::from_slice(&std::fs::read(source.join("config.json")).unwrap()).unwrap(),
        &std::fs::read(source.join("model.safetensors")).unwrap(),
        &std::fs::read(source.join("tokenizer.json")).unwrap(),
        vec![],
        vec![],
        eidetic::PrivacyClass::LocalOnly,
        eidetic::ProvenanceRecord::self_imported("explicit synthetic embedding qualification"),
        eidetic::TrustEnvelope::self_asserted(),
        eidetic::Timestamp::ZERO,
    )
    .await
    .unwrap();
    std::fs::write(root.join("model-id"), model.to_string()).unwrap();
    model
}
async fn seed_generation_model(root: &Path, data: &Path) -> eidetic::ManifestId {
    use distillery::inference::{InferenceModelRow, PromptTemplate};
    let path = djinn::resident_distillery::model_library_path(data, &profile().0, None);
    let mut store = muniment::RedbBackend::open(path).unwrap();
    // RoPE has no learned position tensor; retain the tiny decoder weights
    // while admitting the complete retained-passage prompt in this fixture.
    let mut config: Value = serde_json::from_str(decoder_fixture::CONFIG_JSON).unwrap();
    config["max_position_embeddings"] = 2048.into();
    let model = eidetic::ModelLibrary::save_model_with_components(
        &mut store,
        "qualification/tiny-exact-session",
        "llama",
        "MIT",
        config,
        &decoder_fixture::weights_bytes(),
        decoder_fixture::tokenizer_json().as_bytes(),
        vec![],
        vec![],
        eidetic::PrivacyClass::LocalOnly,
        eidetic::ProvenanceRecord::self_imported("explicit synthetic exact-session fixture"),
        eidetic::TrustEnvelope::self_asserted(),
        eidetic::Timestamp::ZERO,
    )
    .await
    .unwrap();
    let mut template =
        PromptTemplate::from_bytes(include_bytes!("fixtures/esp/smollm2-instruct-v1.json"))
            .unwrap();
    template.system_prompt.clear();
    template.user.prefix = "t1 ".into();
    template.user.suffix = " t2 ".into();
    template.assistant.prefix = "t3 ".into();
    let template = eidetic::typed::save_typed(
        &mut store,
        &template,
        vec![],
        eidetic::PrivacyClass::LocalOnly,
        eidetic::ProvenanceRecord::self_imported("qualification"),
        eidetic::TrustEnvelope::self_asserted(),
        eidetic::Timestamp::ZERO,
    )
    .await
    .unwrap();
    let row = eidetic::typed::save_typed(
        &mut store,
        &InferenceModelRow { model, template },
        vec![],
        eidetic::PrivacyClass::LocalOnly,
        eidetic::ProvenanceRecord::self_imported("qualification"),
        eidetic::TrustEnvelope::self_asserted(),
        eidetic::Timestamp::ZERO,
    )
    .await
    .unwrap();
    std::fs::write(root.join("generation-row-id"), row.to_string()).unwrap();
    row
}
fn source_id(id: &str) -> eidetic::models::recall::RecallSourceId {
    eidetic::models::recall::RecallSourceId {
        kind: eidetic::models::recall::RecallSourceKind::PageText,
        source: format!("https://retained.test/{id}"),
        owner: None,
        node: None,
    }
}
fn expected_passage(benchmark: &Value, id: &str) -> String {
    let text = benchmark["documents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["id"] == id)
        .unwrap()["text"]
        .as_str()
        .unwrap();
    eidetic::models::recall::InferencePassage {
        source: source_id(id),
        version: eidetic::Hash::of(text.as_bytes()).to_string(),
        text_hash: eidetic::Hash::of(text.as_bytes()),
        selector: eidetic::models::recall::PassageSelector {
            start: 0,
            end: text.chars().count(),
        },
        chunker: format!("{CHUNKER_VERSION}/512"),
        text: text.into(),
    }
    .id()
    .to_string()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "explicit marked root and digest-pinned MiniLM artifacts; separate OS-process phases"]
async fn resident_embedding_process_fixture() {
    let root = PathBuf::from(std::env::var_os("EMBEDDING_RECALL_ROOT").expect("marked root"));
    assert_eq!(
        std::fs::read_to_string(root.join("fixture-marker")).unwrap(),
        "embedding-envelope-20261020\n"
    );
    let phase = std::env::var("EMBEDDING_RECALL_PHASE").unwrap();
    assert!(matches!(phase.as_str(), "prepare" | "cold" | "foreign"));
    let data = root.join("data");
    let benchmark: Value = serde_json::from_str(CORPUS).unwrap();
    let (vault, identity) = open_vault(&root);
    if phase == "prepare" {
        write_device_settings(&data, Some(lending()));
    }
    let model = if phase == "prepare" {
        seed_model(&root, &data).await
    } else {
        distillery::inference::parse_model_row(
            &std::fs::read_to_string(root.join("model-id")).unwrap(),
        )
        .unwrap()
    };
    let generation = if phase == "prepare" {
        seed_generation_model(&root, &data).await
    } else {
        distillery::inference::parse_model_row(
            &std::fs::read_to_string(root.join("generation-row-id")).unwrap(),
        )
        .unwrap()
    };
    let embedding = settings(&model.to_string());
    let owner = OwnerSettings {
        inference: Some(InferenceLaneSettings {
            embedding: Some(embedding.clone()),
            recall: RecallConfig {
                max_passages: 16,
                ..Default::default()
            },
            model: InferenceModelSettings::Library {
                weight_cache: None,
                row: generation.to_string(),
                device: djinn::settings::InferenceDeviceSettings::Cpu,
                max_tokens: 2,
            },
        }),
        ..owner(Some(lane()))
    };
    let source_store = Arc::new(muniment::RedbBackend::open(root.join("retained.redb")).unwrap());
    if phase == "prepare" {
        std::fs::write(root.join("origin-corpus.json"), CORPUS).unwrap();
        for doc in benchmark["documents"].as_array().unwrap() {
            eidetic::PageTextStore::new(source_store.as_ref())
                .put(
                    &source_id(doc["id"].as_str().unwrap()).source,
                    doc["text"].as_str().unwrap(),
                    1,
                )
                .await
                .unwrap();
        }
    } else {
        assert!(!root.join("origin-corpus.json").exists());
    }
    let corpus: Arc<dyn RecallCorpus> = Arc::new(PageTextCorpus(source_store.clone()));
    let resident = DjinnResident::open(&identity, &data, &profile(), owner, &vault, unlock())
        .await
        .unwrap();
    let journal = resident.inference().unwrap().journal().unwrap().clone();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    while resident
        .inference()
        .unwrap()
        .lane()
        .runs()
        .session
        .is_none()
    {
        assert!(
            resident
                .inference()
                .unwrap()
                .lane()
                .runs()
                .load_error
                .is_none()
        );
        assert!(tokio::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    if phase == "foreign" {
        let previous: Value =
            serde_json::from_slice(&std::fs::read(root.join("prepare-receipt.json")).unwrap())
                .unwrap();
        let envelope =
            serde_json::from_value(previous["rows"][0]["embedding"]["envelope"].clone()).unwrap();
        let original = journal
            .load_embedding_projection(envelope, embedding.max_index_bytes)
            .unwrap()
            .unwrap();
        let mut refusals = Vec::new();
        for kind in ["space", "chunker", "source", "legacy"] {
            let mut record: Value = serde_json::from_slice(&original).unwrap();
            assert_eq!(record["envelope"]["space"]["dimensions"], 384);
            match kind {
                "space" => {
                    record["envelope"]["space"]["producer"]["revision"] = "foreign/v1".into()
                },
                "chunker" => record["envelope"]["corpus"]["chunker"] = "foreign/v1".into(),
                "source" => {
                    record["envelope"]["corpus"]["sources"][0]["version"] = "foreign/v1".into()
                },
                _ => {
                    record.as_object_mut().unwrap().remove("envelope");
                },
            }
            journal
                .save_embedding_projection(envelope, &serde_json::to_vec(&record).unwrap())
                .unwrap();
            let error = resident
                .configure_inference_recall(
                    corpus.clone(),
                    RecallConfig {
                        max_passages: 16,
                        ..Default::default()
                    },
                )
                .unwrap_err();
            assert!(
                error.contains(match kind {
                    "space" => "foreign embedding space",
                    "legacy" => "embedding cache format",
                    _ => "corpus or chunker changed",
                }),
                "{error}"
            );
            refusals.push(json!({"kind":kind,"same_dimensions":384,"error":error}));
        }
        journal
            .save_embedding_projection(envelope, &original)
            .unwrap();
        std::fs::write(
            root.join("foreign-receipt.json"),
            serde_json::to_vec_pretty(
                &json!({"phase":phase,"pid":std::process::id(),"refusals":refusals}),
            )
            .unwrap(),
        )
        .unwrap();
        resident.shutdown().await.unwrap();
        return;
    }
    resident
        .configure_inference_recall(
            corpus.clone(),
            RecallConfig {
                max_passages: 16,
                ..Default::default()
            },
        )
        .unwrap();
    let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
    let grants = AppRouteGrants::new(AllowedAppRoutes::new([]));
    let route = catalog
        .update(|c| resident.register_inference_route(c))
        .await
        .unwrap()
        .unwrap();
    grants.grant(AppId::new("turnstone"), route);
    let endpoint = endpoint_name();
    let server = {
        let endpoint = endpoint.clone();
        let catalog = catalog.clone();
        let grants = grants.clone();
        tokio::spawn(async move {
            let _ = serve_app_broker(
                &endpoint,
                Arc::new(djinn::keeper::Keeper::new(keeper_host())),
                grants,
                60_000,
                None,
                catalog,
            )
            .await;
        })
    };
    let mut client = None;
    for _ in 0..100 {
        if let Ok(c) = open(&endpoint, "turnstone").await {
            client = Some(c);
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let mut client = client.unwrap();
    let opened = client.open_session().await.unwrap();
    let request = opened.descriptor.projections[0].request.clone();
    let mut rows = Vec::new();
    for (i, query) in benchmark["queries"].as_array().unwrap().iter().enumerate() {
        let q = query["question"].as_str().unwrap();
        let started = std::time::Instant::now();
        let rankings = resident
            .inference()
            .unwrap()
            .lane()
            .recall_rankings(q, &[])
            .unwrap();
        let ranking_micros = started.elapsed().as_micros();
        let expected_key = expected_passage(&benchmark, query["expected"].as_str().unwrap());
        let rank = |keys: &[String]| keys.iter().position(|k| k == &expected_key).map(|i| i + 1);
        let ranks = json!({"lexical":rank(&rankings.lexical),"vector":rank(&rankings.vector),"fused":rank(&rankings.fused)});
        let run = format!("{phase}-query-{i}");
        let ask = RecallAskRequestV1 {
            answer_format: Default::default(),
            ask: AskRequestV1 {
                run: run.clone(),
                question: query["question"].as_str().unwrap().into(),
            },
            scope: vec![],
        };
        assert_eq!(
            intent(
                &mut client,
                &request,
                RECALL_ASK_INTENT,
                serde_json::to_vec(&ask).unwrap()
            )
            .await,
            chirograph::IntentResult::Accepted
        );
        let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
        let answered = loop {
            let runs = read_runs(&mut client, &request).await;
            let found = runs.runs.iter().find(|r| r.run == run);
            if let Some(r) = found {
                if r.state.is_terminal() {
                    break r.clone();
                }
            }
            assert!(tokio::time::Instant::now() < deadline);
            tokio::time::sleep(Duration::from_millis(20)).await;
        };
        assert_eq!(answered.state, RunStateV1::Finished, "{answered:?}");
        let e = answered.recall.as_ref().unwrap();
        let projection = e.embedding.as_ref().unwrap();
        assert_eq!(
            projection.cache,
            if phase == "prepare" {
                "built"
            } else {
                "reopened"
            }
        );
        // Serde may reserve more capacity than the logical 384 floats on
        // reopen. Budget actual vector capacity as well as persisted bytes;
        // CompatibleSearch independently checks every vector's exact width.
        assert!(projection.vector_bytes <= embedding.max_index_bytes);
        assert!(projection.persisted_bytes <= embedding.max_index_bytes);
        let ranked: Vec<_> = e
            .passages
            .iter()
            .map(|p| p.passage.source.source.clone())
            .collect();
        let expected = source_id(query["expected"].as_str().unwrap()).source;
        rows.push(json!({"query":query,"ranks":ranks,"ranking_micros":ranking_micros,"rankings":{"lexical":rankings.lexical,"vector":rankings.vector,"fused":rankings.fused},"rank":ranked.iter().position(|k|k==&expected).map(|i|i+1),"sources":ranked,"embedding":projection,"passages":e.passages}));
    }
    drop(client);
    server.abort();
    let _ = server.await;
    drop(catalog);
    resident.shutdown().await.unwrap();
    drop(journal);
    drop(source_store);
    if phase == "cold" {
        let previous: Value =
            serde_json::from_slice(&std::fs::read(root.join("prepare-receipt.json")).unwrap())
                .unwrap();
        for (a, b) in previous["rows"].as_array().unwrap().iter().zip(&rows) {
            assert_eq!(a["embedding"]["envelope"], b["embedding"]["envelope"]);
            assert_eq!(a["embedding"]["index"], b["embedding"]["index"]);
            assert_eq!(a["rankings"], b["rankings"]);
            assert_eq!(a["passages"], b["passages"]);
        }
    }
    std::fs::write(
        root.join(format!("{phase}-receipt.json")),
        serde_json::to_vec_pretty(
            &json!({"phase":phase,"pid":std::process::id(),"model":model,"rows":rows}),
        )
        .unwrap(),
    )
    .unwrap();
}
