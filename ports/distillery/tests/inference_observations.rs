// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

#![cfg(feature = "inference")]

use apparatus_core::RetentionLimits;
use distillery::inference::{
    InferenceLane, InferenceObservation as Phase, RunJournal, RunRecord, StubInferenceProvider,
};
use distillery::inference_protocol::{AskRequestV1, RunStateV1};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

fn limits() -> RetentionLimits {
    RetentionLimits {
        max_records: 32,
        max_bytes: 8192,
        max_age: Duration::from_secs(60),
    }
}
fn ask(lane: &InferenceLane, name: &str) {
    lane.ask(AskRequestV1 {
        run: name.into(),
        question: "private-prompt".into(),
    })
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if lane
            .runs()
            .runs
            .iter()
            .any(|run| run.run == name && run.state.is_terminal())
        {
            break;
        }
        assert!(Instant::now() < deadline, "worker did not finish");
        std::thread::yield_now();
    }
}
fn lane() -> InferenceLane {
    InferenceLane::open(|| {
        Box::new(StubInferenceProvider::new().with_response("private-prompt", "private-answer"))
    })
}

#[test]
fn real_worker_observations_are_correlated_redacted_and_independently_retained() {
    let lane = lane();
    assert!(lane.observation_cursor().is_err());
    let mut receipt = lane
        .enable_observations("observation-run".into(), limits())
        .unwrap();
    let mut inspector = lane.observation_cursor().unwrap();
    assert!(
        lane.enable_observations("silent-reset".into(), limits())
            .is_err()
    );
    ask(&lane, "private-run-name");
    assert_eq!(lane.runs().runs[0].state, RunStateV1::Finished);
    let inspection = lane.read_observations(&mut inspector, 32).unwrap();
    let evidence = lane.read_observations(&mut receipt, 32).unwrap();
    assert_eq!(inspection, evidence);
    assert_eq!(
        evidence
            .records
            .iter()
            .map(|record| record.payload)
            .collect::<Vec<_>>(),
        [
            Phase::Queued,
            Phase::WorkerStarted,
            Phase::WorkerFinished,
            Phase::RunFinished
        ]
    );
    let operation = &evidence.records[0].envelope.metadata.operation;
    assert!(operation.is_some());
    assert!(
        evidence
            .records
            .iter()
            .all(|record| record.envelope.metadata.operation == *operation
                && record.envelope.metadata.presented_frame.is_none()
                && record.envelope.metadata.semantic_revision.is_none())
    );
    let encoded = serde_json::to_string(&evidence).unwrap();
    for secret in ["private-prompt", "private-answer", "private-run-name"] {
        assert!(!encoded.contains(secret));
    }

    lane.set_observation_limits(RetentionLimits {
        max_records: 1,
        ..limits()
    })
    .unwrap();
    let mut slow = lane.observation_cursor().unwrap();
    let tail = lane.read_observations(&mut slow, 32).unwrap();
    assert_eq!(tail.records.len(), 1);
    assert_eq!(tail.stats.loss.evicted, 3);
    assert_eq!(tail.gaps.len(), 1);

    lane.set_observation_limits(RetentionLimits {
        max_bytes: 1,
        ..limits()
    })
    .unwrap();
    ask(&lane, "too-large-for-diagnostics");
    let mut cursor = lane.observation_cursor().unwrap();
    let omitted = lane.read_observations(&mut cursor, 32).unwrap();
    assert!(omitted.records.is_empty());
    assert_eq!(omitted.stats.loss.rejected_oversized, 4);
    assert!(!omitted.gaps.is_empty());
    lane.set_observation_limits(RetentionLimits {
        max_records: 0,
        ..limits()
    })
    .unwrap();
    ask(&lane, "disabled-diagnostics");
    assert_eq!(
        lane.read_observations(&mut cursor, 32)
            .unwrap()
            .stats
            .loss
            .rejected_disabled,
        4
    );
    lane.close().unwrap();

    let next = self::lane();
    let mut previous_reader = receipt;
    next.enable_observations("new-observation-run".into(), limits())
        .unwrap();
    ask(&next, "fresh-work");
    let fresh = next.read_observations(&mut previous_reader, 32).unwrap();
    assert!(fresh.boundary.is_some());
    assert_eq!(
        fresh.records[0].envelope.metadata.operation, *operation,
        "execution id reuse is scoped by the observation run"
    );
    assert_ne!(
        fresh.records[0].envelope.reference.run,
        evidence.records[0].envelope.reference.run
    );
    next.close().unwrap();
}

struct RefusingJournal;
impl RunJournal for RefusingJournal {
    fn device(&self) -> &str {
        "observation-device"
    }
    fn load(&self) -> Result<Vec<RunRecord>, String> {
        Ok(Vec::new())
    }
    fn append(&self, record: &RunRecord) -> Result<(), String> {
        if serde_json::to_value(record).unwrap()["view"]["state"] == "failed" {
            Err("private-journal-error".into())
        } else {
            Ok(())
        }
    }
}

#[test]
fn unsupported_worker_and_refused_journal_never_become_successful_observations() {
    // The stub truthfully refuses checkpointed generation; then the real
    // receiving journal refuses the failed record. Neither is a success.
    let lane = InferenceLane::try_open_with_journal(
        8,
        || Ok(Box::new(StubInferenceProvider::new())),
        Some(Arc::new(RefusingJournal)),
    )
    .unwrap();
    let mut reader = lane
        .enable_observations("failed-observation-run".into(), limits())
        .unwrap();
    ask(&lane, "private-run-name");
    let view = lane.runs();
    assert_eq!(view.runs[0].state, RunStateV1::Failed);
    assert!(
        view.runs[0]
            .error
            .as_ref()
            .unwrap()
            .contains("private-journal-error")
    );
    let batch = lane.read_observations(&mut reader, 32).unwrap();
    let phases = batch
        .records
        .iter()
        .map(|record| record.payload)
        .collect::<Vec<_>>();
    assert_eq!(
        phases,
        [
            Phase::Queued,
            Phase::WorkerStarted,
            Phase::WorkerFailed,
            Phase::RunFailed
        ]
    );
    assert!(
        !serde_json::to_string(&batch)
            .unwrap()
            .contains("private-journal-error")
    );
    assert!(
        lane.close().is_err(),
        "the failed journal still fails close"
    );
}

#[test]
fn successful_worker_reply_does_not_hide_a_failed_terminal_journal_commit() {
    use distillery::inference::{GenerationCheckpoint, InferError, InferenceProvider};
    use esp::infer::provider::SessionIdentity;
    use esp::infer::{GenerationRequest, ModelCapability};
    use std::ops::ControlFlow;
    struct Complete(StubInferenceProvider);
    impl InferenceProvider for Complete {
        fn capability(&self) -> &ModelCapability {
            self.0.capability()
        }
        fn session_identity(&self) -> Option<SessionIdentity> {
            Some(SessionIdentity {
                session_id: "synthetic-session".into(),
                prompt_template_hash: "synthetic-template".into(),
            })
        }
        fn generate_streaming(
            &self,
            _: &GenerationRequest,
            _: &mut dyn FnMut(&str) -> ControlFlow<()>,
        ) -> Result<String, InferError> {
            unreachable!("the journal asks for checkpointed generation")
        }
        fn generate_checkpointed(
            &self,
            request: &GenerationRequest,
            _: Option<&GenerationCheckpoint>,
            commit: &mut dyn FnMut(&GenerationCheckpoint) -> Result<(), InferError>,
            _: &mut dyn FnMut() -> bool,
        ) -> Result<GenerationCheckpoint, InferError> {
            let checkpoint = GenerationCheckpoint {
                placements: Vec::new(),
                session: self.session_identity(),
                request: request.clone(),
                prompt_ids: vec![1],
                token_ids: vec![2],
                draws: 1,
                text: "private-completed-answer".into(),
                complete: true,
            };
            commit(&checkpoint)?;
            Ok(checkpoint)
        }
    }
    struct RefuseFinish;
    impl RunJournal for RefuseFinish {
        fn device(&self) -> &str {
            "synthetic-device"
        }
        fn load(&self) -> Result<Vec<RunRecord>, String> {
            Ok(Vec::new())
        }
        fn append(&self, record: &RunRecord) -> Result<(), String> {
            if serde_json::to_value(record).unwrap()["view"]["state"] == "finished" {
                Err("private-final-commit-error".into())
            } else {
                Ok(())
            }
        }
    }
    let lane = InferenceLane::try_open_with_journal(
        8,
        || Ok(Box::new(Complete(StubInferenceProvider::new()))),
        Some(Arc::new(RefuseFinish)),
    )
    .unwrap();
    let mut reader = lane
        .enable_observations("commit-refusal-run".into(), limits())
        .unwrap();
    ask(&lane, "private-run-name");
    assert_eq!(lane.runs().runs[0].state, RunStateV1::Failed);
    let batch = lane.read_observations(&mut reader, 32).unwrap();
    assert_eq!(
        batch
            .records
            .iter()
            .map(|record| record.payload)
            .collect::<Vec<_>>(),
        [
            Phase::Queued,
            Phase::WorkerStarted,
            Phase::WorkerFinished,
            Phase::RunFailed
        ]
    );
    let encoded = serde_json::to_string(&batch).unwrap();
    assert!(!encoded.contains("private-completed-answer"));
    assert!(!encoded.contains("private-final-commit-error"));
    assert!(lane.close().is_err());
}

#[test]
fn cancellation_request_is_distinct_from_the_workers_terminal_reply() {
    use distillery::inference::{InferError, InferenceProvider};
    use distillery::inference_protocol::CancelRequestV1;
    use esp::infer::{GenerationRequest, ModelCapability};
    use std::{
        ops::ControlFlow,
        sync::{Mutex, mpsc},
    };
    struct Waiting {
        stub: StubInferenceProvider,
        ready: mpsc::Sender<()>,
        release: Mutex<mpsc::Receiver<()>>,
    }
    impl InferenceProvider for Waiting {
        fn capability(&self) -> &ModelCapability {
            self.stub.capability()
        }
        fn generate_streaming(
            &self,
            _: &GenerationRequest,
            token: &mut dyn FnMut(&str) -> ControlFlow<()>,
        ) -> Result<String, InferError> {
            self.ready.send(()).unwrap();
            self.release
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(10))
                .unwrap();
            let _ = token("private-unaccepted-fragment");
            Ok("private-unaccepted-fragment".into())
        }
    }
    let (ready, started) = mpsc::channel();
    let (release, gate) = mpsc::channel();
    let lane = InferenceLane::open(move || {
        Box::new(Waiting {
            stub: StubInferenceProvider::new(),
            ready,
            release: Mutex::new(gate),
        })
    });
    let mut reader = lane
        .enable_observations("cancel-observation-run".into(), limits())
        .unwrap();
    lane.ask(AskRequestV1 {
        run: "cancelled".into(),
        question: "private-prompt".into(),
    })
    .unwrap();
    started.recv_timeout(Duration::from_secs(10)).unwrap();
    lane.cancel(&CancelRequestV1 {
        run: "cancelled".into(),
    })
    .unwrap();
    let requested = lane.read_observations(&mut reader, 32).unwrap();
    assert!(
        requested
            .records
            .iter()
            .any(|record| record.payload == Phase::CancelRequested)
    );
    assert!(
        !requested
            .records
            .iter()
            .any(|record| record.payload == Phase::RunCancelled)
    );
    release.send(()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !lane.runs().runs[0].state.is_terminal() {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert_eq!(lane.runs().runs[0].state, RunStateV1::Cancelled);
    let terminal = lane.read_observations(&mut reader, 32).unwrap();
    assert!(
        terminal
            .records
            .iter()
            .any(|record| record.payload == Phase::WorkerCancelled)
    );
    assert!(
        terminal
            .records
            .iter()
            .any(|record| record.payload == Phase::RunCancelled)
    );
    assert!(
        !terminal
            .records
            .iter()
            .any(|record| record.payload == Phase::RunFinished)
    );
    lane.close().unwrap();
}
