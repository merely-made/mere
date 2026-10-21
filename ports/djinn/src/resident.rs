// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Product-neutral ownership assembled by the desktop resident.
//!
//! This module owns process-lifetime resources without importing a product UI.
//! Graphshell supplies admitted local sessions; Knot supplies document and sync
//! semantics; Castellan supplies credential records; Djinn keeps their
//! lifetimes and local storage topology coherent.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use armillary::lifecycle::{CloseAction, CloseFuture, close_all};
use castellan::custody::bootstrap::Unlock;
use castellan::lock::VaultLockHolder;
use castellan::resident::{CastellanResident, CredentialSalts};
use graphshell_endpoint::door::route::ResidentEndpointRoute;
use graphshell_host::endpoint_catalog::{ResidentEndpointCatalog, ResidentEndpointCatalogError};
use personae::{IdentityProvider, ProfileId};
use transport::BlobScope;

use crate::resident_blobs::ResidentBlobCustody;
use crate::resident_distillery::ResidentDistillery;
use crate::resident_inference::{self, ResidentInference};
use crate::resident_knot::{KnotGate, ResidentKnot};
use crate::resident_mere::MereRoutes;
use crate::resident_reservoir::{ReservoirLane, ResidentReservoir};
use crate::resident_site::{PublishedSiteEndpoint, PublishedSiteService};
use crate::settings::{KnotResidentSettings, OwnerSettings, ReservoirLaneSettings};

const CASTELLAN_RECORD_SALT: &[u8] = b"mere.djinn/castellan/records/v1";
const CASTELLAN_FRESHNESS_SALT: &[u8] = b"mere.djinn/castellan/freshness/v1";
const CASTELLAN_SALTS: CredentialSalts = CredentialSalts {
    record: CASTELLAN_RECORD_SALT,
    freshness: CASTELLAN_FRESHNESS_SALT,
};

/// Process-wide services retained by one Djinn run.
pub struct DjinnResident {
    credentials: CastellanResident,
    credential_profile: ProfileId,
    blobs: ResidentBlobCustody,
    site: Arc<tokio::sync::Mutex<PublishedSiteService>>,
    knot: Option<ResidentKnot>,
    /// What reopens the Knot lane after a lock: its settings and the gate
    /// its route was registered over (vault lock ruling 48).
    knot_reopen: Option<(PathBuf, KnotResidentSettings, KnotGate)>,
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

impl DjinnResident {
    /// Open a resident with local-only inference. Hosts with personal sync use
    /// `open_deferred_inference`, then start inference against that live host.
    pub async fn open<P: IdentityProvider + ?Sized>(
        identity: &P,
        data_root: &Path,
        profile: &ProfileId,
        owner: OwnerSettings,
        vault_dir: &Path,
        unlock: Unlock,
    ) -> Result<Self, String> {
        let resident =
            Self::open_deferred_inference(identity, data_root, profile, owner, vault_dir, unlock)
                .await?;
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

    /// Open the credential, content, and optional Knot and Distillery
    /// authorities selected for one already-unlocked Personae profile.
    ///
    /// `vault_dir` and `unlock` are threaded through for the Distillery lane
    /// alone. It opens the shared Personae vault a second time, on the same
    /// directory, because its identity is a *profile-derived* mesh author
    /// rather than the process's `IdentityProvider` — and Personae's storage
    /// opens are not exclusive, so the two coexist. Passing an already-opened
    /// vault instead would mean either handing this constructor a Personae
    /// internal or moving the derivation out of Distillery, and both cost more
    /// than one extra open.
    pub async fn open_deferred_inference<P: IdentityProvider + ?Sized>(
        identity: &P,
        data_root: &Path,
        profile: &ProfileId,
        owner: OwnerSettings,
        vault_dir: &Path,
        unlock: Unlock,
    ) -> Result<Self, String> {
        // Before anything opens, so a refusal leaves nothing to unwind.
        resident_inference::validate(&owner)?;
        let credentials = claim_credentials(identity, data_root, profile)?;
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
        let knot_settings = owner.knot.clone();
        let knot = match owner.knot {
            Some(settings) => match ResidentKnot::open(data_root, settings, blobs.clone()).await {
                Ok(knot) => Some(knot),
                Err(open_error) => {
                    drop(site);
                    let close_error = blobs.shutdown().await.err();
                    return Err(match close_error {
                        Some(close_error) => {
                            format!(
                                "open resident Knot: {open_error}; close blob custody: {close_error}"
                            )
                        },
                        None => open_error,
                    });
                },
            },
            None => None,
        };
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
                match ResidentDistillery::open_mesh(
                    data_root,
                    profile,
                    vault_dir,
                    unlock,
                    lane,
                    automatic_mesh,
                )
                .await
                {
                    Ok(distillery) => Some(distillery),
                    Err(open_error) => {
                        // Unwind in reverse construction order: the Knot lane
                        // was built after blob custody and borrows it, so it
                        // has to let go first.
                        let knot_error = match knot {
                            Some(knot) => knot.close().await.err(),
                            None => None,
                        };
                        drop(site);
                        let close_error = blobs.shutdown().await.err();
                        let mut message = format!("open resident Distillery: {open_error}");
                        if let Some(knot_error) = knot_error {
                            message.push_str(&format!("; close resident Knot: {knot_error}"));
                        }
                        if let Some(close_error) = close_error {
                            message.push_str(&format!("; close blob custody: {close_error}"));
                        }
                        return Err(message);
                    },
                }
            },
            None => None,
        };
        let knot_reopen = match (&knot, knot_settings) {
            (Some(knot), Some(settings)) => Some((data_root.to_path_buf(), settings, knot.gate())),
            _ => None,
        };
        Ok(Self {
            credentials,
            credential_profile: profile.clone(),
            blobs,
            site,
            knot,
            knot_reopen,
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
    pub async fn open_reservoir(&mut self, shared_root: &Path) -> &ReservoirLane {
        self.reservoir = ReservoirLane::open(shared_root, &self.reservoir_settings).await;
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

    /// The single record authority behind every Castellan view or adapter.
    pub fn credentials(&self) -> &CastellanResident {
        &self.credentials
    }

    /// The hook the resident host's lock runs (vault lock ruling 1): it
    /// drops the credential keys and re-derives them on unlock.
    pub fn credential_lock_holder(&self) -> Arc<dyn VaultLockHolder> {
        self.credentials
            .lock_holder_for_profile(CASTELLAN_SALTS, self.credential_profile.clone())
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

    /// Claim existing credential stores under each persisted persona's own keys.
    /// This runs during startup, before the app door opens. Empty roots are not
    /// created for profiles which have never kept credentials.
    pub fn credential_stores<S: castellan::custody::IdentityStorage + 'static>(
        &self,
        host: &castellan::authority::PersonaeHost<S>,
        data_root: &Path,
    ) -> Result<std::collections::BTreeMap<ProfileId, CastellanResident>, String> {
        claim_existing_credential_stores(
            host,
            data_root,
            &self.credential_profile,
            &self.credentials,
        )
    }

    /// Clone the shared physical blob custody for a composed product lane.
    pub fn blobs(&self) -> ResidentBlobCustody {
        self.blobs.clone()
    }

    /// Djinn's ordinary immutable-site service. Callers arrive through its
    /// separately admitted first-party route; this never exposes Knot vault or
    /// evidence authority.
    pub fn published_site(&self) -> Arc<tokio::sync::Mutex<PublishedSiteService>> {
        Arc::clone(&self.site)
    }

    /// Register the owner-only published-site route. It shares no Knot route,
    /// sync pairing, or evidence reader authority.
    pub fn register_published_site_route(
        &self,
        catalog: &mut ResidentEndpointCatalog,
    ) -> Result<ResidentEndpointRoute, ResidentEndpointCatalogError> {
        PublishedSiteEndpoint::register(Arc::clone(&self.site), catalog)
    }

    /// Register the optional Knot source under its stable first-party route.
    pub fn register_knot_route(
        &self,
        catalog: &mut ResidentEndpointCatalog,
    ) -> Result<Option<ResidentEndpointRoute>, ResidentEndpointCatalogError> {
        let Some(knot) = self.knot.as_ref() else {
            return Ok(None);
        };
        knot.register(catalog)?;
        Ok(Some(ResidentKnot::route()))
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

    /// Refresh mutable Knot pairing and evidence-reader authority.
    pub async fn refresh(&mut self) -> Result<bool, String> {
        match self.knot.as_mut() {
            Some(knot) => knot.refresh_settings().await,
            None => Ok(false),
        }
    }

    /// Whether this run has an open resident Knot source.
    pub fn knot_enabled(&self) -> bool {
        self.knot.is_some()
    }

    /// Close the Knot lane for a vault lock (ruling 48): the route refuses,
    /// live sessions are cut, sync stops, and the source and its seed go.
    /// Returns how many sessions were cut, or `None` with no lane open.
    pub async fn close_knot(&mut self) -> Result<Option<usize>, String> {
        let Some(knot) = self.knot.take() else {
            return Ok(None);
        };
        let cut = knot.gate().close();
        knot.close().await?;
        Ok(Some(cut))
    }

    /// Reopen the Knot lane after an unlock, through the gate its route was
    /// registered over, so the same route serves again.
    pub async fn reopen_knot(&mut self) -> Result<bool, String> {
        if self.knot.is_some() {
            return Ok(false);
        }
        let Some((data_root, settings, gate)) = self.knot_reopen.clone() else {
            return Ok(false);
        };
        let knot =
            ResidentKnot::open_through(&data_root, settings, self.blobs.clone(), gate).await?;
        self.knot = Some(knot);
        Ok(true)
    }

    /// Loggable Knot transport facts without exposing a source or signing key.
    pub fn knot_network_facts(&self) -> Option<([u8; 32], [u8; 32])> {
        let knot = self.knot.as_ref()?;
        Some((knot.node_id()?, knot.space_id()?))
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

    /// Stop mesh work, then Knot networking, release credential authority, and
    /// finally flush the shared physical store. Call only after every broker
    /// and endpoint view borrowing this resident has stopped.
    pub async fn shutdown(self) -> Result<(), String> {
        let Self {
            credential_profile: _,
            credentials,
            blobs,
            site,
            knot,
            knot_reopen: _,
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
                "Knot",
                Box::new(move || {
                    Box::pin(async move {
                        match knot {
                            Some(knot) => knot.close().await,
                            None => Ok(()),
                        }
                    }) as CloseFuture<'static>
                }) as CloseAction<'static>,
            ),
            (
                "Castellan",
                Box::new(move || {
                    Box::pin(async move {
                        drop(credentials);
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

fn claim_credentials<P: IdentityProvider + ?Sized>(
    identity: &P,
    data_root: &Path,
    profile: &ProfileId,
) -> Result<CastellanResident, String> {
    let profile_root = credential_profile_root(data_root, profile);
    CastellanResident::claim_derived(
        profile_root.join("records"),
        profile_root.join("freshness"),
        identity,
        CASTELLAN_SALTS,
    )
    .map_err(|error| format!("claim Castellan resident: {error}"))
}

fn claim_existing_credential_stores<S: castellan::custody::IdentityStorage + 'static>(
    host: &castellan::authority::PersonaeHost<S>,
    data_root: &Path,
    primary: &ProfileId,
    credentials: &CastellanResident,
) -> Result<std::collections::BTreeMap<ProfileId, CastellanResident>, String> {
    let mut stores = std::collections::BTreeMap::from([(primary.clone(), credentials.clone())]);
    for profile in host
        .snapshot()
        .map_err(|_| "credential persona roster is unavailable".to_string())?
        .profiles
    {
        let id = ProfileId(profile.id);
        if stores.contains_key(&id)
            || !credential_profile_root(data_root, &id)
                .join("records")
                .is_dir()
        {
            continue;
        }
        let provider = host
            .profile_provider(&id)
            .map_err(|_| "credential store persona is unavailable".to_string())?;
        let store = claim_credentials(&provider, data_root, &id)?;
        host.register_lock_holder(store.lock_holder_for_profile(CASTELLAN_SALTS, id.clone()));
        stores.insert(id, store);
    }
    Ok(stores)
}

fn published_site_profile_root(data_root: &Path, profile: &ProfileId) -> PathBuf {
    data_root
        .join("published-sites")
        .join(blake3::hash(profile.0.as_bytes()).to_hex().as_str())
}

fn published_site_profile_scope(profile: &ProfileId) -> BlobScope {
    BlobScope::new(*blake3::hash(profile.0.as_bytes()).as_bytes())
}

fn credential_profile_root(data_root: &Path, profile: &ProfileId) -> PathBuf {
    let segment: String = profile
        .0
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect();
    data_root
        .join("castellan")
        .join("profiles")
        .join(if segment.is_empty() {
            "default"
        } else {
            &segment
        })
}

#[cfg(test)]
mod tests {
    use personae::{IdentityProvider, InMemoryProvider, ProfileId};

    use super::{
        credential_profile_root, published_site_profile_root, published_site_profile_scope,
    };

    #[test]
    fn credential_roots_stay_profile_scoped_and_below_the_data_root() {
        let root = std::path::Path::new("resident-data");
        assert_eq!(
            credential_profile_root(root, &ProfileId("work/../burner".into())),
            root.join("castellan/profiles/work____burner")
        );
    }

    #[test]
    fn credential_derivation_is_distinct_from_the_personal_sync_identity() {
        let identity = InMemoryProvider::from_seed([0x91; 32]);
        assert_ne!(
            identity
                .derive_keypair(super::CASTELLAN_RECORD_SALT)
                .unwrap()
                .to_seed(),
            identity
                .derive_keypair(super::CASTELLAN_FRESHNESS_SALT)
                .unwrap()
                .to_seed()
        );
    }
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
    #[test]
    fn startup_discovers_original_store_after_selected_persona_changes() {
        use castellan::{
            authority::PersonaeHost,
            custody::{
                IdentityStorage, IdentityVault, Profile, SealedProfileStorage, UnlockMethod,
            },
        };
        use graphshell_endpoint::door::identity::VaultProtectionView;
        use personae::{Ed25519Keypair, PersonaId};
        let root = tempfile::tempdir().unwrap();
        let storage = SealedProfileStorage::open_with_key(root.path().join("vault"), [0x53; 32]);
        storage
            .enroll_passphrase(b"synthetic startup receipt")
            .unwrap();
        let family = Profile::new(
            ProfileId("family".into()),
            "Family",
            Ed25519Keypair::from_seed([0x43; 32]),
        );
        let work = Profile::new(
            ProfileId("work".into()),
            "Work",
            Ed25519Keypair::from_seed([0x44; 32]),
        );
        storage.save_profile(&family).unwrap();
        storage.save_profile(&work).unwrap();
        let host = PersonaeHost::new(
            IdentityVault::with_profile(storage, family),
            None,
            VaultProtectionView::Passphrase,
        );
        let original =
            super::claim_credentials(&host, root.path(), &ProfileId("family".into())).unwrap();
        let namespace = PersonaId::new();
        original
            .bind_profile(namespace, &ProfileId("family".into()))
            .unwrap();
        drop(original);
        host.switch_profile(dramatis::intents::SwitchProfileIntentV1 {
            profile: "work".into(),
        })
        .unwrap();
        let current =
            super::claim_credentials(&host, root.path(), &ProfileId("work".into())).unwrap();
        host.register_lock_holder(
            current.lock_holder_for_profile(super::CASTELLAN_SALTS, ProfileId("work".into())),
        );
        let stores = super::claim_existing_credential_stores(
            &host,
            root.path(),
            &ProfileId("work".into()),
            &current,
        )
        .unwrap();
        assert_eq!(stores.len(), 2);
        host.lock_vault().unwrap();
        host.unlock_vault(UnlockMethod::Passphrase(b"synthetic startup receipt"))
            .unwrap();
        assert_eq!(host.current_profile_id(), ProfileId("work".into()));
        assert_eq!(
            stores[&ProfileId("family".into())]
                .profile_personas(&ProfileId("family".into()))
                .unwrap(),
            vec![namespace]
        );
        assert!(
            stores[&ProfileId("work".into())]
                .profile_personas(&ProfileId("family".into()))
                .unwrap()
                .is_empty()
        );
    }
}
