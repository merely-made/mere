// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Redacted observations at the inference owner's actual command/reply boundaries.
//! They are an optional bounded copy, never the run table or durable journal.

use super::{InferUpdate, InferenceLane, RunStateV1};
use apparatus_core::{
    Batch, Cursor, ObservationMetadata, ObservationStore, OperationId, RetentionLimits, RunId,
};
use serde::Serialize;
use std::time::Instant;

/// Fixed categories only: prompts, answers, caller run names and error text are
/// deliberately absent. Worker reply and accepted run outcome are separate facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum InferenceObservation {
    /// The actor inbox accepted this execution request, before execution.
    Queued,
    /// The owner requested a stop; its terminal reply is still outstanding.
    CancelRequested,
    /// The owner received the worker's execution-start notification.
    WorkerStarted,
    /// The worker returned a successful answer, before journal adjudication.
    WorkerFinished,
    /// The worker returned a failure; diagnostic details remain private.
    WorkerFailed,
    /// The worker acknowledged cancellation.
    WorkerCancelled,
    /// The worker acknowledged a checkpoint pause.
    WorkerPaused,
    /// The run table refused a reply for unknown or already terminal work.
    UpdateDiscarded,
    /// The owner accepted a finished state after its required journal barrier.
    RunFinished,
    /// The owner recorded failure, including journal refusal or actor loss.
    RunFailed,
    /// The owner accepted the cancelled state.
    RunCancelled,
    /// The owner accepted the paused state.
    RunPaused,
}

pub(super) struct Observations {
    started: Instant,
    store: ObservationStore<InferenceObservation>,
    error: Option<String>,
}

impl Observations {
    pub(super) fn note(&mut self, id: u64, phase: InferenceObservation) {
        // Unit variants have fixed JSON encodings; account this phase's bytes.
        // Metadata carries the owner-generated execution id, never a user label.
        if let Err(error) = self.store.record(
            phase,
            serde_json::to_vec(&phase)
                .expect("unit observation enum serializes")
                .len(),
            ObservationMetadata {
                operation: Some(OperationId(format!("inference:{id}"))),
                ..ObservationMetadata::default()
            },
            self.started.elapsed(),
        ) {
            self.error.get_or_insert_with(|| error.to_string());
        }
    }
}

pub(super) fn worker_update(update: &InferUpdate) -> Option<(u64, InferenceObservation)> {
    use InferenceObservation::*;
    match update {
        InferUpdate::Started { id } => Some((*id, WorkerStarted)),
        InferUpdate::Finished { id, .. } => Some((*id, WorkerFinished)),
        InferUpdate::Failed { id, .. } => Some((*id, WorkerFailed)),
        InferUpdate::Cancelled { id } => Some((*id, WorkerCancelled)),
        InferUpdate::Paused { id } => Some((*id, WorkerPaused)),
        _ => None,
    }
}

pub(super) fn accepted_state(state: &RunStateV1) -> Option<InferenceObservation> {
    use InferenceObservation::*;
    match state {
        RunStateV1::Finished => Some(RunFinished),
        RunStateV1::Failed => Some(RunFailed),
        RunStateV1::Cancelled => Some(RunCancelled),
        RunStateV1::Paused => Some(RunPaused),
        _ => None,
    }
}

impl InferenceLane {
    /// Opt in before the operations to be observed. The host supplies a fresh,
    /// bounded, nonsecret run identity and retention/export policy. There is no
    /// backfill and no automatic export, screenshot, frame or causal attribution.
    /// Enabling twice refuses rather than silently resetting reader coverage.
    pub fn enable_observations(
        &self,
        run: RunId,
        limits: RetentionLimits,
    ) -> Result<Cursor, String> {
        let mut table = self.shared.table();
        if table.observations.is_some() {
            return Err("inference observations are already enabled".into());
        }
        let store = ObservationStore::new(run, "distillery.inference".into(), limits);
        let cursor = store.cursor();
        table.observations = Some(Observations {
            started: Instant::now(),
            store,
            error: None,
        });
        Ok(cursor)
    }

    /// Another independent reader, starting at the run's beginning. Missing
    /// retained records remain visible as Apparatus gaps and cumulative loss.
    pub fn observation_cursor(&self) -> Result<Cursor, String> {
        self.shared
            .table()
            .observations
            .as_ref()
            .map(|observations| observations.store.cursor())
            .ok_or_else(|| "inference observations are not enabled".into())
    }

    /// Read already-redacted records. Observation failures fail the requested
    /// read, but do not alter inference or claim successful domain completion.
    pub fn read_observations(
        &self,
        cursor: &mut Cursor,
        max_records: usize,
    ) -> Result<Batch<InferenceObservation>, String> {
        let mut table = self.shared.table();
        let observations = table
            .observations
            .as_mut()
            .ok_or("inference observations are not enabled")?;
        if let Some(error) = &observations.error {
            return Err(format!("inference observations: {error}"));
        }
        observations
            .store
            .read(cursor, observations.started.elapsed(), max_records)
            .map_err(|error| error.to_string())
    }

    /// Apply the host's changed retention policy without resetting run identity,
    /// cursors or loss counters. A zero bound disables retention explicitly.
    pub fn set_observation_limits(&self, limits: RetentionLimits) -> Result<(), String> {
        let mut table = self.shared.table();
        let observations = table
            .observations
            .as_mut()
            .ok_or("inference observations are not enabled")?;
        observations
            .store
            .set_limits(limits, observations.started.elapsed())
            .map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::{RunJournal, RunRecord, StubInferenceProvider};
    use crate::inference_protocol::AskRequestV1;
    use std::{
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
            mpsc,
        },
        time::Duration,
    };

    struct Journal(AtomicUsize);
    impl RunJournal for Journal {
        fn device(&self) -> &str {
            "stale-reply-test"
        }
        fn load(&self) -> Result<Vec<RunRecord>, String> {
            Ok(Vec::new())
        }
        fn append(&self, _: &RunRecord) -> Result<(), String> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }
    #[test]
    fn stale_worker_replies_neither_revive_nor_rejournal_terminal_work() {
        let journal = Arc::new(Journal(AtomicUsize::new(0)));
        let lane = InferenceLane::try_open_with_journal(
            8,
            || Ok(Box::new(StubInferenceProvider::new())),
            Some(journal.clone()),
        )
        .unwrap();
        let mut reader = lane
            .enable_observations(
                "stale-reply-run".into(),
                RetentionLimits {
                    max_records: 32,
                    max_bytes: 8192,
                    max_age: Duration::from_secs(60),
                },
            )
            .unwrap();
        lane.ask(AskRequestV1 {
            run: "retained".into(),
            question: "question".into(),
        })
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !lane.runs().runs[0].state.is_terminal() {
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        lane.close().unwrap();
        let (id, before) = {
            let table = lane.shared.table();
            (
                table.runs[0].actor_id,
                serde_json::to_value(&table.runs[0]).unwrap(),
            )
        };
        let writes = journal.0.load(Ordering::SeqCst);
        let generation = lane.generation();
        let (sender, updates) = mpsc::channel();
        sender.send(InferUpdate::Started { id }).unwrap();
        sender
            .send(InferUpdate::Finished {
                id,
                text: "stale answer".into(),
            })
            .unwrap();
        sender
            .send(InferUpdate::Failed {
                id: id + 1,
                error: super::super::InferError::Backend("unknown execution".into()),
            })
            .unwrap();
        drop(sender);
        // Exercise the actual receiving bridge with stale replies after its
        // real actor has closed. It must retain the already accepted outcome.
        super::super::bridge(&lane.shared, &updates);
        let table = lane.shared.table();
        assert!(!table.runs[0].executing);
        assert_eq!(serde_json::to_value(&table.runs[0]).unwrap(), before);
        assert_eq!(journal.0.load(Ordering::SeqCst), writes);
        assert_eq!(table.generation, generation);
        drop(table);
        let batch = lane.read_observations(&mut reader, 32).unwrap();
        assert_eq!(
            batch
                .records
                .iter()
                .filter(|r| r.payload == InferenceObservation::UpdateDiscarded)
                .count(),
            3
        );
        assert!(
            !batch
                .records
                .iter()
                .any(|r| r.payload == InferenceObservation::RunFinished)
        );
    }
}
