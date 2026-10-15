// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The resident inference lane: one esp inference actor behind an
//! application route (resident inference and recall plan, I1).
//!
//! [`InferenceLane`] owns the actor and a table of runs. A bridge thread
//! folds the actor's updates into the table, so the lane needs no async
//! executor and composes the same way in a desktop or an embedded resident.
//! [`InferenceAskEndpoint`] is the route: `Ask` and `Cancel` intents in, a
//! card whose media is the run table out, and a payload-free notice when the
//! table changes. The route's notice poll interval is therefore the rate at
//! which streamed fragments reach a client.
//!
//! What this slice does not do yet: pause and resume (I3), recall and
//! citations (I4), and any model other than the one the caller builds.

use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::Duration;

use armillary::actor::{ActorHandle, Wake};
use chirograph::{
    BoundsRelationship, CachePolicy, CardValueV1, CarrierNotice, ContentHash, EndpointDescriptor,
    IntentInvocation, IntentResult, PortableCardV1, PresentationBinding, PresentationCapability,
    PresentationCodec, PresentationKey, PresentationManifest, PresentationOffer,
    PresentationSemantics, ProjectionOffer, ProjectionRequest, ProjectionSession,
    ProjectionSnapshot, ProtocolVersion, ResourceRequest, ResourceResponse, SemanticRole,
};
use esp::infer::{GenerationRequest, InferCommand, InferUpdate, spawn_inference_actor};
use graphshell_endpoint::{
    IntentSink, PresentationSource, ProjectionCatalog, ProjectionNoticeSource, ProjectionSource,
};
use sceno::{
    Arrangement, Footprint, InstanceId, ProjectedItem, Representation, Scene, Score, Size2,
    SourceRef, Transform2,
};
use scenotime::{Revision, SceneEpoch, SceneSnapshot};
use serde::{Deserialize, Serialize};

/// The provider seam a host builds a lane from, re-exported so a host
/// composes the lane without naming esp.
pub use esp::infer::{InferenceProvider, StubInferenceProvider};

/// The first-party route an application asks through.
pub const INFERENCE_ASK_ROUTE: &str = "inference.ask";

/// How often a session is told the run table moved. Streamed fragments
/// reach a client at most this often (plan R5).
pub const INFERENCE_ASK_NOTICE_POLL: Duration = Duration::from_millis(100);

/// Schema of the run table served as the card's media.
pub const INFERENCE_RUNS_SCHEMA: &str = "distillery.inference-runs/v1";

/// Start a run. Payload: [`AskRequestV1`] as JSON.
pub const ASK_INTENT: &str = "distillery.inference/ask-v1";

/// Cancel a run. Payload: [`CancelRequestV1`] as JSON.
pub const CANCEL_INTENT: &str = "distillery.inference/cancel-v1";

/// The `Ask` intent's payload. The caller names the run, so it can find the
/// run in the table without a reply channel: an intent result carries no data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskRequestV1 {
    /// Caller-chosen run name, unique within this lane.
    pub run: String,
    /// The question, as the prompt the provider receives.
    pub question: String,
}

/// The `Cancel` intent's payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CancelRequestV1 {
    /// The run to cancel.
    pub run: String,
}

/// Where a run stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStateV1 {
    /// Accepted, waiting behind earlier runs.
    Queued,
    /// Generating; `text` grows.
    Running,
    /// Complete; `text` is the whole answer.
    Finished,
    /// Stopped by a cancel; `text` is what was generated before it.
    Cancelled,
    /// The provider refused or failed; see `error`.
    Failed,
}

impl RunStateV1 {
    /// Whether the run will change no further.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Finished | Self::Cancelled | Self::Failed)
    }
}

/// One run, as the route serves it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunV1 {
    /// The caller's run name.
    pub run: String,
    /// The question asked.
    pub question: String,
    /// Where the run stands.
    pub state: RunStateV1,
    /// The answer so far, or in full once finished.
    pub text: String,
    /// Why a failed run failed.
    pub error: Option<String>,
}

/// The run table, as the route serves it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceRunsV1 {
    /// [`INFERENCE_RUNS_SCHEMA`].
    pub schema: String,
    /// The provider's model id, once the actor has built it.
    pub model: Option<String>,
    /// Runs in the order they were asked.
    pub runs: Vec<RunV1>,
}

/// Why the lane would not take an ask or a cancel.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InferenceLaneError {
    /// The run name is already in the table.
    #[error("run {0:?} already exists")]
    DuplicateRun(String),
    /// No run by that name.
    #[error("no run {0:?}")]
    UnknownRun(String),
    /// The run name or question is empty.
    #[error("{0} is empty")]
    Empty(&'static str),
    /// The lane has closed.
    #[error("the inference lane is closed")]
    Closed,
}

struct RunRecord {
    view: RunV1,
    actor_id: u64,
}

#[derive(Default)]
struct RunTable {
    model: Option<String>,
    runs: Vec<RunRecord>,
    by_actor_id: HashMap<u64, usize>,
    next_actor_id: u64,
    /// Bumped on every change a client can see.
    generation: u64,
}

impl RunTable {
    fn view(&self) -> InferenceRunsV1 {
        InferenceRunsV1 {
            schema: INFERENCE_RUNS_SCHEMA.into(),
            model: self.model.clone(),
            runs: self.runs.iter().map(|record| record.view.clone()).collect(),
        }
    }

    fn by_actor(&mut self, id: u64) -> Option<&mut RunV1> {
        let index = *self.by_actor_id.get(&id)?;
        Some(&mut self.runs[index].view)
    }

    /// Fold one actor update. Updates for a run that has already ended are
    /// dropped, so nothing reaches a cancelled run after its cancel lands.
    fn apply(&mut self, update: InferUpdate) {
        match update {
            InferUpdate::Ready { capability } => self.model = Some(capability.model_id),
            InferUpdate::Started { id } => {
                if let Some(run) = self.by_actor(id).filter(|run| !run.state.is_terminal()) {
                    run.state = RunStateV1::Running;
                }
            },
            InferUpdate::Fragment { id, text } => {
                if let Some(run) = self.by_actor(id).filter(|run| !run.state.is_terminal()) {
                    run.text.push_str(&text);
                }
            },
            InferUpdate::Finished { id, text } => {
                if let Some(run) = self.by_actor(id).filter(|run| !run.state.is_terminal()) {
                    run.state = RunStateV1::Finished;
                    run.text = text;
                }
            },
            InferUpdate::Cancelled { id } => {
                if let Some(run) = self.by_actor(id).filter(|run| !run.state.is_terminal()) {
                    run.state = RunStateV1::Cancelled;
                }
            },
            InferUpdate::Failed { id, error } => {
                if let Some(run) = self.by_actor(id).filter(|run| !run.state.is_terminal()) {
                    run.state = RunStateV1::Failed;
                    run.error = Some(error.to_string());
                }
            },
        }
        self.generation += 1;
    }
}

struct Shared {
    table: Mutex<RunTable>,
    actor: Mutex<Option<ActorHandle<InferCommand>>>,
    bridge: Mutex<Option<JoinHandle<()>>>,
}

impl Shared {
    fn table(&self) -> MutexGuard<'_, RunTable> {
        self.table.lock().expect("the run table is never poisoned")
    }
}

/// One inference actor and the runs asked of it. Cheap to clone; every clone
/// is the same lane.
#[derive(Clone)]
pub struct InferenceLane {
    shared: Arc<Shared>,
}

impl InferenceLane {
    /// Start the actor. `build` runs on the actor's own thread and constructs
    /// or loads the provider there.
    pub fn open<F>(build: F) -> Self
    where
        F: FnOnce() -> Box<dyn InferenceProvider> + Send + 'static,
    {
        // The bridge thread blocks on the update channel, so the actor's
        // wake has nothing to do.
        let wake: Wake = Arc::new(|| {});
        let (actor, updates) = spawn_inference_actor(wake, build);
        let shared = Arc::new(Shared {
            table: Mutex::new(RunTable::default()),
            actor: Mutex::new(Some(actor)),
            bridge: Mutex::new(None),
        });
        let bridge = std::thread::Builder::new()
            .name("inference-bridge".into())
            .spawn({
                let shared = Arc::clone(&shared);
                move || bridge(&shared, &updates)
            })
            .expect("spawn the inference bridge thread");
        *shared
            .bridge
            .lock()
            .expect("the bridge slot is never poisoned") = Some(bridge);
        Self { shared }
    }

    /// Ask a question as run `run`.
    pub fn ask(&self, request: AskRequestV1) -> Result<(), InferenceLaneError> {
        if request.run.is_empty() {
            return Err(InferenceLaneError::Empty("run"));
        }
        if request.question.is_empty() {
            return Err(InferenceLaneError::Empty("question"));
        }
        let actor = self
            .shared
            .actor
            .lock()
            .expect("the actor slot is never poisoned");
        let actor = actor.as_ref().ok_or(InferenceLaneError::Closed)?;
        let mut table = self.shared.table();
        if table
            .runs
            .iter()
            .any(|record| record.view.run == request.run)
        {
            return Err(InferenceLaneError::DuplicateRun(request.run));
        }
        let actor_id = table.next_actor_id;
        table.next_actor_id += 1;
        let generation = GenerationRequest {
            prompt: request.question.clone(),
            ..GenerationRequest::default()
        };
        if !actor.command(InferCommand::Generate {
            id: actor_id,
            request: generation,
        }) {
            return Err(InferenceLaneError::Closed);
        }
        let index = table.runs.len();
        table.runs.push(RunRecord {
            view: RunV1 {
                run: request.run,
                question: request.question,
                state: RunStateV1::Queued,
                text: String::new(),
                error: None,
            },
            actor_id,
        });
        table.by_actor_id.insert(actor_id, index);
        table.generation += 1;
        Ok(())
    }

    /// Cancel run `run`. Cancelling a run that has already ended is accepted
    /// and changes nothing.
    pub fn cancel(&self, request: &CancelRequestV1) -> Result<(), InferenceLaneError> {
        let actor = self
            .shared
            .actor
            .lock()
            .expect("the actor slot is never poisoned");
        let actor = actor.as_ref().ok_or(InferenceLaneError::Closed)?;
        let table = self.shared.table();
        let record = table
            .runs
            .iter()
            .find(|record| record.view.run == request.run)
            .ok_or_else(|| InferenceLaneError::UnknownRun(request.run.clone()))?;
        if !record.view.state.is_terminal() {
            actor.command(InferCommand::Cancel {
                id: record.actor_id,
            });
        }
        Ok(())
    }

    /// The run table now.
    pub fn runs(&self) -> InferenceRunsV1 {
        self.shared.table().view()
    }

    /// The table's change counter.
    pub fn generation(&self) -> u64 {
        self.shared.table().generation
    }

    /// Cancel every unfinished run, stop the actor and wait for both of the
    /// lane's threads. Idempotent.
    pub fn close(&self) -> Result<(), String> {
        let actor = self
            .shared
            .actor
            .lock()
            .expect("the actor slot is never poisoned")
            .take();
        if let Some(actor) = actor {
            // Dropping the handle alone would let queued runs drain first.
            let open: Vec<u64> = self
                .shared
                .table()
                .runs
                .iter()
                .filter(|record| !record.view.state.is_terminal())
                .map(|record| record.actor_id)
                .collect();
            for id in open {
                actor.command(InferCommand::Cancel { id });
            }
            actor.join();
        }
        let bridge = self
            .shared
            .bridge
            .lock()
            .expect("the bridge slot is never poisoned")
            .take();
        if let Some(bridge) = bridge {
            bridge
                .join()
                .map_err(|_| "the inference bridge thread panicked".to_string())?;
        }
        Ok(())
    }

    /// The route endpoint for one admitted session.
    pub fn endpoint(&self, session: ProjectionSession) -> InferenceAskEndpoint {
        InferenceAskEndpoint {
            lane: self.clone(),
            session,
            noticed: None,
            current: None,
        }
    }
}

/// Fold updates until the actor is gone and its channel closes.
fn bridge(shared: &Shared, updates: &Receiver<InferUpdate>) {
    while let Ok(update) = updates.recv() {
        shared.table().apply(update);
    }
}

/// The `inference.ask` endpoint for one session.
pub struct InferenceAskEndpoint {
    lane: InferenceLane,
    session: ProjectionSession,
    noticed: Option<u64>,
    /// The bytes the last snapshot named, so its resources stay servable
    /// while the table moves on.
    current: Option<(Vec<u8>, Vec<u8>)>,
}

const CARD_LABEL: &str = "Inference runs";

fn score() -> Score {
    Score::new(Arrangement::Spiral(Default::default()))
}

impl InferenceAskEndpoint {
    fn own_session(&self, session: &ProjectionSession, what: &str) -> Result<(), String> {
        if *session == self.session {
            Ok(())
        } else {
            Err(format!("inference.ask {what} names another session"))
        }
    }

    fn card_snapshot(&self, generation: u64, card_bytes: &[u8]) -> ProjectionSnapshot {
        let mut scene = Scene::new();
        let source = scene.intern_source(SourceRef::new(self.session.0.as_str(), "runs"));
        scene.items.push(ProjectedItem {
            source,
            space: Scene::WORLD,
            transform: Transform2::IDENTITY,
            footprint: Footprint::Rect {
                size: Size2::new(1.0, 1.0),
            },
            representation: Representation::Card,
            layer: 0,
            visible: false,
            hit: None,
            channels: Vec::new(),
        });
        let key = PresentationKey(INFERENCE_RUNS_SCHEMA.into());
        let mut presentation = PresentationManifest::default();
        presentation.bindings.push(PresentationBinding {
            instance: InstanceId(0),
            key: key.clone(),
        });
        presentation.offers.insert(
            key,
            vec![PresentationOffer {
                codec: PresentationCodec::PortableCardV1,
                resource: ContentHash::of(card_bytes),
                byte_size: card_bytes.len() as u64,
                requires: PresentationCapability::PortableCard,
                semantics: PresentationSemantics {
                    label: CARD_LABEL.into(),
                    role: SemanticRole::Graphic,
                    bounds: BoundsRelationship::FitWithinFootprint,
                    actions: Vec::new(),
                },
            }],
        );
        ProjectionSnapshot {
            version: ProtocolVersion::V1,
            session: self.session.clone(),
            scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(generation), scene)
                .expect("a one-item scene is dense"),
            presentation,
            cache_policy: CachePolicy::default(),
        }
    }
}

impl ProjectionCatalog for InferenceAskEndpoint {
    fn describe(&self) -> EndpointDescriptor {
        EndpointDescriptor {
            label: "Resident inference".into(),
            projections: vec![ProjectionOffer {
                label: CARD_LABEL.into(),
                request: ProjectionRequest {
                    version: ProtocolVersion::V1,
                    session: self.session.clone(),
                    score: score(),
                },
            }],
        }
    }
}

impl ProjectionSource for InferenceAskEndpoint {
    type Error = String;

    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, Self::Error> {
        self.own_session(&request.session, "snapshot")?;
        let (runs, generation) = {
            let table = self.lane.shared.table();
            (table.view(), table.generation)
        };
        let runs_bytes = serde_json::to_vec(&runs).map_err(|error| error.to_string())?;
        let finished = runs
            .runs
            .iter()
            .filter(|run| run.state.is_terminal())
            .count();
        let value = |label: &str, value: String| CardValueV1 {
            label: label.into(),
            value,
        };
        let card = PortableCardV1 {
            title: CARD_LABEL.into(),
            values: vec![
                value("model", runs.model.clone().unwrap_or_default()),
                value("runs", runs.runs.len().to_string()),
                value("ended", finished.to_string()),
            ],
            badges: Vec::new(),
            media: vec![ContentHash::of(&runs_bytes)],
        };
        let card_bytes = serde_json::to_vec(&card).map_err(|error| error.to_string())?;
        let snapshot = self.card_snapshot(generation, &card_bytes);
        self.noticed = Some(generation);
        self.current = Some((card_bytes, runs_bytes));
        Ok(snapshot)
    }
}

impl PresentationSource for InferenceAskEndpoint {
    type Error = String;

    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, Self::Error> {
        self.own_session(&request.session, "resource")?;
        let Some((card, runs)) = self.current.as_ref() else {
            return Err("request an inference.ask snapshot first".into());
        };
        let bytes = [card, runs]
            .into_iter()
            .find(|bytes| ContentHash::of(bytes) == request.resource)
            .ok_or("that resource is not in the current inference.ask snapshot")?
            .clone();
        Ok(ResourceResponse {
            session: request.session,
            resource: request.resource,
            bytes,
        })
    }
}

impl IntentSink for InferenceAskEndpoint {
    type Error = String;

    fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, Self::Error> {
        self.own_session(&intent.session, "intent")?;
        let rejected = |reason: String| Ok(IntentResult::Rejected { reason });
        let outcome = match intent.intent.as_str() {
            ASK_INTENT => match serde_json::from_slice::<AskRequestV1>(&intent.payload) {
                Ok(ask) => self.lane.ask(ask),
                Err(error) => return rejected(format!("{ASK_INTENT} payload: {error}")),
            },
            CANCEL_INTENT => match serde_json::from_slice::<CancelRequestV1>(&intent.payload) {
                Ok(cancel) => self.lane.cancel(&cancel),
                Err(error) => return rejected(format!("{CANCEL_INTENT} payload: {error}")),
            },
            other => {
                return rejected(format!(
                    "inference.ask takes {ASK_INTENT} or {CANCEL_INTENT}, not {other:?}"
                ));
            },
        };
        match outcome {
            Ok(()) => Ok(IntentResult::Accepted),
            Err(error) => rejected(error.to_string()),
        }
    }
}

impl ProjectionNoticeSource for InferenceAskEndpoint {
    type Error = String;

    fn poll_notice(&mut self) -> Result<Option<CarrierNotice>, Self::Error> {
        let generation = self.lane.generation();
        if self.noticed == Some(generation) {
            return Ok(None);
        }
        self.noticed = Some(generation);
        Ok(Some(CarrierNotice {
            session: self.session.clone(),
            epoch: SceneEpoch(1),
            revision: Revision(generation),
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::ops::ControlFlow;
    use std::sync::mpsc::{SyncSender, sync_channel};
    use std::time::Instant;

    use esp::infer::{InferError, ModelCapability, StubInferenceProvider};

    use super::*;

    fn session() -> ProjectionSession {
        ProjectionSession("test-session".into())
    }

    fn ask(run: &str, question: &str) -> AskRequestV1 {
        AskRequestV1 {
            run: run.into(),
            question: question.into(),
        }
    }

    fn run<'a>(runs: &'a InferenceRunsV1, name: &str) -> &'a RunV1 {
        runs.runs
            .iter()
            .find(|run| run.run == name)
            .expect("the run is in the table")
    }

    /// Poll until `done` holds or a generous deadline passes.
    fn wait_for(lane: &InferenceLane, done: impl Fn(&InferenceRunsV1) -> bool) -> InferenceRunsV1 {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let runs = lane.runs();
            if done(&runs) {
                return runs;
            }
            assert!(Instant::now() < deadline, "timed out; table: {runs:?}");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// Emits one fragment each time the test lets it, so a cancel can land
    /// mid-stream deterministically.
    struct GatedProvider {
        capability: ModelCapability,
        gate: Mutex<Receiver<()>>,
        emitted: SyncSender<usize>,
    }

    impl InferenceProvider for GatedProvider {
        fn capability(&self) -> &ModelCapability {
            &self.capability
        }

        fn generate_streaming(
            &self,
            _request: &GenerationRequest,
            on_token: &mut dyn FnMut(&str) -> ControlFlow<()>,
        ) -> Result<String, InferError> {
            let gate = self.gate.lock().unwrap();
            let mut out = String::new();
            for i in 0..100 {
                if gate.recv().is_err() {
                    break;
                }
                let fragment = format!("t{i} ");
                if on_token(&fragment).is_break() {
                    break;
                }
                out.push_str(&fragment);
                let _ = self.emitted.send(i);
            }
            Ok(out)
        }
    }

    #[test]
    fn a_stub_ask_streams_to_a_finished_answer() {
        let lane = InferenceLane::open(|| {
            Box::new(StubInferenceProvider::new().with_response("why?", "because it is so"))
        });
        lane.ask(ask("a", "why?")).unwrap();
        let runs = wait_for(&lane, |runs| {
            runs.runs.first().is_some_and(|run| run.state.is_terminal())
        });
        let a = run(&runs, "a");
        assert_eq!(a.state, RunStateV1::Finished);
        assert_eq!(a.text, "because it is so");
        assert_eq!(runs.model.as_deref(), Some("stub/echo"));
        lane.close().unwrap();
    }

    #[test]
    fn a_cancel_mid_stream_keeps_partial_text_and_nothing_after() {
        let (open_gate, gate) = sync_channel::<()>(0);
        let (emitted, fragments) = sync_channel::<usize>(100);
        let lane = InferenceLane::open(move || {
            Box::new(GatedProvider {
                capability: StubInferenceProvider::new().capability().clone(),
                gate: Mutex::new(gate),
                emitted,
            })
        });
        lane.ask(ask("a", "go")).unwrap();
        for expected in 0..3 {
            open_gate.send(()).unwrap();
            assert_eq!(fragments.recv().unwrap(), expected);
        }
        let partial = wait_for(&lane, |runs| run(runs, "a").text == "t0 t1 t2 ");
        assert_eq!(run(&partial, "a").state, RunStateV1::Running);

        lane.cancel(&CancelRequestV1 { run: "a".into() }).unwrap();
        // The provider asks for one more fragment; the actor sees the cancel
        // there and drops it.
        open_gate.send(()).unwrap();
        let runs = wait_for(&lane, |runs| run(runs, "a").state.is_terminal());
        let a = run(&runs, "a");
        assert_eq!(a.state, RunStateV1::Cancelled);
        assert_eq!(a.text, "t0 t1 t2 ");

        // Nothing reaches the run after its cancel.
        drop(open_gate);
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(run(&lane.runs(), "a").text, "t0 t1 t2 ");
        lane.close().unwrap();
    }

    #[test]
    fn duplicate_and_unknown_runs_are_refused() {
        let lane = InferenceLane::open(|| Box::new(StubInferenceProvider::new()));
        lane.ask(ask("a", "one")).unwrap();
        assert_eq!(
            lane.ask(ask("a", "two")),
            Err(InferenceLaneError::DuplicateRun("a".into()))
        );
        assert_eq!(
            lane.cancel(&CancelRequestV1 { run: "b".into() }),
            Err(InferenceLaneError::UnknownRun("b".into()))
        );
        assert_eq!(
            lane.ask(ask("", "q")),
            Err(InferenceLaneError::Empty("run"))
        );
        lane.close().unwrap();
    }

    #[test]
    fn close_cancels_queued_runs_and_refuses_later_asks() {
        let (open_gate, gate) = sync_channel::<()>(0);
        let (emitted, _fragments) = sync_channel::<usize>(100);
        let lane = InferenceLane::open(move || {
            Box::new(GatedProvider {
                capability: StubInferenceProvider::new().capability().clone(),
                gate: Mutex::new(gate),
                emitted,
            })
        });
        lane.ask(ask("a", "first")).unwrap();
        lane.ask(ask("b", "second")).unwrap();
        wait_for(&lane, |runs| run(runs, "a").state == RunStateV1::Running);
        // Unblock the provider once close has sent its cancels, so the actor
        // sees them at the next fragment rather than finishing the run.
        let feeder = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(200));
            while open_gate.send(()).is_ok() {}
        });
        lane.close().unwrap();
        feeder.join().unwrap();
        let runs = lane.runs();
        assert_eq!(run(&runs, "a").state, RunStateV1::Cancelled);
        assert_eq!(run(&runs, "b").state, RunStateV1::Cancelled);
        assert_eq!(lane.ask(ask("c", "late")), Err(InferenceLaneError::Closed));
        lane.close().unwrap();
    }

    fn invoke(
        endpoint: &mut InferenceAskEndpoint,
        intent: &str,
        payload: &impl Serialize,
    ) -> IntentResult {
        endpoint
            .invoke(IntentInvocation {
                session: session(),
                target: InstanceId(0),
                observed_epoch: SceneEpoch(1),
                observed_revision: Revision(0),
                intent: intent.into(),
                payload: serde_json::to_vec(payload).unwrap(),
            })
            .unwrap()
    }

    fn served_runs(endpoint: &mut InferenceAskEndpoint) -> InferenceRunsV1 {
        let request = endpoint.describe().projections[0].request.clone();
        let snapshot = endpoint.snapshot(request).unwrap();
        let offer = snapshot.presentation.offers.values().next().unwrap()[0].clone();
        let card_bytes = endpoint
            .resource(ResourceRequest {
                session: session(),
                resource: offer.resource,
            })
            .unwrap()
            .bytes;
        let card: PortableCardV1 = serde_json::from_slice(&card_bytes).unwrap();
        let runs_bytes = endpoint
            .resource(ResourceRequest {
                session: session(),
                resource: card.media[0],
            })
            .unwrap()
            .bytes;
        serde_json::from_slice(&runs_bytes).unwrap()
    }

    #[test]
    fn the_route_asks_streams_and_rings() {
        let lane = InferenceLane::open(|| {
            Box::new(StubInferenceProvider::new().with_response("why?", "because"))
        });
        let mut endpoint = lane.endpoint(session());
        // Let the actor's Ready land first, so the notices below are ours.
        wait_for(&lane, |runs| runs.model.is_some());
        assert!(endpoint.poll_notice().unwrap().is_some());
        let _ = served_runs(&mut endpoint);
        assert!(endpoint.poll_notice().unwrap().is_none());

        assert_eq!(
            invoke(&mut endpoint, ASK_INTENT, &ask("a", "why?")),
            IntentResult::Accepted
        );
        wait_for(&lane, |runs| run(runs, "a").state.is_terminal());
        let notice = endpoint.poll_notice().unwrap().expect("the table moved");
        assert_eq!(notice.session, session());
        let runs = served_runs(&mut endpoint);
        assert_eq!(runs.schema, INFERENCE_RUNS_SCHEMA);
        assert_eq!(run(&runs, "a").text, "because");
        assert_eq!(run(&runs, "a").state, RunStateV1::Finished);
        assert!(endpoint.poll_notice().unwrap().is_none());

        assert!(matches!(
            invoke(&mut endpoint, ASK_INTENT, &ask("a", "again")),
            IntentResult::Rejected { .. }
        ));
        assert!(matches!(
            invoke(&mut endpoint, "something-else", &ask("x", "y")),
            IntentResult::Rejected { .. }
        ));
        assert!(matches!(
            invoke(&mut endpoint, CANCEL_INTENT, &serde_json::json!({"run": 3})),
            IntentResult::Rejected { .. }
        ));
        lane.close().unwrap();
    }

    #[test]
    fn another_sessions_requests_are_errors() {
        let lane = InferenceLane::open(|| Box::new(StubInferenceProvider::new()));
        let mut endpoint = lane.endpoint(session());
        let request = ProjectionRequest {
            version: ProtocolVersion::V1,
            session: ProjectionSession("other".into()),
            score: score(),
        };
        assert!(endpoint.snapshot(request).is_err());
        lane.close().unwrap();
    }
}
