// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use burn::tensor::Device;
use burn_wgpu::{Wgpu, WgpuDevice};
use cubecl::wgpu::WgpuDeviceKind;
use distillery::{
    BURN_REMOTE_RESOURCE, BlobCustody, Distillery, RemoteSessionService, RemoteSessionSettings,
    RetentionSettings,
};
use esp::embed::EmbeddingProvider;
use esp::embed::bert::{BertEmbeddingProvider, load_cpu};
use identity::{IdentityProvider, InMemoryProvider};
use mesh::{
    DeterminismClass, DeviceConditions, HostFacts, JobId, JobSpec, LeaseId, LeasePolicy,
    LeaseTerms, MESH_AUTHOR_SALT, MemoryBlobSpace, MeshEvent, MeshStore, ReclaimReason,
    RemoteSessionClaim, ResourceId, ResourceRegistry, SyncedMesh,
};
use distillery::mesh_host::{HostConfig, ManualClock, MeshHost, ObservedConditions, Step};
use muniment::MemoryBackend;
use serde::Serialize;
use serde_json::{Value, json};
use transport::P2pandaTransport;

const MESH: [u8; 32] = [0x72; 32];
const NOW_MS: u64 = 5_000;
const REFERENCE_FIRST_8: [f32; 8] = [
    0.045927152,
    -0.0018069973,
    0.02857656,
    0.07433602,
    0.0718927,
    0.053076733,
    -0.010092336,
    0.0085868575,
];
const TOLERANCE: f32 = 0.0001;

type Works = Distillery<MemoryBackend>;

struct NoCustody;

impl BlobCustody for NoCustody {
    fn collect<'a>(
        &'a self,
        _blobs: &'a [mesh::BlobRef],
    ) -> Pin<Box<dyn Future<Output = Result<u64, String>> + Send + 'a>> {
        Box::pin(async { Ok(0) })
    }
}

struct ActiveRun {
    job: JobId,
    lease: LeaseId,
    epoch: u32,
    observed_steps: Vec<Step>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Local,
    Remote,
}

impl Mode {
    fn label(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Remote => "remote",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct AllocatorSnapshot {
    number_allocs: u64,
    bytes_in_use: u64,
    bytes_padding: u64,
    bytes_reserved: u64,
}

impl AllocatorSnapshot {
    fn capture(device: &WgpuDevice) -> Result<Self, String> {
        let usage = cubecl::Device::from(device.clone()).client().memory_usage();
        Ok(Self {
            number_allocs: usage.number_allocs,
            bytes_in_use: usage.bytes_in_use,
            bytes_padding: usage.bytes_padding,
            bytes_reserved: usage.bytes_reserved,
        })
    }

    fn active_matches(&self, baseline: &Self) -> bool {
        self.number_allocs == baseline.number_allocs && self.bytes_in_use == baseline.bytes_in_use
    }

    fn active_exceeds(&self, baseline: &Self) -> bool {
        self.number_allocs > baseline.number_allocs && self.bytes_in_use > baseline.bytes_in_use
    }
}

async fn await_allocator_baseline(
    device: &WgpuDevice,
    baseline: &AllocatorSnapshot,
    stage: &str,
) -> Result<(AllocatorSnapshot, f64), String> {
    let started = Instant::now();
    let mut last = AllocatorSnapshot::capture(device)?;
    for _ in 0..400 {
        if last.active_matches(baseline) {
            return Ok((last, started.elapsed().as_secs_f64() * 1_000.0));
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
        last = AllocatorSnapshot::capture(device)?;
    }
    Err(format!(
        "CubeCL allocator did not return to baseline after {stage}: baseline={baseline:?}, last={last:?}"
    ))
}

fn usage() -> String {
    "usage: distillery-remote-minilm-fixture [local|remote] <model-dir> <input> [cancellation-batch]".into()
}

fn feature_receipt() -> Value {
    json!({
        "fusion": cfg!(feature = "fusion"),
        "autotune": cfg!(feature = "autotune"),
        "fusion_autotune": cfg!(feature = "fusion-autotune")
    })
}

fn backend_profile() -> &'static str {
    match (cfg!(feature = "fusion"), cfg!(feature = "autotune")) {
        (false, false) => "plain",
        (true, false) => "fusion-only",
        (false, true) => "autotune-only",
        (true, true) => "fusion-autotune",
    }
}

fn report_stage(stage: &str) {
    eprintln!("distillery-remote-minilm stage: {stage}");
    let hold_ms = std::env::var("DISTILLERY_REMOTE_STAGE_HOLD_MS")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or_default();
    if hold_ms > 0 {
        std::thread::sleep(Duration::from_millis(hold_ms));
    }
}

fn max_abs_error(left: &[f32], right: &[f32]) -> Result<f32, String> {
    if left.len() != right.len() {
        return Err(format!(
            "numerical comparison length mismatch: {} != {}",
            left.len(),
            right.len()
        ));
    }
    if !left.iter().chain(right).all(|value| value.is_finite()) {
        return Err("numerical comparison contains a non-finite value".to_string());
    }
    Ok(left
        .iter()
        .zip(right)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f32::max))
}

fn l2_norm(values: &[f32]) -> f32 {
    values.iter().map(|value| value * value).sum::<f32>().sqrt()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

async fn await_run(
    works: &mut Works,
    service: &RemoteSessionService<Wgpu>,
    job: JobId,
    minimum_epoch: u32,
) -> Result<ActiveRun, String> {
    let mut lease = None;
    let mut epoch = None;
    let mut observed_steps = Vec::new();
    for _ in 0..400 {
        let steps = works.tick().await.map_err(|error| error.to_string())?;
        for step in &steps {
            if let Step::Granted {
                job: granted_job,
                epoch: granted_epoch,
                lease: granted_lease,
            } = step
                && *granted_job == job
                && *granted_epoch >= minimum_epoch
            {
                lease = Some(*granted_lease);
                epoch = Some(*granted_epoch);
            }
        }
        observed_steps.extend(steps);
        if let (Some(lease), Some(epoch)) = (lease, epoch)
            && service.is_active(job, lease)
        {
            return Ok(ActiveRun {
                job,
                lease,
                epoch,
                observed_steps,
            });
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    Err(format!(
        "host did not activate job {} at epoch >= {minimum_epoch}: {observed_steps:#?}",
        hex(&job.0)
    ))
}

async fn reclaim(
    works: &mut Works,
    conditions: &ObservedConditions,
    service: &RemoteSessionService<Wgpu>,
    run: &ActiveRun,
) -> Result<(bool, bool, Vec<Step>), String> {
    conditions.in_use();
    let first = works.tick().await.map_err(|error| error.to_string())?;
    let awaiting_stop = first
        .iter()
        .any(|step| matches!(step, Step::AwaitingStop { job } if *job == run.job));
    let reclaimed_too_early = first.iter().any(
        |step| matches!(step, Step::Reclaimed { job, lease, .. } if *job == run.job && *lease == run.lease),
    );
    if !awaiting_stop || reclaimed_too_early {
        return Err(format!(
            "reclaim did not expose stop-before-fact ordering: {first:#?}"
        ));
    }

    let mut observed = first;
    let mut reclaimed = false;
    for _ in 0..400 {
        let steps = works.tick().await.map_err(|error| error.to_string())?;
        reclaimed |= steps.iter().any(
            |step| matches!(step, Step::Reclaimed { job, lease, .. } if *job == run.job && *lease == run.lease),
        );
        observed.extend(steps);
        if reclaimed && service.session_count(run.job, run.lease).await == 0 {
            return Ok((awaiting_stop, reclaimed, observed));
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    Err(format!(
        "reclaim did not close the lease-bound session: {observed:#?}"
    ))
}

/// Owner reclaim of every run on the device, for shutdown. Unlike [`reclaim`] it does not
/// assert stop-before-fact ordering, which the first reclaim already proves; it waits for
/// `run`'s lease to end and its sessions to close.
async fn reclaim_for_shutdown(
    works: &mut Works,
    conditions: &ObservedConditions,
    service: &RemoteSessionService<Wgpu>,
    run: &ActiveRun,
) -> Result<bool, String> {
    conditions.in_use();
    let mut observed = Vec::new();
    let mut ended = false;
    for _ in 0..400 {
        let steps = works.tick().await.map_err(|error| error.to_string())?;
        ended |= steps.iter().any(|step| {
            matches!(step, Step::Reclaimed { job, lease, .. } | Step::Completed { job, lease: Some(lease) }
                if *job == run.job && *lease == run.lease)
        });
        observed.extend(steps);
        if ended && service.session_count(run.job, run.lease).await == 0 {
            return Ok(true);
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    Err(format!(
        "shutdown reclaim did not end the kept lease: {observed:#?}"
    ))
}

async fn post_job(
    works: &Works,
    poster_key: &identity::Ed25519Keypair,
    request: &mesh::BlobRef,
    nonce: u64,
) -> Result<JobId, String> {
    let posted = works
        .host()
        .synced()
        .author(
            poster_key,
            &MeshEvent::JobPostedV2 {
                spec: Box::new(
                    JobSpec::simple(
                        ResourceId::parse(BURN_REMOTE_RESOURCE)
                            .map_err(|error| error.to_string())?,
                        "request",
                        request.clone(),
                        "receipt",
                        512,
                        DeterminismClass::Observed,
                    )
                    .leased(LeaseTerms::new(600_000, 60_000)),
                ),
                nonce,
                at_ms: NOW_MS,
            },
        )
        .await
        .map_err(|error| error.to_string())?;
    Ok(JobId(*posted.hash.as_bytes()))
}

/// Tick until the host has lost `run`'s lease and no Burn session remains on it.
async fn await_lease_lost(
    works: &mut Works,
    service: &RemoteSessionService<Wgpu>,
    run: &ActiveRun,
) -> Result<Vec<Step>, String> {
    let mut observed = Vec::new();
    let mut lost = false;
    for _ in 0..400 {
        let steps = works.tick().await.map_err(|error| error.to_string())?;
        lost |= steps.iter().any(
            |step| matches!(step, Step::LeaseLost { job, lease } if *job == run.job && *lease == run.lease),
        );
        observed.extend(steps);
        if lost && service.session_count(run.job, run.lease).await == 0 {
            return Ok(observed);
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    Err(format!(
        "the revoked lease was not lost and closed: {observed:#?}"
    ))
}

async fn remote_provider(
    client: &P2pandaTransport,
    server: &P2pandaTransport,
    service: &RemoteSessionService<Wgpu>,
    poster_key: &identity::Ed25519Keypair,
    run: &ActiveRun,
    model_dir: &Path,
) -> Result<(Arc<BertEmbeddingProvider>, f64), String> {
    let credential = RemoteSessionClaim::signed(
        poster_key,
        MESH,
        run.job,
        run.lease,
        run.epoch,
        service.server_peer(),
        0,
    )
    .encode()
    .map_err(|error| error.to_string())?;
    let endpoint = client
        .protocol_endpoint()
        .endpoint()
        .await
        .map_err(|error| error.to_string())?;
    let server_addr = server
        .endpoint_addr()
        .await
        .map_err(|error| error.to_string())?;
    let device = Device::remote_iroh_authorized(&endpoint, server_addr, 0, credential);
    let started = Instant::now();
    let provider = BertEmbeddingProvider::load(model_dir, device)
        .map_err(|error| format!("load remote provider: {error}"))?;
    Ok((
        Arc::new(provider),
        started.elapsed().as_secs_f64() * 1_000.0,
    ))
}

fn numerical_receipt(output: &[f32], reference: &[f32]) -> Result<Value, String> {
    let first_8 = output.iter().take(8).copied().collect::<Vec<_>>();
    let browser_reference_error = max_abs_error(&first_8, &REFERENCE_FIRST_8)?;
    let native_reference_error = max_abs_error(output, reference)?;
    let norm = l2_norm(output);
    let passes = output.len() == 384
        && output.iter().all(|value| value.is_finite())
        && (norm - 1.0).abs() <= TOLERANCE
        && browser_reference_error <= TOLERANCE
        && native_reference_error <= TOLERANCE;
    if !passes {
        return Err(format!(
            "MiniLM numerical gate failed: dims={}, norm={norm}, browser max error={browser_reference_error}, native max error={native_reference_error}",
            output.len()
        ));
    }
    Ok(json!({
        "dimensions": output.len(),
        "all_finite": true,
        "l2_norm": norm,
        "first_8": first_8,
        "browser_reference_first_8": REFERENCE_FIRST_8,
        "browser_reference_max_abs_error": browser_reference_error,
        "native_reference_max_abs_error": native_reference_error,
        "tolerance": TOLERANCE,
        "passes": true
    }))
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os().skip(1);
    let first = arguments.next().ok_or_else(usage)?;
    let (mode, model_dir) = match first.to_str() {
        Some("local") => (
            Mode::Local,
            arguments.next().map(PathBuf::from).ok_or_else(usage)?,
        ),
        Some("remote") => (
            Mode::Remote,
            arguments.next().map(PathBuf::from).ok_or_else(usage)?,
        ),
        _ => (Mode::Remote, PathBuf::from(first)),
    };
    let input = arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(usage)?;
    let cancellation_argument = arguments.next();
    let local_cancellation_argument = mode == Mode::Local && cancellation_argument.is_some();
    let cancellation_batch = cancellation_argument
        .map(|value| {
            value
                .into_string()
                .map_err(|_| usage())?
                .parse::<usize>()
                .map_err(|_| usage())
        })
        .transpose()?
        .unwrap_or(512);
    if arguments.next().is_some() || cancellation_batch == 0 || local_cancellation_argument {
        return Err(usage());
    }

    match mode {
        Mode::Local => run_local(model_dir, input).await,
        Mode::Remote => run_remote(model_dir, input, cancellation_batch).await,
    }
}

async fn run_local(model_dir: PathBuf, input: String) -> Result<(), String> {
    let native_load_started = Instant::now();
    let native = load_cpu(&model_dir).map_err(|error| format!("load native control: {error}"))?;
    let native_load_ms = native_load_started.elapsed().as_secs_f64() * 1_000.0;
    let native_started = Instant::now();
    let native_output = native
        .embed_one(&input)
        .map_err(|error| error.to_string())?;
    let native_execution_ms = native_started.elapsed().as_secs_f64() * 1_000.0;

    let wgpu_load_started = Instant::now();
    let provider = BertEmbeddingProvider::load(
        &model_dir,
        Device::wgpu(burn::tensor::DeviceKind::DiscreteGpu(0)),
    )
    .map_err(|error| format!("load local WGPU provider: {error}"))?;
    let wgpu_load_ms = wgpu_load_started.elapsed().as_secs_f64() * 1_000.0;
    let wgpu_started = Instant::now();
    let output = tokio::time::timeout(Duration::from_secs(30), provider.embed_one_async(&input))
        .await
        .map_err(|_| "local WGPU MiniLM execution timed out".to_string())?
        .map_err(|error| error.to_string())?;
    let wgpu_execution_ms = wgpu_started.elapsed().as_secs_f64() * 1_000.0;
    let numerical = numerical_receipt(&output, &native_output)?;

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema": "distillery.remote-minilm-matrix-row/v1",
            "source": {
                "commit": std::env::var("DISTILLERY_REMOTE_PROBE_COMMIT").unwrap_or_else(|_| "unknown".into()),
                "owned_paths_dirty": std::env::var("DISTILLERY_REMOTE_PROBE_DIRTY").unwrap_or_else(|_| "unknown".into())
            },
            "mode": Mode::Local.label(),
            "backend_profile": backend_profile(),
            "features": feature_receipt(),
            "model": {
                "id": "sentence-transformers/all-MiniLM-L6-v2",
                "revision": "1110a243fdf4706b3f48f1d95db1a4f5529b4d41",
                "model_dir": model_dir,
                "input": input,
                "weights_bytes": 90_868_376_u64,
                "weights_sha256": "53aa51172d142c89d9012cce15ae4d6cc0ca6895895114379cacb4fab128d9db"
            },
            "numerical": numerical,
            "timings_ms": {
                "wgpu_load": wgpu_load_ms,
                "wgpu_execution": wgpu_execution_ms,
                "native_control_load": native_load_ms,
                "native_control_execution": native_execution_ms
            },
            "passes": true
        }))
        .map_err(|error| error.to_string())?
    );
    Ok(())
}

async fn run_remote(
    model_dir: PathBuf,
    input: String,
    cancellation_batch: usize,
) -> Result<(), String> {
    report_stage("allocator-baseline");
    let server_device = WgpuDevice::new(WgpuDeviceKind::DiscreteGpu(0));
    let allocator_baseline = AllocatorSnapshot::capture(&server_device)?;
    report_stage("bind-peers");
    let poster_provider = InMemoryProvider::from_seed([31; 32]);
    let server_provider = InMemoryProvider::from_seed([32; 32]);
    let poster_key = poster_provider
        .derive_keypair(MESH_AUTHOR_SALT)
        .map_err(|error| error.to_string())?;
    let server_key = server_provider
        .derive_keypair(MESH_AUTHOR_SALT)
        .map_err(|error| error.to_string())?;
    let server_transport = Arc::new(
        P2pandaTransport::builder(server_provider.master_keypair())
            .gossip()
            .bind()
            .await
            .map_err(|error| error.to_string())?,
    );
    let client_transport = P2pandaTransport::builder(poster_provider.master_keypair())
        .bind()
        .await
        .map_err(|error| error.to_string())?;
    let server_endpoint = server_transport
        .protocol_endpoint()
        .endpoint()
        .await
        .map_err(|error| error.to_string())?;
    let client_endpoint = client_transport
        .protocol_endpoint()
        .endpoint()
        .await
        .map_err(|error| error.to_string())?;
    if server_endpoint.id() == client_endpoint.id() {
        return Err("the remote receipt requires two distinct p2panda peers".into());
    }

    let clock = Arc::new(ManualClock::at(NOW_MS));
    let service = RemoteSessionService::<Wgpu>::mount(
        &server_transport,
        vec![cubecl::Device::Wgpu(server_device.clone())],
        MESH,
        server_key.public_key().to_bytes(),
        clock.clone(),
        LeasePolicy { max_skew_ms: 0 },
        RemoteSessionSettings::gpu(512, Duration::from_millis(5)),
    )
    .await
    .map_err(|error| error.to_string())?;
    report_stage("service-mounted");

    let (endpoint, gossip) = server_transport
        .sync_parts()
        .ok_or_else(|| "server transport did not expose sync parts".to_string())?;
    let synced = SyncedMesh::join(endpoint, gossip, MeshStore::in_memory(), MESH)
        .await
        .map_err(|error| error.to_string())?;
    let space = Arc::new(MemoryBlobSpace::in_memory());
    let request = space
        .put(b"MiniLM over a lease-bound Burn Remote session")
        .await
        .map_err(|error| error.to_string())?;
    let conditions = Arc::new(ObservedConditions::spare());
    let mut registry = ResourceRegistry::new();
    registry
        .register(service.resource())
        .map_err(|error| error.to_string())?;
    let mut config = HostConfig::supervised(space.clone());
    config.registry = registry;
    config.clock = clock;
    config.conditions = conditions.clone();
    config.facts = HostFacts {
        memory_mib: 16_384,
        gpu: true,
    };
    config.policy = mesh::DevicePolicy::conservative();
    // Room for the second-live-lease stage: its two leases plus a re-grant of the first job.
    // The earlier stages post one job at a time, so they are unaffected.
    config.policy.max_concurrent_jobs = 3;
    config.lease = LeasePolicy { max_skew_ms: 0 };
    let host = MeshHost::new(synced, server_key.clone(), config);
    let mut works = Distillery::new(host, Arc::new(NoCustody), RetentionSettings::default());
    works.attach_remote_sessions(service.clone());

    works
        .host()
        .synced()
        .author(
            &poster_key,
            &MeshEvent::DeviceAttested {
                attestation: Box::new(
                    poster_provider
                        .attest_derived_key(MESH_AUTHOR_SALT)
                        .map_err(|error| error.to_string())?,
                ),
            },
        )
        .await
        .map_err(|error| error.to_string())?;
    works
        .host()
        .synced()
        .author(
            &server_key,
            &MeshEvent::DeviceAttested {
                attestation: Box::new(
                    server_provider
                        .attest_derived_key(MESH_AUTHOR_SALT)
                        .map_err(|error| error.to_string())?,
                ),
            },
        )
        .await
        .map_err(|error| error.to_string())?;
    let posted = works
        .host()
        .synced()
        .author(
            &poster_key,
            &MeshEvent::JobPostedV2 {
                spec: Box::new(
                    JobSpec::simple(
                        ResourceId::parse(BURN_REMOTE_RESOURCE)
                            .map_err(|error| error.to_string())?,
                        "request",
                        request.clone(),
                        "receipt",
                        512,
                        DeterminismClass::Observed,
                    )
                    .leased(LeaseTerms::new(600_000, 60_000)),
                ),
                nonce: 1,
                at_ms: NOW_MS,
            },
        )
        .await
        .map_err(|error| error.to_string())?;
    let job = JobId(*posted.hash.as_bytes());
    let first_run = await_run(&mut works, &service, job, 0).await?;
    report_stage("first-lease-active");

    let (remote, remote_load_ms) = remote_provider(
        &client_transport,
        &server_transport,
        &service,
        &poster_key,
        &first_run,
        &model_dir,
    )
    .await?;
    report_stage("first-provider-loaded");
    let first_started = Instant::now();
    let remote_output =
        tokio::time::timeout(Duration::from_secs(30), remote.embed_one_async(&input))
            .await
            .map_err(|_| "first remote MiniLM execution timed out".to_string())?
            .map_err(|error| error.to_string())?;
    report_stage("first-remote-executed");
    let first_execution_ms = first_started.elapsed().as_secs_f64() * 1_000.0;
    let native_load_started = Instant::now();
    let native = load_cpu(&model_dir).map_err(|error| format!("load native control: {error}"))?;
    let native_load_ms = native_load_started.elapsed().as_secs_f64() * 1_000.0;
    let native_started = Instant::now();
    let native_output = native
        .embed_one(&input)
        .map_err(|error| error.to_string())?;
    let native_execution_ms = native_started.elapsed().as_secs_f64() * 1_000.0;
    let first_numerical = numerical_receipt(&remote_output, &native_output)?;
    report_stage("native-control-passed");
    let allocator_after_first_execute = AllocatorSnapshot::capture(&server_device)?;
    if !allocator_after_first_execute.active_exceeds(&allocator_baseline) {
        return Err(format!(
            "MiniLM execution did not force observable CubeCL allocations: baseline={allocator_baseline:?}, after={allocator_after_first_execute:?}"
        ));
    }
    let sessions_before_reclaim = service.session_count(job, first_run.lease).await;
    if sessions_before_reclaim != 1 {
        return Err(format!(
            "expected one live model session before reclaim, got {sessions_before_reclaim}"
        ));
    }

    let cancellation_provider = remote.clone();
    let cancellation_input = input.clone();
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let cancellation = tokio::spawn(async move {
        let texts = vec![cancellation_input; cancellation_batch];
        let refs = texts.iter().map(String::as_str).collect::<Vec<_>>();
        let _ = started_tx.send(());
        cancellation_provider.embed_async(&refs).await
    });
    started_rx
        .await
        .map_err(|_| "cancellation request did not start".to_string())?;
    tokio::time::sleep(Duration::from_millis(25)).await;
    let in_flight_at_reclaim = !cancellation.is_finished();
    if !in_flight_at_reclaim {
        return Err(format!(
            "the {cancellation_batch}-row MiniLM batch finished before reclaim; increase cancellation-batch"
        ));
    }
    let (awaiting_stop, reclaimed, _) =
        reclaim(&mut works, &conditions, &service, &first_run).await?;
    let cancellation_result = tokio::time::timeout(Duration::from_secs(30), cancellation)
        .await
        .map_err(|_| "the cancelled MiniLM request hung".to_string())?
        .map_err(|error| format!("the cancelled MiniLM task panicked: {error}"))?;
    let cancellation_error = match cancellation_result {
        Ok(_) => return Err("the in-flight MiniLM request completed after reclaim".into()),
        Err(error) => error.to_string(),
    };
    drop(remote);
    let allocator_immediate_after_first_reclaim = AllocatorSnapshot::capture(&server_device)?;
    let (allocator_after_first_reclaim, first_cleanup_wait_ms) =
        await_allocator_baseline(&server_device, &allocator_baseline, "first reclaim").await?;
    report_stage("first-reclaim-clean");

    conditions.set(DeviceConditions::spare());
    let recovery_run = await_run(&mut works, &service, job, first_run.epoch + 1).await?;
    if recovery_run.lease == first_run.lease {
        return Err("recovery reused the reclaimed lease".into());
    }
    let (recovered, recovery_load_ms) = remote_provider(
        &client_transport,
        &server_transport,
        &service,
        &poster_key,
        &recovery_run,
        &model_dir,
    )
    .await?;
    report_stage("recovery-provider-loaded");
    let recovery_started = Instant::now();
    let recovery_output =
        tokio::time::timeout(Duration::from_secs(30), recovered.embed_one_async(&input))
            .await
            .map_err(|_| "recovery remote MiniLM execution timed out".to_string())?
            .map_err(|error| error.to_string())?;
    report_stage("recovery-remote-executed");
    let recovery_execution_ms = recovery_started.elapsed().as_secs_f64() * 1_000.0;
    let recovery_numerical = numerical_receipt(&recovery_output, &native_output)?;
    let recovery_vs_first = max_abs_error(&recovery_output, &remote_output)?;
    if recovery_vs_first > TOLERANCE {
        return Err(format!(
            "fresh-session recovery diverged from the first remote run: {recovery_vs_first}"
        ));
    }
    let allocator_after_recovery_execute = AllocatorSnapshot::capture(&server_device)?;
    if !allocator_after_recovery_execute.active_exceeds(&allocator_baseline) {
        return Err(format!(
            "recovery MiniLM execution did not force observable CubeCL allocations: baseline={allocator_baseline:?}, after={allocator_after_recovery_execute:?}"
        ));
    }
    drop(recovered);
    let (_, recovery_reclaimed, _) =
        reclaim(&mut works, &conditions, &service, &recovery_run).await?;
    let final_session_count = service.session_count(job, recovery_run.lease).await;
    let allocator_immediate_after_recovery_reclaim = AllocatorSnapshot::capture(&server_device)?;
    let (allocator_after_recovery_reclaim, recovery_cleanup_wait_ms) =
        await_allocator_baseline(&server_device, &allocator_baseline, "recovery reclaim").await?;
    report_stage("recovery-reclaim-clean");

    let allocator_passes = allocator_after_first_execute.active_exceeds(&allocator_baseline)
        && allocator_after_first_reclaim.active_matches(&allocator_baseline)
        && allocator_after_recovery_execute.active_exceeds(&allocator_baseline)
        && allocator_after_recovery_reclaim.active_matches(&allocator_baseline);
    if !allocator_passes {
        return Err("CubeCL allocator lifecycle gate failed".into());
    }

    // Second live lease (ruling 508): close one lease while another stays live on the same
    // device. The live lease must keep its lease, its session and its tensor values, and the
    // allocator must return to a baseline that still contains it.
    conditions.set(DeviceConditions::spare());
    let kept_job = post_job(&works, &poster_key, &request, 2).await?;
    let kept_run = await_run(&mut works, &service, kept_job, 0).await?;
    let (kept, kept_load_ms) = remote_provider(
        &client_transport,
        &server_transport,
        &service,
        &poster_key,
        &kept_run,
        &model_dir,
    )
    .await?;
    let kept_output = tokio::time::timeout(Duration::from_secs(30), kept.embed_one_async(&input))
        .await
        .map_err(|_| "kept-lease MiniLM execution timed out".to_string())?
        .map_err(|error| error.to_string())?;
    let kept_numerical = numerical_receipt(&kept_output, &native_output)?;
    report_stage("kept-lease-executed");
    // Settle the device before taking the kept lease's baseline. Only the baseline gets this
    // help; the close under test below is observed without any fixture-side wait.
    cubecl::Device::from(server_device.clone())
        .client()
        .sync()
        .await
        .map_err(|error| format!("baseline settle sync failed: {error:?}"))?;
    let allocator_kept_baseline = AllocatorSnapshot::capture(&server_device)?;
    if !allocator_kept_baseline.active_exceeds(&allocator_baseline) {
        return Err(format!(
            "the kept lease holds no observable CubeCL allocations: baseline={allocator_baseline:?}, kept={allocator_kept_baseline:?}"
        ));
    }

    let closed_job = post_job(&works, &poster_key, &request, 3).await?;
    let closed_run = await_run(&mut works, &service, closed_job, 0).await?;
    let (closed, closed_load_ms) = remote_provider(
        &client_transport,
        &server_transport,
        &service,
        &poster_key,
        &closed_run,
        &model_dir,
    )
    .await?;
    let closed_output =
        tokio::time::timeout(Duration::from_secs(30), closed.embed_one_async(&input))
            .await
            .map_err(|_| "closed-lease MiniLM execution timed out".to_string())?
            .map_err(|error| error.to_string())?;
    let closed_numerical = numerical_receipt(&closed_output, &native_output)?;
    let allocator_with_both = AllocatorSnapshot::capture(&server_device)?;
    if !allocator_with_both.active_exceeds(&allocator_kept_baseline) {
        return Err(format!(
            "the second lease added no observable CubeCL allocations: kept={allocator_kept_baseline:?}, both={allocator_with_both:?}"
        ));
    }
    report_stage("second-lease-executed");

    // Close the second lease alone: its holder revokes it, the host loses that lease, and
    // Distillery closes its sessions through burn-remote's targeted close.
    works
        .host()
        .synced()
        .author(
            &server_key,
            &MeshEvent::LeaseRevokedByOwner {
                job: closed_job.0,
                lease: closed_run.lease.0,
                reason: ReclaimReason::Manual,
                at_ms: NOW_MS,
            },
        )
        .await
        .map_err(|error| error.to_string())?;
    let close_steps = await_lease_lost(&mut works, &service, &closed_run).await?;
    drop(closed);
    let allocator_immediate_after_close = AllocatorSnapshot::capture(&server_device)?;
    let (allocator_after_close, close_wait_ms) = await_allocator_baseline(
        &server_device,
        &allocator_kept_baseline,
        "second-lease close",
    )
    .await?;
    report_stage("second-lease-closed");

    let kept_disturbed = close_steps.iter().any(|step| {
        matches!(step, Step::LeaseLost { job, .. } | Step::Reclaimed { job, .. } if *job == kept_job)
    });
    let kept_still_active = service.is_active(kept_job, kept_run.lease);
    let kept_sessions_after_close = service.session_count(kept_job, kept_run.lease).await;
    let closed_sessions_after_close = service.session_count(closed_job, closed_run.lease).await;
    if kept_disturbed || !kept_still_active || kept_sessions_after_close != 1 {
        return Err(format!(
            "closing one lease disturbed the other: disturbed={kept_disturbed}, active={kept_still_active}, sessions={kept_sessions_after_close}"
        ));
    }
    let kept_again = tokio::time::timeout(Duration::from_secs(30), kept.embed_one_async(&input))
        .await
        .map_err(|_| "kept-lease MiniLM re-execution timed out".to_string())?
        .map_err(|error| error.to_string())?;
    let kept_again_numerical = numerical_receipt(&kept_again, &native_output)?;
    let kept_vs_before = max_abs_error(&kept_again, &kept_output)?;
    if kept_vs_before != 0.0 {
        return Err(format!(
            "the kept lease's output changed after the other lease closed: {kept_vs_before}"
        ));
    }
    report_stage("kept-lease-intact");

    let kept_reclaimed = reclaim_for_shutdown(&mut works, &conditions, &service, &kept_run).await?;
    drop(kept);
    let allocator_immediate_after_final = AllocatorSnapshot::capture(&server_device)?;
    let (allocator_after_final, final_cleanup_wait_ms) =
        await_allocator_baseline(&server_device, &allocator_baseline, "final reclaim").await?;
    report_stage("final-reclaim-clean");
    let second_lease_passes = allocator_after_close.active_matches(&allocator_kept_baseline)
        && allocator_after_final.active_matches(&allocator_baseline)
        && closed_sessions_after_close == 0
        && kept_reclaimed;
    if !second_lease_passes {
        return Err("second-live-lease gate failed".into());
    }

    works.shutdown().await.map_err(|error| error.to_string())?;
    client_transport
        .close()
        .await
        .map_err(|error| error.to_string())?;
    server_transport
        .close()
        .await
        .map_err(|error| error.to_string())?;
    report_stage("shutdown-complete");

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema": "distillery.remote-minilm-receipt/v1",
            "source": {
                "commit": std::env::var("DISTILLERY_REMOTE_PROBE_COMMIT").unwrap_or_else(|_| "unknown".into()),
                "owned_paths_dirty": std::env::var("DISTILLERY_REMOTE_PROBE_DIRTY").unwrap_or_else(|_| "unknown".into())
            },
            "mode": Mode::Remote.label(),
            "backend_profile": backend_profile(),
            "features": feature_receipt(),
            "model": {
                "id": "sentence-transformers/all-MiniLM-L6-v2",
                "revision": "1110a243fdf4706b3f48f1d95db1a4f5529b4d41",
                "model_dir": model_dir,
                "input": input,
                "weights_bytes": 90_868_376_u64,
                "weights_sha256": "53aa51172d142c89d9012cce15ae4d6cc0ca6895895114379cacb4fab128d9db"
            },
            "topology": {
                "transport": "two distinct application-owned p2panda/Iroh endpoints",
                "server_peer": server_endpoint.id().to_string(),
                "client_peer": client_endpoint.id().to_string(),
                "same_endpoint": false,
                "server_backend": format!("burn-wgpu 0.22.0-pre.4 Wgpu/AutoCompiler DiscreteGpu(0), {}", backend_profile()),
                "client_backend": "Burn Dispatch Remote over authorized Iroh"
            },
            "first_run": {
                "job": hex(&job.0),
                "lease": hex(&first_run.lease.0),
                "epoch": first_run.epoch,
                "claim_and_start_steps": first_run.observed_steps.iter().map(|step| format!("{step:?}")).collect::<Vec<_>>(),
                "sessions_before_reclaim": sessions_before_reclaim,
                "numerical": first_numerical,
                "timings_ms": {
                    "remote_load": remote_load_ms,
                    "remote_execution": first_execution_ms,
                    "native_control_load": native_load_ms,
                    "native_control_execution": native_execution_ms
                }
            },
            "owner_reclaim": {
                "cancellation_batch": cancellation_batch,
                "request_in_flight_at_reclaim": in_flight_at_reclaim,
                "awaiting_stop_before_reclaim_fact": awaiting_stop,
                "reclaimed": reclaimed,
                "session_count_after_reclaim": service.session_count(job, first_run.lease).await,
                "in_flight_request_error": cancellation_error,
                "passes": true
            },
            "fresh_session_recovery": {
                "lease": hex(&recovery_run.lease.0),
                "epoch": recovery_run.epoch,
                "new_lease": recovery_run.lease != first_run.lease,
                "numerical": recovery_numerical,
                "max_abs_error_vs_first_remote": recovery_vs_first,
                "timings_ms": {
                    "remote_load": recovery_load_ms,
                    "remote_execution": recovery_execution_ms
                },
                "reclaimed_for_shutdown": recovery_reclaimed,
                "final_session_count": final_session_count,
                "passes": recovery_reclaimed && final_session_count == 0
            },
            "second_live_lease": {
                "scope": "one lease closed by its holder's revoke while another stays live on the same server device",
                "kept_job": hex(&kept_job.0),
                "kept_lease": hex(&kept_run.lease.0),
                "closed_job": hex(&closed_job.0),
                "closed_lease": hex(&closed_run.lease.0),
                "close_steps": close_steps.iter().map(|step| format!("{step:?}")).collect::<Vec<_>>(),
                "closed_sessions_after_close": closed_sessions_after_close,
                "kept_disturbed_by_close": kept_disturbed,
                "kept_still_active_after_close": kept_still_active,
                "kept_sessions_after_close": kept_sessions_after_close,
                "kept_numerical": kept_numerical,
                "closed_numerical": closed_numerical,
                "kept_after_close_numerical": kept_again_numerical,
                "kept_max_abs_error_vs_before_close": kept_vs_before,
                "kept_reclaimed_for_shutdown": kept_reclaimed,
                "allocator": {
                    "kept_baseline_settled_by_fixture_sync": allocator_kept_baseline,
                    "with_both": allocator_with_both,
                    "immediate_after_close": allocator_immediate_after_close,
                    "after_close": allocator_after_close,
                    "close_wait_ms": close_wait_ms,
                    "immediate_after_final_reclaim": allocator_immediate_after_final,
                    "after_final_reclaim": allocator_after_final,
                    "final_cleanup_wait_ms": final_cleanup_wait_ms
                },
                "timings_ms": {
                    "kept_remote_load": kept_load_ms,
                    "closed_remote_load": closed_load_ms
                },
                "passes": second_lease_passes
            },
            "physical_gpu_allocation_release": {
                "measured": false,
                "allocator_level_measured": true,
                "claim": "lease reclamation closes the Burn session, CubeCL active allocations return to baseline, and a fresh WGPU-backed session recovers exact model behavior",
                "remaining_gate": "process- or driver-level VRAM telemetry; allocator reserved bytes are cache policy rather than live-allocation truth"
            },
            "cubecl_allocator_release": {
                "scope": "ComputeClient memory usage aggregated across every server-device stream",
                "baseline": allocator_baseline,
                "after_first_execute": allocator_after_first_execute,
                "immediate_after_first_reclaim": allocator_immediate_after_first_reclaim,
                "after_first_reclaim_cleanup": allocator_after_first_reclaim,
                "first_cleanup_wait_ms": first_cleanup_wait_ms,
                "after_recovery_execute": allocator_after_recovery_execute,
                "immediate_after_recovery_reclaim": allocator_immediate_after_recovery_reclaim,
                "after_recovery_reclaim_cleanup": allocator_after_recovery_reclaim,
                "recovery_cleanup_wait_ms": recovery_cleanup_wait_ms,
                "required": {
                    "model_execution_exceeds_baseline": true,
                    "number_allocs_returns_to_baseline": true,
                    "bytes_in_use_returns_to_baseline": true,
                    "bytes_reserved_returns_to_baseline": false
                },
                "driver_vram_claimed": false,
                "passes": allocator_passes
            },
            "passes": true
        }))
        .map_err(|error| error.to_string())?
    );
    Ok(())
}

#[cfg(test)]
mod verification_tests {
    use super::*;

    fn finite_receipt_fixture() -> Vec<f32> {
        let mut output = vec![0.0; 384];
        output[..8].copy_from_slice(&REFERENCE_FIRST_8);
        let first_8_squared = REFERENCE_FIRST_8.iter().map(|value| value * value).sum::<f32>();
        assert!(first_8_squared < 1.0);
        output[8] = (1.0 - first_8_squared).sqrt();
        output
    }

    #[test]
    fn error_accepts_finite_equal_lengths() {
        assert_eq!(max_abs_error(&[0.0, -1.0], &[0.0, -0.5]).unwrap(), 0.5);
    }

    #[test]
    fn error_rejects_non_finite_values_on_either_side() {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(max_abs_error(&[value], &[0.0])
                .unwrap_err()
                .contains("non-finite"));
            assert!(max_abs_error(&[0.0], &[value])
                .unwrap_err()
                .contains("non-finite"));
        }
    }

    #[test]
    fn error_rejects_unequal_lengths() {
        assert!(max_abs_error(&[0.0], &[]).unwrap_err().contains("length mismatch"));
        assert!(max_abs_error(&[], &[0.0]).unwrap_err().contains("length mismatch"));
    }

    #[test]
    fn receipt_accepts_finite_native_reference() {
        let output = finite_receipt_fixture();
        let receipt = numerical_receipt(&output, &output).unwrap();
        assert_eq!(receipt["dimensions"], 384);
        assert_eq!(receipt["native_reference_max_abs_error"], 0.0);
        assert_eq!(receipt["passes"], true);
    }

    #[test]
    fn receipt_rejects_non_finite_native_reference_beyond_first_eight() {
        let output = finite_receipt_fixture();
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut reference = output.clone();
            reference[383] = value;
            assert!(numerical_receipt(&output, &reference)
                .unwrap_err()
                .contains("non-finite"));
        }
    }

    #[test]
    fn receipt_rejects_wrong_lengths_including_equal_truncation() {
        let output = finite_receipt_fixture();
        assert!(numerical_receipt(&output, &output[..383]).is_err());
        assert!(numerical_receipt(&output[..383], &output).is_err());
        assert!(numerical_receipt(&output[..383], &output[..383]).is_err());
        assert!(numerical_receipt(&[], &[]).is_err());
    }
}
