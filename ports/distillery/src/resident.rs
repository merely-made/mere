// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Long-lived Distillery lifecycle and its durable blob custody.

use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use crate::lifecycle::{CloseAction, CloseFuture, close_all};
use crate::mesh_host::{MeshHost, Step, TransportBlobSpace};
use mesh::JobBoard;
use mesh::ingress::{IngressConfig, IngressHandle, IngressStats, RecordIngress};
use muniment::Backend;
use tokio::time::{Instant, MissedTickBehavior, interval, interval_at};
use transport::{BlobError, BlobStore, P2pandaTransport};

use crate::{Distillery, DistilleryError, MaintenanceReport, RetentionSettings};

/// Owner-selected cadence and retention behavior for one resident authority.
///
/// There is deliberately no `Default`: a process entry point must project an
/// owner's settings into every cadence rather than quietly choosing policy in
/// the service layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResidentSettings {
    /// How often the non-blocking mesh supervisor gets a turn.
    pub tick_every: Duration,
    /// How often to checkpoint an advanced event frontier. `None` leaves
    /// maintenance as an explicit owner command.
    pub maintenance_every: Option<Duration>,
    /// How often the physical store looks for content with no custody tag.
    pub blob_gc_every: Duration,
    /// What an accepted maintenance checkpoint may release.
    pub retention: RetentionSettings,
}

impl ResidentSettings {
    /// Refuse cadences which would spin continuously or which Tokio cannot run.
    pub fn validate(&self) -> Result<(), ResidentError> {
        if self.tick_every.is_zero() {
            return Err(ResidentError::InvalidSettings(
                "supervisor tick cadence must be greater than zero",
            ));
        }
        if self.maintenance_every.is_some_and(|every| every.is_zero()) {
            return Err(ResidentError::InvalidSettings(
                "maintenance cadence must be greater than zero when enabled",
            ));
        }
        if self.blob_gc_every.is_zero() {
            return Err(ResidentError::InvalidSettings(
                "blob collection cadence must be greater than zero",
            ));
        }
        Ok(())
    }
}

/// Disk-backed bytes and the mesh-scoped custody view over them.
pub struct ResidentStorage {
    root: PathBuf,
    mesh_id: [u8; 32],
    gc_every: Duration,
    blobs: Arc<BlobStore>,
    space: Arc<TransportBlobSpace>,
}

impl ResidentStorage {
    /// Open the persistent collecting store used by a resident mesh host.
    pub async fn open(
        root: impl AsRef<Path>,
        mesh_id: [u8; 32],
        gc_every: Duration,
    ) -> Result<Self, ResidentError> {
        if gc_every.is_zero() {
            return Err(ResidentError::InvalidSettings(
                "blob collection cadence must be greater than zero",
            ));
        }
        let root = root.as_ref().to_path_buf();
        let blobs = Arc::new(BlobStore::open_collecting(&root, gc_every).await?);
        let space = Arc::new(TransportBlobSpace::for_mesh(blobs.clone(), mesh_id));
        Ok(Self {
            root,
            mesh_id,
            gc_every,
            blobs,
            space,
        })
    }

    /// Store root selected by the owner.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Mesh whose custody tags this storage view owns.
    pub fn mesh_id(&self) -> [u8; 32] {
        self.mesh_id
    }

    /// Physical collection cadence active in the opened store.
    pub fn gc_every(&self) -> Duration {
        self.gc_every
    }

    /// Persistent bytes to mount on the p2p transport.
    pub fn blobs(&self) -> Arc<BlobStore> {
        self.blobs.clone()
    }

    /// Mesh-scoped read, write, and custody access for [`MeshHost`].
    pub fn space(&self) -> Arc<TransportBlobSpace> {
        self.space.clone()
    }

    async fn shutdown(self) -> Result<(), BlobError> {
        let Self { blobs, space, .. } = self;
        drop(space);
        blobs.shutdown().await
    }
}

/// One exact observation from the resident authority lifecycle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResidentReceipt {
    /// One supervisor turn and the substrate receipts it returned.
    Tick {
        /// Exact receipts returned by `MeshHost`.
        steps: Vec<Step>,
    },
    /// The event frontier advanced and maintenance committed.
    MaintenanceCompleted(Box<MaintenanceReport>),
    /// The cadence fired, but the frontier had not advanced.
    MaintenanceIdle,
    /// Maintenance was refused or its custody operation failed. This is
    /// observable and non-fatal; a live lease is an expected example.
    MaintenanceFailed {
        /// Display form of the maintenance refusal.
        error: String,
    },
    /// The supervisor itself failed, so the resident loop is ending.
    SupervisorFailed {
        /// Display form of the fatal supervisor error.
        error: String,
    },
    /// The caller's shutdown signal won the lifecycle race.
    StopRequested,
}

/// A resident Distillery failure.
#[derive(Debug, thiserror::Error)]
pub enum ResidentError {
    /// The owner-selected admitted service could not mount.
    #[error("resident ingress: {0}")]
    InvalidIngress(String),
    /// An owner setting cannot form a safe timer.
    #[error("resident settings: {0}")]
    InvalidSettings(&'static str),
    /// The durable custody scope and joined mesh disagree.
    #[error("resident storage belongs to another mesh")]
    MeshMismatch,
    /// The reported settings and already-opened storage cadence disagree.
    #[error("resident settings name another blob collection cadence than the open store")]
    StorageCadenceMismatch,
    /// The Distillery authority could not continue.
    #[error(transparent)]
    Authority(#[from] DistilleryError),
    /// The persistent blob store could not open.
    #[error(transparent)]
    Storage(#[from] BlobError),
    /// Shutdown tried all owned resources and reports every refusal.
    #[error(
        "resident shutdown failed (transport: {transport:?}, authority: {authority:?}, blob store: {storage:?})"
    )]
    Shutdown {
        /// Failure while closing the network endpoint.
        transport: Option<String>,
        /// Failure while stopping work and releasing the mesh store.
        authority: Option<String>,
        /// Failure while flushing and closing persistent blob storage.
        storage: Option<String>,
    },
}

/// The long-lived device authority, including resources whose shutdown order
/// matters.
///
/// Build `host` with [`ResidentStorage::space`] and mount
/// [`ResidentStorage::blobs`] on `transport` before moving all three here. The
/// resident then owns the endpoint lifetime, the joined mesh, and the final
/// blob-store flush as one lifecycle.
pub struct ResidentAuthority<B: Backend + Clone + Send + Sync + 'static> {
    authority: Distillery<B>,
    transport: Arc<P2pandaTransport>,
    storage: ResidentStorage,
    settings: ResidentSettings,
    ingress: Option<IngressHandle<B>>,
    #[cfg(feature = "inference-stages")]
    stages: Vec<Arc<crate::inference_stage::StageService>>,
}

impl<B: Backend + Clone + Send + Sync + 'static> ResidentAuthority<B> {
    /// Take ownership of a fully composed resident host.
    pub fn new(
        host: MeshHost<B>,
        transport: Arc<P2pandaTransport>,
        storage: ResidentStorage,
        settings: ResidentSettings,
    ) -> Result<Self, ResidentError> {
        settings.validate()?;
        if host.synced().mesh_id() != storage.mesh_id() {
            return Err(ResidentError::MeshMismatch);
        }
        if settings.blob_gc_every != storage.gc_every() {
            return Err(ResidentError::StorageCadenceMismatch);
        }
        let authority = Distillery::new(host, storage.space(), settings.retention);
        Ok(Self {
            authority,
            transport,
            storage,
            settings,
            ingress: None,
            #[cfg(feature = "inference-stages")]
            stages: Vec::new(),
        })
    }

    /// The owner settings projected into this run.
    pub fn settings(&self) -> ResidentSettings {
        self.settings
    }

    /// The product authority for posting jobs and read-only projections.
    pub fn authority(&self) -> &Distillery<B> {
        &self.authority
    }

    /// Mutable owner policy access between resident runs.
    pub fn authority_mut(&mut self) -> &mut Distillery<B> {
        &mut self.authority
    }

    /// The live transport, for pairing and reachability projection.
    pub fn transport(&self) -> &Arc<P2pandaTransport> {
        &self.transport
    }

    /// Compose leased stages on this resident's existing mesh and endpoint.
    /// The owner supplies the same clock/lease policy used by its supervisor
    /// and live personal graph read authority. Shutdown drains stages before
    /// closing transport, leases or storage.
    #[cfg(feature = "inference-stages")]
    pub async fn enable_inference_stages(
        &mut self,
        clock: Arc<dyn crate::mesh_host::Clock>,
        policy: mesh::LeasePolicy,
        settings: crate::inference_stage::StageSettings,
        factory: Arc<crate::inference_stage::StageFactory>,
        readers: Arc<crate::inference_stage::StageReaders>,
    ) -> Result<Arc<crate::inference_stage::StageService>, String> {
        if !self.stages.is_empty() {
            return Err("this resident already has a stage service".into());
        }
        if settings.requires.memory_mib > self.authority.host().facts().memory_mib {
            return Err("stage offer exceeds the resident's composed host facts".into());
        }
        let service = crate::inference_stage::StageService::mount(
            self.transport.clone(),
            self.storage.mesh_id(),
            self.authority.host().me(),
            clock,
            policy,
            settings,
            factory,
            readers,
        )
        .await?;
        if let Err(error) = self
            .authority
            .host_mut()
            .register_resource(service.resource())
        {
            service.shutdown().await?;
            return Err(error.to_string());
        }
        service.refresh(self.authority.board().await.map_err(|e| e.to_string())?);
        self.authority.attach_remote_sessions(service.clone());
        self.stages.push(service.clone());
        Ok(service)
    }

    /// Owned private stage workers, for the embedding host's vault lock hook.
    #[cfg(feature = "inference-stages")]
    pub fn inference_stage_services(&self) -> Vec<Arc<crate::inference_stage::StageService>> {
        self.stages.clone()
    }

    /// Opt in to admitted signed-record submission and separately authorized exact
    /// inventory on this resident's endpoint and existing mesh store. One queue and
    /// active-call budget serve both actions. Mounting never binds another endpoint.
    pub async fn enable_record_ingress(
        &mut self,
        config: IngressConfig,
    ) -> Result<(), ResidentError> {
        if self.ingress.is_some() {
            return Err(ResidentError::InvalidSettings(
                "record ingress already mounted",
            ));
        }
        let store = self.authority.host().synced().store().clone();
        let mesh = self.authority.host().synced().mesh_id();
        let service = RecordIngress::new(store, mesh, config)
            .map_err(|e| ResidentError::InvalidIngress(e.to_string()))?;
        self.ingress = Some(
            service
                .mount(self.transport.clone())
                .await
                .map_err(|e| ResidentError::InvalidIngress(e.to_string()))?,
        );
        Ok(())
    }

    /// Read the optional ingress service's own observations.
    pub fn record_ingress_stats(&self) -> Option<IngressStats> {
        self.ingress.as_ref().map(|h| h.service().stats())
    }

    /// The persistent custody surface.
    pub fn storage(&self) -> &ResidentStorage {
        &self.storage
    }

    /// Drive the authority until `shutdown` resolves.
    ///
    /// Receipts are delivered synchronously and in authority order, so an
    /// embeddable view can project them without rebuilding mesh truth. A
    /// maintenance failure remains visible and the supervisor keeps running;
    /// a supervisor failure emits its receipt and ends the run.
    pub async fn run_until<S, O>(
        &mut self,
        shutdown: S,
        mut observe: O,
    ) -> Result<(), ResidentError>
    where
        S: Future<Output = ()>,
        O: FnMut(ResidentReceipt),
    {
        self.run_until_with_board(shutdown, |receipt, _board| observe(receipt))
            .await
    }

    /// Drive the authority until `shutdown` resolves, folding the current
    /// board before delivering each receipt.
    ///
    /// This is the resident-to-projection seam: the board is folded by the
    /// Distillery authority after the real supervisor or maintenance event,
    /// then delivered beside the exact receipt that caused the observation.
    /// An application owner can therefore update a read-only observer in the
    /// same callback without reaching through `MeshHost` or reconstructing
    /// the board from receipts.
    pub async fn run_until_with_board<S, O>(
        &mut self,
        shutdown: S,
        mut observe: O,
    ) -> Result<(), ResidentError>
    where
        S: Future<Output = ()>,
        O: FnMut(ResidentReceipt, JobBoard),
    {
        let mut shutdown = Box::pin(shutdown);
        let mut ticks = interval(self.settings.tick_every);
        ticks.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let mut maintenance = self.settings.maintenance_every.map(|every| {
            let mut interval = interval_at(Instant::now() + every, every);
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            interval
        });

        loop {
            tokio::select! {
                biased;
                _ = &mut shutdown => {
                    observe(ResidentReceipt::StopRequested, self.authority.board().await?);
                    return Ok(());
                }
                _ = ticks.tick() => {
                    match self.authority.tick().await {
                        Ok(steps) => {
                            let board = self.authority.board().await?;
                            observe(ResidentReceipt::Tick { steps }, board);
                        }
                        Err(error) => {
                            observe(ResidentReceipt::SupervisorFailed {
                                error: error.to_string(),
                            }, self.authority.board().await?);
                            return Err(error.into());
                        }
                    }
                }
                _ = async {
                    match maintenance.as_mut() {
                        Some(interval) => {
                            interval.tick().await;
                        }
                        None => std::future::pending::<()>().await,
                    }
                } => {
                    let receipt = match self.authority.maintain_if_advanced().await {
                        Ok(Some(report)) => {
                            ResidentReceipt::MaintenanceCompleted(Box::new(report))
                        }
                        Ok(None) => ResidentReceipt::MaintenanceIdle,
                        Err(error) => ResidentReceipt::MaintenanceFailed {
                            error: error.to_string(),
                        },
                    };
                    observe(receipt, self.authority.board().await?);
                }
            }
        }
    }

    /// Close the endpoint, drop the joined mesh, then flush and close blob
    /// storage. All close attempts run even when an earlier one fails.
    pub async fn shutdown(self) -> Result<(), ResidentError> {
        let Self {
            authority,
            transport,
            storage,
            ingress,
            #[cfg(feature = "inference-stages")]
            stages,
            ..
        } = self;
        let report = close_all(vec![
            #[cfg(feature = "inference-stages")]
            (
                "inference-stages",
                Box::new(move || {
                    Box::pin(async move {
                        let mut errors = Vec::new();
                        for service in stages {
                            if let Err(error) = service.shutdown().await {
                                errors.push(error);
                            }
                        }
                        if errors.is_empty() {
                            Ok(())
                        } else {
                            Err(errors.join("; "))
                        }
                    }) as CloseFuture<'static>
                }) as CloseAction<'static>,
            ),
            (
                "transport",
                Box::new(move || {
                    Box::pin(async move {
                        let ingress_error = match ingress {
                            Some(handle) => handle.shutdown().await.err().map(|e| e.to_string()),
                            None => None,
                        };
                        let transport_error = transport.close().await.err().map(|e| e.to_string());
                        match (ingress_error, transport_error) {
                            (None, None) => Ok(()),
                            (a, b) => Err(format!("ingress: {a:?}; transport: {b:?}")),
                        }
                    }) as CloseFuture<'static>
                }) as CloseAction<'static>,
            ),
            (
                "authority",
                Box::new(move || {
                    Box::pin(async move {
                        authority
                            .shutdown()
                            .await
                            .map_err(|error| error.to_string())
                    }) as CloseFuture<'static>
                }) as CloseAction<'static>,
            ),
            (
                "storage",
                Box::new(move || {
                    Box::pin(
                        async move { storage.shutdown().await.map_err(|error| error.to_string()) },
                    ) as CloseFuture<'static>
                }) as CloseAction<'static>,
            ),
        ])
        .await;
        let transport_error = report.failure("transport").map(str::to_owned);
        let mut authority_error = report.failure("authority").map(str::to_owned);
        #[cfg(feature = "inference-stages")]
        if let Some(error) = report.failure("inference-stages") {
            authority_error = Some(format!(
                "stage shutdown: {error}; host: {authority_error:?}"
            ));
        }
        let storage_error = report.failure("storage").map(str::to_owned);
        if transport_error.is_none() && authority_error.is_none() && storage_error.is_none() {
            Ok(())
        } else {
            Err(ResidentError::Shutdown {
                transport: transport_error,
                authority: authority_error,
                storage: storage_error,
            })
        }
    }
}
