// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

#![cfg(feature = "inference")]
use distillery::inference::{InferenceLane, InferenceLaneError, RunJournal, RunRecord};
use distillery::inference_protocol::{AskRequestV1, CancelRequestV1, RunStateV1};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

#[derive(Default)]
struct Journal(Mutex<Vec<RunRecord>>);
impl RunJournal for Journal {
    fn device(&self) -> &str {
        "shutdown-test"
    }
    fn load(&self) -> Result<Vec<RunRecord>, String> {
        Ok(self.0.lock().unwrap().clone())
    }
    fn append(&self, record: &RunRecord) -> Result<(), String> {
        let mut rows = self.0.lock().unwrap();
        let name = serde_json::to_value(record).unwrap()["view"]["run"].clone();
        if let Some(row) = rows
            .iter_mut()
            .find(|r| serde_json::to_value(r).unwrap()["view"]["run"] == name)
        {
            *row = record.clone();
        } else {
            rows.push(record.clone());
        }
        Ok(())
    }
}
fn ask(run: &str) -> AskRequestV1 {
    AskRequestV1 {
        run: run.into(),
        question: "synthetic shutdown test".into(),
    }
}
fn await_disconnect(lane: &InferenceLane) {
    let until = Instant::now() + Duration::from_secs(5);
    while lane.runs().load_error.is_none() {
        assert!(Instant::now() < until, "actor loss was not materialized");
        std::thread::yield_now();
    }
}
#[test]
fn actor_loss_fails_pending_work_and_refuses_later_commands() {
    let (release, gate) = mpsc::channel();
    let journal = Arc::new(Journal::default());
    let lane = InferenceLane::try_open_with_journal(
        8,
        move || {
            gate.recv().unwrap();
            panic!("synthetic model-construction panic");
        },
        Some(journal.clone()),
    )
    .unwrap();
    lane.ask(ask("pending")).unwrap();
    release.send(()).unwrap();
    await_disconnect(&lane);
    assert_eq!(lane.runs().runs[0].state, RunStateV1::Failed);
    let saved = serde_json::to_value(journal.load().unwrap()).unwrap();
    assert_eq!(saved[0]["view"]["state"], "failed");
    assert_eq!(lane.ask(ask("late")), Err(InferenceLaneError::Closed));
    assert_eq!(
        lane.cancel(&CancelRequestV1 {
            run: "pending".into()
        }),
        Err(InferenceLaneError::Closed)
    );
    assert_eq!(
        lane.pause("pending", "user"),
        Err(InferenceLaneError::Closed)
    );
    assert_eq!(lane.runs().runs.len(), 1);
    let first = lane.close().unwrap_err();
    assert!(first.contains("actor"));
    assert_eq!(lane.close().unwrap_err(), first);

    let mut paused = saved[0].clone();
    paused["view"]["state"] = "paused".into();
    paused["pause_reason"] = "user".into();
    let paused: RunRecord = serde_json::from_value(paused).unwrap();
    let journal = Arc::new(Journal(Mutex::new(vec![paused])));
    let before = serde_json::to_value(journal.load().unwrap()).unwrap();
    let lane = InferenceLane::try_open_with_journal(
        8,
        || panic!("synthetic resume provider panic"),
        Some(journal.clone()),
    )
    .unwrap();
    await_disconnect(&lane);
    assert_eq!(
        lane.resume("pending", "user"),
        Err(InferenceLaneError::Closed)
    );
    assert_eq!(
        serde_json::to_value(journal.load().unwrap()).unwrap(),
        before
    );
    assert!(lane.close().is_err());
}
