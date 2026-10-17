//! Public inference-lane crash observations. No production/original-test edits.
use distillery::inference::{
    AskRequestV1, CancelRequestV1, InferError, InferenceLane, InferenceLaneError,
    RunJournal, RunRecord, RunStateV1,
};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

#[derive(Default)]
struct MemoryJournal(Mutex<Vec<RunRecord>>);
impl RunJournal for MemoryJournal {
    fn device(&self) -> &str { "audit-device" }
    fn load(&self) -> Result<Vec<RunRecord>, String> { Ok(self.0.lock().unwrap().clone()) }
    fn append(&self, record: &RunRecord) -> Result<(), String> {
        let mut rows = self.0.lock().unwrap();
        let name = serde_json::to_value(record).unwrap()["view"]["run"].clone();
        if let Some(row) = rows.iter_mut().find(|row| serde_json::to_value(row).unwrap()["view"]["run"] == name) {
            *row = record.clone();
        } else { rows.push(record.clone()); }
        Ok(())
    }
}
fn ask(name: &str) -> AskRequestV1 {
    AskRequestV1 { run: name.into(), question: "public synthetic audit question".into() }
}
fn dead(lane: &InferenceLane) {
    let deadline = Instant::now() + Duration::from_secs(5);
    for index in 0..1000 {
        if matches!(lane.ask(ask(&format!("after-crash-{index}"))), Err(InferenceLaneError::Closed)) {
            return;
        }
        assert!(Instant::now() < deadline, "actor did not disconnect");
        std::thread::sleep(Duration::from_millis(1));
    }
    panic!("actor did not disconnect within bounded observation count");
}
fn state(lane: &InferenceLane, name: &str) -> RunStateV1 {
    lane.runs().runs.into_iter().find(|run| run.run == name).unwrap().state
}

#[test]
fn crashed_actor_leaves_queued_work_while_cancel_pause_and_close_report_success() {
    let (release, gate) = mpsc::channel();
    let journal = Arc::new(MemoryJournal::default());
    let lane = InferenceLane::try_open_with_journal(8, move || {
        gate.recv().unwrap();
        panic!("synthetic audit model-construction panic");
    }, Some(journal.clone())).unwrap();
    lane.ask(ask("before-crash")).unwrap();
    release.send(()).unwrap();
    dead(&lane);
    assert_eq!(state(&lane, "before-crash"), RunStateV1::Queued);
    assert!(lane.runs().load_error.is_none());
    lane.cancel(&CancelRequestV1 { run: "before-crash".into() }).unwrap();
    lane.pause("before-crash", "user").unwrap();
    assert_eq!(state(&lane, "before-crash"), RunStateV1::Queued);
    lane.close().unwrap();
    assert_eq!(state(&lane, "before-crash"), RunStateV1::Queued);
    let saved = journal.load().unwrap();
    let first = saved.iter().map(|r| serde_json::to_value(r).unwrap())
        .find(|v| v["view"]["run"] == "before-crash").unwrap();
    assert_eq!(first["view"]["state"], "queued");
    assert!(saved.iter().any(|r| serde_json::to_value(r).unwrap()["view"]["run"]
        .as_str().unwrap().starts_with("after-crash-")));
    println!("OBSERVATION: actor disconnected; cancel/pause/close all returned success; admitted run and rejected late ask remain queued, including journal state");
}

#[test]
fn resume_commits_queued_state_and_reports_success_after_actor_disconnect() {
    // Obtain the full record schema from the public lane, then retain a user pause.
    let (release, gate) = mpsc::channel();
    let source = Arc::new(MemoryJournal::default());
    let seed = InferenceLane::try_open_with_journal(8, move || {
        gate.recv().unwrap();
        panic!("synthetic audit seed-construction panic");
    }, Some(source.clone())).unwrap();
    seed.ask(ask("paused")).unwrap();
    release.send(()).unwrap();
    dead(&seed);
    seed.close().unwrap();
    let record = source.load().unwrap().into_iter()
        .find(|r| serde_json::to_value(r).unwrap()["view"]["run"] == "paused").unwrap();
    let mut value = serde_json::to_value(record).unwrap();
    value["view"]["state"] = "paused".into();
    value["pause_reason"] = "user".into();
    let retained = Arc::new(MemoryJournal(Mutex::new(vec![serde_json::from_value(value).unwrap()])));
    let (release, gate) = mpsc::channel();
    let lane = InferenceLane::try_open_with_journal(8, move || {
        gate.recv().unwrap();
        panic!("synthetic audit resumed-model construction panic");
    }, Some(retained.clone())).unwrap();
    assert_eq!(state(&lane, "paused"), RunStateV1::Paused);
    release.send(()).unwrap();
    dead(&lane);
    lane.resume("paused", "user").unwrap();
    assert_eq!(state(&lane, "paused"), RunStateV1::Queued);
    let saved = retained.load().unwrap().into_iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .find(|v| v["view"]["run"] == "paused").unwrap();
    assert_eq!(saved["view"]["state"], "queued");
    assert_eq!(saved["continuations"].as_array().unwrap().len(), 2);
    lane.close().unwrap();
    println!("OBSERVATION: resume returned success and journaled a queued continuation after actor disconnection");
}

#[test]
fn ordinary_model_errors_produce_a_terminal_failed_run() {
    let lane = InferenceLane::try_open(|| Err(InferError::InvalidConfig("synthetic expected load error".into())));
    lane.ask(ask("ordinary-error")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !state(&lane, "ordinary-error").is_terminal() {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert_eq!(state(&lane, "ordinary-error"), RunStateV1::Failed);
    assert!(lane.runs().load_error.unwrap().contains("synthetic expected load error"));
    lane.close().unwrap();
    println!("CONTROL: returned model error is visible and the ask becomes Failed");
}
