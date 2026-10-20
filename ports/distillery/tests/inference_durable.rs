// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

#![cfg(feature = "inference-model")]
use distillery::inference::durable::PersonaRunJournal;
use distillery::inference::*;
use eidetic::typed::TypedPayload;
use esp::infer::decoder::{DecoderDevice, DecoderProvider};
use esp::infer::prompt::{PromptTemplate, SessionProvider};
use esp::infer::provider::{GenerationCheckpoint, SessionIdentity};
use esp::infer::{BoundModelSession, ModelSession};
use esp::infer::{GenerationRequest, InferError};
use esp_test_support::decoder as fixture;
use muniment::{MemoryBackend, RedbBackend};
use personae::{Ed25519Keypair, IdentityProvider, InMemoryProvider};
use std::ops::ControlFlow;
use std::sync::Arc;
use std::time::{Duration, Instant};
use transport::P2pandaTransport;

fn provider(changed: bool) -> Box<dyn InferenceProvider> {
    let decoder = DecoderProvider::from_bytes(
        fixture::CONFIG_JSON.as_bytes(),
        fixture::tokenizer_json().as_bytes(),
        &fixture::weights_bytes(),
        "fixture",
        "burn-ndarray",
        &DecoderDevice::ndarray(),
    )
    .unwrap();
    let mut template = PromptTemplate::from_bytes(esp_test_support::SMOLLM2_INSTRUCT_V1).unwrap();
    template.bos.clear();
    template.eos.clear();
    template.system_prompt.clear();
    template.user.prefix.clear();
    template.user.suffix = " ".into();
    template.assistant.prefix.clear();
    template.assistant.suffix.clear();
    template.stop.clear();
    if changed {
        template.user.suffix.push(' ');
    }
    let bytes = template.serialize_to_bytes().unwrap();
    let session = ModelSession {
        base_model_ref: eidetic::ManifestId::of_blob(&fixture::weights_bytes()),
        model_id: "fixture".into(),
        tokenizer_ref: eidetic::ManifestId::of_blob(fixture::tokenizer_json().as_bytes()),
        prompt_template_hash: eidetic::Hash::of(&bytes),
        quantization: None,
        loader: "burn-ndarray".into(),
        adapters: Vec::new(),
    };
    let bound = BoundModelSession::bind(session, decoder).unwrap();
    Box::new(Slow(SessionProvider::new(bound, bytes, 8).unwrap()))
}
struct Slow(SessionProvider<DecoderProvider>);
impl InferenceProvider for Slow {
    fn capability(&self) -> &esp::infer::ModelCapability {
        self.0.capability()
    }
    fn session_identity(&self) -> Option<SessionIdentity> {
        self.0.session_identity()
    }
    fn generate_streaming(
        &self,
        request: &GenerationRequest,
        on_token: &mut dyn FnMut(&str) -> ControlFlow<()>,
    ) -> Result<String, InferError> {
        self.0.generate_streaming(request, on_token)
    }
    fn generate_checkpointed(
        &self,
        request: &GenerationRequest,
        resume: Option<&GenerationCheckpoint>,
        commit: &mut dyn FnMut(&GenerationCheckpoint) -> Result<(), InferError>,
        stop: &mut dyn FnMut() -> bool,
    ) -> Result<GenerationCheckpoint, InferError> {
        self.0.generate_checkpointed(
            request,
            resume,
            &mut |cp| {
                commit(cp)?;
                if cp.token_ids.len() == 3 && std::env::var_os("DISTILLERY_CRASH_CHILD").is_some() {
                    // Parent owns this process and kills it after the durable
                    // third token. No timing assumption can finish the ask.
                    loop {
                        std::thread::park();
                    }
                }
                std::thread::sleep(Duration::from_millis(12));
                Ok(())
            },
            stop,
        )
    }
}
fn ask() -> AskRequestV1 {
    AskRequestV1 {
        run: "one".into(),
        question: "t1 t5".into(),
    }
}
fn sample() -> GenerationRequest {
    GenerationRequest {
        prompt: "t1 t5".into(),
        max_tokens: 8,
        temperature: 0.85,
        top_p: Some(0.95),
        seed: Some(42),
        ..Default::default()
    }
}
fn wait(lane: &InferenceLane, test: impl Fn(&RunV1) -> bool) -> RunV1 {
    let end = Instant::now() + Duration::from_secs(15);
    loop {
        if let Some(run) = lane.runs().runs.first() {
            if test(run) {
                return run.clone();
            }
        }
        assert!(Instant::now() < end, "timed out: {:?}", lane.runs());
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn open<B: muniment::Backend + Clone + Send + Sync + 'static>(
    journal: Arc<PersonaRunJournal<B>>,
    changed: bool,
) -> InferenceLane {
    InferenceLane::try_open_with_journal(8, move || Ok(provider(changed)), Some(journal)).unwrap()
}
fn author() -> Ed25519Keypair {
    Ed25519Keypair::from_seed([0x29; 32])
}
#[test]
fn restart_resumes_committed_seeded_tokens_and_retains_eidetic_evidence() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("asks.redb");
    let journal = Arc::new(PersonaRunJournal::new(
        RedbBackend::open(&path).unwrap(),
        [7; 32],
        &author(),
        "device-a".into(),
        [9; 32],
    ));
    let lane = open(journal.clone(), false);
    lane.ask_with_generation(ask(), sample()).unwrap();
    wait(&lane, |run| run.tokens >= 3);
    lane.close().unwrap();
    let paused = lane.runs().runs[0].clone();
    assert_eq!(paused.state, RunStateV1::Paused);
    let evidence = journal.retained_record("one").unwrap();
    assert_eq!(
        evidence.status,
        eidetic::models::inference::InferenceRunStatus::Paused
    );
    assert_eq!(
        journal
            .resolve_evidence_blob(evidence.question_blob)
            .unwrap()
            .unwrap(),
        b"t1 t5"
    );
    drop(lane);
    drop(journal);
    let journal = Arc::new(PersonaRunJournal::new(
        RedbBackend::open(&path).unwrap(),
        [7; 32],
        &author(),
        "device-a".into(),
        [9; 32],
    ));
    let lane = open(journal.clone(), false);
    let run = wait(&lane, |run| run.state.is_terminal());
    let expected = provider(false)
        .generate_checkpointed(&sample(), None, &mut |_| Ok(()), &mut || false)
        .unwrap();
    assert_eq!(run.state, RunStateV1::Finished);
    assert_eq!(run.text, expected.text);
    assert_eq!(run.tokens, 8);
    assert_eq!(run.continuations[1].reason, "restart");
    assert_eq!(run.continuations[1].at_token, paused.tokens);
    let cp = journal
        .retained_record("one")
        .unwrap()
        .checkpoint_blob
        .unwrap();
    let actual: GenerationCheckpoint =
        serde_json::from_slice(&journal.resolve_evidence_blob(cp).unwrap().unwrap()).unwrap();
    assert_eq!(actual.token_ids, expected.token_ids);
    assert_eq!(journal.record_comparison("one", &expected).unwrap(), None);
    assert!(
        journal
            .retained_record("one")
            .unwrap()
            .continuations
            .last()
            .unwrap()
            .compared_token_ids_hash
            .is_some()
    );
    assert_eq!(actual.draws, 8);
    lane.close().unwrap();
}
#[test]
fn lock_pauses_and_refuses_the_route_then_unlock_resumes_at_the_boundary() {
    let journal = Arc::new(PersonaRunJournal::new(
        MemoryBackend::new(),
        [7; 32],
        &author(),
        "device-a".into(),
        [9; 32],
    ));
    let lane = open(journal.clone(), false);
    lane.ask_with_generation(ask(), sample()).unwrap();
    wait(&lane, |run| run.tokens >= 2);
    lane.set_locked(true).unwrap();
    let paused = lane.runs().runs[0].clone();
    assert_eq!(paused.state, RunStateV1::Paused);
    assert_eq!(
        lane.ask(AskRequestV1 {
            run: "locked".into(),
            question: "private".into()
        }),
        Err(InferenceLaneError::Locked)
    );
    let mut endpoint = lane.endpoint(chirograph::ProjectionSession("s".into()));
    use graphshell_endpoint::{ProjectionCatalog, ProjectionSource};
    assert!(
        endpoint
            .snapshot(endpoint.describe().projections[0].request.clone())
            .unwrap_err()
            .contains("locked")
    );
    std::thread::sleep(Duration::from_millis(45));
    assert_eq!(lane.runs().runs[0].tokens, paused.tokens);
    journal.lock();
    assert!(journal.retained_record("one").is_err());
    journal.unlock([9; 32]);
    lane.set_locked(false).unwrap();
    let finished = wait(&lane, |run| run.state.is_terminal());
    assert_eq!(finished.state, RunStateV1::Finished);
    assert_eq!(
        finished.continuations.last().unwrap().at_token,
        paused.tokens
    );
    lane.close().unwrap();
}
#[test]
fn explicit_pause_stays_paused_and_cancelled_partial_survives_reopen() {
    let journal = Arc::new(PersonaRunJournal::new(
        MemoryBackend::new(),
        [7; 32],
        &author(),
        "device-a".into(),
        [9; 32],
    ));
    let lane = open(journal.clone(), false);
    lane.ask_with_generation(ask(), sample()).unwrap();
    wait(&lane, |run| run.tokens >= 1);
    lane.pause("one", "user").unwrap();
    wait(&lane, |run| run.state == RunStateV1::Paused);
    lane.set_locked(true).unwrap();
    lane.set_locked(false).unwrap();
    assert_eq!(lane.runs().runs[0].state, RunStateV1::Paused);
    lane.cancel(&CancelRequestV1 { run: "one".into() }).unwrap();
    lane.close().unwrap();
    let lane = open(journal, false);
    assert_eq!(lane.runs().runs[0].state, RunStateV1::Cancelled);
    assert!(!lane.runs().runs[0].text.is_empty());
    lane.close().unwrap();
}
#[test]
fn changed_session_refuses_before_another_token() {
    let journal = Arc::new(PersonaRunJournal::new(
        MemoryBackend::new(),
        [7; 32],
        &author(),
        "device-a".into(),
        [9; 32],
    ));
    let lane = open(journal.clone(), false);
    lane.ask_with_generation(ask(), sample()).unwrap();
    wait(&lane, |run| run.tokens >= 2);
    lane.close().unwrap();
    let before = lane.runs().runs[0].clone();
    let lane = open(journal, true);
    let failed = wait(&lane, |run| run.state.is_terminal());
    assert_eq!(failed.state, RunStateV1::Failed);
    assert!(failed.error.unwrap().contains("different exact session"));
    assert_eq!(failed.tokens, before.tokens);
    lane.close().unwrap();
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn checkpoint_log_syncs_on_existing_transport_and_continues_on_device_b() {
    let pa = InMemoryProvider::from_seed([1; 32]);
    let pb = InMemoryProvider::from_seed([2; 32]);
    let a = P2pandaTransport::builder_for(&pa.derived_keypair(b"transport").unwrap())
        .gossip()
        .bind()
        .await
        .unwrap();
    let b = P2pandaTransport::builder_for(&pb.derived_keypair(b"transport").unwrap())
        .gossip()
        .bind()
        .await
        .unwrap();
    a.add_peer(b.endpoint_addr().await.unwrap()).await.unwrap();
    b.add_peer(a.endpoint_addr().await.unwrap()).await.unwrap();
    let overlay = transport::sync_overlay_topic([7; 32]);
    let a_id =
        transport::PeerID::from_public_key(pa.derived_keypair(b"transport").unwrap().public_key());
    let b_id =
        transport::PeerID::from_public_key(pb.derived_keypair(b"transport").unwrap().public_key());
    a.set_topics(b_id, &[overlay]).await.unwrap();
    b.set_topics(a_id, &[overlay]).await.unwrap();
    let ja = Arc::new(PersonaRunJournal::new(
        MemoryBackend::new(),
        [7; 32],
        &author(),
        "device-a".into(),
        [9; 32],
    ));
    let jb = Arc::new(PersonaRunJournal::new(
        MemoryBackend::new(),
        [7; 32],
        &author(),
        "device-b".into(),
        [9; 32],
    ));
    let (ep, gossip) = a.sync_parts().unwrap();
    ja.join(ep, gossip).await.unwrap();
    let lane = open(ja.clone(), false);
    lane.ask_with_generation(ask(), sample()).unwrap();
    wait(&lane, |run| run.tokens >= 3);
    lane.pause("one", "user").unwrap();
    let paused = wait(&lane, |run| run.state == RunStateV1::Paused);
    // B joins after A paused: this proves offline log reconciliation as well
    // as live continuation, without inventing a second work transport.
    let (ep, gossip) = b.sync_parts().unwrap();
    jb.join(ep, gossip).await.unwrap();
    let end = Instant::now() + Duration::from_secs(25);
    loop {
        if jb.retained_record("one").is_ok_and(|record| {
            record.status == eidetic::models::inference::InferenceRunStatus::Paused
        }) {
            break;
        }
        assert!(
            Instant::now() < end,
            "log did not replicate: {:?}",
            jb.sync_status()
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    assert!(jb.sync_status().unwrap().ops_received > 0);
    let lane_b = open(jb.clone(), false);
    assert_eq!(lane_b.runs().runs[0].state, RunStateV1::Paused);
    lane_b.resume("one", "device-transfer").unwrap();
    let finished = wait(&lane_b, |run| run.state.is_terminal());
    assert_eq!(finished.state, RunStateV1::Finished);
    let expected = provider(false)
        .generate_checkpointed(&sample(), None, &mut |_| Ok(()), &mut || false)
        .unwrap();
    assert_eq!(finished.text, expected.text);
    assert_eq!(finished.continuations.last().unwrap().device, "device-b");
    assert_eq!(
        finished.continuations.last().unwrap().at_token,
        paused.tokens
    );
    let cp = jb.retained_record("one").unwrap().checkpoint_blob.unwrap();
    let actual: GenerationCheckpoint =
        serde_json::from_slice(&jb.resolve_evidence_blob(cp).unwrap().unwrap()).unwrap();
    assert_eq!(actual.token_ids, expected.token_ids);
    assert_eq!(jb.record_comparison("one", &expected).unwrap(), None);
    lane_b.close().unwrap();
    lane.close().unwrap();
    ja.shutdown().await.unwrap();
    jb.shutdown().await.unwrap();
    a.close().await.unwrap();
    b.close().await.unwrap();
}

#[test]
fn crash_child() {
    let Some(root) = std::env::var_os("DISTILLERY_CRASH_CHILD") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    let journal = Arc::new(PersonaRunJournal::new(
        RedbBackend::open(root.join("asks.redb")).unwrap(),
        [7; 32],
        &author(),
        "device-a".into(),
        [9; 32],
    ));
    let lane = open(journal, false);
    lane.ask_with_generation(ask(), sample()).unwrap();
    wait(&lane, |run| run.tokens >= 3);
    std::fs::write(root.join("committed"), b"committed").unwrap();
    loop {
        std::thread::park();
    }
}
#[test]
fn killing_the_process_mid_answer_recovers_its_last_committed_draw() {
    let temp = tempfile::tempdir().unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "crash_child", "--nocapture"])
        .env("DISTILLERY_CRASH_CHILD", temp.path())
        .spawn()
        .unwrap();
    let end = Instant::now() + Duration::from_secs(15);
    while !temp.path().join("committed").exists() {
        if let Some(status) = child.try_wait().unwrap() {
            panic!("child exited before checkpoint: {status}");
        }
        if Instant::now() >= end {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("child never committed");
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    let journal = Arc::new(PersonaRunJournal::new(
        RedbBackend::open(temp.path().join("asks.redb")).unwrap(),
        [7; 32],
        &author(),
        "device-a".into(),
        [9; 32],
    ));
    let before = journal
        .retained_record("one")
        .unwrap()
        .checkpoint_blob
        .unwrap();
    let before: GenerationCheckpoint =
        serde_json::from_slice(&journal.resolve_evidence_blob(before).unwrap().unwrap()).unwrap();
    assert!(before.token_ids.len() >= 3);
    assert!(!before.complete);
    let lane = open(journal.clone(), false);
    assert_eq!(
        wait(&lane, |run| run.state.is_terminal()).state,
        RunStateV1::Finished
    );
    let expected = provider(false)
        .generate_checkpointed(&sample(), None, &mut |_| Ok(()), &mut || false)
        .unwrap();
    assert_eq!(journal.record_comparison("one", &expected).unwrap(), None);
    lane.close().unwrap();
}

#[test]
fn a_new_loaded_session_does_not_relabel_a_retained_failure_without_tokens() {
    let journal = Arc::new(PersonaRunJournal::new(
        MemoryBackend::new(),
        [7; 32],
        &author(),
        "device-a".into(),
        [9; 32],
    ));
    let lane = open(journal.clone(), false);
    lane.ask(AskRequestV1 {
        run: "one".into(),
        question: "t1 ".repeat(30),
    })
    .unwrap();
    let failed = wait(&lane, |run| run.state.is_terminal());
    assert_eq!(failed.state, RunStateV1::Failed);
    assert_eq!(failed.tokens, 0);
    assert!(failed.session.is_some());
    lane.close().unwrap();
    let lane = open(journal, true);
    let end = Instant::now() + Duration::from_secs(5);
    while lane.runs().session.is_none() {
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_ne!(lane.runs().session, failed.session);
    assert_eq!(lane.runs().runs[0].session, failed.session);
    lane.close().unwrap();
}
