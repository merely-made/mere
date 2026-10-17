// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Layer-stage work on the resident's shared iroh endpoint. Every call is
//! admitted against an active supervised mesh lease and its signed input.
//! Prompt-free discovery and reservation precede that lease.

mod fallback;
pub use fallback::{StageFallbackProvider, StreamingLoader};
mod lane;
pub use lane::{
    StageChainFactory, StagedLaneDevice, open_automatic_library_lane_with_journal,
    open_staged_library_lane_with_journal,
};
mod discovery;
pub use discovery::{StageCatalog, StageOffer, library_stage_catalog};
pub mod automatic;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use esp::infer::decoder::stage::{
    DecoderStage, LayerRange, StageExecutor, StageHidden, decode_hidden, encode_hidden,
};
use esp::infer::{
    InferError,
    decoder::{DecoderDevice, WeightFormat},
};
use mesh::{
    JobBoard, JobControl, JobId, LeaseId, LeasePolicy, MeshResource, Prepared, RemoteAdmission,
    RemoteSessionClaim, ResourceDescriptor, ResourceError, ResourceId, RunContext,
};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use transport::{Alpn, P2pandaTransport, PeerID, Transport};

use crate::authority::RemoteSessionProjection;
use crate::gpu::{GpuAccess, GpuGuard};
use crate::mesh_host::Clock;

/// Protocol on the resident's existing authenticated iroh endpoint.
pub const STAGE_ALPN: &str = "mere/inference-stage/v1";
const CONTROL_BYTES: usize = 16 * 1024;

/// Device-addressed resource offers prevent another host claiming a range
/// assigned to this device. The algorithm remains the same for every offer.
pub fn stage_resource(peer: [u8; 32]) -> ResourceId {
    let suffix: String = peer.iter().map(|b| format!("{b:02x}")).collect();
    ResourceId::parse(&format!("esp.inference.stage.{suffix}/v1")).expect("bounded resource id")
}

/// Signed, content-addressed job input. A factory verifies the row's exact
/// session and loader before loading the granted range from its library.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageRequest {
    /// Immutable library row available to this worker.
    pub row: eidetic::ManifestId,
    /// Hash of the complete model/template/format/backend session.
    pub session: String,
    /// Base backend loader, before its layer-index suffix.
    pub loader: String,
    /// Absolute half-open layer range this job grants.
    pub range: LayerRange,
}

/// Owner limits. Hidden bytes cover a full prefill; calls and pending accepts
/// are bounded independently of the model's context window.
#[derive(Clone, Debug)]
pub struct StageSettings {
    /// Maximum encoded hidden/logits frame, at most 256 MiB.
    pub max_hidden_bytes: usize,
    /// Queued-call deadline and maximum idle time in an open stage session.
    pub call_timeout: Duration,
    /// Cancellation, lock and current-seat observation cadence.
    pub poll_every: Duration,
    /// Capability floor of this device's stage offer.
    pub requires: mesh::ResourceRequirements,
}

impl StageSettings {
    fn validate(&self) -> Result<(), String> {
        if self.max_hidden_bytes < 20
            || self.max_hidden_bytes > 256 * 1024 * 1024
            || self.call_timeout.is_zero()
            || self.poll_every.is_zero()
        {
            return Err("stage byte budget or cadence is invalid".into());
        }
        Ok(())
    }
}

/// Returned by the owner factory inside the supervised run. Guard outlives
/// tensor drain and destruction, including cancellation during model load.
pub struct StageRuntime {
    stage: Option<DecoderStage>,
    guard: Option<GpuGuard>,
    busy: Option<BusyPermit>,
    loading_device: Option<DecoderDevice>,
}

struct BusyPermit(Arc<AtomicBool>);
impl Drop for BusyPermit {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

impl StageRuntime {
    /// A CPU stage without a GPU ownership guard.
    pub fn cpu(stage: DecoderStage) -> Self {
        Self {
            stage: Some(stage),
            guard: None,
            busy: None,
            loading_device: None,
        }
    }
    /// A stage whose hardware remains owned through drain and destruction.
    pub fn guarded(stage: DecoderStage, guard: GpuGuard) -> Self {
        Self {
            stage: Some(stage),
            guard: Some(guard),
            busy: None,
            loading_device: None,
        }
    }
    fn close(&mut self) -> Result<(), String> {
        let mut result = self
            .stage
            .as_ref()
            .map(DecoderStage::drain)
            .transpose()
            .map_err(|e| e.to_string());
        if self.stage.is_none() {
            if let Some(device) = self.loading_device.as_ref() {
                result = device.sync().map(|_| None).map_err(|e| e.to_string());
            }
        }
        self.stage.take();
        self.loading_device.take();
        self.guard.take();
        self.busy.take();
        result.map(|_| ())
    }
}
impl Drop for StageRuntime {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

/// Owner factory invoked on a blocking worker inside a supervised lease.
pub type StageFactory =
    dyn Fn(&StageRequest, &JobControl) -> Result<StageRuntime, String> + Send + Sync;
/// Live current-epoch personal graph membership, queried by attested root.
/// The resident supplies this from its existing key-group owner.
pub type StageReaders = dyn Fn([u8; 32]) -> bool + Send + Sync;

/// Borrow the existing library writer. Shared tensors resolve to host bytes;
/// only assigned layer tensors reach the worker device. A GPU stage uses the
/// mesh background guard; the host's ordinary owner reclaim preempts it.
pub fn library_stage_factory<B: eidetic::Store + Send + 'static>(
    library: Arc<tokio::sync::Mutex<B>>,
    device: DecoderDevice,
    loader: String,
    gpu: Option<GpuAccess>,
    resident_layer_budget: u64,
) -> Arc<StageFactory> {
    Arc::new(move |request, control| {
        if request.loader != loader {
            return Err("stage backend class differs from owner offer".into());
        }
        control.check().map_err(|e| e.to_string())?;
        let guard = gpu
            .as_ref()
            .map(|access| {
                access
                    .try_background()
                    .ok_or("stage GPU is reclaimed or occupied".to_string())
            })
            .transpose()?;
        let mut runtime = StageRuntime {
            stage: None,
            guard,
            busy: None,
            loading_device: Some(device.clone()),
        };
        let resolved = {
            let mut store = library.blocking_lock();
            pollster::block_on(esp::infer::library::resolve_library_session(
                &mut *store,
                request.row,
            ))
            .map_err(|e| e.to_string())?
        };
        if resolved
            .session(&loader)
            .id()
            .map_err(|e| e.to_string())?
            .to_string()
            != request.session
        {
            return Err("stage input names another exact session".into());
        }
        let costs = resolved.stage_layer_costs().map_err(|e| e.to_string())?;
        request
            .range
            .validate(costs.len())
            .map_err(|e| e.to_string())?;
        let required = costs[request.range.first..request.range.end]
            .iter()
            .try_fold(0u64, |sum, n| sum.checked_add(*n))
            .ok_or("stage payload size overflow")?;
        if required > resident_layer_budget {
            return Err("assigned stage exceeds the owner's resident layer/KV budget".into());
        }
        let source = crate::inference::library_layer_source(library.clone(), &resolved)
            .ok_or("stage requires a layer-indexed row")?;
        let guarded_source = esp::infer::decoder::streaming::with_expert_reader(
            |index, limit| {
                control.check().map_err(|_| InferError::Interrupted)?;
                source.read_layer(index, limit)
            },
            |layer, expert, limit| {
                control.check().map_err(|_| InferError::Interrupted)?;
                source.read_expert(layer, expert, limit)
            },
        );
        let stage = resolved
            .load_stage(&device, &loader, request.range, &guarded_source)
            .map_err(|e| e.to_string())?;
        runtime.stage = Some(stage);
        control.check().map_err(|e| e.to_string())?;
        Ok(runtime)
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RunKey {
    job: JobId,
    lease: LeaseId,
}

struct ActiveRun {
    request: StageRequest,
    control: JobControl,
    runtime: Mutex<StageRuntime>,
    closed: AtomicBool,
    finished: AtomicBool,
    seen_live: AtomicBool,
    last_call: Mutex<std::time::Instant>,
}

#[derive(Default)]
struct AdmissionState {
    board: Option<JobBoard>,
    active: BTreeMap<RunKey, Arc<ActiveRun>>,
}

/// Service lifetime is explicit: close stage calls and drain all workers
/// before shutting down the library or shared transport.
pub struct StageService {
    transport: Arc<P2pandaTransport>,
    mesh: [u8; 32],
    author: [u8; 32],
    clock: Arc<dyn Clock>,
    policy: LeasePolicy,
    settings: StageSettings,
    descriptor: ResourceDescriptor,
    factory: Arc<StageFactory>,
    readers: Arc<StageReaders>,
    state: RwLock<AdmissionState>,
    stopped: AtomicBool,
    task: Mutex<Option<tokio::task::JoinHandle<()>>>,
    busy: Arc<AtomicBool>,
    catalog_busy: Arc<AtomicBool>,
    last_refusal: Mutex<Option<String>>,
    private_locked: AtomicBool,
    catalog: RwLock<Option<Arc<StageCatalog>>>,
    reservation: Mutex<Option<discovery::StageReservation>>,
}

impl StageService {
    /// Mount one bounded, serial stage service. Admission remains closed until
    /// the owner projects a board and its supervised resource becomes active.
    pub async fn mount(
        transport: Arc<P2pandaTransport>,
        mesh: [u8; 32],
        author: [u8; 32],
        clock: Arc<dyn Clock>,
        policy: LeasePolicy,
        settings: StageSettings,
        factory: Arc<StageFactory>,
        readers: Arc<StageReaders>,
    ) -> Result<Arc<Self>, String> {
        settings.validate()?;
        if transport.local_peer_id().to_bytes() != author {
            return Err("stage transport must be the attested mesh author".into());
        }
        transport
            .register_bounded_alpn(Alpn::new(STAGE_ALPN), 4, settings.call_timeout)
            .await
            .map_err(|e| e.to_string())?;
        let service = Arc::new(Self {
            descriptor: ResourceDescriptor {
                resource: stage_resource(author),
                implementation: mesh::ImplementationId::parse("distillery.inference-stage/v1")
                    .unwrap(),
                requires: settings.requires,
                verification: mesh::VerificationClass::ProducerOnly,
            },
            transport,
            mesh,
            author,
            clock,
            policy,
            settings,
            factory,
            readers,
            state: RwLock::new(AdmissionState::default()),
            stopped: AtomicBool::new(false),
            task: Mutex::new(None),
            busy: Arc::new(AtomicBool::new(false)),
            catalog_busy: Arc::new(AtomicBool::new(false)),
            last_refusal: Mutex::new(None),
            private_locked: AtomicBool::new(false),
            catalog: RwLock::new(None),
            reservation: Mutex::new(None),
        });
        let weak = Arc::downgrade(&service);
        let network = service.transport.clone();
        let task = tokio::spawn(async move {
            // One call at a time on one stage avoids interleaved KV mutation.
            loop {
                let (accepted, arrived) = match network.accept_observed(Alpn::new(STAGE_ALPN)).await
                {
                    Ok(s) => s,
                    Err(_) => break,
                };
                let Some(service) = weak.upgrade() else { break };
                if service.stopped.load(Ordering::SeqCst) {
                    break;
                }
                if let Ok(Err(error)) = tokio::time::timeout_at(
                    arrived + service.settings.call_timeout,
                    service.serve(accepted),
                )
                .await
                {
                    *service.last_refusal.lock().unwrap() = Some(error);
                }
            }
        });
        *service.task.lock().unwrap() = Some(task);
        Ok(service)
    }

    /// Adapter to register with the same resident's mesh supervisor.
    pub fn resource(self: &Arc<Self>) -> Arc<dyn MeshResource> {
        Arc::new(StageResource(self.clone()))
    }
    /// Install the library owner's exact-session discovery callback. Older
    /// explicit stage services remain undiscoverable until this is installed.
    pub fn set_catalog(&self, catalog: Arc<StageCatalog>) {
        *self.catalog.write().unwrap() = Some(catalog);
    }
    /// Authenticated transport/mesh-author identity of this worker.
    pub fn server_peer(&self) -> [u8; 32] {
        self.author
    }
    /// Owner diagnostic only; contains protocol/authority errors, never hidden values.
    pub fn last_refusal(&self) -> Option<String> {
        self.last_refusal.lock().unwrap().clone()
    }
    /// Replace lease and attested-root facts after a supervisor tick.
    pub fn refresh(&self, board: JobBoard) {
        self.state.write().unwrap().board = Some(board);
    }
    /// Whether this exact job/lease currently has a live runtime.
    pub fn is_active(&self, job: JobId, lease: LeaseId) -> bool {
        self.state
            .read()
            .unwrap()
            .active
            .get(&RunKey { job, lease })
            .is_some_and(|r| !r.closed.load(Ordering::SeqCst))
    }
    /// Refuse immediately; active resources observe this between calls and
    /// drain KV and weights before releasing their hardware/busy guards.
    pub fn set_private_locked(&self, locked: bool) {
        self.private_locked.store(locked, Ordering::SeqCst);
    }
    /// A synchronous vault holder calls this on its blocking lock path. Async
    /// hosts can await `drain_private` without blocking their executor.
    pub fn private_drained(&self) -> bool {
        !self.busy.load(Ordering::SeqCst) && !self.catalog_busy.load(Ordering::SeqCst)
    }
    /// Lock private stage work and await actual tensor/guard destruction.
    pub async fn drain_private(&self) {
        self.set_private_locked(true);
        while !self.private_drained() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }
    fn run_permitted(&self, key: RunKey, run: &ActiveRun) -> bool {
        if self.private_locked.load(Ordering::SeqCst) {
            return false;
        }
        let state = self.state.read().unwrap();
        let Some(board) = state.board.as_ref() else {
            return !run.seen_live.load(Ordering::SeqCst);
        };
        let Some(job) = board.job(key.job) else {
            return !run.seen_live.load(Ordering::SeqCst);
        };
        for author in [job.posted_by, self.author] {
            if board
                .devices()
                .master_of(&author)
                .is_some_and(|root| !(self.readers)(root))
            {
                return false;
            }
        }
        if matches!(job.lease_at(self.clock.now_ms(),&self.policy),mesh::LeasePhase::Held{lease,holder,..} if lease==key.lease && holder==self.author)
        {
            run.seen_live.store(true, Ordering::SeqCst);
            return true;
        }
        !job.state.is_terminal() && !run.seen_live.load(Ordering::SeqCst)
    }
    fn authorize_claim(
        &self,
        claim: &RemoteSessionClaim,
        peer: [u8; 32],
        state: &AdmissionState,
    ) -> Result<(), String> {
        if self.stopped.load(Ordering::SeqCst) || self.private_locked.load(Ordering::SeqCst) {
            return Err("stage service closed".into());
        }
        let board = state.board.as_ref().ok_or("stage board not observed")?;
        for author in [claim.client(), self.author] {
            let root = board
                .devices()
                .master_of(&author)
                .ok_or("stage device has no root attestation")?;
            if !(self.readers)(root) {
                return Err("stage device is not a current keyed personal graph reader".into());
            }
        }
        claim
            .authorize(&RemoteAdmission {
                mesh_id: self.mesh,
                board,
                server_author: self.author,
                server_peer: self.author,
                connected_peer: peer,
                requested_device: 0,
                offered_devices: &BTreeSet::from([0]),
                expected_resource: &self.descriptor.resource,
                now_ms: self.clock.now_ms(),
                lease_policy: &self.policy,
            })
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn admit(&self, claim: &RemoteSessionClaim, peer: [u8; 32]) -> Result<Arc<ActiveRun>, String> {
        let state = self.state.read().unwrap();
        self.authorize_claim(claim, peer, &state)?;
        let run = state
            .active
            .get(&RunKey {
                job: claim.job(),
                lease: claim.lease(),
            })
            .ok_or("stage lease is not active in this host")?
            .clone();
        if run.closed.load(Ordering::SeqCst)
            || run.control.signal() != mesh::ControlSignal::Continue
        {
            return Err("stage is stopping".into());
        }
        run.seen_live.store(true, Ordering::SeqCst);
        *run.last_call.lock().unwrap() = std::time::Instant::now();
        Ok(run)
    }

    async fn serve(
        &self,
        accepted: transport::AcceptedSession<transport::P2pandaStream>,
    ) -> Result<(), String> {
        if !accepted.is_transport_authenticated() || accepted.protocol != Alpn::new(STAGE_ALPN) {
            return Err("stage carrier identity".into());
        }
        let peer = accepted.peer.ok_or("stage peer missing")?.to_bytes();
        let mut stream = accepted.stream;
        let bytes = notochord::read_frame(&mut stream, CONTROL_BYTES as u32)
            .await
            .map_err(|e| e.to_string())?;
        let call: StageCall = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if matches!(
            call.operation,
            StageOperation::Discover | StageOperation::Reserve | StageOperation::ReleaseReservation
        ) {
            return self.serve_discovery(peer, call, &mut stream).await;
        }
        let claim = RemoteSessionClaim::decode(&call.credential).map_err(|e| e.to_string())?;
        if call.operation == StageOperation::Probe {
            let ready = {
                let state = self.state.read().unwrap();
                self.authorize_claim(&claim, peer, &state)?;
                let job = state
                    .board
                    .as_ref()
                    .unwrap()
                    .job(claim.job())
                    .ok_or("stage job missing")?;
                let bytes = serde_json::to_vec(&call.request).map_err(|e| e.to_string())?;
                if job.spec.as_ref().and_then(|spec| spec.input("stage"))
                    != Some(&mesh::BlobRef::blake3(&bytes))
                {
                    return Err("stage probe disagrees with signed input".into());
                }
                state
                    .active
                    .get(&RunKey {
                        job: claim.job(),
                        lease: claim.lease(),
                    })
                    .is_some_and(|run| {
                        let ready = run.request == call.request
                            && !run.closed.load(Ordering::SeqCst)
                            && run.control.signal() == mesh::ControlSignal::Continue;
                        if ready {
                            *run.last_call.lock().unwrap() = std::time::Instant::now();
                        }
                        ready
                    })
            };
            notochord::write_frame(
                &mut stream,
                if ready { b"ready" } else { b"preparing" },
                CONTROL_BYTES as u32,
            )
            .await
            .map_err(|e| e.to_string())?;
            stream.shutdown().await.map_err(|e| e.to_string())?;
            return Ok(());
        }
        let run = self.admit(&claim, peer)?;
        if call.request != run.request {
            return Err("stage call disagrees with signed job input".into());
        }
        if call.operation == StageOperation::Finish {
            run.finished.store(true, Ordering::SeqCst);
            self.reservation.lock().unwrap().take();
            notochord::write_frame(&mut stream, b"finished", CONTROL_BYTES as u32)
                .await
                .map_err(|e| e.to_string())?;
            stream.shutdown().await.map_err(|e| e.to_string())?;
            return Ok(());
        }
        let hidden = zeroize::Zeroizing::new(
            notochord::read_frame(&mut stream, self.settings.max_hidden_bytes as u32)
                .await
                .map_err(|e| e.to_string())?,
        );
        self.admit(&claim, peer)?;
        let work = run.clone();
        let limit = self.settings.max_hidden_bytes;
        let output = tokio::task::spawn_blocking(move || {
            let mut runtime = work
                .runtime
                .lock()
                .map_err(|_| "stage runtime poisoned".to_string())?;
            if work.closed.load(Ordering::SeqCst)
                || work.control.signal() != mesh::ControlSignal::Continue
            {
                return Err("stage reclaimed".into());
            }
            let stage = runtime.stage.as_mut().ok_or("stage runtime closed")?;
            let output = stage
                .forward_bytes(&hidden, call.start, limit)
                .map_err(|e| e.to_string())?;
            work.control
                .report(work.control.progress().done.saturating_add(1), 0);
            Ok::<_, String>(zeroize::Zeroizing::new(output))
        })
        .await
        .map_err(|e| e.to_string())??;
        self.admit(&claim, peer)?;
        notochord::write_frame(&mut stream, &output, self.settings.max_hidden_bytes as u32)
            .await
            .map_err(|e| e.to_string())?;
        stream.shutdown().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn close_run(&self, key: RunKey, run: Arc<ActiveRun>) -> Result<(), String> {
        run.closed.store(true, Ordering::SeqCst);
        self.state.write().unwrap().active.remove(&key);
        tokio::task::spawn_blocking(move || {
            run.runtime
                .lock()
                .map_err(|_| "stage runtime poisoned".to_string())?
                .close()
        })
        .await
        .map_err(|e| e.to_string())?
    }

    /// Stop accepting and drain active or detached blocking stage work.
    pub async fn shutdown(&self) -> Result<(), String> {
        self.stopped.store(true, Ordering::SeqCst);
        self.transport.stop_accepting(&Alpn::new(STAGE_ALPN));
        let task = self.task.lock().unwrap().take();
        if let Some(task) = task {
            task.abort();
            let _ = task.await;
        }
        let runs: Vec<_> = self
            .state
            .read()
            .unwrap()
            .active
            .iter()
            .map(|(k, r)| (*k, r.clone()))
            .collect();
        for (key, run) in runs {
            self.close_run(key, run).await?;
        }
        // An aborted resource may already have removed admission while its
        // blocking computation still owns the runtime and busy permit.
        while !self.private_drained() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        Ok(())
    }
}
impl RemoteSessionProjection for StageService {
    fn refresh(&self, board: JobBoard) {
        self.refresh(board);
    }
}

impl Drop for StageService {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::SeqCst);
        self.transport.stop_accepting(&Alpn::new(STAGE_ALPN));
        if let Ok(task) = self.task.get_mut() {
            if let Some(task) = task.take() {
                task.abort();
            }
        }
    }
}

struct StageResource(Arc<StageService>);

// MeshHost may abort an async lease-lost run. Removing admission is immediate;
// an uncancellable blocking drain retains the hardware guard and busy permit.
struct RunRegistration {
    service: Arc<StageService>,
    key: RunKey,
    run: Arc<ActiveRun>,
    closed: bool,
}
impl Drop for RunRegistration {
    fn drop(&mut self) {
        if self.closed {
            return;
        }
        self.run.closed.store(true, Ordering::SeqCst);
        self.service.state.write().unwrap().active.remove(&self.key);
        let run = self.run.clone();
        tokio::task::spawn_blocking(move || {
            if let Ok(mut runtime) = run.runtime.lock() {
                let _ = runtime.close();
            }
        });
    }
}
impl MeshResource for StageResource {
    fn descriptor(&self) -> &ResourceDescriptor {
        &self.0.descriptor
    }
    fn available_for(&self, job: &mesh::Job) -> bool {
        if self.0.stopped.load(Ordering::SeqCst)
            || self.0.private_locked.load(Ordering::SeqCst)
            || self.0.busy.load(Ordering::SeqCst)
        {
            return false;
        }
        if !self.0.reservation_permits(job) {
            return false;
        }
        let state = self.0.state.read().unwrap();
        let Some(board) = state.board.as_ref() else {
            return false;
        };
        [self.0.author, job.posted_by].iter().all(|author| {
            board
                .devices()
                .master_of(author)
                .is_some_and(|root| (self.0.readers)(root))
        })
    }
    fn prepare<'a>(
        &'a self,
        ns: &'a mesh::JobNamespaceView<'a>,
    ) -> mesh::namespace::BoxFuture<'a, Result<Prepared, ResourceError>> {
        Box::pin(async move {
            let bytes = ns.read("stage").await?;
            if bytes.len() > CONTROL_BYTES {
                return Err(ResourceError::input("stage", "request budget"));
            }
            let request: StageRequest = serde_json::from_slice(&bytes)
                .map_err(|e| ResourceError::input("stage", e.to_string()))?;
            if request.session.len() > 256
                || request.loader.len() > 128
                || request.range.first >= request.range.end
            {
                return Err(ResourceError::input("stage", "invalid stage input"));
            }
            if serde_json::to_vec(&request)
                .map_err(|e| ResourceError::input("stage", e.to_string()))?
                != bytes
            {
                return Err(ResourceError::input(
                    "stage",
                    "stage input must use the canonical struct encoding",
                ));
            }
            Ok(Prepared::new(request))
        })
    }
    fn execute<'a>(
        &'a self,
        _: Prepared,
        _: &'a JobControl,
    ) -> mesh::namespace::BoxFuture<'a, Result<Vec<u8>, ResourceError>> {
        Box::pin(async {
            Err(ResourceError::Backend(
                "stage requires supervised lease context".into(),
            ))
        })
    }
    fn execute_for<'a>(
        &'a self,
        prepared: Prepared,
        control: &'a JobControl,
        context: RunContext,
    ) -> mesh::namespace::BoxFuture<'a, Result<Vec<u8>, ResourceError>> {
        Box::pin(async move {
            let request = prepared.take::<StageRequest>()?;
            let key = RunKey {
                job: context.job,
                lease: context
                    .lease
                    .ok_or_else(|| ResourceError::Backend("stage requires a mesh lease".into()))?,
            };
            let factory = self.0.factory.clone();
            if self.0.private_locked.load(Ordering::SeqCst) {
                return Err(ResourceError::Backend(
                    "private stage work is locked".into(),
                ));
            }
            if self
                .0
                .busy
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                return Err(ResourceError::Backend(
                    "stage device already assigned".into(),
                ));
            }
            let permit = BusyPermit(self.0.busy.clone());
            let load_request = request.clone();
            let load_control = control.clone();
            let runtime = tokio::task::spawn_blocking(move || {
                let mut runtime = factory(&load_request, &load_control)?;
                runtime.busy = Some(permit);
                Ok::<_, String>(runtime)
            })
            .await
            .map_err(|e| ResourceError::Backend(e.to_string()))?
            .map_err(ResourceError::Backend)?;
            let run = Arc::new(ActiveRun {
                request,
                control: control.clone(),
                runtime: Mutex::new(runtime),
                closed: AtomicBool::new(false),
                finished: AtomicBool::new(false),
                seen_live: AtomicBool::new(false),
                last_call: Mutex::new(std::time::Instant::now()),
            });
            {
                let mut state = self.0.state.write().unwrap();
                if !state.active.is_empty() || self.0.stopped.load(Ordering::SeqCst) {
                    return Err(ResourceError::Backend(
                        "stage device already assigned or closed".into(),
                    ));
                }
                state.active.insert(key, run.clone());
            }
            let mut registration = RunRegistration {
                service: self.0.clone(),
                key,
                run: run.clone(),
                closed: false,
            };
            let mut idle = false;
            while !self.0.stopped.load(Ordering::SeqCst)
                && !run.finished.load(Ordering::SeqCst)
                && control.signal() == mesh::ControlSignal::Continue
                && self.0.run_permitted(key, &run)
            {
                if run.last_call.lock().unwrap().elapsed() >= self.0.settings.call_timeout {
                    idle = true;
                    break;
                }
                tokio::time::sleep(self.0.settings.poll_every).await;
            }
            let finished = run.finished.load(Ordering::SeqCst);
            self.0
                .close_run(key, run)
                .await
                .map_err(ResourceError::Backend)?;
            registration.closed = true;
            if control.signal() != mesh::ControlSignal::Continue
                || self.0.stopped.load(Ordering::SeqCst)
                || (!finished && !idle)
            {
                return Err(mesh::Cancelled.into());
            }
            Ok(if idle {
                b"esp.inference.stage/v1 closed-idle".to_vec()
            } else {
                b"esp.inference.stage/v1 finished".to_vec()
            })
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum StageOperation {
    Discover,
    Reserve,
    ReleaseReservation,
    Probe,
    Forward,
    Finish,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StageCall {
    credential: Vec<u8>,
    request: StageRequest,
    start: usize,
    operation: StageOperation,
}

/// A single assigned lease. Credentials are signed by the job poster; the
/// server rechecks them on every forward, after input, and before output.
#[derive(Clone)]
pub struct StagePeer {
    /// Authenticated worker transport address.
    pub peer: PeerID,
    /// The poster's signed claim for this exact live lease.
    pub credential: Vec<u8>,
    /// Exact content-addressed job input checked again by the worker.
    pub request: StageRequest,
}

/// Completed tensor transfers. Wire framing, QUIC overhead and incomplete
/// failed calls are excluded; a partial chain's completed hops still count.
#[derive(Clone, Debug, Default, Serialize)]
pub struct StageTraffic {
    /// Complete chain passes, including prefill and a terminating EOS pass.
    pub forwards: u64,
    /// Completed hidden-state request payload bytes.
    pub payload_bytes_sent: u64,
    /// Completed hidden-state or final-logits reply payload bytes.
    pub payload_bytes_received: u64,
    /// Wall time in complete chain passes; includes their device readback.
    pub elapsed_seconds: f64,
}

/// Dimensions bound to the selected model. The final worker returns exactly
/// one f32 vocabulary row; intermediate workers retain the hidden width.
#[derive(Clone, Copy, Debug)]
pub struct StageShape {
    /// Intermediate tensor width.
    pub hidden: usize,
    /// Final f32 logits width.
    pub vocab: usize,
    /// Absolute model context limit.
    pub context: usize,
    /// Total layers the assignments must cover once.
    pub layers: usize,
}

/// Ordered disjoint stages, relayed by the coordinator. Only hidden states
/// move; no individual Burn operations or remote weight downloads occur.
pub struct StageChain {
    transport: Arc<P2pandaTransport>,
    runtime: tokio::runtime::Handle,
    peers: Vec<StagePeer>,
    settings: StageSettings,
    format: WeightFormat,
    width: usize,
    vocab: usize,
    max_seq: usize,
    traffic: Arc<Mutex<StageTraffic>>,
    peers_permitted: Arc<dyn Fn(PeerID) -> bool + Send + Sync>,
    cleanup_timeout: Duration,
    finished: AtomicBool,
}
impl std::fmt::Debug for StageChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StageChain")
            .field("stages", &self.peers.len())
            .finish()
    }
}
impl StageChain {
    /// Validate distinct devices and a complete, ordered session-bound cover.
    pub fn new(
        transport: Arc<P2pandaTransport>,
        runtime: tokio::runtime::Handle,
        peers: Vec<StagePeer>,
        settings: StageSettings,
        format: WeightFormat,
        shape: StageShape,
        peers_permitted: Arc<dyn Fn(PeerID) -> bool + Send + Sync>,
    ) -> Result<Self, String> {
        settings.validate()?;
        let StageShape {
            hidden: width,
            vocab,
            context: max_seq,
            layers,
        } = shape;
        if peers.len() < 2 || width == 0 || max_seq == 0 || vocab == 0 {
            return Err("stage chain needs two devices and valid hidden dimensions".into());
        }
        let session = &peers[0].request.session;
        let mut end = 0;
        for (i, peer) in peers.iter().enumerate() {
            peer.request
                .range
                .validate(layers)
                .map_err(|e| e.to_string())?;
            if peer.request.range.first != end
                || peer.request.session != *session
                || peer.request.row != peers[0].request.row
                || peer.request.loader != peers[0].request.loader
                || peers[..i].iter().any(|p| p.peer == peer.peer)
            {
                return Err("stage chain gap, overlap, repeated device or session mismatch".into());
            }
            end = peer.request.range.end;
        }
        if end != layers {
            return Err("stage chain does not cover all layers".into());
        }
        Ok(Self {
            transport,
            runtime,
            peers,
            cleanup_timeout: settings.call_timeout,
            settings,
            format,
            width,
            vocab,
            max_seq,
            traffic: Arc::new(Mutex::new(StageTraffic::default())),
            finished: AtomicBool::new(false),
            peers_permitted,
        })
    }
    /// Shared counters survive the scoped decoder provider.
    pub fn traffic(&self) -> Arc<Mutex<StageTraffic>> {
        self.traffic.clone()
    }

    /// Bound the entire best-effort close, independently of a compute call.
    pub fn with_cleanup_timeout(mut self, timeout: Duration) -> Result<Self, String> {
        if timeout.is_zero() {
            return Err("stage cleanup timeout must be positive".into());
        }
        self.cleanup_timeout = timeout;
        Ok(self)
    }
    /// Compact compute provenance for the coordinator's sealed checkpoint.
    /// Device ids are transport authors; their roots remain in the mesh directory.
    pub fn placement_description(&self) -> String {
        let workers = self
            .peers
            .iter()
            .map(|peer| {
                let id: String = peer
                    .peer
                    .to_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect();
                format!(
                    "{id} layers {}..{}",
                    peer.request.range.first, peer.request.range.end
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        format!("layer stages: {workers}")
    }
    async fn call(
        &self,
        peer: &StagePeer,
        start: usize,
        hidden: &[u8],
        operation: StageOperation,
    ) -> Result<Vec<u8>, String> {
        tokio::time::timeout(self.settings.call_timeout, async {
            if !(self.peers_permitted)(peer.peer) {
                return Err("stage peer no longer has personal graph read authority".into());
            }
            let mut stream = self
                .transport
                .connect(peer.peer, Alpn::new(STAGE_ALPN))
                .await
                .map_err(|e| e.to_string())?;
            let call = serde_json::to_vec(&StageCall {
                credential: peer.credential.clone(),
                request: peer.request.clone(),
                start,
                operation,
            })
            .map_err(|e| e.to_string())?;
            notochord::write_frame(&mut stream, &call, CONTROL_BYTES as u32)
                .await
                .map_err(|e| e.to_string())?;
            if !(self.peers_permitted)(peer.peer) {
                return Err("stage peer read authority changed".into());
            }
            if operation == StageOperation::Forward {
                notochord::write_frame(&mut stream, hidden, self.settings.max_hidden_bytes as u32)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            let limit = if operation == StageOperation::Forward {
                self.settings.max_hidden_bytes
            } else {
                CONTROL_BYTES
            };
            notochord::read_frame(&mut stream, limit as u32)
                .await
                .map_err(|e| e.to_string())
        })
        .await
        .map_err(|_| "stage call timed out".to_string())?
    }
    /// Wait for tensor preparation without sending a prompt or hidden state.
    /// A bound deadline turns an unavailable plan into the streamed fallback.
    pub fn wait_ready(&self) -> Result<(), InferError> {
        self.runtime
            .block_on(async {
                tokio::time::timeout(self.settings.call_timeout, async {
                    loop {
                        let mut ready = true;
                        for peer in &self.peers {
                            if !(self.peers_permitted)(peer.peer) {
                                return Err(
                                    "stage peer no longer has graph read authority".to_string()
                                );
                            }
                            match self.call(peer, 0, &[], StageOperation::Probe).await {
                                Ok(reply) if reply == b"ready" => {},
                                _ => ready = false,
                            }
                        }
                        if ready {
                            return Ok(());
                        }
                        tokio::time::sleep(Duration::from_millis(25)).await;
                    }
                })
                .await
                .map_err(|_| "stage preparation deadline expired".to_string())?
            })
            .map_err(InferError::StageUnavailable)
    }
    /// Best-effort lease-session completion on the actor thread. Unreachable
    /// workers remain subject to their host's cancellation/lease expiry.
    pub fn finish(&self) {
        if self.finished.swap(true, Ordering::SeqCst) {
            return;
        }
        self.runtime.block_on(async {
            let _ = tokio::time::timeout(self.cleanup_timeout, async {
                for peer in &self.peers {
                    let _ = self.call(peer, 0, &[], StageOperation::Finish).await;
                    let _ = discovery::query(
                        &self.transport,
                        peer.peer,
                        peer.request.clone(),
                        StageOperation::ReleaseReservation,
                        self.cleanup_timeout,
                    )
                    .await;
                }
            })
            .await;
        });
    }
}
impl StageExecutor for StageChain {
    fn returns_logits(&self) -> bool {
        true
    }
    fn session_id(&self) -> Option<&str> {
        Some(&self.peers[0].request.session)
    }
    fn forward(&self, hidden: StageHidden, start: usize) -> Result<StageHidden, InferError> {
        if self.finished.load(Ordering::SeqCst) {
            return Err(InferError::StageUnavailable(
                "stage placement retired".into(),
            ));
        }
        let [batch, seq, width] = hidden.dims();
        if batch != 1
            || width != self.width
            || hidden.dtype() != self.format.activation_dtype()
            || start.checked_add(seq).is_none_or(|end| end > self.max_seq)
        {
            return Err(InferError::InvalidRequest(
                "stage chain hidden dimensions, dtype or context".into(),
            ));
        }
        let device = hidden.device();
        let now = std::time::Instant::now();
        let mut bytes = zeroize::Zeroizing::new(
            encode_hidden(hidden, self.settings.max_hidden_bytes).map_err(|e| {
                InferError::StageUnavailable(format!("stage prefill is unavailable: {e}"))
            })?,
        );
        for peer in &self.peers {
            let sent = bytes.len() as u64;
            bytes = zeroize::Zeroizing::new(
                self.runtime
                    .block_on(self.call(peer, start, &bytes, StageOperation::Forward))
                    .map_err(|e| InferError::StageUnavailable(e))?,
            );
            let mut traffic = self.traffic.lock().unwrap();
            traffic.payload_bytes_sent += sent;
            traffic.payload_bytes_received += bytes.len() as u64;
        }
        let result = decode_hidden(&bytes, self.vocab, 1, WeightFormat::F32, &device)?;
        let mut traffic = self.traffic.lock().unwrap();
        traffic.forwards += 1;
        traffic.elapsed_seconds += now.elapsed().as_secs_f64();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use personae::{IdentityProvider, InMemoryProvider};

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn aborted_registration_removes_admission_before_detached_work_releases_the_guard() {
        let identity = InMemoryProvider::from_seed([121; 32]);
        let key = identity.derived_keypair(mesh::MESH_AUTHOR_SALT).unwrap();
        let network = Arc::new(P2pandaTransport::builder_for(&key).bind().await.unwrap());
        let service = StageService::mount(
            network.clone(),
            [1; 32],
            key.public_key().to_bytes(),
            Arc::new(crate::mesh_host::ManualClock::at(1_000)),
            LeasePolicy::default(),
            StageSettings {
                max_hidden_bytes: 1024,
                call_timeout: Duration::from_secs(1),
                poll_every: Duration::from_millis(5),
                requires: mesh::ResourceRequirements::cpu(),
            },
            Arc::new(|_, _| Err("not used".into())),
            Arc::new(|_| true),
        )
        .await
        .unwrap();
        let access = GpuAccess::default();
        service.busy.store(true, Ordering::SeqCst);
        let (_, control) = JobControl::new();
        let run = Arc::new(ActiveRun {
            request: StageRequest {
                row: eidetic::ManifestId::of_blob(b"test"),
                session: "test".into(),
                loader: "test".into(),
                range: LayerRange { first: 0, end: 1 },
            },
            control,
            runtime: Mutex::new(StageRuntime {
                stage: None,
                guard: Some(access.try_background().unwrap()),
                busy: Some(BusyPermit(service.busy.clone())),
                loading_device: None,
            }),
            closed: AtomicBool::new(false),
            finished: AtomicBool::new(false),
            seen_live: AtomicBool::new(false),
            last_call: Mutex::new(std::time::Instant::now()),
        });
        let key = RunKey {
            job: JobId([2; 32]),
            lease: LeaseId([3; 32]),
        };
        service
            .state
            .write()
            .unwrap()
            .active
            .insert(key, run.clone());
        let worker = run.clone();
        let (entered, received) = std::sync::mpsc::channel();
        let (release, released) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            let _work = worker.runtime.lock().unwrap();
            entered.send(()).unwrap();
            released.recv().unwrap();
        });
        received.recv_timeout(Duration::from_secs(1)).unwrap();
        drop(RunRegistration {
            service: service.clone(),
            key,
            run: run.clone(),
            closed: false,
        });
        assert!(!service.is_active(key.job, key.lease));
        assert!(access.is_occupied());
        assert!(!service.private_drained());
        release.send(()).unwrap();
        thread.join().unwrap();
        tokio::time::timeout(Duration::from_secs(2), async {
            while !service.private_drained() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
        assert!(!access.is_occupied());
        service.shutdown().await.unwrap();
        network.close().await.unwrap();
    }
}
