// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Real resident settings, graph key agreement, app route and supervisor leases.
#![cfg(feature = "inference-stages")]

mod common;
#[path = "fixtures/esp/decoder.rs"]
mod fixture;

use common::inference_door::{endpoint_name, intent, keeper_host, open, read_runs};
use common::{lane, lending, open_vault, owner, profile, unlock, write_device_settings};
use distillery::inference::{ASK_INTENT, AskRequestV1, GenerationCheckpoint, RunStateV1};
use distillery::inference_protocol::AutomaticStageSettings;
use djinn::sync::personal_sync_host::{PersonalSyncHost, PersonalSyncHostConfig};
use djinn::{
    resident::DjinnResident,
    settings::{
        InferenceDeviceSettings, InferenceLaneSettings, InferenceModelSettings, SyncSettings,
    },
};
use personae::IdentityProvider;
use std::{sync::Arc, time::Duration};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ordinary_asks_discover_keyed_stages_and_recover_when_a_worker_leaves() {
    exercise(false).await;
}

#[cfg(feature = "inference-gpu")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires I8 qualified SmolLM2 f16 libraries and a discrete GPU"]
async fn real_smollm2_ordinary_asks_select_stages_and_recover() {
    exercise(true).await;
}

async fn save_library(data: &std::path::Path, root: &std::path::Path) -> eidetic::ManifestId {
    use distillery::inference::{InferenceModelRow, PromptTemplate, import_layered_model};
    use eidetic::{PrivacyClass, ProvenanceRecord, Timestamp, TrustEnvelope};
    let mut config: serde_json::Value = serde_json::from_str(fixture::CONFIG_JSON).unwrap();
    config["max_position_embeddings"] = serde_json::json!(512);
    std::fs::write(
        root.join("config.json"),
        serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    std::fs::write(root.join("model.safetensors"), fixture::weights_bytes()).unwrap();
    std::fs::write(root.join("tokenizer.json"), fixture::tokenizer_json()).unwrap();
    let path = djinn::resident_distillery::model_library_path(data, &profile().0, None);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut library = muniment::RedbBackend::open(path).unwrap();
    let model = import_layered_model(
        &mut library,
        root,
        "fixture/automatic",
        "MPL-2.0",
        Default::default(),
    )
    .await
    .unwrap();
    let mut template =
        PromptTemplate::from_bytes(include_bytes!("fixtures/esp/smollm2-instruct-v1.json"))
            .unwrap();
    template.bos.clear();
    template.eos.clear();
    template.system_prompt.clear();
    template.user.prefix.clear();
    template.user.suffix = " ".into();
    template.assistant.prefix.clear();
    template.assistant.suffix.clear();
    template.stop.clear();
    let template = eidetic::typed::save_typed(
        &mut library,
        &template,
        vec![],
        PrivacyClass::LocalOnly,
        ProvenanceRecord::self_imported("local"),
        TrustEnvelope::self_asserted(),
        Timestamp::ZERO,
    )
    .await
    .unwrap();
    eidetic::typed::save_typed(
        &mut library,
        &InferenceModelRow { model, template },
        vec![],
        PrivacyClass::LocalOnly,
        ProvenanceRecord::self_imported("local"),
        TrustEnvelope::self_asserted(),
        Timestamp::ZERO,
    )
    .await
    .unwrap()
}

async fn exercise(native: bool) {
    tokio::task::LocalSet::new()
        .run_until(exercise_inner(native))
        .await;
}

async fn exercise_inner(native: bool) {
    use djinn::door::app_broker::{AppEndpointCatalog, serve_app_broker};
    use graphshell::personal_sync::{SyncRoster, SyncSelection};
    use graphshell_endpoint::door::app_admission::{AllowedAppRoutes, AppId, AppRouteGrants};
    use graphshell_host::endpoint_catalog::ResidentEndpointCatalog;

    let started = std::time::Instant::now();
    let mut results = Vec::new();
    let mut old_device = None;
    let temp = tempfile::tempdir().unwrap();
    let roots: Vec<_> = (0..3).map(|i| temp.path().join(i.to_string())).collect();
    for root in &roots {
        std::fs::create_dir_all(root).unwrap();
    }
    let vaults: Vec<_> = roots.iter().map(|root| open_vault(root)).collect();
    let graph_name = "automatic-inference";
    let graph = djinn::personal_sync::personal_graph_id(graph_name);
    let roster = SyncRoster::new(
        vaults
            .iter()
            .map(|(_, id)| id.master_public_key().to_bytes()),
    );
    let mut devices = Vec::new();
    let mut hosts: Vec<Arc<PersonalSyncHost>> = Vec::new();
    let mut row = None;
    for (i, root) in roots.iter().enumerate() {
        let data = root.join("data");
        let mut posture = lending();
        posture.allowed_resources.clear(); // explicit test posture permits device-addressed stages
        write_device_settings(&data, Some(posture));
        let selected = if native {
            let source = std::path::PathBuf::from(
                std::env::var_os("ESP_STAGE_WORK_DIR").expect("I8 fixture root"),
            );
            let result: serde_json::Value =
                serde_json::from_slice(&std::fs::read(source.join("stage-result.json")).unwrap())
                    .unwrap();
            let target = djinn::resident_distillery::model_library_path(&data, &profile().0, None);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::copy(
                source
                    .join(["coordinator", "worker-a", "worker-b"][i])
                    .join("model.redb"),
                target,
            )
            .unwrap();
            distillery::inference::parse_model_row(result["row"].as_str().unwrap()).unwrap()
        } else {
            save_library(&data, root).await
        };
        if let Some(row) = row {
            assert_eq!(row, selected);
        } else {
            row = Some(selected);
        }
        let settings = djinn::settings::OwnerSettings {
            sync: Some(SyncSettings {
                graph: graph_name.into(),
                store_path: None,
                roster_roots: vec![],
                paired_devices: vec![],
                lanes: Default::default(),
                encrypted: false,
                relay_urls: vec![],
            }),
            inference: Some(InferenceLaneSettings {
                embedding: None,
                recall: Default::default(),
                model: InferenceModelSettings::Automatic {
                    row: selected.to_string(),
                    device: if native {
                        InferenceDeviceSettings::Gpu
                    } else {
                        InferenceDeviceSettings::Cpu
                    },
                    max_tokens: if native { 16 } else { 64 },
                    stages: AutomaticStageSettings {
                        local_resident_bytes: if i == 0 { 0 } else { 2 << 30 },
                        worker_layer_bytes: if i == 0 {
                            0
                        } else if native {
                            482406400
                        } else {
                            18752
                        },
                        max_peers: 8,
                        max_hidden_bytes: 16 << 20,
                        discovery_timeout_ms: 2000,
                        discovery_every_ms: 100,
                        lease_wait_ms: 10000,
                        call_timeout_ms: 60000,
                        poll_every_ms: 5,
                        lease_duration_ms: 600000,
                        heartbeat_ms: 10000,
                    },
                },
            }),
            ..owner(Some(lane()))
        };
        // Round-trip through the production JSON shape before composition.
        let settings: djinn::settings::OwnerSettings =
            serde_json::from_slice(&serde_json::to_vec(&settings).unwrap()).unwrap();
        let (vault, identity) = &vaults[i];
        if i == 0 {
            let mut prior = settings.clone();
            prior.sync = None;
            prior.inference.as_mut().unwrap().model = InferenceModelSettings::Library {
                weight_cache: None,
                row: selected.to_string(),
                device: if native {
                    InferenceDeviceSettings::Gpu
                } else {
                    InferenceDeviceSettings::Cpu
                },
                max_tokens: 8,
            };
            let prior = DjinnResident::open(identity, &data, &profile(), prior, vault, unlock())
                .await
                .unwrap();
            prior
                .inference()
                .unwrap()
                .lane()
                .ask(AskRequestV1 {
                    run: "before-automatic".into(),
                    question: if native {
                        "What is the capital of France?".into()
                    } else {
                        "t4".into()
                    },
                })
                .unwrap();
            let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
            loop {
                if let Some(run) = prior
                    .inference()
                    .unwrap()
                    .lane()
                    .runs()
                    .runs
                    .iter()
                    .find(|r| r.run == "before-automatic" && r.state.is_terminal())
                {
                    assert_eq!(run.state, RunStateV1::Finished);
                    break;
                }
                assert!(
                    tokio::time::Instant::now() < deadline,
                    "historical ask did not finish"
                );
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            old_device = Some(
                distillery::inference::RunJournal::device(
                    prior.inference().unwrap().journal().unwrap().as_ref(),
                )
                .to_string(),
            );
            prior.shutdown().await.unwrap();
        }
        let mut resident = DjinnResident::open_deferred_inference(
            identity,
            &data,
            &profile(),
            settings,
            vault,
            unlock(),
        )
        .await
        .unwrap();
        let custody = resident.blobs();
        custody
            .bind_scope(transport::BlobScope::new(graph))
            .await
            .unwrap();
        let host = Arc::new(
            PersonalSyncHost::open_with_blob_custody(
                identity,
                PersonalSyncHostConfig {
                    graph,
                    store_path: root.join("personal.redb"),
                    roster: roster.clone(),
                    selection: SyncSelection::default(),
                    peer_tickets: if i == 0 {
                        vec![]
                    } else {
                        vec![hosts[0].ticket().await.unwrap()]
                    },
                    peer_hints: vec![],
                    paired_nodes: vec![],
                    relay_urls: vec![],
                },
                custody.blobs(),
                custody.authorizer(),
            )
            .await
            .unwrap(),
        );
        if i > 0 {
            hosts[0].pair_node(host.node_id()).await.unwrap();
        }
        resident
            .start_inference(identity, Some(host.clone()))
            .await
            .unwrap();
        hosts.push(host);
        devices.push(resident);
    }
    let row = row.unwrap();
    assert_ne!(
        vaults[0].1.master_public_key(),
        vaults[1].1.master_public_key()
    );
    assert!(
        devices
            .windows(2)
            .all(|p| p[0].distillery().unwrap().mesh_id() == p[1].distillery().unwrap().mesh_id())
    );
    assert!(
        devices[0]
            .inference()
            .unwrap()
            .lane()
            .ask(AskRequestV1 {
                run: "unkeyed".into(),
                question: "t4".into()
            })
            .unwrap_err()
            .to_string()
            .contains("keyed reader")
    );
    hosts[0].enable_encryption().await.unwrap();
    for (_, identity) in &vaults[1..] {
        hosts[0]
            .admit_reader(identity.master_public_key().to_bytes(), "worker")
            .await
            .unwrap();
    }
    let deadline = tokio::time::Instant::now() + Duration::from_secs(45);
    loop {
        for host in &hosts {
            host.key_paired_devices().await.unwrap();
        }
        if hosts.iter().all(|h| h.seals().can_write()) {
            let probe = hosts[0].seals().seal(b"same epoch").unwrap();
            if hosts.iter().all(|h| h.seals().open(&probe).is_ok()) {
                break;
            }
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "graph key agreement timed out"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
    let route = catalog
        .update(|c| devices[0].register_inference_route(c))
        .await
        .unwrap()
        .unwrap();
    let grants = AppRouteGrants::new(AllowedAppRoutes::new([(AppId::new("turnstone"), route)]));
    let endpoint = endpoint_name();
    let server_endpoint = endpoint.clone();
    let server = tokio::spawn(async move {
        let _ = serve_app_broker(
            &server_endpoint,
            Arc::new(djinn::keeper::Keeper::new(keeper_host())),
            grants,
            600000,
            None,
            catalog,
        )
        .await;
    });
    let deadline = tokio::time::Instant::now() + Duration::from_secs(180);
    let mut client = loop {
        if let Ok(client) = open(&endpoint, "turnstone").await {
            break client;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "app door did not open"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    };
    let session = client.open_session().await.unwrap();
    let projection = session.descriptor.projections[0].request.clone();
    let journal = devices[0].inference().unwrap().journal().unwrap().clone();
    assert_eq!(
        old_device.as_deref(),
        Some(distillery::inference::RunJournal::device(journal.as_ref()))
    );
    devices[0]
        .inference()
        .unwrap()
        .lane()
        .refresh_retained_runs()
        .unwrap();
    assert!(
        journal.retained_record("before-automatic").is_ok(),
        "changing execution mesh hid history"
    );
    let library = devices[0].distillery().unwrap().model_library().unwrap();
    let lane = devices[0].inference().unwrap().lane().clone();
    let worker_lane = devices[2].inference().unwrap().lane().clone();
    let worker_journal = devices[2].inference().unwrap().journal().unwrap().clone();
    let works = devices[0].distillery().unwrap();
    let mesh = works.mesh_id();
    let planner = works.authority().stage_planner().unwrap().clone();
    let services: Vec<_> = devices
        .iter()
        .flat_map(|d| d.distillery().unwrap().inference_stage_services())
        .collect();
    let board_store = works.authority().host().synced().store().clone();
    let mut tasks = Vec::new();
    let mut stops = Vec::new();
    for (index, device) in devices.iter_mut().enumerate() {
        let mut works = device.take_distillery().unwrap();
        let (stop, receive) = tokio::sync::oneshot::channel();
        stops.push(Some(stop));
        tasks.push(tokio::task::spawn_blocking(move || {
            // Each software device drives its own resident loop, as separate
            // processes do. A busy supervisor must not starve another device's
            // shutdown or the app client's observation on a shared LocalSet.
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(async move {
                works
                    .run_until(
                        async {
                            let _ = receive.await;
                        },
                        move |receipt| {
                            if let distillery::ResidentReceipt::Tick { steps } = &receipt {
                                if steps
                                    .iter()
                                    .any(|s| !matches!(s, distillery::mesh_host::Step::Idle))
                                {
                                    println!("automatic host {index}: {steps:?}");
                                }
                            }
                        },
                    )
                    .await
                    .unwrap();
                works.close().await.unwrap();
            });
        }));
    }
    // The bridge alone supplies mesh endpoints and attestations. No stage
    // addresses, assignments, jobs or credentials are supplied by this fixture.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(180);
    while board_store.board(mesh).await.unwrap().devices().len() < 3 {
        assert!(
            tokio::time::Instant::now() < deadline,
            "attested mesh endpoints did not reconcile"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let question = if native {
        "What is the capital of France?"
    } else {
        "t4"
    };
    for (run, depart) in [("staged", false), ("departure", true), ("offline", false)] {
        // Snapshot construction may wait behind sealed journal writes. Observe
        // committed progress on a separate thread so removing the worker does
        // not depend on the app carrier winning that scheduling race.
        let departure =
            if depart {
                let lane = lane.clone();
                let stop = stops[1].take().unwrap();
                Some(tokio::task::spawn_blocking(move || {
                    let deadline = std::time::Instant::now() + Duration::from_secs(180);
                    loop {
                        if lane.runs().runs.iter().any(|r| {
                            r.run == "departure" && r.tokens >= 3 && !r.state.is_terminal()
                        }) {
                            stop.send(()).unwrap();
                            return;
                        }
                        assert!(
                            std::time::Instant::now() < deadline,
                            "no mid-answer prefix: {:?}",
                            lane.runs()
                        );
                        std::thread::sleep(Duration::from_millis(1));
                    }
                }))
            } else {
                None
            };
        let ask_started = std::time::Instant::now();
        assert_eq!(
            intent(
                &mut client,
                &projection,
                ASK_INTENT,
                serde_json::to_vec(&AskRequestV1 {
                    run: run.into(),
                    question: question.into()
                })
                .unwrap()
            )
            .await,
            chirograph::IntentResult::Accepted
        );
        println!(
            "automatic {run}: app accepted after {:.3}s",
            ask_started.elapsed().as_secs_f64()
        );
        let deadline = tokio::time::Instant::now() + Duration::from_secs(180);
        loop {
            let runs = read_runs(&mut client, &projection).await;
            let current = runs.runs.iter().find(|r| r.run == run).unwrap();
            if current.state.is_terminal() {
                assert_eq!(current.state, RunStateV1::Finished, "{current:?}");
                break;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "automatic ask timed out: {runs:?}"
            );
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        println!(
            "automatic {run}: app finished after {:.3}s",
            ask_started.elapsed().as_secs_f64()
        );
        if let Some(departure) = departure {
            departure.await.unwrap();
        }
        let record = journal.retained_record(run).unwrap();
        let bytes = journal
            .resolve_evidence_blob(record.checkpoint_blob.unwrap())
            .unwrap()
            .unwrap();
        let cp: GenerationCheckpoint = serde_json::from_slice(&bytes).unwrap();
        results.push(
            serde_json::json!({ "run": run, "session": cp.session.clone(),
            "token_ids": cp.token_ids.clone(), "placements": cp.placements.clone(),
            "text": cp.text.clone(), "ask_seconds": ask_started.elapsed().as_secs_f64() }),
        );
        if run == "staged" {
            if !cp
                .placements
                .iter()
                .any(|p| p.reason.starts_with("layer stages:"))
            {
                println!(
                    "automatic diagnostic: {:?}; board: {:?}",
                    planner.last_unavailable(),
                    board_store.board(mesh).await.unwrap()
                );
                println!(
                    "automatic worker refusals: {:?}",
                    services
                        .iter()
                        .map(|s| s.last_refusal())
                        .collect::<Vec<_>>()
                );
            }
            assert!(
                cp.placements
                    .iter()
                    .any(|p| p.reason.starts_with("layer stages:")),
                "{cp:?}"
            );
        }
        if depart {
            assert!(
                cp.placements.iter().any(
                    |p| p.at_token > 0 && p.reason.contains("resumed by local layer streaming")
                ),
                "{cp:?}"
            );
        }
        if run == "offline" {
            assert!(
                cp.placements
                    .iter()
                    .any(|p| p.reason.contains("stage plan unavailable")),
                "{cp:?}"
            );
        }
        let library = library.clone();
        let expected = tokio::task::spawn_blocking(move || {
            use distillery::inference::{
                InferError, InferenceDevice, InferenceProvider, resolve_library_session,
            };
            let resolved =
                pollster::block_on(resolve_library_session(&mut *library.blocking_lock(), row))
                    .unwrap();
            let artifacts = resolved.layer_artifacts().unwrap();
            let source = library.clone();
            let layer_source = Box::new(move |index: usize, limit: usize| {
                pollster::block_on(artifacts[index].read(&mut *source.blocking_lock(), limit))
                    .map_err(|e| InferError::InvalidWeights(e.to_string()))
            });
            #[cfg(feature = "inference-gpu")]
            let device = if native {
                distillery::inference::discrete_gpu_inference_device()
                    .unwrap()
                    .0
            } else {
                InferenceDevice::ndarray()
            };
            #[cfg(not(feature = "inference-gpu"))]
            let device = InferenceDevice::ndarray();
            let loader = if native { "burn-wgpu" } else { "burn-ndarray" };
            let provider = resolved
                .load_resident_with_layer_source(
                    &device,
                    loader,
                    cp.request.max_tokens,
                    layer_source,
                )
                .unwrap();
            let reference = provider
                .bound()
                .provider()
                .generate_checkpointed(&cp.request, None, &mut |_| Ok(()), &mut || false)
                .unwrap();
            assert_eq!(reference.token_ids, cp.token_ids, "{run}");
            device.sync().unwrap();
            cp.token_ids.clone()
        })
        .await
        .unwrap();
        println!("automatic {run}: {} exact reference tokens", expected.len());
        if run == "staged" {
            worker_lane
                .ask(AskRequestV1 {
                    run: "resident".into(),
                    question: question.into(),
                })
                .unwrap();
            let deadline = tokio::time::Instant::now() + Duration::from_secs(180);
            loop {
                if let Some(run) = worker_lane
                    .runs()
                    .runs
                    .iter()
                    .find(|r| r.run == "resident" && r.state.is_terminal())
                {
                    assert_eq!(run.state, RunStateV1::Finished);
                    break;
                }
                assert!(
                    tokio::time::Instant::now() < deadline,
                    "resident rung did not finish"
                );
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            let record = worker_journal.retained_record("resident").unwrap();
            let bytes = worker_journal
                .resolve_evidence_blob(record.checkpoint_blob.unwrap())
                .unwrap()
                .unwrap();
            let cp: GenerationCheckpoint = serde_json::from_slice(&bytes).unwrap();
            assert!(
                cp.placements
                    .iter()
                    .any(|p| p.reason == "whole model resident within owner payload budget")
            );
            assert_eq!(cp.token_ids, expected, "resident and staged rungs diverged");
            results.push(
                serde_json::json!({ "run": "resident", "session": cp.session,
                "token_ids": cp.token_ids, "placements": cp.placements, "text": cp.text }),
            );
            while board_store
                .board(mesh)
                .await
                .unwrap()
                .jobs()
                .any(|j| !j.state.is_terminal())
            {
                assert!(
                    tokio::time::Instant::now() < deadline,
                    "completed stages did not drain and commit"
                );
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }
    assert!(lane.runs().load_error.is_none());
    client.close().await.unwrap();
    server.abort();
    let _ = server.await;
    for stop in &mut stops {
        if let Some(stop) = stop.take() {
            let _ = stop.send(());
        }
    }
    for task in tasks {
        task.await.unwrap();
    }
    drop(lane);
    drop(worker_lane);
    drop(planner);
    drop(services);
    // Let each ResidentInference become its host's final owner and await the
    // personal transport drain before the enclosing blob store closes.
    drop(hosts);
    for device in devices {
        device.shutdown().await.unwrap();
    }
    drop(board_store);
    drop(library);
    drop(journal);
    if let Some(root) = std::env::var_os("DJINN_AUTOMATIC_RECEIPT_DIR") {
        let path = std::path::PathBuf::from(root).join(if native {
            "automatic-gpu-result.json"
        } else {
            "automatic-cpu-result.json"
        });
        std::fs::write(path, serde_json::to_vec_pretty(&serde_json::json!({
            "backend": if native { "burn-wgpu/f16" } else { "burn-ndarray/f32" }, "row": row.to_string(),
            "independent_roots": 3, "physical_computers": 1, "rung_change_preserved_history": true,
            "runs": results, "fixture_seconds": started.elapsed().as_secs_f64()
        })).unwrap()).unwrap();
    }
}
