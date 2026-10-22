// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Native resident receipt: a real GPU trainer yields its live lease to an
//! inference ask. Both arms use the composed library, device gate and host.
#![cfg(all(feature = "inference-gpu", feature = "trainer-gpu"))]

mod common;

use common::*;
use distillery::inference::{AskRequestV1, InferenceModelRow, PromptTemplate, RunStateV1};
use distillery::mesh_host::Step;
use distillery::{
    ResidentReceipt, TRAINER_REQUEST_INPUT, TRAINER_RESOURCE, TrainRequest, TrainerSettings,
};
use djinn::resident::DjinnResident;
use djinn::settings::{
    InferenceDeviceSettings, InferenceLaneSettings, InferenceModelSettings, TrainerLaneSettings,
};
use eidetic::typed::{TypedPayload, save_typed};
use eidetic::{ManifestId, PrivacyClass, ProvenanceRecord, Timestamp, TrustEnvelope};
use mesh::spec::{DeterminismClass, JobSpec};
use mesh::{LeaseTerms, ReclaimReason, ResourceId};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

async fn save<T: TypedPayload>(store: &mut muniment::RedbBackend, payload: &T) -> ManifestId {
    save_typed(
        store,
        payload,
        vec![],
        PrivacyClass::LocalOnly,
        ProvenanceRecord::self_imported("local"),
        TrustEnvelope::self_asserted(),
        Timestamp::ZERO,
    )
    .await
    .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires a discrete GPU; trains and reclaims real native GPU work"]
async fn an_ask_reclaims_a_gpu_trainer_through_the_lease_owner() {
    reclaim(false, mesh::CheckpointClass::Restart).await;
}

#[cfg(feature = "trainer-autodiff")]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires a discrete GPU; trains and reclaims real native GPU work"]
async fn an_ask_reclaims_an_autodiff_gpu_trainer_through_the_lease_owner() {
    reclaim(true, mesh::CheckpointClass::Restart).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires a discrete GPU; verifies checkpoint requests cannot strand inference"]
async fn an_ask_cancels_a_trainer_that_was_posted_as_checkpointable() {
    reclaim(false, mesh::CheckpointClass::Resumable).await;
}

async fn reclaim(autodiff: bool, checkpoint: mesh::CheckpointClass) {
    let directory = tempfile::tempdir().unwrap();
    let data_root = directory.path().join("data");
    let (vault_dir, identity) = open_vault(directory.path());
    let mut posture = lending();
    posture.allowed_resources = vec![TRAINER_RESOURCE.into()];
    posture.accepted_checkpoints = vec!["restart".into(), "resumable".into()];
    write_device_settings(&data_root, Some(posture));
    let path = djinn::resident_distillery::model_library_path(&data_root, &profile().0, None);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let (base_model_ref, tokenizer_ref, corpus_ref, row) = {
        let mut store = muniment::RedbBackend::open(&path).unwrap();
        let (model, tokenizer, corpus) = author_fixture(&mut store, "inference-reclaim").await;
        let mut template =
            PromptTemplate::from_bytes(include_bytes!("fixtures/esp/smollm2-instruct-v1.json"))
                .unwrap();
        template.system_prompt.clear();
        template.bos.clear();
        template.user.prefix.clear();
        template.user.suffix = " ".into();
        template.assistant.prefix.clear();
        let template = save(&mut store, &template).await;
        let row = save(&mut store, &InferenceModelRow { model, template }).await;
        (model, tokenizer, corpus, row)
    };
    let mut trains = lane();
    trains.trainer = Some(TrainerLaneSettings {
        device: "gpu".into(),
        library_root: None,
    });
    let mut settings = owner(Some(trains));
    settings.inference = Some(InferenceLaneSettings {
        embedding: None,
        recall: Default::default(),
        model: InferenceModelSettings::Library {
            weight_cache: None,
            row: row.to_string(),
            device: InferenceDeviceSettings::Gpu,
            max_tokens: 4,
        },
    });
    let mut resident = DjinnResident::open(
        &identity,
        &data_root,
        &profile(),
        settings,
        &vault_dir,
        unlock(),
    )
    .await
    .unwrap();
    let inference = resident.inference().unwrap().lane().clone();
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let runs = inference.runs();
            assert!(runs.load_error.is_none(), "{:?}", runs.load_error);
            if runs.session.is_some() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let mut works = resident.take_distillery().unwrap();
    let access = works.gpu_access();
    assert!(!access.foreground_requested());
    assert!(works.authority().host().facts().gpu);
    let mut trainer_settings = if autodiff {
        autodiff_settings()
    } else {
        finite_difference_settings()
    };
    // Far beyond this test's failure bound; completion cannot masquerade as reclaim.
    match &mut trainer_settings {
        TrainerSettings::FiniteDifference(settings) => settings.steps = 100_000,
        TrainerSettings::Autodiff(settings) => settings.steps = 100_000,
    }
    let request = TrainRequest {
        base_model_ref,
        tokenizer_ref,
        corpus_ref,
        adapter_name: "cancelled-trainer".into(),
        prompt_template: "{{ prompt }}".into(),
        metric: eidetic::models::EvalMetric::RankingAt { limit: 3 },
        settings: trainer_settings,
        created_at: now_ms(),
    };
    let blob = works
        .space()
        .put(&serde_json::to_vec(&request).unwrap())
        .await
        .unwrap();
    let mut spec = JobSpec::simple(
        ResourceId::parse(TRAINER_RESOURCE).unwrap(),
        TRAINER_REQUEST_INPUT,
        blob,
        "receipt",
        65536,
        DeterminismClass::Observed,
    )
    .leased(LeaseTerms::new(600_000, 1000));
    spec.checkpoint = checkpoint;
    works.post_job(spec, 1, now_ms()).await.unwrap();
    let library = works.model_library().unwrap();
    let mut steps = Vec::new();
    works
        .run_until(
            async {
                tokio::time::timeout(Duration::from_secs(60), async {
                    loop {
                        if library.try_lock().is_err() && access.try_background().is_none() {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                })
                .await
                .expect("trainer acquired its library and actual GPU guard");
            },
            |receipt| {
                if let ResidentReceipt::Tick { steps: tick } = receipt {
                    steps.extend(tick);
                }
            },
        )
        .await
        .unwrap();
    assert!(
        steps
            .iter()
            .any(|step| matches!(step, Step::Started { lease: Some(_), .. }))
    );
    // Let real tensor work begin, then ask while the supervisor is stopped so
    // the waiting ownership and withdrawn HostFacts can be inspected directly.
    tokio::time::sleep(Duration::from_millis(100)).await;
    inference
        .ask(AskRequestV1 {
            run: "foreground".into(),
            question: "t3 t11 t5 t29".into(),
        })
        .unwrap();
    tokio::time::timeout(Duration::from_secs(10), async {
        while !access.foreground_requested() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert!(
        !works.authority().host().facts().gpu,
        "withdrawn as soon as the ask waits"
    );
    assert!(access.try_background().is_none());
    let reclaimed = Arc::new(AtomicBool::new(false));
    let saw_reclaim = reclaimed.clone();
    works
        .run_until(
            async {
                tokio::time::timeout(Duration::from_secs(180), async {
                    loop {
                        let runs = inference.runs();
                        if let Some(run) = runs.runs.first() {
                            if run.state.is_terminal() && reclaimed.load(Ordering::SeqCst) {
                                assert_eq!(run.state, RunStateV1::Finished, "{run:?}");
                                assert!(!run.text.is_empty());
                                break;
                            }
                        }
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                })
                .await
                .expect("owner reclaim and GPU ask both finished");
            },
            |receipt| {
                if let ResidentReceipt::Tick { steps: tick } = receipt {
                    for step in &tick {
                        if matches!(
                            step,
                            Step::Reclaimed {
                                reason: ReclaimReason::ForegroundActivity,
                                ..
                            }
                        ) {
                            saw_reclaim.store(true, Ordering::SeqCst);
                        }
                        assert!(
                            !matches!(
                                step,
                                Step::LeaseLost { .. }
                                    | Step::Released { .. }
                                    | Step::Completed { .. }
                            ),
                            "trainer must end as owner reclaim: {step:?}"
                        );
                    }
                    steps.extend(tick);
                }
            },
        )
        .await
        .unwrap();
    assert!(!access.foreground_requested());
    assert!(
        works.authority().host().facts().gpu,
        "GPU offer restored after inference"
    );
    println!("GPU trainer reclaim (autodiff={autodiff}): {steps:?}");
    println!("inference: {:?}", inference.runs());
    if let Some(path) = std::env::var_os("DJINN_INFERENCE_RECLAIM_RECEIPT") {
        let mut receipt: serde_json::Value = std::fs::read(&path)
            .ok()
            .map(|bytes| serde_json::from_slice(&bytes).unwrap())
            .unwrap_or_else(|| serde_json::json!({}));
        receipt[if checkpoint == mesh::CheckpointClass::Resumable {
            "checkpoint_request"
        } else if autodiff {
            "autodiff"
        } else {
            "finite_difference"
        }] = serde_json::json!({
            "adapter": works.trainer_adapter(), "recorded_at_ms": now_ms(), "checkpoint_class": checkpoint,
            "inference": inference.runs(), "steps": format!("{steps:?}"),
            "offer_withdrawn_while_waiting": true, "offer_restored_after_inference": true,
            "native_in_process": true,
        });
        std::fs::write(path, serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
    }
    resident.restore_distillery(Some(works));
    resident.shutdown().await.unwrap();
    assert!(
        !access.is_occupied(),
        "shutdown drained the actual GPU worker"
    );
}
