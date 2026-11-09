// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Product-neutral ownership assembled by the desktop resident.
//!
//! This module owns process-lifetime resources without importing a product UI.
//! Graphshell supplies admitted local sessions; Castellan supplies credential records; Djinn keeps their
//! lifetimes and local storage topology coherent.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use armillary::lifecycle::{CloseAction, CloseFuture, close_all};
#[cfg(feature = "inference")]
use dramatis_resident::custody_host::LockHolder as VaultLockHolder;
use ostiary::route::ResidentEndpointRoute;
use graphshell_host::endpoint_catalog::{ResidentEndpointCatalog, ResidentEndpointCatalogError};
use personae::{IdentityProvider, ProfileId};
use transport::BlobScope;

use crate::resident_blobs::ResidentBlobCustody;
use crate::resident_distillery::ResidentDistillery;
use crate::resident_inference::{self, ResidentInference};
use crate::supervisor::AppSupervisor;
use crate::resident_mere::MereRoutes;
use crate::resident_reservoir::{ReservoirLane, ResidentReservoir};
use crate::resident_site::{PublishedSiteEndpoint, PublishedSiteService};
use crate::settings::{OwnerSettings, ReservoirLaneSettings};

/// Process-wide services retained by one Djinn run.
pub struct DjinnResident {
    /// The profile whose keys seal this run's inference data.
    #[cfg_attr(not(feature = "inference"), allow(dead_code))]
    credential_profile: ProfileId,
    blobs: ResidentBlobCustody,
    site: Arc<tokio::sync::Mutex<PublishedSiteService>>,
    /// The application the owner named to run while the vault is unlocked
    /// (cycle breaks P2 step 3). The resident names no application; the
    /// door's endpoint arrives with [`DjinnResident::start_supervised`].
    supervised: Option<AppSupervisor>,
    distillery: Option<ResidentDistillery>,
    /// The inference lane, which borrows the Distillery lane's model
    /// library and so closes before it.
    inference: Option<ResidentInference>,
    #[cfg(feature = "inference")]
    inference_pending: Option<crate::settings::InferenceLaneSettings>,
    #[cfg(feature = "inference")]
    data_root: PathBuf,
    reservoir_settings: ReservoirLaneSettings,
    /// Off until [`DjinnResident::open_reservoir`] opens it under a shared
    /// root the caller names.
    reservoir: ReservoirLane,
}

/// Where the resident's custody-derived material comes from (custody
/// isolation plan, H2): Distillery's mesh author (exactly
/// [`distillery::InstalledAuthority::release_salts`]).
/// Never a second vault open, and never a wallet read in this process.
#[derive(Clone)]
pub enum MeshAuthor {
    /// Captured from the identity the resident was given.
    Resident,
    /// Custody itself: the independently launched custodian in production.
    Custody(Arc<dyn dramatis_resident::custody_host::ResidentCustody>),
}

impl MeshAuthor {
    fn identity<P: IdentityProvider + ?Sized>(
        &self,
        resident: &P,
        _profile: &ProfileId,
    ) -> Result<(Arc<dyn IdentityProvider>, String), String> {
        let salts = distillery::InstalledAuthority::release_salts();
        let (captured, protection) = match self {
            Self::Resident => (personae::RetainedKeys::capture(resident, &salts), "derived keys captured from the resident's identity"),
            // A consent petition may wait on the owner; the worker is lent.
            Self::Custody(host) => (
                tokio::task::block_in_place(|| personae::RetainedKeys::capture(host.as_ref(), &salts)),
                "derived keys released by the custodian",
            ),
        };
        captured
            .map(|keys| (Arc::new(keys) as Arc<dyn IdentityProvider>, protection.to_string()))
            .map_err(|error| format!("the Distillery mesh author: {error}"))
    }
}

impl DjinnResident {
    /// Open a resident with local-only inference. Hosts with personal sync use
    /// `open_deferred_inference`, then start inference against that live host.
    pub async fn open<P: IdentityProvider + ?Sized>(
        identity: &P,
        data_root: &Path,
        profile: &ProfileId,
        owner: OwnerSettings,
        mesh_author: MeshAuthor,
    ) -> Result<Self, String> {
        let resident =
            Self::open_deferred_inference(identity, data_root, profile, owner, mesh_author).await?;
        #[cfg(feature = "inference")]
        let mut resident = resident;
        #[cfg(feature = "inference")]
        if let Err(error) = resident.start_inference(identity, None).await {
            let cleanup = resident.shutdown().await.err();
            return Err(format!(
                "open resident inference: {error}; cleanup: {cleanup:?}"
            ));
        }
        Ok(resident)
    }

    /// Open the credential, content, and optional supervised-application and Distillery
    /// authorities selected for one already-unlocked Personae profile.
    ///
    /// The Distillery lane's identity is a profile-derived mesh author. It
    /// used to come from a second open of the same vault under a kept copy
    /// of the start's passphrase; it now comes from `mesh_author`: exactly
    /// the works' two release salts, captured from `identity` or released by
    /// the custodian (custody isolation plan, §5.1). No vault is opened here.
    pub async fn open_deferred_inference<P: IdentityProvider + ?Sized>(
        identity: &P,
        data_root: &Path,
        profile: &ProfileId,
        mut owner: OwnerSettings,
        mesh_author: MeshAuthor,
    ) -> Result<Self, String> {
        // Before anything opens, so a refusal leaves nothing to unwind.
        resident_inference::validate(&owner)?;
        let blobs = ResidentBlobCustody::open(data_root, &owner.content).await?;
        let site_root = published_site_profile_root(data_root, profile);
        let site_scope = published_site_profile_scope(profile);
        let site = match PublishedSiteService::open(&site_root, site_scope, blobs.clone()).await {
            Ok(site) => Arc::new(tokio::sync::Mutex::new(site)),
            Err(open_error) => {
                let close_error = blobs.shutdown().await.err();
                return Err(match close_error {
                    Some(close_error) => format!(
                        "open published-site service: {open_error}; close blob custody: {close_error}"
                    ),
                    None => open_error,
                });
            },
        };
        let supervised = owner
            .supervised
            .take()
            .map(|settings| AppSupervisor::new(settings, data_root));
        let automatic_mesh = if matches!(
            owner.inference.as_ref().map(|s| &s.model),
            Some(crate::settings::InferenceModelSettings::Automatic { .. })
        ) {
            Some(resident_inference::compute_space(
                crate::personal_sync::personal_graph_id(
                    &owner
                        .sync
                        .as_ref()
                        .ok_or("automatic inference requires personal sync settings")?
                        .graph,
                ),
            ))
        } else {
            None
        };
        let distillery = match owner.distillery {
            Some(lane) => {
                let opened = match mesh_author.identity(identity, profile) {
                    Ok((author, protection)) => {
                        ResidentDistillery::open_mesh(
                            data_root,
                            profile,
                            author,
                            protection,
                            lane,
                            automatic_mesh,
                        )
                        .await
                    },
                    Err(error) => Err(error),
                };
                match opened {
                    Ok(distillery) => Some(distillery),
                    Err(open_error) => {
                        drop(site);
                        let close_error = blobs.shutdown().await.err();
                        let mut message = format!("open resident Distillery: {open_error}");
                        if let Some(close_error) = close_error {
                            message.push_str(&format!("; close blob custody: {close_error}"));
                        }
                        return Err(message);
                    },
                }
            },
            None => None,
        };
        Ok(Self {
            credential_profile: profile.clone(),
            blobs,
            site,
            supervised,
            distillery,
            inference: None,
            #[cfg(feature = "inference")]
            inference_pending: owner.inference,
            #[cfg(feature = "inference")]
            data_root: data_root.to_path_buf(),
            reservoir_settings: owner.reservoir,
            reservoir: ReservoirLane::Off,
        })
    }

    /// Complete inference composition after personal sync opens, before its
    /// lock holder or route is registered. No actor runs during this handoff.
    #[cfg(feature = "inference")]
    pub async fn start_inference<P: IdentityProvider + ?Sized>(
        &mut self,
        identity: &P,
        personal: Option<Arc<crate::sync::personal_sync_host::PersonalSyncHost>>,
    ) -> Result<(), String> {
        let Some(settings) = self.inference_pending.as_ref() else {
            return Ok(());
        };
        let works = self
            .distillery
            .as_mut()
            .ok_or("inference needs Distillery during composition")?;
        let library = if settings.model.library().is_some() || settings.embedding.is_some() {
            Some(works.inference_model_library(&self.data_root)?)
        } else {
            None
        };
        let journal = if library.is_some() {
            let key = identity
                .derive_keypair(resident_inference::INFERENCE_DATA_SALT)
                .map_err(|error| error.to_string())?
                .to_seed();
            let local = works.inference_journal(key)?;
            Some(match personal.as_ref() {
                Some(host) => resident_inference::personal::open(
                    identity,
                    works.inference_root(),
                    host,
                    local,
                    key,
                )?,
                None => local,
            })
        } else {
            None
        };
        #[cfg(feature = "inference-stages")]
        let stages = if let crate::settings::InferenceModelSettings::Automatic {
            row,
            device,
            stages,
            ..
        } = &settings.model
        {
            Some(
                works
                    .prepare_automatic_inference(
                        &self.data_root,
                        personal
                            .as_ref()
                            .ok_or("automatic inference requires the live keyed personal graph")?,
                        row,
                        *device,
                        stages.clone(),
                        identity
                            .attest_derived_key(mesh::MESH_AUTHOR_SALT)
                            .map_err(|e| e.to_string())?,
                    )
                    .await?,
            )
        } else {
            None
        };
        #[cfg(feature = "inference-embedding")]
        let embedding = if let Some(embedding) = settings.embedding.clone() {
            let library = library
                .clone()
                .ok_or("embedding recall requires the resident model library")?;
            let cache = journal
                .clone()
                .ok_or("embedding recall requires the private resident journal")?;
            Some(
                tokio::task::spawn_blocking(move || {
                    distillery::inference::recall_embedding::open_embedding_recall(
                        embedding, library, cache,
                    )
                    .map(Arc::new)
                })
                .await
                .map_err(|e| format!("embedding recall startup: {e}"))??,
            )
        } else {
            None
        };
        let mut inference = ResidentInference::open(
            settings,
            library,
            works.gpu_access(),
            journal.clone(),
            #[cfg(feature = "inference-stages")]
            stages,
        )?;
        #[cfg(feature = "inference-embedding")]
        if let Some(embedding) = embedding {
            if let Err(error) = inference.lane().configure_embedding_recall(embedding) {
                let closed = inference.close().await.err();
                return Err(format!(
                    "embedding recall composition: {error}; close: {closed:?}"
                ));
            }
        }
        if let (Some(host), Some(journal)) = (&personal, &journal) {
            if let Err(error) = resident_inference::personal::join(host, journal).await {
                let closed = inference.close().await.err();
                return Err(format!(
                    "join private inference log: {error}; close: {closed:?}"
                ));
            }
            inference.retain_personal_sync(host.clone());
        }
        self.inference = Some(inference);
        self.inference_pending = None;
        Ok(())
    }

    /// Open the reservoir lane under `shared_root` as the owner's settings say.
    ///
    /// Kept apart from [`DjinnResident::open`] so the shared root is always
    /// named by the caller: the binary passes the family-shared root, and a
    /// test passes a scratch directory rather than the owner's real one. It
    /// never fails the resident; a reservoir that cannot open leaves the lane
    /// unavailable with its reason.
    pub async fn open_reservoir(&mut self, shared_root: &Path, personas: Arc<dyn dramatis_resident::custody_host::PersonaIndex>) -> &ReservoirLane {
        self.reservoir = ReservoirLane::open(shared_root, &self.reservoir_settings, personas).await;
        &self.reservoir
    }

    /// The reservoir lane's current state.
    pub fn reservoir(&self) -> &ReservoirLane {
        &self.reservoir
    }

    /// Register the reservoir route when the lane is open, serving each mere
    /// it ensures through `routes`.
    pub fn register_reservoir_route(
        &self,
        catalog: &mut ResidentEndpointCatalog,
        routes: Option<MereRoutes>,
    ) -> Result<Option<ResidentEndpointRoute>, ResidentEndpointCatalogError> {
        let Some(reservoir) = self.reservoir.reservoir() else {
            return Ok(None);
        };
        reservoir.register(catalog, routes)?;
        Ok(Some(ResidentReservoir::route()))
    }

    #[cfg(feature = "inference")]
    pub fn inference_lock_holder(&self) -> Option<Arc<dyn VaultLockHolder>> {
        self.inference
            .as_ref()
            .map(|lane| lane.lock_holder(self.credential_profile.clone()))
    }

    /// Private stage workers remain lock holders even without a local ask lane.
    #[cfg(feature = "inference-stages")]
    pub fn inference_stage_lock_holder(&self) -> Option<Arc<dyn VaultLockHolder>> {
        self.distillery
            .as_ref()
            .and_then(|works| works.inference_stage_lock_holder())
    }


    /// Clone the shared physical blob custody for a composed product lane.
    pub fn blobs(&self) -> ResidentBlobCustody {
        self.blobs.clone()
    }

    /// Djinn's ordinary immutable-site service. Callers arrive through its
    /// separately admitted first-party route; this never exposes any supervised application's vault or
    /// evidence authority.
    pub fn published_site(&self) -> Arc<tokio::sync::Mutex<PublishedSiteService>> {
        Arc::clone(&self.site)
    }

    /// Register the owner-only published-site route. It shares no application's route,
    /// sync pairing, or evidence reader authority.
    pub fn register_published_site_route(
        &self,
        catalog: &mut ResidentEndpointCatalog,
    ) -> Result<ResidentEndpointRoute, ResidentEndpointCatalogError> {
        PublishedSiteEndpoint::register(Arc::clone(&self.site), catalog)
    }

    /// Register the inference route when this run composes the lane.
    pub fn register_inference_route(
        &self,
        catalog: &mut ResidentEndpointCatalog,
    ) -> Result<Option<ResidentEndpointRoute>, ResidentEndpointCatalogError> {
        self.inference
            .as_ref()
            .map(|inference| inference.register(catalog))
            .transpose()
    }

    /// The open inference lane, when this run composes it.
    pub fn inference(&self) -> Option<&ResidentInference> {
        self.inference.as_ref()
    }

    /// Borrow a retained corpus from its source owner; never reopen its store.
    /// Desktop and embedded compositions share this adapter before admitting
    /// `ask-recall-v1`. Source acquisition and disclosure remain caller-owned.
    #[cfg(feature = "inference")]
    pub fn configure_inference_recall(
        &self,
        corpus: Arc<dyn distillery::inference::recall::RecallCorpus>,
        config: distillery::inference::recall::RecallConfig,
    ) -> Result<(), String> {
        self.inference
            .as_ref()
            .ok_or("inference is not open")?
            .lane()
            .configure_recall(corpus, config)
    }

    /// Whether the owner named an application to supervise.
    pub fn supervised_enabled(&self) -> bool {
        self.supervised.is_some()
    }

    /// The supervised application's id, when one is named.
    pub fn supervised_app(&self) -> Option<&str> {
        self.supervised.as_ref().map(AppSupervisor::app)
    }

    /// Start the supervised application against the door at `endpoint`, once
    /// the door is up. Returns whether a process was started.
    pub fn start_supervised(&mut self, endpoint: &str) -> Result<bool, String> {
        match self.supervised.as_mut() {
            Some(supervised) => supervised.start(endpoint),
            None => Ok(false),
        }
    }

    /// Start the supervised application again if it exited (after a back-off).
    pub fn ensure_supervised(&mut self, endpoint: &str) -> Result<bool, String> {
        match self.supervised.as_mut() {
            Some(supervised) => supervised.ensure_running(endpoint),
            None => Ok(false),
        }
    }

    /// End the supervised application for a vault lock (ruling 48). The door
    /// has already refused it new keys. Returns whether a process was ended.
    pub async fn stop_supervised(&mut self) -> Result<bool, String> {
        match self.supervised.as_mut() {
            Some(supervised) => supervised.stop().await,
            None => Ok(false),
        }
    }

    /// Whether this run composes the resident Distillery works.
    pub fn distillery_enabled(&self) -> bool {
        self.distillery.is_some()
    }

    /// The composed works, for read-only projection and job posting.
    pub fn distillery(&self) -> Option<&ResidentDistillery> {
        self.distillery.as_ref()
    }

    /// Clone the resident-owned Chronicle source before the works are lifted
    /// into their long-running loop. The caller can register that source with
    /// Graphshell while the mutable works borrow is active; only endpoint
    /// source state crosses the split, never job or authority access.
    pub fn distillery_chronicle_observer(&self) -> Option<distillery::ChronicleObserver> {
        self.distillery
            .as_ref()
            .map(ResidentDistillery::chronicle_observer)
    }

    /// Lift the works out so a caller can drive
    /// [`ResidentDistillery::run_until`] for the length of its run loop.
    ///
    /// This exists because that future needs `&mut` on the lane for the whole
    /// loop while the Knot arm of the same `select!` still needs `&mut` on this
    /// resident. Lifting the lane out splits the borrow that Rust cannot split
    /// on its own; [`restore_distillery`](Self::restore_distillery) hands it
    /// back before shutdown so the ordered close still owns it.
    pub fn take_distillery(&mut self) -> Option<ResidentDistillery> {
        self.distillery.take()
    }

    /// Return a lane taken by [`take_distillery`](Self::take_distillery).
    pub fn restore_distillery(&mut self, distillery: Option<ResidentDistillery>) {
        self.distillery = distillery;
    }

    /// Stop mesh work, then the supervised application, release credential authority, and
    /// finally flush the shared physical store. Call only after every broker
    /// and endpoint view borrowing this resident has stopped.
    pub async fn shutdown(self) -> Result<(), String> {
        let Self {
            credential_profile: _,
            blobs,
            site,
            supervised,
            distillery,
            inference,
            #[cfg(feature = "inference")]
                inference_pending: _,
            #[cfg(feature = "inference")]
                data_root: _,
            reservoir_settings: _,
            reservoir,
        } = self;
        let report = close_all(vec![
            (
                // Before the Distillery lane, whose model library it borrows.
                "inference",
                Box::new(move || {
                    Box::pin(async move {
                        match inference {
                            Some(inference) => inference.close().await,
                            None => Ok(()),
                        }
                    }) as CloseFuture<'static>
                }) as CloseAction<'static>,
            ),
            (
                // First: the works borrow nothing from blob custody or
                // credentials, and stopping work the ring believes is running
                // is the one close that other devices can observe. Everything
                // else here is local teardown.
                "Distillery",
                Box::new(move || {
                    Box::pin(async move {
                        match distillery {
                            Some(distillery) => distillery.close().await,
                            None => Ok(()),
                        }
                    }) as CloseFuture<'static>
                }) as CloseAction<'static>,
            ),
            (
                "published site",
                Box::new(move || {
                    Box::pin(async move {
                        let site = Arc::try_unwrap(site)
                            .map_err(|_| {
                                "published-site route still has active borrowers".to_string()
                            })?
                            .into_inner();
                        site.shutdown();
                        Ok(())
                    }) as CloseFuture<'static>
                }) as CloseAction<'static>,
            ),
            (
                // Local teardown that borrows neither blobs nor credentials:
                // releasing the reservoir frees its lock for the next owner.
                "reservoir",
                Box::new(move || {
                    Box::pin(async move { reservoir.close().await }) as CloseFuture<'static>
                }) as CloseAction<'static>,
            ),
            (
                "supervised application",
                Box::new(move || {
                    Box::pin(async move {
                        match supervised {
                            Some(mut supervised) => supervised.stop().await.map(drop),
                            None => Ok(()),
                        }
                    }) as CloseFuture<'static>
                }) as CloseAction<'static>,
            ),
            (
                "Castellan",
                Box::new(move || {
                    Box::pin(async move {
                        Ok(())
                    }) as CloseFuture<'static>
                }) as CloseAction<'static>,
            ),
            (
                "blob custody",
                Box::new(move || {
                    Box::pin(async move { blobs.shutdown().await }) as CloseFuture<'static>
                }) as CloseAction<'static>,
            ),
        ])
        .await;
        if report.is_clean() {
            Ok(())
        } else {
            let failures = report
                .into_failures()
                .into_iter()
                .map(|failure| format!("{}: {}", failure.resource, failure.error))
                .collect::<Vec<_>>()
                .join("; ");
            Err(format!("Djinn resident shutdown failed ({failures})"))
        }
    }
}

fn published_site_profile_root(data_root: &Path, profile: &ProfileId) -> PathBuf {
    data_root
        .join("published-sites")
        .join(blake3::hash(profile.0.as_bytes()).to_hex().as_str())
}

fn published_site_profile_scope(profile: &ProfileId) -> BlobScope {
    BlobScope::new(*blake3::hash(profile.0.as_bytes()).as_bytes())
}

#[cfg(test)]
mod tests {
    use personae::ProfileId;

    use super::{published_site_profile_root, published_site_profile_scope};

    #[test]
    fn published_sites_keep_profile_paths_and_custody_scopes_distinct() {
        let root = std::path::Path::new("resident-data");
        let work = ProfileId("work/../burner".into());
        let personal = ProfileId("personal".into());
        assert_ne!(
            published_site_profile_root(root, &work),
            published_site_profile_root(root, &personal)
        );
        assert_ne!(
            published_site_profile_scope(&work),
            published_site_profile_scope(&personal)
        );
        assert!(published_site_profile_root(root, &work).starts_with(root));
    }
}

