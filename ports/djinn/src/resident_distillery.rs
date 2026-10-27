// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Resident Distillery composition inside Djinn.
//!
//! Distillery owns the works: the supervisor loop, retention maintenance, and
//! blob custody for one mesh. Mesh owns job grammar and leases. Pandect owns
//! the device's lending posture. Personae owns the identity. This module is
//! only the place those authorities are put together, and its whole discipline
//! is that **it invents nothing**.
//!
//! That is a stronger claim than it sounds, so here is what it costs:
//!
//! - **No `DevicePolicy::permissive()`.** The lending policy is converted from
//!   pandect's device-scoped `mesh_lending` block. An enabled lane on a device
//!   with no such block is refused, because a composition that needs a lending
//!   posture and cannot find one must say so rather than lend everything.
//! - **No fabricated condition.** Conditions come from
//!   [`DeviceConditionSensor`], which reports each signal's provenance, and
//!   [`validate_policy_coverage`] refuses before the resident binds if any
//!   enabled rule rests on a signal nobody can supply. Thermal is not sensed on
//!   this platform, so a `max_thermal_c` above zero with no stated temperature
//!   refuses composition — loudly, at start, rather than by never lending.
//! - **No invented capacity.** [`mesh::HostFacts::memory_mib`] is read from the
//!   operating system — `GlobalMemoryStatusEx` on Windows, `MemTotal:` from
//!   `/proc/meminfo` on Linux, the `hw.memsize` sysctl on macOS — and a build
//!   on a target with none of those refuses rather than advertising a number.
//! - **No unearned GPU.** [`mesh::HostFacts::gpu`] follows the resource this
//!   composition actually registered, never the hardware present. A `"gpu"`
//!   trainer is composed only after Distillery's adapter probe says the wgpu
//!   runtime would bind a real discrete adapter — because cubecl, asked for a
//!   discrete GPU it cannot find, substitutes an unclassified adapter instead
//!   of refusing. A machine with a GPU and a CPU trainer on it still
//!   advertises `false`.
//! - **No borrowed identity.** The mesh id is derived under Distillery's own
//!   salt from the same Personae profile Djinn resolved, and the retention
//!   checkpoint authority is that profile's derived mesh author.
//!
//! ## What this lane deliberately does not adopt
//!
//! It builds its own [`transport::P2pandaTransport`] and its own job-byte
//! store, exactly as [`InstalledAuthority::bind_resident`] already does. It
//! does **not** borrow Djinn's shared [`crate::resident_blobs`] custody. Mesh
//! job payloads are governed by a mesh retention policy with its own checkpoint
//! authority, and personal-graph transfers and Knot evidence are governed by
//! their own leases; folding them into one physical store would make one
//! policy's collection decision reach into another policy's bytes.

use std::collections::BTreeSet;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use distillery::{
    ChronicleObserver, ChronicleRevision, Distillery, InstalledAuthority, InstalledSettings,
    ResidentAuthority, ResidentReceipt, ResidentSettings, RetentionSettings,
};
use graphshell_endpoint::door::route::ResidentEndpointRoute;
use graphshell_host::endpoint_catalog::{ResidentEndpointCatalog, ResidentEndpointCatalogError};
use mesh::spec::JobSpec;
use mesh::{AvailabilityPolicy, DevicePolicy, ErasurePolicy, JobBoard, MeshRetentionPolicy};
use muniment::RedbBackend;
use notochord::LocalNetworkPolicy;
use personae::{Ed25519Keypair, ProfileId};
use tokio::sync::Mutex;

use crate::conditions::{DeviceConditionSensor, StatedConditions, validate_policy_coverage};
use crate::settings::{DistilleryLaneSettings, parse_keep_bound, sanitize_profile};

/// Stable first-party route for the resident Distillery Chronicle.
pub const RESIDENT_DISTILLERY_CHRONICLE_ROUTE: &str = "distillery.chronicle";

/// Notice cadence for live Chronicle revisions on the first-party route.
pub const RESIDENT_DISTILLERY_CHRONICLE_NOTICE_POLL: Duration = Duration::from_millis(250);

#[cfg(feature = "inference-stages")]
struct StageVaultLock {
    services: Vec<Arc<distillery::inference_stage::StageService>>,
    profile: ProfileId,
}
#[cfg(feature = "inference-stages")]
impl crate::custody_host::LockHolder for StageVaultLock {
    fn name(&self) -> &str {
        "resident inference stages"
    }
    fn key_profile(&self) -> Option<&ProfileId> {
        Some(&self.profile)
    }
    fn lock(&self) {
        for service in &self.services {
            service.set_private_locked(true);
        }
        // Djinn runs lock holders on its blocking lock path while the Tokio
        // resident workers continue draining their leased computation.
        while self
            .services
            .iter()
            .any(|service| !service.private_drained())
        {
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    fn unlock(
        &self,
        _vault: &dyn personae::IdentityProvider,
    ) -> Result<(), personae::IdentityError> {
        for service in &self.services {
            service.set_private_locked(false);
        }
        Ok(())
    }
}

/// One resident Distillery works, bound to this profile's personal mesh.
pub struct ResidentDistillery {
    resident: ResidentAuthority<RedbBackend>,
    /// The read-only projection materialized from the actual board this
    /// resident opened. It is separate from the work authority: a session can
    /// read it, but cannot reach the job host or mutate a job through it.
    chronicle: ChronicleObserver,
    /// Monotonic observation timebase consumed by the resident-owned
    /// Chronicle. It records lifecycle observations without retaining the
    /// unbounded receipt stream.
    chronicle_tick: u64,
    /// The next materialization version for the resident-owned Chronicle.
    /// The epoch is stable for this resident; generation and revision advance
    /// together for each observation tick.
    chronicle_revision: ChronicleRevision,
    /// Retained because on a single device the poster and the runner are the
    /// same process: this resident authors its own job posts as itself. The
    /// key is already resident inside the supervisor for the lane's whole
    /// life, so keeping the handle exposes nothing the composition did not
    /// already hold.
    author: Ed25519Keypair,
    mesh_id: [u8; 32],
    profile: String,
    mesh_root: PathBuf,
    #[cfg(feature = "inference")]
    inference_root: PathBuf,
    #[cfg(feature = "inference")]
    inference_space: [u8; 32],
    /// The persona-scoped model library the trainer writes artifacts into,
    /// held for the lane's life so the resource and any reader see one handle
    /// on one redb file. Opened for a trainer or a library-backed inference
    /// lane; `None` when neither consumer needs it.
    model_library: Option<Arc<Mutex<RedbBackend>>>,
    gpu_access: distillery::gpu::GpuAccess,
    #[cfg(feature = "inference-stages")]
    stage_network: Option<super::resident_inference::mesh_discovery::StageNetwork>,
    /// The adapter the composed trainer actually runs on, when this lane
    /// composed a GPU trainer.
    ///
    /// Kept because the claim `HostFacts::gpu == true` is only as good as the
    /// evidence behind it, and the evidence is this: the name, backend and
    /// class of the adapter the wgpu runtime reported *before* the device was
    /// constructed. A receipt reads it to prove the run went to the discrete
    /// GPU rather than to whatever cubecl would have substituted.
    #[cfg(feature = "trainer-gpu")]
    trainer_adapter: Option<distillery::GpuAdapterFacts>,
}

impl ResidentDistillery {
    /// Compose discovery, the worker offer and the actor's bounded planner on
    /// this existing mesh. Current graph keys remain the admission boundary.
    #[cfg(feature = "inference-stages")]
    pub async fn prepare_automatic_inference(
        &mut self,
        data_root: &Path,
        personal: &Arc<crate::sync::personal_sync_host::PersonalSyncHost>,
        row: &str,
        device: crate::settings::InferenceDeviceSettings,
        limits: distillery::inference_protocol::AutomaticStageSettings,
        proof: insigne::DerivedKeyAttestation,
    ) -> Result<Arc<distillery::inference_stage::StageChainFactory>, String> {
        use distillery::inference_stage::{automatic::StagePlanner, library_stage_catalog};
        limits.validate()?;
        if self.mesh_id != super::resident_inference::compute_space(personal.graph()) {
            return Err("automatic inference graph differs from the configured graph".into());
        }
        self.resident
            .authority()
            .host()
            .announce(proof.clone())
            .await
            .map_err(|e| e.to_string())?;
        let seals = personal.seals();
        let local = self.author.public_key().to_bytes();
        self.resident
            .authority_mut()
            .host_mut()
            .set_admission(Arc::new(move |job, board| {
                [local, job.posted_by].iter().all(|author| {
                    board
                        .devices()
                        .master_of(author)
                        .is_some_and(|root| seals.current_reader(&root))
                })
            }));
        let row = distillery::inference::parse_model_row(row)?;
        let library = self.inference_model_library(data_root)?;
        let resolved = {
            let mut store = library.lock().await;
            distillery::inference::resolve_library_session(&mut *store, row)
                .await
                .map_err(|e| e.to_string())?
        };
        if resolved.layer_artifacts().is_none() {
            return Err("automatic inference requires a layer-indexed library row".into());
        }
        let half = resolved.weight_format().is_some();
        let loader = match device {
            crate::settings::InferenceDeviceSettings::Cpu if half => "burn-flex",
            crate::settings::InferenceDeviceSettings::Cpu => "burn-ndarray",
            crate::settings::InferenceDeviceSettings::Gpu => "burn-wgpu",
        };
        resolved.capability(loader).map_err(|e| e.to_string())?;
        let memory_mib = u32::try_from(limits.worker_layer_bytes.div_ceil(1 << 20))
            .map_err(|_| "stage layer budget exceeds host memory range")?;
        let requires = mesh::ResourceRequirements {
            memory_mib,
            compute: if device == crate::settings::InferenceDeviceSettings::Gpu {
                mesh::ComputeClass::Gpu
            } else {
                mesh::ComputeClass::Cpu
            },
        };
        if limits.worker_layer_bytes > 0 {
            let service = self
                .enable_inference_stages(
                    data_root,
                    personal,
                    loader,
                    limits.worker_layer_bytes,
                    limits.transport(requires),
                )
                .await?;
            service.set_catalog(library_stage_catalog(
                library,
                loader.into(),
                (device == crate::settings::InferenceDeviceSettings::Gpu)
                    .then(|| self.gpu_access.clone()),
                limits.worker_layer_bytes,
            ));
        }
        let transport = self.resident.transport().clone();
        let blobs = self.resident.storage().blobs();
        self.resident
            .authority_mut()
            .host_mut()
            .set_courier(Arc::new(distillery::mesh_host::TransportCourier::for_mesh(
                transport.clone(),
                blobs,
                self.mesh_id,
            )));
        let planner = StagePlanner::new(
            transport,
            self.mesh_id,
            self.author.clone(),
            Arc::new(distillery::mesh_host::SystemClock),
            mesh::LeasePolicy::default(),
            crate::resident_inference::personal::stage_readers(personal),
            limits.clone(),
        )?;
        self.resident
            .authority_mut()
            .attach_stage_planner(planner.clone())?;
        self.stage_network = Some(
            super::resident_inference::mesh_discovery::StageNetwork::open(
                personal.clone(),
                self.resident.transport().clone(),
                proof,
                self.author.clone(),
                &limits,
            )
            .await?,
        );
        Ok(planner.factory(row, loader.into()))
    }
    /// Explicit embedding composition for a stage worker. Uses this profile's
    /// existing library, mesh leases, trainer gate and shared endpoint. The
    /// caller reserves shared/activation/backend memory before giving a stage
    /// its layer/KV budget and installs the vault holder before opening a door.
    #[cfg(feature = "inference-stages")]
    pub async fn enable_inference_stages(
        &mut self,
        data_root: &Path,
        personal: &crate::sync::personal_sync_host::PersonalSyncHost,
        loader: &str,
        resident_layer_budget: u64,
        settings: distillery::inference_stage::StageSettings,
    ) -> Result<Arc<distillery::inference_stage::StageService>, String> {
        let (device, gpu) = match loader {
            "burn-ndarray" => (distillery::inference::InferenceDevice::ndarray(), None),
            "burn-flex" => (distillery::inference::InferenceDevice::flex(), None),
            #[cfg(feature = "inference-gpu")]
            "burn-wgpu" => (
                distillery::inference::discrete_gpu_inference_device()?.0,
                Some(self.gpu_access.clone()),
            ),
            _ => return Err("stage loader is unavailable in this resident build".into()),
        };
        if (settings.requires.compute == mesh::ComputeClass::Gpu) != gpu.is_some() {
            return Err("stage requirements disagree with the composed backend".into());
        }
        let library = self.inference_model_library(data_root)?;
        let factory = distillery::inference_stage::library_stage_factory(
            library,
            device,
            loader.into(),
            gpu,
            resident_layer_budget,
        );
        let policy = self.resident.authority().host().policy();
        let resource =
            distillery::inference_stage::stage_resource(self.author.public_key().to_bytes());
        if (!policy.allowed_resources.is_empty() && !policy.allowed_resources.contains(&resource))
            || (!policy.accepted_checkpoints.is_empty()
                && !policy
                    .accepted_checkpoints
                    .contains(&mesh::CheckpointClass::Restart))
        {
            return Err("mesh lending settings do not permit this device's stage resource and restart class".into());
        }
        self.resident
            .enable_inference_stages(
                Arc::new(distillery::mesh_host::SystemClock),
                mesh::LeasePolicy::default(),
                settings,
                factory,
                crate::resident_inference::personal::stage_readers(personal),
            )
            .await
    }

    /// Register alongside the ask lane's holder. An idle resident with only
    /// stage workers still participates in private-data lock handling.
    #[cfg(feature = "inference-stages")]
    pub fn inference_stage_lock_holder(&self) -> Option<Arc<dyn crate::custody_host::LockHolder>> {
        let services = self.resident.inference_stage_services();
        if services.is_empty() {
            None
        } else {
            Some(Arc::new(StageVaultLock {
                services,
                profile: ProfileId(self.profile.clone()),
            }))
        }
    }

    /// Owner handles for live worker diagnostics and lifecycle verification.
    #[cfg(feature = "inference-stages")]
    pub fn inference_stage_services(&self) -> Vec<Arc<distillery::inference_stage::StageService>> {
        self.resident.inference_stage_services()
    }
    /// Open the works for one already-resolved Personae profile. `identity`
    /// holds exactly [`InstalledAuthority::release_salts`]; `protection` says
    /// where they came from. No vault is opened here (custody isolation
    /// plan, §5.1): the resident captured them from its own open identity or
    /// the custodian released them.
    pub async fn open(
        data_root: &Path,
        profile: &ProfileId,
        identity: Arc<dyn personae::IdentityProvider>,
        protection: String,
        lane: DistilleryLaneSettings,
    ) -> Result<Self, String> {
        Self::open_mesh(data_root, profile, identity, protection, lane, None).await
    }

    /// Automatic inference names a common graph compute space even when the
    /// devices have distinct profile roots. Existing profile meshes stay local.
    pub async fn open_mesh(
        data_root: &Path,
        profile: &ProfileId,
        identity: Arc<dyn personae::IdentityProvider>,
        protection: String,
        lane: DistilleryLaneSettings,
        mesh: Option<[u8; 32]>,
    ) -> Result<Self, String> {
        lane.validate()?;

        // The gate's honesty, checked before anything is opened: a build with
        // no trainer compiled in cannot answer a lane whose settings say it
        // trains. Composing anyway would leave a device advertising work it
        // would then fail, which is the same sin as a permissive policy
        // default, one layer up.
        #[cfg(not(feature = "trainer"))]
        if lane.trainer.is_some() {
            return Err(
                "distillery.trainer states this lane composes a trainer, but this build \
                 carries no trainer; rebuild with the `trainer` feature or set \
                 distillery.trainer to null. A works must not compose settings it cannot \
                 honour."
                    .into(),
            );
        }

        // The same gate, one word narrower: a build may carry the trainer and
        // still not carry the GPU half of it, and the two are separate
        // features because the GPU half pulls wgpu and cubecl's whole runtime
        // in behind burn. Refused at the same point and for the same reason —
        // training on the CPU because the owner's word was unanswerable would
        // be exactly the quiet substitution this lane exists to refuse.
        #[cfg(all(feature = "trainer", not(feature = "trainer-gpu")))]
        if lane
            .trainer
            .as_ref()
            .is_some_and(|trainer| trainer.device.trim() == "gpu")
        {
            return Err(
                "distillery.trainer.device is \"gpu\", but this build carries no GPU \
                 trainer; rebuild with the `trainer-gpu` feature or say \"cpu\". A works \
                 must not compose settings it cannot honour, and training on the CPU \
                 instead would leave the owner's own settings file reading as proof the \
                 device trains on its GPU."
                    .into(),
            );
        }

        // Djinn already resolved which face this device speaks as, and that
        // resolution IS the owner's explicit choice; asking them to configure
        // the same profile a second time in Distillery's own settings would be
        // ceremony. A settings file naming a DIFFERENT profile is another
        // matter: two faces in one resident is a surprise, not a shorthand.
        match InstalledSettings::load(data_root)
            .map_err(|error| format!("read Distillery installed settings: {error}"))?
        {
            Some(installed) if installed.profile_id() != *profile => {
                return Err(format!(
                    "the Distillery works are configured for Personae profile {:?} but this \
                     resident speaks as {:?}; reconfigure one of them rather than running two \
                     faces in one process",
                    installed.profile, profile.0
                ));
            },
            Some(_) => {},
            None => {
                InstalledAuthority::configure(data_root, profile.clone())
                    .map_err(|error| format!("configure the Distillery works: {error}"))?;
            },
        }

        // The works receive their two derived keys; neither Distillery nor
        // this lane opens a vault (dramatis repo plan, D5; custody plan §5.1).
        let authority = InstalledAuthority::open(data_root, identity, protection)
            .map_err(|error| format!("open the Distillery works: {error}"))?;
        let personal_mesh = authority
            .personal_mesh_id()
            .map_err(|error| format!("derive the personal mesh id: {error}"))?;
        #[cfg(feature = "inference")]
        let inference_paths = authority.paths(personal_mesh);
        #[cfg(feature = "inference")]
        inference_paths.prepare().map_err(|e| e.to_string())?;
        let mesh_id = match mesh {
            Some(mesh) => mesh,
            None => personal_mesh,
        };
        let paths = authority.paths(mesh_id);
        paths
            .prepare()
            .map_err(|error| format!("prepare {}: {error}", paths.root().display()))?;

        let lending = load_lending(data_root)?;
        let policy = device_policy(&lending)?;
        let sensor = Arc::new(DeviceConditionSensor::new(stated_conditions(&lending)?));
        validate_policy_coverage(&policy, &sensor.coverage()).map_err(|refusal| {
            format!(
                "this device's mesh lending policy cannot be honestly evaluated here: {refusal}"
            )
        })?;

        let author = authority
            .mesh_author()
            .map_err(|error| format!("derive the mesh author: {error}"))?;
        let store = mesh::MeshStore::at_path_with_retention(
            paths.mesh_store_path(),
            retention_policy(&lane, author.public_key().to_bytes())?,
        )
        .map_err(|error| {
            format!(
                "open the mesh store at {}: {error}",
                paths.mesh_store_path().display()
            )
        })?;

        // Paths are read before `bind_resident` consumes the authority, so
        // nothing below has to reopen a vault to say where it is. The host
        // facts are *not* read here: `gpu` is a fact about what got composed,
        // so it cannot be known until the registry below is assembled.
        let profile_name = authority.profile().0;
        let mesh_root = paths.root().to_path_buf();

        // The trainer's artifact store, opened before the host config closure
        // so a failure to open it refuses composition rather than being
        // swallowed inside a closure that can only return a config.
        #[cfg(feature = "trainer")]
        let model_library = match lane.trainer.as_ref() {
            Some(trainer) => Some(open_model_library(
                data_root,
                &profile_name,
                trainer.library_root.as_deref(),
            )?),
            None => None,
        };
        #[cfg(not(feature = "trainer"))]
        let model_library: Option<Arc<Mutex<RedbBackend>>> = None;

        // The adapter a GPU trainer was actually composed on, filled in below
        // if one was. It stays `None` for a CPU trainer and for no trainer at
        // all, and it is what [`Self::trainer_adapter`] hands out.
        #[cfg(feature = "trainer-gpu")]
        let mut trainer_adapter: Option<distillery::GpuAdapterFacts> = None;

        let gpu_access = distillery::gpu::GpuAccess::default();

        // Registering is fallible, and the closure below cannot be, so the
        // registry is assembled here and only *installed* there.
        #[cfg(feature = "trainer")]
        let registry = match model_library.as_ref() {
            Some(store) => {
                // The owner's device word, turned into a device. `"cpu"` is
                // `ndarray`, as it has always been. `"gpu"` goes through
                // Distillery's probe-then-compose helper, which refuses unless
                // the wgpu runtime would really bind a discrete adapter —
                // cubecl would otherwise substitute an unclassified one and
                // say nothing, and this lane would then advertise a GPU it
                // could not name.
                let device = match lane.trainer.as_ref().map(|trainer| trainer.device.trim()) {
                    #[cfg(feature = "trainer-gpu")]
                    Some("gpu") => {
                        let (device, adapter) =
                            distillery::discrete_gpu_trainer_device().map_err(|refusal| {
                                format!(
                                    "distillery.trainer.device is \"gpu\", and this build \
                                     carries the GPU trainer, but this machine cannot \
                                     honour it: {refusal}"
                                )
                            })?;
                        tracing::info!(
                            adapter = %adapter.name,
                            backend = %adapter.backend,
                            device_type = %adapter.device_type,
                            "composing the Distillery trainer on the discrete GPU"
                        );
                        trainer_adapter = Some(adapter);
                        device
                    },
                    _ => distillery::TrainerDevice::ndarray(),
                };
                let mut resource = distillery::TrainerResource::new(Arc::clone(store), device);
                if lane
                    .trainer
                    .as_ref()
                    .is_some_and(|trainer| trainer.device.trim() == "gpu")
                {
                    resource = resource.with_gpu_access(gpu_access.clone());
                }
                let mut registry = mesh::ResourceRegistry::builtin();
                registry
                    .register(Arc::new(resource))
                    .map_err(|error| format!("register the trainer resource: {error}"))?;
                Some(registry)
            },
            None => None,
        };

        // The one place `HostFacts::gpu` is decided, and it is decided by what
        // was registered a few lines up rather than by what is plugged in.
        #[cfg(feature = "trainer-gpu")]
        let composed_gpu = trainer_adapter.is_some();
        #[cfg(not(feature = "trainer-gpu"))]
        let composed_gpu = false;
        let facts = host_facts(composed_gpu)?;

        let mut resident = authority
            .bind_resident(mesh_id, store, resident_settings(&lane), move |space| {
                let mut config = distillery::mesh_host::HostConfig::supervised(space);
                config.policy = policy;
                config.conditions = sensor;
                config.facts = facts;
                // `SystemClock` stays: leases are wall-clock promises other
                // devices read, so a resident that ran on a hand-set clock
                // would be lying to the ring about when it let go.
                //
                // `NoCourier` stays, and on a single device that is truthful
                // rather than a gap: the poster and the runner are this same
                // process, so a job's inputs are already in this blob space
                // and there is nobody to pull them from. A second device makes
                // this a `TransportCourier`, not before.
                //
                // `ResourceRegistry::builtin()` is the floor, and it is all a
                // default build installs. Distillery's trainer resource is
                // added on top only when the owner stated one *and* this build
                // compiled the `trainer` feature, because the resource pulls
                // the burn stack into the desktop resident: a weight every
                // Djinn pays for otherwise, for a capability most of them never
                // use. The two halves are kept honest together — settings that
                // name a trainer a build cannot carry refuse `open` above.
                #[cfg(feature = "trainer")]
                if let Some(registry) = registry {
                    config.registry = registry;
                }
                config
            })
            .await
            .map_err(|error| format!("bind the Distillery resident: {error}"))?;

        resident
            .authority_mut()
            .host_mut()
            .set_gpu_access(gpu_access.clone());

        // Chronicle needs a board snapshot but owns neither the board nor its
        // lifecycle. Take that snapshot only after the resident authority is
        // completely assembled, then retain the observer beside the works so
        // reconnecting first-party sessions share its source without ever
        // sharing an admitted session.
        //
        // This is materialization one. The lifecycle path advances it from
        // fresh folded boards and the observation tick after every event.
        let board = resident.authority().board().await.map_err(|error| {
            format!("read the resident Distillery board for Chronicle: {error}")
        })?;
        let chronicle = ChronicleObserver::new_at_tick(&board, 0, ChronicleRevision::new(1, 1, 1));

        Ok(Self {
            resident,
            chronicle,
            chronicle_tick: 0,
            chronicle_revision: ChronicleRevision::new(1, 1, 1),
            author,
            mesh_id,
            profile: profile_name,
            mesh_root,
            #[cfg(feature = "inference")]
            inference_root: inference_paths.root().to_path_buf(),
            #[cfg(feature = "inference")]
            inference_space: personal_mesh,
            model_library,
            gpu_access,
            #[cfg(feature = "inference-stages")]
            stage_network: None,
            #[cfg(feature = "trainer-gpu")]
            trainer_adapter,
        })
    }

    /// The single local device gate, shared with the registered trainer.
    pub fn gpu_access(&self) -> distillery::gpu::GpuAccess {
        self.gpu_access.clone()
    }

    /// The mesh this works joined, derived rather than configured.
    pub fn mesh_id(&self) -> [u8; 32] {
        self.mesh_id
    }

    /// The Personae profile this works speaks as.
    pub fn profile(&self) -> &str {
        &self.profile
    }

    /// The product-private root holding this mesh's durable state.
    pub fn mesh_root(&self) -> &Path {
        &self.mesh_root
    }

    /// Durable ask authority remains profile-scoped when execution moves to a
    /// graph mesh. Changing rungs must not hide old local or sealed graph runs.
    #[cfg(feature = "inference")]
    pub fn inference_root(&self) -> &Path {
        &self.inference_root
    }

    /// This device's mesh author key, as the ring sees it.
    pub fn author(&self) -> [u8; 32] {
        self.author.public_key().to_bytes()
    }

    /// The persona-scoped library shared by the trainer and inference,
    /// or `None` when neither consumer opened it.
    ///
    /// Handing the same handle out rather than a path is deliberate: redb is
    /// single-writer, so a reader that opened the file itself would either
    /// fail or read a stale snapshot. Everything that wants the adapter
    /// manifests and evaluation reports this device produced goes through the
    /// one open store the trainer is writing.
    pub fn model_library(&self) -> Option<Arc<Mutex<RedbBackend>>> {
        self.model_library.as_ref().map(Arc::clone)
    }

    /// Borrow the trainer's handle, or open the same persona library when
    /// inference is its first consumer. Distillery retains the sole owner.
    #[cfg(feature = "inference")]
    pub fn inference_model_library(
        &mut self,
        data_root: &Path,
    ) -> Result<Arc<Mutex<RedbBackend>>, String> {
        if self.model_library.is_none() {
            self.model_library = Some(open_model_library(data_root, &self.profile, None)?);
        }
        Ok(Arc::clone(
            self.model_library.as_ref().expect("just opened"),
        ))
    }

    /// Open the persona's ask log before starting replication into it.
    #[cfg(feature = "inference")]
    pub fn inference_journal(
        &self,
        key: [u8; 32],
    ) -> Result<Arc<distillery::inference::durable::PersonaRunJournal<RedbBackend>>, String> {
        use rand_core::RngCore;
        let device_path = self.inference_root.join("inference-device");
        let device = match std::fs::read_to_string(&device_path) {
            Ok(device) => device,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let mut bytes = [0u8; 16];
                rand_core::OsRng.fill_bytes(&mut bytes);
                let id: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
                let device = format!(
                    "{}:{id}",
                    std::env::var("COMPUTERNAME")
                        .or_else(|_| std::env::var("HOSTNAME"))
                        .unwrap_or_else(|_| "device".into())
                );
                std::fs::write(&device_path, &device).map_err(|error| error.to_string())?;
                device
            },
            Err(error) => return Err(error.to_string()),
        };
        let backend = RedbBackend::open(self.inference_root.join("inference.redb"))
            .map_err(|error| error.to_string())?;
        let journal = Arc::new(distillery::inference::durable::PersonaRunJournal::new(
            backend,
            self.inference_space,
            &self.author,
            device,
            key,
        ));
        Ok(journal)
    }

    /// The GPU adapter this lane's trainer was composed on, or `None` when it
    /// composed a CPU trainer or none at all.
    ///
    /// This is the evidence behind [`mesh::HostFacts::gpu`] on this lane, and
    /// it is exposed rather than merely logged because a log line is not
    /// something a receipt can assert on. `Some(_)` here and
    /// `HostFacts::gpu == true` are the same fact seen from two sides; they
    /// are set from one value at composition and cannot drift apart.
    #[cfg(feature = "trainer-gpu")]
    pub fn trainer_adapter(&self) -> Option<&distillery::GpuAdapterFacts> {
        self.trainer_adapter.as_ref()
    }

    /// The product authority, for read-only board and progress projections.
    pub fn authority(&self) -> &Distillery<RedbBackend> {
        self.resident.authority()
    }

    /// Clone the resident-owned Chronicle source for first-party route
    /// assembly. The clone carries source state only; each admitted route
    /// factory still creates its own session-bound endpoint.
    pub fn chronicle_observer(&self) -> ChronicleObserver {
        self.chronicle.clone()
    }

    /// Register a Chronicle source held by this resident under its stable
    /// first-party route.
    ///
    /// The observer is passed by value so a caller can retain it across
    /// [`Self::run_until`]'s long mutable borrow. Every factory invocation
    /// receives Graphshell's already-admitted context and binds the endpoint
    /// to that context's projection session.
    pub fn register_chronicle_route(
        observer: ChronicleObserver,
        catalog: &mut ResidentEndpointCatalog,
    ) -> Result<ResidentEndpointRoute, ResidentEndpointCatalogError> {
        catalog.register_resumable_notifying(
            RESIDENT_DISTILLERY_CHRONICLE_ROUTE,
            "Distillery Chronicle",
            move |context| Ok(observer.endpoint(context.session().clone())),
        )?;
        Ok(Self::chronicle_route())
    }

    /// Build the live Chronicle host a WebRTC signaling owner hands joined
    /// browsers into. The caller supplies network policy and signaling; Djinn
    /// supplies only the resident-owned source and its fixed first-party
    /// route.
    pub fn chronicle_projection_host(
        observer: ChronicleObserver,
        policy: LocalNetworkPolicy,
    ) -> Result<
        graphshell_host::projection_host::ResidentProjectionHost,
        ResidentEndpointCatalogError,
    > {
        let mut catalog = ResidentEndpointCatalog::new();
        let route = Self::register_chronicle_route(observer, &mut catalog)?;
        Ok(graphshell_host::projection_host::ResidentProjectionHost::new(policy, route, catalog))
    }

    /// Local route descriptor granted to an installed first-party client.
    pub fn chronicle_route() -> ResidentEndpointRoute {
        ResidentEndpointRoute::new(
            RESIDENT_DISTILLERY_CHRONICLE_ROUTE,
            RESIDENT_DISTILLERY_CHRONICLE_NOTICE_POLL,
        )
        .expect("the resident Distillery Chronicle route is valid")
    }

    /// The mesh-scoped blob space this works reads and writes.
    ///
    /// Staging a job's input goes through here, which is also why the lane
    /// needs no courier while it is the only device: what it posts, it already
    /// holds.
    pub fn space(&self) -> Arc<distillery::mesh_host::TransportBlobSpace> {
        self.resident.storage().space()
    }

    /// Post a job onto this mesh as this device.
    ///
    /// Authoring needs the mesh author key, and on a single device the poster
    /// and the runner are the same process, so this resident signs its own
    /// posts. `at_ms` is the caller's reading of the wall clock, not one this
    /// module invents.
    pub async fn post_job(&self, spec: JobSpec, nonce: u64, at_ms: u64) -> Result<(), String> {
        self.resident
            .authority()
            .host()
            .synced()
            .author(
                &self.author,
                &mesh::MeshEvent::JobPostedV2 {
                    spec: Box::new(spec),
                    nonce,
                    at_ms,
                },
            )
            .await
            .map(|_| ())
            .map_err(|error| format!("post a Distillery job: {error}"))
    }

    /// Drive the works until `shutdown` resolves, reporting every receipt.
    pub async fn run_until<S, O>(&mut self, shutdown: S, observe: O) -> Result<(), String>
    where
        S: Future<Output = ()>,
        O: FnMut(ResidentReceipt),
    {
        let Self {
            resident,
            chronicle,
            chronicle_tick,
            chronicle_revision,
            ..
        } = self;
        let mut observe = observe;
        resident
            .run_until_with_board(shutdown, |receipt, board| {
                advance_chronicle(chronicle, chronicle_tick, chronicle_revision, &board);
                observe(receipt);
            })
            .await
            .map_err(|error| format!("resident Distillery works: {error}"))
    }

    /// Stop mesh networking, release the joined mesh, then flush blob storage.
    pub async fn close(mut self) -> Result<(), String> {
        #[cfg(feature = "inference-stages")]
        if let Some(network) = self.stage_network.take() {
            network.close().await?;
        }
        self.resident
            .shutdown()
            .await
            .map_err(|error| format!("could not close the resident Distillery works: {error}"))
    }
}

/// Advance the resident-owned Chronicle before exposing a lifecycle receipt to
/// the rest of Djinn. The authority supplied this board after the event, so
/// Chronicle remains a projection of current mesh truth rather than a second
/// board fold or a fixture replay.
fn advance_chronicle(
    chronicle: &ChronicleObserver,
    observation_tick: &mut u64,
    revision: &mut ChronicleRevision,
    board: &JobBoard,
) {
    *observation_tick = (*observation_tick)
        .checked_add(1)
        .expect("a resident Chronicle observation tick cannot wrap");
    let generation = revision
        .generation
        .checked_add(1)
        .expect("a resident Chronicle generation cannot wrap");
    let next_revision = ChronicleRevision::new(
        generation,
        revision.epoch.0,
        revision
            .revision
            .0
            .checked_add(1)
            .expect("a resident Chronicle revision cannot wrap"),
    );
    chronicle
        .observe_at_tick(board, *observation_tick, next_revision)
        .expect("a resident Chronicle version advances monotonically in one epoch");
    *revision = next_revision;
}

/// Where this persona's trained artifacts live.
///
/// `override_root` is the owner's word and is taken as the file path itself;
/// otherwise the path is derived as
/// `<data_root>/models/<profile>/library.redb`.
///
/// The scoping is the ruling, and two places it deliberately is **not** are
/// worth naming:
///
/// - **Not the personal-graph store.** Adapters are not the owner's notes.
///   Folding them into the graph would put multi-megabyte tensors under a
///   store whose retention, sync and privacy classes were written for a
///   personal graph, and would sync them to every paired device by default.
/// - **Not under the mesh root.** A mesh's root is governed by that mesh's
///   retention policy, and its blob custody may collect at a checkpoint. A
///   trained adapter is a durable possession of the persona; it must outlive
///   any one mesh, including a mesh the owner leaves and rejoins under a new
///   id.
///
/// The profile segment goes through the same [`sanitize_profile`] the settings
/// filename uses, because a profile id reaches this process from an argument
/// or an environment variable and is about to become a directory name.
/// It is `pub` rather than crate-private because the answer is the owner's
/// business: a host that wants to tell them where their trained adapters live,
/// or to back that directory up, should not have to guess the derivation. It
/// is also answerable in a build with no `trainer` feature — the path is where
/// the library *would* be, and saying so costs nothing.
pub fn model_library_path(
    data_root: &Path,
    profile: &str,
    override_root: Option<&Path>,
) -> PathBuf {
    match override_root {
        Some(stated) => stated.to_path_buf(),
        None => data_root
            .join("models")
            .join(sanitize_profile(profile))
            .join("library.redb"),
    }
}

/// Open (creating if absent) the persona's model library.
#[cfg(any(feature = "trainer", feature = "inference"))]
fn open_model_library(
    data_root: &Path,
    profile: &str,
    override_root: Option<&Path>,
) -> Result<Arc<Mutex<RedbBackend>>, String> {
    let path = model_library_path(data_root, profile, override_root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("prepare {}: {error}", parent.display()))?;
    }
    let store = RedbBackend::open(&path)
        .map_err(|error| format!("open the model library at {}: {error}", path.display()))?;
    Ok(Arc::new(Mutex::new(store)))
}

/// The device's lending posture, or a refusal naming what is missing.
fn load_lending(data_root: &Path) -> Result<pandect::MeshLendingSettings, String> {
    let settings = pandect::load_device_settings(data_root)
        .map_err(|error| format!("read device settings: {error}"))?;
    settings
        .and_then(|settings| settings.mesh_lending)
        .ok_or_else(|| {
            format!(
                "the Distillery works are enabled but this device has never stated a mesh \
                 lending posture: write a `mesh_lending` block in {} before running work for \
                 anyone. A composition that needs a lending posture must refuse rather than \
                 assume one.",
                pandect::device_settings_path(data_root).display()
            )
        })
}

/// Convert the device's stated lending posture into mesh's vocabulary.
///
/// This conversion lives here rather than in pandect on purpose: pandect must
/// stay mesh-free, so it stores the closed-set strings and the consumer types
/// them.
fn device_policy(lending: &pandect::MeshLendingSettings) -> Result<DevicePolicy, String> {
    Ok(DevicePolicy {
        min_idle_ms: lending.min_idle_ms,
        min_battery_pct: lending.min_battery_pct,
        max_thermal_c: lending.max_thermal_c,
        min_network: network_class(&lending.min_network, "mesh_lending.min_network")?,
        max_bandwidth_in_use_kbps: lending.max_bandwidth_in_use_kbps,
        quiet_hours: lending.quiet_hours.map(|hours| mesh::QuietHours {
            start_hour: hours.start_hour,
            end_hour: hours.end_hour,
        }),
        max_concurrent_jobs: lending.max_concurrent_jobs,
        // Empty is still mesh's own word for "no restriction" here, but it is
        // now a value the owner chose rather than one this lane invented: an
        // empty `allowed_resources` or `accepted_checkpoints` in the device
        // settings block means the owner wrote `[]`, and pandect refuses a
        // file that omits the field entirely. Each stated entry is converted
        // into mesh's own type below, and a value that does not convert
        // refuses composition rather than being dropped.
        allowed_resources: resource_set(&lending.allowed_resources)?,
        accepted_checkpoints: checkpoint_set(&lending.accepted_checkpoints)?,
        reclaim_grace_ms: lending.reclaim_grace_ms,
        supervises_leases: lending.supervises_leases,
    })
}

/// Convert the owner's stated fallbacks into the sensor's typed form.
fn stated_conditions(lending: &pandect::MeshLendingSettings) -> Result<StatedConditions, String> {
    let stated = &lending.stated;
    Ok(StatedConditions {
        idle_ms: stated.idle_ms,
        battery_pct: stated.battery_pct,
        on_mains: stated.on_mains,
        thermal_c: stated.thermal_c,
        network: stated
            .network
            .as_deref()
            .map(|value| network_class(value, "mesh_lending.stated.network"))
            .transpose()?,
        bandwidth_in_use_kbps: stated.bandwidth_in_use_kbps,
    })
}

/// Read one link class out of pandect's closed set.
fn network_class(value: &str, field: &str) -> Result<mesh::NetworkClass, String> {
    match value.trim() {
        "offline" => Ok(mesh::NetworkClass::Offline),
        "metered" => Ok(mesh::NetworkClass::Metered),
        "wifi" => Ok(mesh::NetworkClass::Wifi),
        "wired" => Ok(mesh::NetworkClass::Wired),
        other => Err(format!(
            "{field} must be one of \"offline\", \"metered\", \"wifi\", \"wired\" (got {other:?})"
        )),
    }
}

/// Parse the owner's stated resource ids with mesh's real grammar. Pandect
/// only checked that each entry could plausibly be an identifier; this is
/// where a string that is syntactically plausible but not a valid
/// `ResourceId` is actually caught, and it is refused by the parser's own
/// message so the offending string is named rather than swallowed.
fn resource_set(ids: &[String]) -> Result<BTreeSet<mesh::ResourceId>, String> {
    ids.iter()
        .map(|id| {
            mesh::ResourceId::parse(id).map_err(|error| {
                format!(
                    "mesh_lending.allowed_resources: {id:?} is not a valid resource id: {error}"
                )
            })
        })
        .collect()
}

/// Convert the owner's stated checkpoint-class strings into mesh's enum.
///
/// Pandect's own `validate` already refuses any string outside the closed
/// set before a device settings file can be saved or loaded, which makes an
/// unknown value here unreachable through that path. This conversion is
/// still not allowed to default or silently drop one: a caller that built
/// `MeshLendingSettings` some other way (a test, a future importer) gets a
/// named refusal instead of a policy that quietly accepted less than it was
/// told to.
fn checkpoint_set(values: &[String]) -> Result<BTreeSet<mesh::CheckpointClass>, String> {
    values
        .iter()
        .map(|value| match value.as_str() {
            "restart" => Ok(mesh::CheckpointClass::Restart),
            "resumable" => Ok(mesh::CheckpointClass::Resumable),
            "non-interruptible" => Ok(mesh::CheckpointClass::NonInterruptible),
            other => Err(format!(
                "mesh_lending.accepted_checkpoints: unknown checkpoint class {other:?}"
            )),
        })
        .collect()
}

/// The cadences the owner stated, as Distillery's resident settings.
fn resident_settings(lane: &DistilleryLaneSettings) -> ResidentSettings {
    ResidentSettings {
        tick_every: Duration::from_millis(lane.tick_every_ms),
        maintenance_every: lane.maintenance_every_ms.map(Duration::from_millis),
        blob_gc_every: Duration::from_millis(lane.blob_gc_every_ms),
        retention: RetentionSettings {
            collect_after_checkpoint: lane.collect_after_checkpoint,
        },
    }
}

/// The retention posture the owner stated, under their governance tag.
///
/// `checkpoint_authority` is not a setting: it is this profile's derived mesh
/// author. On a single device the owner's works govern their own retention, and
/// a typed key here could only ever hand that governance to somebody else by
/// accident.
fn retention_policy(
    lane: &DistilleryLaneSettings,
    checkpoint_authority: [u8; 32],
) -> Result<MeshRetentionPolicy, String> {
    Ok(MeshRetentionPolicy {
        revision: mesh::PolicyRevision(lane.retention_revision_bytes()?),
        checkpoint_authority,
        availability: AvailabilityPolicy {
            promised_floor: parse_keep_bound(&lane.promised_floor, "distillery.promised_floor")?,
        },
        erasure: ErasurePolicy {
            privacy_ceiling: parse_keep_bound(&lane.privacy_ceiling, "distillery.privacy_ceiling")?,
            terminal_job_payload: if lane.erase_terminal_at_checkpoint {
                mesh::PayloadRule::EraseTerminalAtCheckpoint
            } else {
                mesh::PayloadRule::Keep
            },
        },
        lease: mesh::LeasePolicy {
            max_skew_ms: lane.max_skew_ms,
        },
    })
}

/// What this machine can honestly say it has.
///
/// `memory_mib` is total physical memory, read once at composition from
/// [`total_memory_mib`] — `GlobalMemoryStatusEx` on Windows, `MemTotal:` from
/// `/proc/meminfo` on Linux, the `hw.memsize` sysctl on macOS, and nothing at
/// all anywhere else, where this refuses. It is a ceiling rather than a live
/// reading, and it is deliberately not "free right now": [`mesh::HostFacts`]
/// is a static offer other devices match job requirements against, so a number
/// that moved every time a browser opened would make this device's offers
/// meaningless. A job that would fit the machine but not this moment is a
/// scheduling concern, and scheduling by live pressure is a later slice.
///
/// `gpu` is an argument rather than a constant, and the rule behind it is the
/// whole point: **the fact follows the composed resource, never the hardware.**
/// It is `true` only when this composition actually registered a resource
/// running on a GPU device — which today means a `"gpu"` trainer, on a build
/// carrying `trainer-gpu`, whose adapter probe found a real discrete adapter.
/// A machine with a discrete GPU in it and a CPU trainer composed on top still
/// advertises `false`, because [`mesh::HostFacts`] is an offer to the ring:
/// what it promises is work this device will *run*, not silicon it happens to
/// contain. The builtin resources are all CPU work, so nothing else can move
/// this bit.
fn host_facts(gpu: bool) -> Result<mesh::HostFacts, String> {
    Ok(mesh::HostFacts {
        memory_mib: total_memory_mib().ok_or(
            "this build cannot read total physical memory, and a device must not advertise a \
             capacity it never measured",
        )?,
        gpu,
    })
}

// ─── Total physical memory, one reading per platform ────────────────────────
//
// Three sources, and naming them is the whole doc: `GlobalMemoryStatusEx` on
// Windows, the `MemTotal:` line of `/proc/meminfo` on Linux, the `hw.memsize`
// sysctl on macOS. Each is the operating system's own answer, and none of them
// is guessed from anything else — a device that could not reach its platform's
// answer reports `None` and `host_facts` refuses.
//
// The arms are cheap enough to be per-platform rather than per-crate: Windows
// and macOS each go through a dependency this port already carries for its
// own reasons, and the Linux arm is `std` alone.

/// Windows: `GlobalMemoryStatusEx`, through the `windows` crate
/// [`crate::conditions`] already carries.
#[cfg(windows)]
fn total_memory_mib() -> Option<u32> {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    let mut status = MEMORYSTATUSEX {
        dwLength: u32::try_from(size_of::<MEMORYSTATUSEX>()).ok()?,
        ..Default::default()
    };
    // SAFETY: `status` is a live, correctly sized `MEMORYSTATUSEX`; the call
    // only writes through the pointer it is given.
    unsafe { GlobalMemoryStatusEx(&mut status) }.ok()?;
    u32::try_from(status.ullTotalPhys / (1024 * 1024)).ok()
}

/// Linux: the `MemTotal:` line of `/proc/meminfo`, which the kernel reports in
/// kibibytes despite writing `kB`.
///
/// `std` alone, deliberately: `/proc/meminfo`'s first line has been
/// `MemTotal:` for the life of the interface, and taking a dependency to read
/// one integer out of a text file would cost the resident more than it buys.
/// Anything unexpected — no `/proc`, no such line, a value that will not parse
/// — is `None` rather than a guess.
#[cfg(target_os = "linux")]
fn total_memory_mib() -> Option<u32> {
    let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;
    let kib: u64 = meminfo
        .lines()
        .find_map(|line| line.strip_prefix("MemTotal:"))?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    Some(u32::try_from(kib / 1024).unwrap_or(u32::MAX))
}

/// macOS: the `hw.memsize` sysctl, which reports bytes.
///
/// `sysctlbyname` through `libc` rather than spawning `sysctl(8)`: a resident
/// that forked a process to learn its own memory size would be paying for an
/// answer the kernel hands over directly. A non-zero return, or a reply that
/// is not the `u64` this asked for, is `None`.
#[cfg(target_os = "macos")]
fn total_memory_mib() -> Option<u32> {
    let mut bytes: u64 = 0;
    let mut written = size_of::<u64>();
    // SAFETY: the name is a NUL-terminated C string, `bytes` and `written` are
    // live locals, and `written` is exactly the size of the buffer the second
    // pointer addresses; the call writes only through the pointers it is given.
    let answered = unsafe {
        libc::sysctlbyname(
            c"hw.memsize".as_ptr(),
            (&raw mut bytes).cast(),
            &raw mut written,
            std::ptr::null_mut(),
            0,
        )
    };
    if answered != 0 || written != size_of::<u64>() {
        return None;
    }
    Some(u32::try_from(bytes / (1024 * 1024)).unwrap_or(u32::MAX))
}

/// Everywhere else: nothing, and [`host_facts`] refuses rather than advertise
/// a capacity this build never measured. The same posture
/// [`crate::conditions`] takes with its sensed signals: a port that wants this
/// lane supplies a real reading first.
#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
fn total_memory_mib() -> Option<u32> {
    None
}

#[cfg(test)]
mod tests {
    use pandect::{MeshLendingSettings, QuietHoursSettings, StatedConditionSettings};

    use super::*;

    fn lending() -> MeshLendingSettings {
        MeshLendingSettings {
            min_idle_ms: 120_000,
            min_battery_pct: 40,
            max_thermal_c: 0,
            min_network: "wifi".into(),
            max_bandwidth_in_use_kbps: 0,
            quiet_hours: Some(QuietHoursSettings {
                start_hour: 22,
                end_hour: 8,
            }),
            max_concurrent_jobs: 2,
            allowed_resources: Vec::new(),
            accepted_checkpoints: Vec::new(),
            reclaim_grace_ms: 5_000,
            supervises_leases: true,
            stated: StatedConditionSettings::default(),
        }
    }

    fn lane() -> DistilleryLaneSettings {
        DistilleryLaneSettings {
            tick_every_ms: 25,
            maintenance_every_ms: None,
            blob_gc_every_ms: 1_000,
            collect_after_checkpoint: true,
            retention_revision: "2a".repeat(32),
            promised_floor: "forever".into(),
            privacy_ceiling: "until-checkpoint".into(),
            erase_terminal_at_checkpoint: true,
            max_skew_ms: 0,
            trainer: None,
        }
    }

    #[test]
    fn the_stated_posture_becomes_the_policy_with_nothing_added() {
        let policy = device_policy(&lending()).unwrap();
        assert_eq!(policy.min_idle_ms, 120_000);
        assert_eq!(policy.min_battery_pct, 40);
        assert_eq!(policy.min_network, mesh::NetworkClass::Wifi);
        assert_eq!(policy.max_concurrent_jobs, 2);
        assert_eq!(policy.reclaim_grace_ms, 5_000);
        assert!(policy.supervises_leases);
        assert_eq!(
            policy.quiet_hours,
            Some(mesh::QuietHours {
                start_hour: 22,
                end_hour: 8
            })
        );
        assert_ne!(
            policy,
            DevicePolicy::permissive(),
            "a stated posture that converted into the permissive default would \
             mean the owner's settings never reached the supervisor"
        );
    }

    #[test]
    fn a_link_class_outside_the_closed_set_is_refused_by_name() {
        let mut broken = lending();
        broken.min_network = "ethernet".into();
        let refusal = device_policy(&broken).expect_err("`ethernet` is not a link class");
        assert!(refusal.contains("min_network"), "{refusal}");

        let mut stated = lending();
        stated.stated.network = Some("fibre".into());
        let refusal = stated_conditions(&stated).expect_err("`fibre` is not a link class");
        assert!(refusal.contains("stated.network"), "{refusal}");
    }

    #[test]
    fn allowed_resources_and_accepted_checkpoints_convert_into_mesh_types() {
        let mut stated = lending();
        stated.allowed_resources = vec!["mesh.blake3/v1".into(), "mesh.echo/v1".into()];
        stated.accepted_checkpoints = vec!["restart".into(), "resumable".into()];

        let policy = device_policy(&stated).unwrap();
        assert_eq!(
            policy.allowed_resources,
            BTreeSet::from([
                mesh::ResourceId::parse("mesh.blake3/v1").unwrap(),
                mesh::ResourceId::parse("mesh.echo/v1").unwrap(),
            ])
        );
        assert_eq!(
            policy.accepted_checkpoints,
            BTreeSet::from([
                mesh::CheckpointClass::Restart,
                mesh::CheckpointClass::Resumable,
            ])
        );
    }

    #[test]
    fn an_unparseable_resource_id_is_refused_by_the_parsers_own_message() {
        let mut stated = lending();
        stated.allowed_resources = vec!["Not A Valid Id".into()];
        let refusal =
            device_policy(&stated).expect_err("`Not A Valid Id` is not a valid resource id");
        assert!(refusal.contains("Not A Valid Id"), "{refusal}");
    }

    #[test]
    fn an_unknown_checkpoint_string_is_refused_by_name() {
        let mut stated = lending();
        stated.accepted_checkpoints = vec!["paused".into()];
        let refusal = device_policy(&stated).expect_err("`paused` is not a checkpoint class");
        assert!(refusal.contains("paused"), "{refusal}");
    }

    #[test]
    fn a_thermal_limit_without_a_thermal_reading_cannot_compose() {
        // The composition-time half of the same refusal the receipt proves end
        // to end: thermal is never sensed, so an enabled thermal rule needs a
        // stated temperature or it rests on nothing.
        let mut hot = lending();
        hot.max_thermal_c = 85;
        let policy = device_policy(&hot).unwrap();
        let sensor = DeviceConditionSensor::new(stated_conditions(&hot).unwrap());
        let refusal = validate_policy_coverage(&policy, &sensor.coverage())
            .expect_err("a thermal limit on a device with no thermometer is not a policy");
        assert!(refusal.contains("thermal"), "{refusal}");

        // State every fallback the policy's *other* rules rest on as well, so
        // the only open question is thermal on every platform. On Windows the
        // sensed idle, battery and network readings beat these anyway; off
        // Windows nothing is sensed, and without them this half of the test
        // would fail on idle, battery and network rather than pass on thermal
        // (it did, the first time it ran on the Fedora ThinkPad, 2026-09-02).
        hot.stated = StatedConditionSettings {
            idle_ms: Some(600_000),
            battery_pct: Some(90),
            on_mains: Some(true),
            thermal_c: Some(45),
            network: Some("wifi".into()),
            bandwidth_in_use_kbps: None,
        };
        let sensor = DeviceConditionSensor::new(stated_conditions(&hot).unwrap());
        assert_eq!(
            validate_policy_coverage(&policy, &sensor.coverage()),
            Ok(()),
            "the owner's word is a provenance, so stating one satisfies the rule"
        );
    }

    #[test]
    fn the_retention_policy_carries_the_owners_revision_and_self_authority() {
        let author = [0x7c; 32];
        let policy = retention_policy(&lane(), author).unwrap();
        assert_eq!(policy.revision, mesh::PolicyRevision([0x2a; 32]));
        assert_eq!(
            policy.checkpoint_authority, author,
            "the works govern their own retention on a single device"
        );
        assert_eq!(policy.availability.promised_floor, mesh::KeepBound::Forever);
        assert_eq!(
            policy.erasure.privacy_ceiling,
            mesh::KeepBound::UntilCheckpoint
        );
        assert_eq!(
            policy.erasure.terminal_job_payload,
            mesh::PayloadRule::EraseTerminalAtCheckpoint
        );
        assert_eq!(policy.lease.max_skew_ms, 0);
    }

    #[test]
    fn the_cadences_are_exactly_the_ones_the_owner_stated() {
        let settings = resident_settings(&lane());
        assert_eq!(settings.tick_every, Duration::from_millis(25));
        assert_eq!(settings.maintenance_every, None);
        assert_eq!(settings.blob_gc_every, Duration::from_millis(1_000));
        assert!(settings.retention.collect_after_checkpoint);
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn the_model_library_is_persona_scoped_and_lives_outside_every_mesh() {
        let data_root = Path::new("C:").join("djinn").join("data");
        let derived = model_library_path(&data_root, "works", None);
        assert_eq!(
            derived,
            data_root.join("models").join("works").join("library.redb"),
            "the derived path is <data_root>/models/<profile>/library.redb"
        );
        assert!(
            !derived.starts_with(data_root.join("distillery")),
            "a trained adapter must not sit under a mesh root whose retention \
             policy could collect it: {}",
            derived.display()
        );
        assert_ne!(
            model_library_path(&data_root, "works", None),
            model_library_path(&data_root, "burner", None),
            "two faces on one device do not share a model library"
        );
    }

    #[test]
    fn a_profile_that_could_escape_the_data_root_is_sanitised_into_one_segment() {
        let data_root = Path::new("C:").join("djinn").join("data");
        let escaped = model_library_path(&data_root, "../../evil", None);
        assert_eq!(
            escaped,
            data_root
                .join("models")
                .join("______evil")
                .join("library.redb"),
            "a profile id arrives from an argument and is about to be a directory name"
        );
    }

    #[test]
    fn a_stated_library_root_is_taken_as_the_owners_word() {
        let data_root = Path::new("C:").join("djinn").join("data");
        let stated = Path::new("D:").join("archive").join("adapters.redb");
        assert_eq!(
            model_library_path(&data_root, "works", Some(&stated)),
            stated,
            "an override is a path the owner chose, not a hint to be re-derived"
        );
    }

    /// Windows, Linux and macOS each have a real reading, so on all three this
    /// asks the same two questions: the number is the machine's, and the GPU
    /// bit is the composition's.
    #[cfg(any(windows, target_os = "linux", target_os = "macos"))]
    #[test]
    fn this_machine_reports_a_real_memory_size_and_a_gpu_fact_that_follows_composition() {
        let facts = host_facts(false).expect("this platform reads physical memory");
        assert!(
            facts.memory_mib > 512,
            "a machine running this test has more than half a gibibyte, so a smaller \
             answer means the reading is wrong rather than the machine small: {}",
            facts.memory_mib
        );
        assert!(
            !facts.gpu,
            "a composition that registered no GPU resource may not claim one — and this \
             runs on machines that do have a GPU in them, which is exactly the case that \
             would go wrong if the fact were read off the hardware"
        );
        assert!(
            host_facts(true)
                .expect("this platform reads physical memory")
                .gpu,
            "and the composed answer is carried through rather than overridden"
        );
    }

    /// Anywhere else there is no reading, and the refusal is the receipt.
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    #[test]
    fn a_build_that_cannot_measure_memory_refuses_to_advertise_one() {
        assert!(host_facts(false).is_err());
        assert!(host_facts(true).is_err());
    }
}
