// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The resident inference lane (resident inference and recall plan, I1).
//!
//! One esp inference actor, owned by Distillery's [`InferenceLane`], served
//! on the `inference.ask` route to Turnstone (ruling 8). The lane borrows the
//! Distillery lane's model library (ruling 3), so settings that name it
//! without a Distillery lane are refused before anything opens, as are
//! settings that name it in a build without the `inference` feature.
//!
//! [`InferenceLane`]: distillery::inference::InferenceLane

use crate::settings::OwnerSettings;

/// The graph's compute space is a name, independent of any device root.
pub fn compute_space(graph: [u8; 32]) -> [u8; 32] {
    let mut hash = blake3::Hasher::new();
    hash.update(b"mere.djinn/personal-inference-mesh/v1");
    hash.update(&graph);
    *hash.finalize().as_bytes()
}

#[cfg(feature = "inference-stages")]
pub(crate) mod mesh_discovery;

#[cfg(feature = "inference")]
pub const INFERENCE_DATA_SALT: &[u8] = b"mere.djinn/inference/data/v1";

/// Refuse inference settings this build or this resident cannot honour.
/// Called before any lane opens, so a refusal leaves nothing to unwind.
pub fn validate(owner: &OwnerSettings) -> Result<(), String> {
    if owner.inference.is_none() {
        return Ok(());
    }
    if !cfg!(feature = "inference") {
        return Err(
            "inference names the resident inference lane, but this build carries no \
             inference lane; rebuild with the `inference` feature or remove the inference \
             block. A resident must not compose settings it cannot honour."
                .into(),
        );
    }
    owner.inference.as_ref().unwrap().recall.validate()?;
    if let crate::settings::InferenceModelSettings::Library {
        device,
        weight_cache: Some(cache),
        ..
    } = &owner.inference.as_ref().unwrap().model
    {
        usize::try_from(cache.host_bytes).map_err(|_| "host cache budget exceeds address space")?;
        if *device != crate::settings::InferenceDeviceSettings::Gpu
            && (cache.host_bytes != 0 || cache.device_layer_bytes != 0)
        {
            return Err("weight caching requires the GPU library lane".into());
        }
    }

    if let Some(embedding) = &owner.inference.as_ref().unwrap().embedding {
        if !cfg!(feature = "inference-embedding") {
            return Err("embedding recall requires the inference-embedding feature".into());
        }
        embedding.validate()?;
    }
    if owner.distillery.is_none() {
        return Err(
            "inference needs the distillery lane: it borrows that lane's model library \
             (resident inference and recall plan, ruling 3); add a distillery block or \
             remove the inference block"
                .into(),
        );
    }
    if let crate::settings::InferenceModelSettings::Automatic { stages, .. } =
        &owner.inference.as_ref().unwrap().model
    {
        if !cfg!(feature = "inference-stages") {
            return Err("automatic inference requires the `inference-stages` feature".into());
        }
        stages.validate()?;
        if owner.sync.is_none() {
            return Err("automatic inference requires personal sync settings".into());
        }
    }
    if let Some((row, device, max_tokens)) = owner.inference.as_ref().unwrap().model.library() {
        if max_tokens == 0 {
            return Err("inference.max_tokens must be greater than zero".into());
        }
        #[cfg(feature = "inference")]
        distillery::inference::parse_model_row(row)?;
        #[cfg(not(feature = "inference"))]
        let _ = row;
        if device == crate::settings::InferenceDeviceSettings::Gpu
            && !cfg!(feature = "inference-gpu")
        {
            return Err("inference.model.library.device gpu requires the `inference-gpu` feature; rebuild with it or select cpu".into());
        }
    }
    Ok(())
}

#[cfg(feature = "inference")]
pub use lane::ResidentInference;

#[cfg(feature = "inference")]
pub mod personal;

/// Without the feature the lane cannot exist: [`validate`] refuses its
/// settings, so no value of this type is ever made.
#[cfg(not(feature = "inference"))]
pub enum ResidentInference {}

#[cfg(not(feature = "inference"))]
impl ResidentInference {
    /// Uninhabited without the feature.
    pub fn register(
        &self,
        _: &mut graphshell_host::endpoint_catalog::ResidentEndpointCatalog,
    ) -> Result<
        graphshell_endpoint::door::route::ResidentEndpointRoute,
        graphshell_host::endpoint_catalog::ResidentEndpointCatalogError,
    > {
        match *self {}
    }

    /// Uninhabited without the feature.
    pub async fn close(self) -> Result<(), String> {
        match self {}
    }
}

#[cfg(feature = "inference")]
mod lane {
    use distillery::inference::{
        INFERENCE_ASK_NOTICE_POLL, INFERENCE_ASK_ROUTE, InferenceLane, StubInferenceProvider,
    };
    use graphshell_endpoint::door::route::ResidentEndpointRoute;
    use graphshell_host::endpoint_catalog::{
        ResidentEndpointCatalog, ResidentEndpointCatalogError,
    };

    use crate::settings::{InferenceLaneSettings, InferenceModelSettings};

    /// The open lane.
    pub struct ResidentInference {
        lane: InferenceLane,
        personal: Option<std::sync::Arc<crate::sync::personal_sync_host::PersonalSyncHost>>,
        journal: Option<
            std::sync::Arc<
                distillery::inference::durable::PersonaRunJournal<muniment::RedbBackend>,
            >,
        >,
    }

    impl ResidentInference {
        /// Start the actor on the model the settings name.
        pub fn open(
            settings: &InferenceLaneSettings,
            library: Option<std::sync::Arc<tokio::sync::Mutex<muniment::RedbBackend>>>,
            gpu_access: distillery::gpu::GpuAccess,
            journal: Option<
                std::sync::Arc<
                    distillery::inference::durable::PersonaRunJournal<muniment::RedbBackend>,
                >,
            >,
            #[cfg(feature = "inference-stages")] stages: Option<
                std::sync::Arc<distillery::inference_stage::StageChainFactory>,
            >,
        ) -> Result<Self, String> {
            #[cfg(not(feature = "inference-gpu"))]
            let _ = gpu_access;
            let lane = match &settings.model {
                #[cfg(feature = "inference-stages")]
                InferenceModelSettings::Automatic {
                    row,
                    device,
                    max_tokens,
                    stages: limits,
                } => {
                    let device = match device {
                        crate::settings::InferenceDeviceSettings::Cpu => {
                            distillery::inference_stage::StagedLaneDevice::Cpu
                        },
                        #[cfg(feature = "inference-gpu")]
                        crate::settings::InferenceDeviceSettings::Gpu => {
                            distillery::inference_stage::StagedLaneDevice::Gpu(gpu_access)
                        },
                        #[cfg(not(feature = "inference-gpu"))]
                        _ => return Err("automatic GPU inference requires inference-gpu".into()),
                    };
                    let label = distillery::inference::RunJournal::device(
                        journal
                            .as_ref()
                            .ok_or("automatic inference requires its journal")?
                            .as_ref(),
                    )
                    .to_string();
                    distillery::inference_stage::open_automatic_library_lane_with_journal(
                        library.ok_or("automatic inference requires its library")?,
                        distillery::inference::parse_model_row(row)?,
                        *max_tokens,
                        device,
                        stages.ok_or("automatic stage planner was not composed")?,
                        limits.local_resident_bytes,
                        label,
                        journal.as_ref().map(|j| {
                            j.clone() as std::sync::Arc<dyn distillery::inference::RunJournal>
                        }),
                    )?
                },
                #[cfg(not(feature = "inference-stages"))]
                InferenceModelSettings::Automatic { .. } => {
                    return Err("automatic inference requires inference-stages".into());
                },
                InferenceModelSettings::Stub => {
                    InferenceLane::open(|| Box::new(StubInferenceProvider::new()))
                },
                #[cfg(feature = "inference-gpu")]
                InferenceModelSettings::Library {
                    row,
                    max_tokens,
                    device: crate::settings::InferenceDeviceSettings::Gpu,
                    weight_cache,
                } => distillery::inference::open_gpu_library_lane_with_journal_and_cache(
                    library.expect("resident opens the model library first"),
                    distillery::inference::parse_model_row(row).expect("validated model row"),
                    *max_tokens,
                    gpu_access,
                    journal.as_ref().map(|journal| {
                        journal.clone() as std::sync::Arc<dyn distillery::inference::RunJournal>
                    }),
                    usize::try_from(weight_cache.as_ref().map_or(0, |c| c.host_bytes))
                        .map_err(|_| "host cache budget exceeds address space")?,
                    weight_cache.as_ref().map_or(0, |c| c.device_layer_bytes),
                )?,
                InferenceModelSettings::Library {
                    row,
                    max_tokens,
                    device: crate::settings::InferenceDeviceSettings::Cpu,
                    weight_cache,
                } => {
                    if weight_cache
                        .as_ref()
                        .is_some_and(|c| c.host_bytes != 0 || c.device_layer_bytes != 0)
                    {
                        return Err("weight caching requires the GPU library lane".into());
                    }
                    distillery::inference::open_library_lane_with_journal(
                        library.expect("resident opens the model library first"),
                        distillery::inference::parse_model_row(row)
                            .expect("validated before opening lanes"),
                        *max_tokens,
                        journal.as_ref().map(|journal| {
                            journal.clone() as std::sync::Arc<dyn distillery::inference::RunJournal>
                        }),
                    )?
                },
                #[cfg(not(feature = "inference-gpu"))]
                InferenceModelSettings::Library {
                    device: crate::settings::InferenceDeviceSettings::Gpu,
                    ..
                } => unreachable!("validate refuses GPU settings without inference-gpu"),
            };
            lane.configure_published_recall(settings.recall.clone())?;
            Ok(Self {
                lane,
                journal,
                personal: None,
            })
        }

        /// Retain the endpoint/key owner until after the private lane drains.
        pub fn retain_personal_sync(
            &mut self,
            host: std::sync::Arc<crate::sync::personal_sync_host::PersonalSyncHost>,
        ) {
            self.personal = Some(host);
        }

        /// The sealed record authority, for in-process evidence readers.
        pub fn journal(
            &self,
        ) -> Option<
            &std::sync::Arc<
                distillery::inference::durable::PersonaRunJournal<muniment::RedbBackend>,
            >,
        > {
            self.journal.as_ref()
        }

        /// The lane itself, for in-process callers.
        pub fn lane(&self) -> &InferenceLane {
            &self.lane
        }

        pub fn lock_holder(
            &self,
            profile: personae::ProfileId,
        ) -> std::sync::Arc<dyn castellan::lock::VaultLockHolder> {
            std::sync::Arc::new(InferenceLock {
                lane: self.lane.clone(),
                journal: self.journal.clone(),
                profile,
            })
        }

        /// Register `inference.ask`; each admitted session gets its own
        /// endpoint over the one lane.
        pub fn register(
            &self,
            catalog: &mut ResidentEndpointCatalog,
        ) -> Result<ResidentEndpointRoute, ResidentEndpointCatalogError> {
            let lane = self.lane.clone();
            catalog.register_notifying(
                INFERENCE_ASK_ROUTE,
                "Resident inference",
                move |context| Ok(lane.endpoint(context.session().clone())),
            )?;
            Ok(Self::route())
        }

        /// The route descriptor granted to a first-party client.
        pub fn route() -> ResidentEndpointRoute {
            ResidentEndpointRoute::new(INFERENCE_ASK_ROUTE, INFERENCE_ASK_NOTICE_POLL)
                .expect("the inference route is valid")
        }

        /// Cancel unfinished runs and join the lane's threads.
        pub async fn close(self) -> Result<(), String> {
            let closed = tokio::task::spawn_blocking(move || self.lane.close())
                .await
                .map_err(|error| format!("join the inference lane's close: {error}"))
                .and_then(|result| result);
            let drained = match self.journal {
                Some(journal) => journal.shutdown().await,
                None => Ok(()),
            };
            // When the enclosing task scope has released its watchers and
            // sessions, this is the final host owner. Await its transport drain
            // before the resident checks shared blob custody for borrowers.
            // An embedded caller retaining its own host keeps that ownership.
            let personal = match self.personal {
                Some(host) => match std::sync::Arc::try_unwrap(host) {
                    Ok(host) => host
                        .close()
                        .await
                        .map_err(|error| format!("close personal sync: {error}")),
                    Err(_) => Ok(()),
                },
                None => Ok(()),
            };
            let failures: Vec<_> = [closed, drained, personal]
                .into_iter()
                .filter_map(Result::err)
                .collect();
            if failures.is_empty() {
                Ok(())
            } else {
                Err(failures.join("; "))
            }
        }
    }
    struct InferenceLock {
        lane: InferenceLane,
        journal: Option<
            std::sync::Arc<
                distillery::inference::durable::PersonaRunJournal<muniment::RedbBackend>,
            >,
        >,
        profile: personae::ProfileId,
    }
    impl castellan::lock::VaultLockHolder for InferenceLock {
        fn name(&self) -> &str {
            "resident inference"
        }
        fn key_profile(&self) -> Option<&personae::ProfileId> {
            Some(&self.profile)
        }
        fn lock(&self) {
            if let Err(error) = self.lane.set_locked(true) {
                tracing::error!(%error, "inference pause failed at lock");
            }
            self.lane.forget_locked_materialization();
            if let Some(journal) = &self.journal {
                journal.lock();
            }
        }
        fn unlock(
            &self,
            vault: &dyn personae::IdentityProvider,
        ) -> Result<(), personae::IdentityError> {
            if let Some(journal) = &self.journal {
                journal.unlock(vault.derive_keypair(super::INFERENCE_DATA_SALT)?.to_seed());
            }
            if let Err(error) = self.lane.refresh_retained_runs() {
                tracing::error!(%error, "inference retained runs could not reopen");
                return Err(personae::IdentityError::Backend(error));
            }
            if let Err(error) = self.lane.set_locked(false) {
                tracing::error!(%error, "inference resume failed at unlock");
                return Err(personae::IdentityError::Backend(error));
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{InferenceLaneSettings, InferenceModelSettings};

    fn stub() -> Option<InferenceLaneSettings> {
        Some(InferenceLaneSettings {
            embedding: None,
            recall: Default::default(),
            model: InferenceModelSettings::Stub,
        })
    }

    #[test]
    fn absent_settings_pass() {
        assert_eq!(validate(&OwnerSettings::default()), Ok(()));
    }

    #[test]
    fn weight_cache_is_explicit_and_legacy_settings_keep_their_shape() {
        let raw = serde_json::json!({"library":{"row":"blake3:0000000000000000000000000000000000000000000000000000000000000000","device":"gpu","max_tokens":32}});
        let model: InferenceModelSettings = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(serde_json::to_value(model).unwrap(), raw);
        let mut configured = raw.clone();
        configured["library"]["weight_cache"] = serde_json::json!({"host_bytes":3 * 1024_u64.pow(3),"device_layer_bytes":4 * 1024_u64.pow(3)});
        let model: InferenceModelSettings = serde_json::from_value(configured.clone()).unwrap();
        assert_eq!(serde_json::to_value(model).unwrap(), configured);
        configured["library"]["weight_cache"]["unbounded"] = true.into();
        assert!(serde_json::from_value::<InferenceModelSettings>(configured).is_err());
    }

    #[cfg(feature = "inference")]
    #[test]
    fn cpu_cache_budgets_are_refused_before_opening_any_lane() {
        let model = InferenceModelSettings::Library {
            row: format!("blake3:{}", "00".repeat(32)),
            device: crate::settings::InferenceDeviceSettings::Cpu,
            max_tokens: 4,
            weight_cache: Some(crate::settings::InferenceWeightCacheSettings {
                host_bytes: 1,
                device_layer_bytes: 1,
            }),
        };
        let owner = OwnerSettings {
            inference: Some(InferenceLaneSettings {
                embedding: None,
                recall: Default::default(),
                model,
            }),
            ..Default::default()
        };
        assert!(
            validate(&owner)
                .unwrap_err()
                .contains("weight caching requires the GPU")
        );
    }

    #[cfg(feature = "inference")]
    #[test]
    fn the_lane_without_a_distillery_lane_is_refused() {
        let owner = OwnerSettings {
            inference: stub(),
            ..OwnerSettings::default()
        };
        let refusal = validate(&owner).unwrap_err();
        assert!(refusal.contains("needs the distillery lane"), "{refusal}");
    }

    #[cfg(not(feature = "inference"))]
    #[test]
    fn the_lane_in_a_build_without_it_is_refused() {
        let owner = OwnerSettings {
            inference: stub(),
            ..OwnerSettings::default()
        };
        let refusal = validate(&owner).unwrap_err();
        assert!(refusal.contains("`inference` feature"), "{refusal}");
    }

    #[test]
    fn settings_name_the_stub_in_kebab_case() {
        let owner: OwnerSettings =
            serde_json::from_str(r#"{"inference": {"model": "stub"}}"#).unwrap();
        assert_eq!(owner.inference, stub());
        assert!(
            serde_json::from_str::<OwnerSettings>(r#"{"inference": {"model": "llama-70b"}}"#)
                .is_err()
        );
    }

    #[test]
    fn library_settings_are_explicit_and_refuse_unknown_fields() {
        let row = format!("blake3:{}", "00".repeat(32));
        let value = serde_json::json!({"inference": {"model": {"library": {"row": row, "device": "cpu", "max_tokens": 32}}}});
        let owner: OwnerSettings = serde_json::from_value(value.clone()).unwrap();
        assert!(matches!(
            owner.inference.unwrap().model,
            InferenceModelSettings::Library { max_tokens: 32, .. }
        ));
        let mut bad = value.clone();
        bad["inference"]["model"]["library"]["unknown"] = true.into();
        assert!(serde_json::from_value::<OwnerSettings>(bad).is_err());
        for field in ["row", "device", "max_tokens"] {
            let mut bad = value.clone();
            bad["inference"]["model"]["library"]
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(
                serde_json::from_value::<OwnerSettings>(bad).is_err(),
                "{field} must be stated"
            );
        }
    }

    #[cfg(feature = "inference")]
    #[test]
    fn invalid_library_rows_limits_and_gpu_are_refused_before_opening() {
        use crate::settings::{DistilleryLaneSettings, InferenceDeviceSettings};
        let mut owner = OwnerSettings {
            distillery: Some(DistilleryLaneSettings {
                tick_every_ms: 25,
                maintenance_every_ms: None,
                blob_gc_every_ms: 1000,
                collect_after_checkpoint: true,
                retention_revision: "00".repeat(32),
                promised_floor: "forever".into(),
                privacy_ceiling: "until-checkpoint".into(),
                erase_terminal_at_checkpoint: true,
                max_skew_ms: 0,
                trainer: None,
            }),
            ..Default::default()
        };
        let set_model = |row: String, device, max_tokens| {
            Some(InferenceLaneSettings {
                embedding: None,
                recall: Default::default(),
                model: InferenceModelSettings::Library {
                    weight_cache: None,
                    row,
                    device,
                    max_tokens,
                },
            })
        };
        owner.inference = set_model("bad".into(), InferenceDeviceSettings::Cpu, 32);
        assert!(validate(&owner).unwrap_err().contains("model row"));
        let row = format!("blake3:{}", "00".repeat(32));
        owner.inference = set_model(row.clone(), InferenceDeviceSettings::Cpu, 0);
        assert!(validate(&owner).unwrap_err().contains("max_tokens"));
        owner.inference = set_model(row.clone(), InferenceDeviceSettings::Gpu, 32);
        if cfg!(feature = "inference-gpu") {
            assert!(validate(&owner).is_ok());
        } else {
            assert!(validate(&owner).unwrap_err().contains("inference-gpu"));
        }
        owner.inference = set_model(row.clone(), InferenceDeviceSettings::Cpu, 32);
        assert!(validate(&owner).is_ok());
        let model: InferenceModelSettings = serde_json::from_value(serde_json::json!({
            "automatic": { "row": row, "device": "cpu", "max_tokens": 32, "stages": {
                "local_resident_bytes": 0, "worker_layer_bytes": 1048576, "max_peers": 8,
                "max_hidden_bytes": 1048576, "discovery_timeout_ms": 1000, "discovery_every_ms": 1000,
                "lease_wait_ms": 10000, "call_timeout_ms": 60000, "poll_every_ms": 25,
                "lease_duration_ms": 600000, "heartbeat_ms": 10000
            } }
        })).unwrap();
        owner.inference.as_mut().unwrap().model = model;
        if cfg!(feature = "inference-stages") {
            assert!(validate(&owner).unwrap_err().contains("personal sync"));
            owner.sync = Some(crate::settings::SyncSettings {
                graph: "personal".into(),
                store_path: None,
                roster_roots: vec![],
                paired_devices: vec![],
                lanes: Default::default(),
                encrypted: false,
                relay_urls: vec![],
            });
            assert!(validate(&owner).is_ok());
            if let InferenceModelSettings::Automatic { stages, .. } =
                &mut owner.inference.as_mut().unwrap().model
            {
                stages.max_peers = 0;
            }
            assert!(validate(&owner).unwrap_err().contains("bounds"));
        } else {
            assert!(validate(&owner).unwrap_err().contains("inference-stages"));
        }
    }
}
