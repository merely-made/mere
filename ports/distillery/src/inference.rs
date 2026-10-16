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
//! `inference-model` adds local model-library resolution and a session-bound
//! declared template. A persona journal adds token checkpoints and lock pause/
//! resume (I3); recall and citations remain I4.

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
use esp::infer::actor::spawn_inference_actor_with_checkpoints;
use esp::infer::provider::GenerationCheckpoint;
use esp::infer::{GenerationRequest, InferCommand, InferUpdate};
#[cfg(feature = "inference-model")]
pub mod durable;
use graphshell_endpoint::{
    IntentSink, PresentationSource, ProjectionCatalog, ProjectionNoticeSource, ProjectionSource,
};
use sceno::{
    Arrangement, Footprint, InstanceId, ProjectedItem, Representation, Scene, Score, Size2,
    SourceRef, Transform2,
};
use scenotime::{Revision, SceneEpoch, SceneSnapshot};
use serde::{Deserialize, Serialize};

#[cfg(feature = "inference-gpu")]
mod gpu;
#[cfg(feature = "inference-gpu")]
pub use gpu::{
    discrete_gpu_inference_device, open_gpu_library_lane, open_gpu_library_lane_with_journal,
};

#[cfg(feature = "inference-model")]
pub use esp::infer::decoder::DecoderDevice as InferenceDevice;
/// The provider seam a host builds a lane from, re-exported so a host
/// composes the lane without naming esp.
pub use esp::infer::{InferenceProvider, StubInferenceProvider};
#[cfg(feature = "inference-model")]
pub use esp::infer::{
    library::{InferenceModelRow, load_library_session},
    prompt::PromptTemplate,
};

/// Parse the self-describing content reference from owner settings.
#[cfg(feature = "inference-model")]
pub fn parse_model_row(row: &str) -> Result<eidetic::ManifestId, String> {
    eidetic::Hash::parse(row)
        .map(eidetic::ManifestId::from_hash)
        .map_err(|error| format!("inference model row {row:?}: {error}"))
}

/// The library lock covers resolution only. Generation never holds it, so a
/// trainer can publish another artifact while an ask is running on the CPU.
#[cfg(feature = "inference-model")]
pub fn open_library_lane<B: eidetic::Store + Send + 'static>(
    library: Arc<tokio::sync::Mutex<B>>,
    row: eidetic::ManifestId,
    max_tokens: usize,
) -> InferenceLane {
    open_library_lane_with_journal(library, row, max_tokens, None).expect("ephemeral library lane")
}

#[cfg(feature = "inference-model")]
pub fn open_library_lane_with_journal<B: eidetic::Store + Send + 'static>(
    library: Arc<tokio::sync::Mutex<B>>,
    row: eidetic::ManifestId,
    max_tokens: usize,
    journal: Option<Arc<dyn RunJournal>>,
) -> Result<InferenceLane, String> {
    InferenceLane::try_open_with_journal(
        max_tokens,
        move || {
            let resolved = {
                let mut store = library.blocking_lock();
                pollster::block_on(esp::infer::library::resolve_library_session(
                    &mut *store,
                    row,
                ))?
            };
            let provider =
                resolved.load(&InferenceDevice::ndarray(), "burn-ndarray", max_tokens)?;
            Ok(Box::new(provider))
        },
        journal,
    )
}

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
/// Pause a durable run, retaining its token checkpoint.
pub const PAUSE_INTENT: &str = "distillery.inference/pause-v1";
/// Resume a retained run with its recorded generation settings.
pub const RESUME_INTENT: &str = "distillery.inference/resume-v1";
/// Ask with explicit sampling settings, still bounded by the owner cap.
pub const ASK_SAMPLED_INTENT: &str = "distillery.inference/ask-sampled-v1";

/// Sampling controls accepted by [`ASK_SAMPLED_INTENT`].
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SampledAskRequestV1 {
    pub ask: AskRequestV1,
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: Option<f32>,
    pub seed: Option<u64>,
}

/// The `Ask` intent's payload. The caller names the run, so it can find the
/// run in the table without a reply channel: an intent result carries no data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskRequestV1 {
    /// Caller-chosen run name, unique within this lane.
    pub run: String,
    /// The raw question; a library provider renders its declared template.
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
    /// Stopped at a durable token boundary.
    Paused,
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
    /// Exact session used for this run, absent for a stub or failed load.
    pub session: Option<esp::infer::provider::SessionIdentity>,
    #[serde(default)]
    pub tokens: usize,
    #[serde(default)]
    pub continuations: Vec<Continuation>,
}

/// The run table, as the route serves it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceRunsV1 {
    /// [`INFERENCE_RUNS_SCHEMA`].
    pub schema: String,
    /// The provider's model id, once the actor has built it.
    pub model: Option<String>,
    /// Exact loaded session and template, absent for an unbound stub.
    pub session: Option<esp::infer::provider::SessionIdentity>,
    /// Provider construction error; asks fail with this instead of falling back.
    pub load_error: Option<String>,
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
    #[error("the inference route is locked")]
    Locked,
    #[error("inference journal: {0}")]
    Journal(String),
}

/// Retained host state. Journals serialize this beside Eidetic evidence;
/// execution IDs and in-flight flags are local materialization details.
#[derive(Clone, Serialize, Deserialize)]
pub struct RunRecord {
    view: RunV1,
    actor_id: u64,
    request: GenerationRequest,
    checkpoint: Option<GenerationCheckpoint>,
    #[serde(default)]
    continuations: Vec<Continuation>,
    #[serde(default)]
    pause_reason: Option<String>,
    #[serde(skip)]
    executing: bool,
}

#[cfg(feature = "inference-model")]
impl Drop for RunRecord {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.view.question.zeroize();
        self.view.text.zeroize();
        self.request.clear_private();
    }
}

/// One actual visit to a device, recorded before continuation starts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Continuation {
    pub device: String,
    pub at_token: usize,
    pub reason: String,
    pub first_differing_token: Option<usize>,
    #[serde(default)]
    pub compared_token_ids_hash: Option<String>,
}

/// Synchronous durable barrier supplied by the persona's log owner.
pub trait RunJournal: Send + Sync {
    fn device(&self) -> &str;
    fn load(&self) -> Result<Vec<RunRecord>, String>;
    fn append(&self, record: &RunRecord) -> Result<(), String>;
    /// Historical reads can remain available after execution authority ends.
    fn execution_permitted(&self) -> bool {
        true
    }
    fn remote_revision(&self) -> u64 {
        0
    }
}

#[derive(Default)]
struct RunTable {
    model: Option<String>,
    session: Option<esp::infer::provider::SessionIdentity>,
    load_error: Option<String>,
    runs: Vec<RunRecord>,
    by_actor_id: HashMap<u64, usize>,
    next_actor_id: u64,
    locked: bool,
    seen_remote_revision: u64,
    /// Bumped on every change a client can see.
    generation: u64,
}

impl RunTable {
    fn view(&self) -> InferenceRunsV1 {
        InferenceRunsV1 {
            schema: INFERENCE_RUNS_SCHEMA.into(),
            model: self.model.clone(),
            session: self.session.clone(),
            load_error: self.load_error.clone(),
            runs: self
                .runs
                .iter()
                .map(|record| {
                    let mut view = record.view.clone();
                    view.tokens = record
                        .checkpoint
                        .as_ref()
                        .map_or(0, |cp| cp.token_ids.len());
                    view.continuations = record.continuations.clone();
                    view
                })
                .collect(),
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
            InferUpdate::Session { identity } => {
                for record in &mut self.runs {
                    if record.view.session.is_none()
                        && matches!(record.view.state, RunStateV1::Queued | RunStateV1::Running)
                    {
                        record.view.session = Some(identity.clone());
                    }
                }
                self.session = Some(identity);
            },
            InferUpdate::LoadFailed { error } => self.load_error = Some(error.to_string()),
            InferUpdate::Started { id } => {
                if let Some(index) = self.by_actor_id.get(&id).copied() {
                    self.runs[index].executing = true;
                }
                if let Some(run) = self.by_actor(id).filter(|run| !run.state.is_terminal()) {
                    run.state = RunStateV1::Running;
                }
            },
            InferUpdate::Paused { id } => {
                if let Some(index) = self.by_actor_id.get(&id).copied() {
                    self.runs[index].executing = false;
                }
                if let Some(run) = self.by_actor(id).filter(|run| !run.state.is_terminal()) {
                    run.state = RunStateV1::Paused;
                }
            },
            // The synchronous commit barrier already published this state.
            // Replaying an earlier queued update would regress its token count.
            InferUpdate::Checkpoint { .. } => {},
            InferUpdate::Fragment { id, text } => {
                if let Some(run) = self.by_actor(id).filter(|run| !run.state.is_terminal()) {
                    run.text.push_str(&text);
                }
            },
            InferUpdate::Finished { id, text } => {
                if let Some(index) = self.by_actor_id.get(&id).copied() {
                    self.runs[index].executing = false;
                }
                if let Some(run) = self.by_actor(id).filter(|run| !run.state.is_terminal()) {
                    run.state = RunStateV1::Finished;
                    run.text = text;
                }
            },
            InferUpdate::Cancelled { id } => {
                if let Some(index) = self.by_actor_id.get(&id).copied() {
                    self.runs[index].executing = false;
                }
                if let Some(run) = self.by_actor(id).filter(|run| !run.state.is_terminal()) {
                    run.state = RunStateV1::Cancelled;
                }
            },
            InferUpdate::Failed { id, error } => {
                if let Some(index) = self.by_actor_id.get(&id).copied() {
                    self.runs[index].executing = false;
                }
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
    table: Arc<Mutex<RunTable>>,
    journal: Option<Arc<dyn RunJournal>>,
    actor: Mutex<Option<ActorHandle<InferCommand>>>,
    bridge: Mutex<Option<JoinHandle<()>>>,
    max_tokens: usize,
    caches: Mutex<Vec<std::sync::Weak<Mutex<Option<(Vec<u8>, Vec<u8>)>>>>>,
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
        Self::try_open(move || Ok(build()))
    }

    /// Start a model-backed actor; load failures remain visible on the route.
    pub fn try_open<F>(build: F) -> Self
    where
        F: FnOnce() -> Result<Box<dyn InferenceProvider>, esp::infer::InferError> + Send + 'static,
    {
        Self::try_open_with_max_tokens(GenerationRequest::default().max_tokens, build)
    }

    fn try_open_with_max_tokens<F>(max_tokens: usize, build: F) -> Self
    where
        F: FnOnce() -> Result<Box<dyn InferenceProvider>, esp::infer::InferError> + Send + 'static,
    {
        Self::try_open_with_journal(max_tokens, build, None)
            .expect("an ephemeral lane has no journal to fail")
    }

    /// Open retained runs before admitting work; local interrupted asks resume.
    pub fn try_open_with_journal<F>(
        max_tokens: usize,
        build: F,
        journal: Option<Arc<dyn RunJournal>>,
    ) -> Result<Self, String>
    where
        F: FnOnce() -> Result<Box<dyn InferenceProvider>, esp::infer::InferError> + Send + 'static,
    {
        let mut table = RunTable::default();
        let mut recover = Vec::new();
        if let Some(journal) = &journal {
            for mut record in journal.load()? {
                record.actor_id = table.next_actor_id;
                table.next_actor_id += 1;
                let local = record
                    .continuations
                    .last()
                    .is_some_and(|visit| visit.device == journal.device());
                if !record.view.state.is_terminal() {
                    if local
                        && journal.execution_permitted()
                        && (record.view.state != RunStateV1::Paused
                            || record.pause_reason.as_deref() == Some("shutdown")
                            || record.pause_reason.as_deref() == Some("lock"))
                    {
                        recover.push(record.actor_id);
                    }
                    record.view.state = RunStateV1::Paused;
                }
                let index = table.runs.len();
                table.by_actor_id.insert(record.actor_id, index);
                table.runs.push(record);
            }
        }
        let table = Arc::new(Mutex::new(table));
        let commit_table = table.clone();
        let commit_journal = journal.clone();
        let wake: Wake = Arc::new(|| {});
        let (actor, updates) =
            spawn_inference_actor_with_checkpoints(wake, build, move |id, checkpoint| {
                if commit_journal.is_some() && checkpoint.session.is_none() {
                    return Err(esp::infer::InferError::InvalidRequest(
                        "durable asks require an exact bound model session".into(),
                    ));
                }
                let mut table = commit_table.lock().unwrap();
                let index = *table
                    .by_actor_id
                    .get(&id)
                    .ok_or_else(|| esp::infer::InferError::Backend("missing durable run".into()))?;
                let record = &mut table.runs[index];
                let mut next = record.clone();
                next.checkpoint = Some(checkpoint.clone());
                next.view.text = checkpoint.text.clone();
                next.view.session = checkpoint.session.clone();
                // A GPU provider still has to synchronize and release its
                // guard after its final token. The actor ends the run.
                next.view.state = RunStateV1::Running;
                if let Some(journal) = &commit_journal {
                    journal
                        .append(&next)
                        .map_err(esp::infer::InferError::Backend)?;
                }
                *record = next;
                table.generation += 1;
                Ok(())
            });
        let shared = Arc::new(Shared {
            table,
            journal,
            actor: Mutex::new(Some(actor)),
            bridge: Mutex::new(None),
            max_tokens,
            caches: Mutex::new(Vec::new()),
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
        let lane = Self { shared };
        for id in recover {
            let name = lane
                .shared
                .table()
                .runs
                .iter()
                .find(|record| record.actor_id == id)
                .unwrap()
                .view
                .run
                .clone();
            if let Err(error) = lane.resume(&name, "restart") {
                let _ = lane.close();
                return Err(error.to_string());
            }
        }
        Ok(lane)
    }

    /// Ask a question as run `run`.
    pub fn ask(&self, request: AskRequestV1) -> Result<(), InferenceLaneError> {
        let generation = GenerationRequest {
            prompt: request.question.clone(),
            max_tokens: self.shared.max_tokens,
            ..GenerationRequest::default()
        };
        self.ask_with_generation(request, generation)
    }

    /// Sampling parameters are recorded with the ask; the owner still caps it.
    pub fn ask_with_generation(
        &self,
        request: AskRequestV1,
        mut generation: GenerationRequest,
    ) -> Result<(), InferenceLaneError> {
        generation.prompt = request.question.clone();
        generation.max_tokens = generation.max_tokens.min(self.shared.max_tokens);
        if generation.max_tokens == 0
            || !generation.temperature.is_finite()
            || generation.temperature < 0.0
            || generation
                .top_p
                .is_some_and(|p| !p.is_finite() || p <= 0.0 || p > 1.0)
        {
            return Err(InferenceLaneError::Journal(
                "invalid token limit, temperature or top-p".into(),
            ));
        }
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
        if table.locked {
            return Err(InferenceLaneError::Locked);
        }
        if table
            .runs
            .iter()
            .any(|record| record.view.run == request.run)
        {
            return Err(InferenceLaneError::DuplicateRun(request.run));
        }
        let actor_id = table.next_actor_id;
        table.next_actor_id += 1;
        let index = table.runs.len();
        let session = table.session.clone();
        table.runs.push(RunRecord {
            view: RunV1 {
                run: request.run,
                question: request.question,
                state: RunStateV1::Queued,
                text: String::new(),
                error: None,
                session,
                tokens: 0,
                continuations: Vec::new(),
            },
            actor_id,
            request: generation.clone(),
            checkpoint: None,
            continuations: self
                .shared
                .journal
                .as_ref()
                .map(|journal| {
                    vec![Continuation {
                        device: journal.device().into(),
                        at_token: 0,
                        reason: "ask".into(),
                        first_differing_token: None,
                        compared_token_ids_hash: None,
                    }]
                })
                .unwrap_or_default(),
            pause_reason: None,
            executing: false,
        });
        if let Some(journal) = &self.shared.journal {
            if let Err(error) = journal.append(&table.runs[index]) {
                table.runs.pop();
                return Err(InferenceLaneError::Journal(error));
            }
        }
        table.by_actor_id.insert(actor_id, index);
        let command = if self.shared.journal.is_some() {
            InferCommand::Checkpointed {
                id: actor_id,
                request: generation,
                resume: None,
            }
        } else {
            InferCommand::Generate {
                id: actor_id,
                request: generation,
            }
        };
        if !actor.command(command) {
            return Err(InferenceLaneError::Closed);
        }
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
        let mut table = self.shared.table();
        if table.locked {
            return Err(InferenceLaneError::Locked);
        }
        let record = table
            .runs
            .iter_mut()
            .find(|record| record.view.run == request.run)
            .ok_or_else(|| InferenceLaneError::UnknownRun(request.run.clone()))?;
        if record.view.state == RunStateV1::Paused {
            let mut next = record.clone();
            next.view.state = RunStateV1::Cancelled;
            if let Some(journal) = &self.shared.journal {
                journal.append(&next).map_err(InferenceLaneError::Journal)?;
            }
            *record = next;
            table.generation += 1;
        } else if !record.view.state.is_terminal() {
            actor.command(InferCommand::Cancel {
                id: record.actor_id,
            });
        }
        Ok(())
    }

    /// Pause a run without cancelling its retained answer.
    pub fn pause(&self, run: &str, reason: &str) -> Result<(), InferenceLaneError> {
        let actor = self.shared.actor.lock().unwrap();
        let actor = actor.as_ref().ok_or(InferenceLaneError::Closed)?;
        let mut table = self.shared.table();
        if table.locked && reason != "lock" && reason != "shutdown" {
            return Err(InferenceLaneError::Locked);
        }
        let record = table
            .runs
            .iter_mut()
            .find(|record| record.view.run == run)
            .ok_or_else(|| InferenceLaneError::UnknownRun(run.into()))?;
        if !record.view.state.is_terminal() {
            record.pause_reason = Some(reason.into());
            actor.command(InferCommand::Pause {
                id: record.actor_id,
            });
        }
        Ok(())
    }

    /// Continue a paused run from its exact retained token and draw position.
    pub fn resume(&self, run: &str, reason: &str) -> Result<(), InferenceLaneError> {
        let actor = self.shared.actor.lock().unwrap();
        let actor = actor.as_ref().ok_or(InferenceLaneError::Closed)?;
        let mut table = self.shared.table();
        if table.locked {
            return Err(InferenceLaneError::Locked);
        }
        let record = table
            .runs
            .iter_mut()
            .find(|record| record.view.run == run)
            .ok_or_else(|| InferenceLaneError::UnknownRun(run.into()))?;
        if record.view.state != RunStateV1::Paused {
            return Err(InferenceLaneError::Journal(
                "only paused runs can resume".into(),
            ));
        }
        let journal = self.shared.journal.as_ref().ok_or_else(|| {
            InferenceLaneError::Journal("this lane has no durable token journal".into())
        })?;
        let mut next = record.clone();
        next.continuations.push(Continuation {
            device: journal.device().into(),
            at_token: record
                .checkpoint
                .as_ref()
                .map_or(0, |cp| cp.token_ids.len()),
            reason: reason.into(),
            first_differing_token: None,
            compared_token_ids_hash: None,
        });
        next.view.state = RunStateV1::Queued;
        next.pause_reason = None;
        journal.append(&next).map_err(InferenceLaneError::Journal)?;
        *record = next;
        actor.command(InferCommand::Checkpointed {
            id: record.actor_id,
            request: record.request.clone(),
            resume: record.checkpoint.clone(),
        });
        table.generation += 1;
        Ok(())
    }

    /// Refuse route access immediately, pause work, and wait for the durable
    /// boundary. Unlock resumes only runs paused by this lock.
    pub fn set_locked(&self, locked: bool) -> Result<(), String> {
        self.shared.table().locked = locked;
        let names: Vec<_> = self
            .shared
            .table()
            .runs
            .iter()
            .filter(|record| {
                !record.view.state.is_terminal()
                    && (if locked {
                        record.executing
                            || matches!(record.view.state, RunStateV1::Running | RunStateV1::Queued)
                    } else {
                        record.pause_reason.as_deref() == Some("lock")
                    })
            })
            .map(|record| record.view.run.clone())
            .collect();
        if locked {
            for name in &names {
                self.pause(name, "lock")
                    .map_err(|error| error.to_string())?;
            }
            loop {
                if self.shared.table().runs.iter().all(|record| {
                    !record.executing
                        && !matches!(record.view.state, RunStateV1::Running | RunStateV1::Queued)
                }) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(2));
            }
        } else if self
            .shared
            .journal
            .as_ref()
            .is_none_or(|journal| journal.execution_permitted())
        {
            for name in names {
                self.resume(&name, "unlock")
                    .map_err(|error| error.to_string())?;
            }
        }
        Ok(())
    }

    /// Drop materialized persona data after a lock boundary. Actor IDs remain
    /// monotonic; an unlock rehydrates records through the unlocked journal.
    #[cfg(feature = "inference-model")]
    pub fn forget_locked_materialization(&self) {
        let mut table = self.shared.table();
        assert!(table.locked);
        table.runs.clear();
        table.by_actor_id.clear();
        table.generation += 1;
        for cache in self
            .shared
            .caches
            .lock()
            .unwrap()
            .iter()
            .filter_map(|cache| cache.upgrade())
        {
            if let Some((mut card, mut runs)) = cache.lock().unwrap().take() {
                use zeroize::Zeroize;
                card.zeroize();
                runs.zeroize();
            }
        }
    }

    fn refresh_remote(&self) -> Result<(), String> {
        if self.is_locked() {
            return Ok(());
        }
        let Some(journal) = &self.shared.journal else {
            return Ok(());
        };
        let revision = journal.remote_revision();
        if self.shared.table().seen_remote_revision == revision {
            return Ok(());
        }
        match self.refresh_retained(false) {
            Err(error) if error.contains("missing parent") => return Ok(()),
            result => result?,
        }
        self.shared.table().seen_remote_revision = revision;
        Ok(())
    }

    /// Rehydrate the current log heads, retaining active local execution IDs.
    pub fn refresh_retained_runs(&self) -> Result<(), String> {
        self.refresh_retained(true)
    }

    fn refresh_retained(&self, allow_locked: bool) -> Result<(), String> {
        let Some(journal) = &self.shared.journal else {
            return Ok(());
        };
        let records = journal.load()?;
        let mut table = self.shared.table();
        if table.locked && !allow_locked {
            return Ok(());
        }
        for mut record in records {
            if let Some(existing) = table
                .runs
                .iter_mut()
                .find(|existing| existing.view.run == record.view.run)
            {
                if matches!(
                    existing.view.state,
                    RunStateV1::Running | RunStateV1::Queued
                ) {
                    continue;
                }
                record.actor_id = existing.actor_id;
                if !record.view.state.is_terminal() {
                    record.view.state = RunStateV1::Paused;
                }
                *existing = record;
            } else {
                record.actor_id = table.next_actor_id;
                table.next_actor_id += 1;
                if !record.view.state.is_terminal() {
                    record.view.state = RunStateV1::Paused;
                }
                let index = table.runs.len();
                table.by_actor_id.insert(record.actor_id, index);
                table.runs.push(record);
            }
        }
        table.generation += 1;
        Ok(())
    }

    /// Whether the route currently refuses persona-data access.
    pub fn is_locked(&self) -> bool {
        self.shared.table().locked
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
                .filter(|record| {
                    matches!(record.view.state, RunStateV1::Running | RunStateV1::Queued)
                })
                .map(|record| record.actor_id)
                .collect();
            for id in open {
                if self.shared.journal.is_some() {
                    let mut table = self.shared.table();
                    let index = table.by_actor_id[&id];
                    table.runs[index].pause_reason = Some("shutdown".into());
                    actor.command(InferCommand::Pause { id });
                } else {
                    actor.command(InferCommand::Cancel { id });
                }
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
        let current = Arc::new(Mutex::new(None));
        self.shared
            .caches
            .lock()
            .unwrap()
            .retain(|cache| cache.strong_count() > 0);
        self.shared
            .caches
            .lock()
            .unwrap()
            .push(Arc::downgrade(&current));
        InferenceAskEndpoint {
            lane: self.clone(),
            session,
            noticed: None,
            current,
        }
    }
}

/// Fold updates until the actor is gone and its channel closes.
fn bridge(shared: &Shared, updates: &Receiver<InferUpdate>) {
    while let Ok(update) = updates.recv() {
        let changed = match &update {
            InferUpdate::Paused { id }
            | InferUpdate::Cancelled { id }
            | InferUpdate::Failed { id, .. }
            | InferUpdate::Finished { id, .. } => Some(*id),
            _ => None,
        };
        let mut table = shared.table();
        table.apply(update);
        if let (Some(id), Some(journal)) = (changed, &shared.journal) {
            if let Some(index) = table.by_actor_id.get(&id).copied() {
                if let Err(error) = journal.append(&table.runs[index]) {
                    table.runs[index].view.state = RunStateV1::Failed;
                    table.runs[index].view.error =
                        Some(format!("checkpoint commit failed: {error}"));
                }
            }
        }
    }
}

/// The `inference.ask` endpoint for one session.
pub struct InferenceAskEndpoint {
    lane: InferenceLane,
    session: ProjectionSession,
    noticed: Option<u64>,
    /// The bytes the last snapshot named, so its resources stay servable
    /// while the table moves on.
    current: Arc<Mutex<Option<(Vec<u8>, Vec<u8>)>>>,
}

const CARD_LABEL: &str = "Inference runs";

fn score() -> Score {
    Score::new(Arrangement::Spiral(Default::default()))
}

impl InferenceAskEndpoint {
    fn own_session(&self, session: &ProjectionSession, what: &str) -> Result<(), String> {
        if self.lane.is_locked() {
            return Err("inference.ask is locked".into());
        }
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
        self.lane.refresh_remote()?;
        let table = self.lane.shared.table();
        if table.locked {
            return Err("inference.ask is locked".into());
        }
        let (runs, generation) = (table.view(), table.generation);
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
        *self.current.lock().unwrap() = Some((card_bytes, runs_bytes));
        Ok(snapshot)
    }
}

impl PresentationSource for InferenceAskEndpoint {
    type Error = String;

    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, Self::Error> {
        self.own_session(&request.session, "resource")?;
        let table = self.lane.shared.table();
        if table.locked {
            return Err("inference.ask is locked".into());
        }
        let current = self.current.lock().unwrap();
        let Some((card, runs)) = current.as_ref() else {
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
            ASK_SAMPLED_INTENT => {
                match serde_json::from_slice::<SampledAskRequestV1>(&intent.payload) {
                    Ok(sampled) => self.lane.ask_with_generation(
                        sampled.ask,
                        GenerationRequest {
                            max_tokens: sampled.max_tokens,
                            temperature: sampled.temperature,
                            top_p: sampled.top_p,
                            seed: sampled.seed,
                            ..Default::default()
                        },
                    ),
                    Err(error) => return rejected(error.to_string()),
                }
            },
            PAUSE_INTENT => match serde_json::from_slice::<CancelRequestV1>(&intent.payload) {
                Ok(run) => self.lane.pause(&run.run, "user"),
                Err(error) => return rejected(error.to_string()),
            },
            RESUME_INTENT => match serde_json::from_slice::<CancelRequestV1>(&intent.payload) {
                Ok(run) => self.lane.resume(&run.run, "user"),
                Err(error) => return rejected(error.to_string()),
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
        if self.lane.is_locked() {
            self.current.lock().unwrap().take();
            return Err("inference.ask is locked".into());
        }
        self.lane.refresh_remote()?;
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
    fn retained_asks_do_not_auto_execute_without_current_authority() {
        struct Historical;
        impl RunJournal for Historical {
            fn device(&self) -> &str {
                "retired"
            }
            fn execution_permitted(&self) -> bool {
                false
            }
            fn load(&self) -> Result<Vec<RunRecord>, String> {
                Ok(vec![RunRecord {
                    view: RunV1 {
                        run: "private".into(),
                        question: "retained question".into(),
                        state: RunStateV1::Paused,
                        text: "prefix".into(),
                        error: None,
                        session: None,
                        tokens: 0,
                        continuations: Vec::new(),
                    },
                    actor_id: 0,
                    request: GenerationRequest::default(),
                    checkpoint: None,
                    continuations: vec![Continuation {
                        device: "retired".into(),
                        at_token: 0,
                        reason: "ask".into(),
                        first_differing_token: None,
                        compared_token_ids_hash: None,
                    }],
                    pause_reason: Some("lock".into()),
                    executing: false,
                }])
            }
            fn append(&self, _: &RunRecord) -> Result<(), String> {
                Err("not currently keyed".into())
            }
        }
        let lane = InferenceLane::try_open_with_journal(
            8,
            || Ok(Box::new(StubInferenceProvider::new())),
            Some(Arc::new(Historical)),
        )
        .unwrap();
        assert_eq!(lane.runs().runs[0].state, RunStateV1::Paused);
        lane.set_locked(true).unwrap();
        lane.set_locked(false).unwrap();
        assert_eq!(lane.runs().runs[0].state, RunStateV1::Paused);
        lane.close().unwrap();
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
    fn a_load_failure_is_visible_and_each_ask_fails() {
        let lane = InferenceLane::try_open(|| {
            Err(esp::infer::InferError::InvalidConfig(
                "missing template".into(),
            ))
        });
        lane.ask(ask("a", "why?")).unwrap();
        let runs = wait_for(&lane, |runs| run(runs, "a").state.is_terminal());
        assert!(runs.model.is_none());
        assert!(runs.session.is_none());
        assert!(
            runs.load_error
                .as_deref()
                .unwrap()
                .contains("missing template")
        );
        assert_eq!(run(&runs, "a").state, RunStateV1::Failed);
        assert!(run(&runs, "a").text.is_empty());
        lane.close().unwrap();
    }

    #[test]
    fn close_cancels_an_ask_waiting_for_a_gpu_worker_to_stop() {
        struct WaitingProvider {
            stub: StubInferenceProvider,
            access: crate::gpu::GpuAccess,
        }
        impl InferenceProvider for WaitingProvider {
            fn capability(&self) -> &esp::infer::ModelCapability {
                self.stub.capability()
            }
            fn generate_streaming(
                &self,
                _: &GenerationRequest,
                _: &mut dyn FnMut(&str) -> std::ops::ControlFlow<()>,
            ) -> Result<String, esp::infer::InferError> {
                unreachable!("actor must supply cooperative cancellation")
            }
            fn generate_streaming_controlled(
                &self,
                _: &GenerationRequest,
                _: &mut dyn FnMut(&str) -> std::ops::ControlFlow<()>,
                cancelled: &mut dyn FnMut() -> bool,
            ) -> Result<String, esp::infer::InferError> {
                let request = self.access.request();
                assert!(
                    request.acquire(cancelled).is_none(),
                    "the worker still owns the GPU"
                );
                Ok(String::new())
            }
        }
        let access = crate::gpu::GpuAccess::default();
        let worker = access.try_background().unwrap();
        let provider_access = access.clone();
        let lane = InferenceLane::open(move || {
            Box::new(WaitingProvider {
                stub: StubInferenceProvider::new(),
                access: provider_access,
            })
        });
        lane.ask(ask("waiting", "why?")).unwrap();
        for _ in 0..100 {
            if access.foreground_requested() {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(access.foreground_requested());
        lane.close().unwrap();
        assert_eq!(run(&lane.runs(), "waiting").state, RunStateV1::Cancelled);
        assert!(!access.foreground_requested());
        assert!(
            access.try_background().is_none(),
            "close cannot release another worker's guard"
        );
        drop(worker);
        assert!(access.try_background().is_some());
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
