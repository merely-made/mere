// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Resident Knot composition inside Djinn.
//!
//! Djinn owns process topology. Knot still owns document semantics; djinn's
//! custody derives the persona's Knot keys, the same release its custody
//! route answers Knot's own processes with (dramatis DR-C); Murm owns
//! transport, and iroh-blobs owns the physical content store. This module
//! only keeps those authorities alive in one resident and registers the
//! stable local route.
//!
//! ## Closed while the vault is locked (vault lock ruling 48)
//!
//! The route is registered once, over a [`KnotGate`]. While the lane is
//! closed the gate refuses every open, and closing it cuts every live
//! session, so no route or session keeps the source (and its signing seed)
//! alive. Reopening refills the gate: the route serves again.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use chirograph::{
    CarrierNotice, EndpointDescriptor, IntentInvocation, IntentResult, ProjectionRequest,
    ProjectionSnapshot, ResourceChunkRequest, ResourceChunkResponse, ResourceRequest,
    ResourceResponse, ResumeReply, ResumeRequest,
};
use graphshell::lifecycle::AdmittedEndpointContext;
use graphshell::native::endpoint_catalog::{ResidentEndpoint, ResidentEndpointSession};
use graphshell_endpoint::{
    IntentSink, PresentationSource, ProjectionCatalog, ProjectionNoticeSource, ProjectionSource,
};

use knot_editor::{
    KnotContentRetentionPort, KnotResidentSource, KnotRosetteConfig, KnotSettings,
    KnotSpaceAuthoritySnapshot, KnotSyncHost, KnotSyncHostConfig, KnotWriteGrant,
    PersonalVaultKeys, StartupUnlockedPersonalVault, knot_settings_path, persona_vault_root,
};
use transport::BlobScope;

use crate::resident_blobs::{LEGACY_KNOT_LEASE, LegacyBlobMigration, ResidentBlobCustody};
use crate::settings::KnotResidentSettings;
use graphshell::native::endpoint_catalog::{
    ResidentEndpointCatalog, ResidentEndpointCatalogError, ResidentEndpointRoute,
};

/// Stable first-party route for the resident Djot vault.
pub const RESIDENT_KNOT_ROUTE: &str = "knot";

/// Notice cadence for live document revisions on the local route.
pub const RESIDENT_KNOT_NOTICE_POLL: Duration = Duration::from_millis(50);

const CLOSED: &str = "Knot is closed while the vault is locked";

/// What the registered route opens through: the lane's sessions while it is
/// open, refusals while it is closed.
#[derive(Clone, Default)]
pub struct KnotGate(Arc<Mutex<GateState>>);

#[derive(Default)]
struct GateState {
    /// The open lane's own catalog, its factory holding the source.
    lane: Option<ResidentEndpointCatalog>,
    sessions: Vec<Weak<Mutex<Option<ResidentEndpointSession>>>>,
}

impl KnotGate {
    fn lock(&self) -> std::sync::MutexGuard<'_, GateState> {
        self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn arm(&self, source: KnotResidentSource, rosette: KnotRosetteConfig) {
        let mut lane = ResidentEndpointCatalog::new();
        lane.register_resumable_notifying(RESIDENT_KNOT_ROUTE, "Knot", move |_| {
            Ok(source
                .session(Some(KnotWriteGrant::new(rosette.max_source_bytes)))
                .with_rosette_config(rosette))
        })
        .expect("a fresh lane catalog takes its one route");
        self.lock().lane = Some(lane);
    }

    /// Refuse new opens and cut every live session. Returns how many were
    /// cut.
    pub fn close(&self) -> usize {
        let mut state = self.lock();
        state.lane = None;
        let mut cut = 0;
        for session in state.sessions.drain(..) {
            if let Some(session) = session.upgrade() {
                let mut held = session.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                cut += usize::from(held.take().is_some());
            }
        }
        cut
    }

    /// Whether the lane is open.
    pub fn is_open(&self) -> bool {
        self.lock().lane.is_some()
    }

    fn open(&self, context: &AdmittedEndpointContext) -> Result<Box<dyn ResidentEndpoint>, String> {
        let mut state = self.lock();
        let lane = state.lane.as_mut().ok_or_else(|| CLOSED.to_string())?;
        let session = lane
            .open(RESIDENT_KNOT_ROUTE, context)
            .map_err(|error| error.to_string())?;
        let held = Arc::new(Mutex::new(Some(session)));
        state.sessions.retain(|weak| weak.strong_count() > 0);
        state.sessions.push(Arc::downgrade(&held));
        Ok(Box::new(GatedSession(held)))
    }
}

/// One live Knot session, cut when the lane closes.
struct GatedSession(Arc<Mutex<Option<ResidentEndpointSession>>>);

impl GatedSession {
    fn with<R>(&self, f: impl FnOnce(&mut ResidentEndpointSession) -> Result<R, String>) -> Result<R, String> {
        let mut held = self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        f(held.as_mut().ok_or_else(|| CLOSED.to_string())?)
    }
}

impl ResidentEndpoint for GatedSession {
    fn describe(&self) -> EndpointDescriptor {
        self.with(|session| Ok(session.describe()))
            .unwrap_or_else(|_| EndpointDescriptor {
                label: CLOSED.to_string(),
                projections: Vec::new(),
            })
    }

    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, String> {
        self.with(|session| session.snapshot(request))
    }

    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, String> {
        self.with(|session| session.resource(request))
    }

    fn resource_chunk(
        &mut self,
        request: ResourceChunkRequest,
    ) -> Result<ResourceChunkResponse, String> {
        self.with(|session| session.resource_chunk(request))
    }

    fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, String> {
        self.with(|session| session.invoke(intent))
    }

    fn poll_notice(&mut self) -> Result<Option<CarrierNotice>, String> {
        self.with(|session| session.poll_notice())
    }

    fn resume(&mut self, request: ResumeRequest) -> Result<ResumeReply, String> {
        self.with(|session| session.resume(request))
    }
}

/// One startup-unlocked Knot source, optional network host, and settings view.
pub struct ResidentKnot {
    sync: Option<KnotSyncHost>,
    settings_file: PathBuf,
    gate: KnotGate,
}

impl ResidentKnot {
    /// Open the selected persona once and keep its source resident.
    pub async fn open(
        data_root: &Path,
        config: KnotResidentSettings,
        blob_custody: ResidentBlobCustody,
    ) -> Result<Self, String> {
        Self::open_through(data_root, config, blob_custody, KnotGate::default()).await
    }

    /// [`Self::open`], serving the route through `gate` (a reopen keeps the
    /// gate its route was registered over).
    pub async fn open_through(
        data_root: &Path,
        config: KnotResidentSettings,
        blob_custody: ResidentBlobCustody,
        gate: KnotGate,
    ) -> Result<Self, String> {
        let persona_uuid = uuid::Uuid::parse_str(config.persona.trim())
            .map_err(|error| format!("invalid resident Knot persona UUID: {error}"))?;
        let persona = personae::PersonaId::from_uuid(persona_uuid);
        let settings_file = knot_settings_path(data_root, persona);
        let settings = KnotSettings::load(&settings_file)
            .map_err(|error| format!("could not load resident Knot settings: {error}"))?;
        let authority = settings
            .sync
            .as_ref()
            .map(KnotSpaceAuthoritySnapshot::from_personal_settings)
            .transpose()
            .map_err(|error| format!("could not read resident Knot pairing: {error}"))?
            .unwrap_or_default();
        let device_label = settings
            .sync
            .as_ref()
            .map(|sync| sync.device_label.trim())
            .filter(|label| !label.is_empty())
            .unwrap_or_else(|| config.device_label.trim());
        if device_label.is_empty() {
            return Err("resident Knot device label is empty".into());
        }
        let released = crate::custody::epoch_keys(
            &crate::custody::ReleasePolicy::default(),
            data_root,
            persona,
            Some(device_label),
            &PersonalVaultKeys::requests(),
        )
        .map_err(|error| format!("could not derive resident Knot keys: {error}"))?;
        let keys = PersonalVaultKeys::from_released(&released)?;
        let startup =
            StartupUnlockedPersonalVault::open(data_root, persona, keys, authority.writers())?;
        // Held across the awaits below, so cleared on drop (vault lock ruling 49).
        let signing_seed = zeroize::Zeroizing::new(*startup.signing_seed());
        let store = startup.store().clone();
        let source = startup.into_resident_source()?;

        let scope = BlobScope::new(store.space_id());
        let readers = blob_custody.authorizer();
        readers.replace_readers(scope, authority.evidence_readers());
        let max_artifact_bytes = config.max_artifact_bytes;
        let evidence_root = config
            .evidence_root
            .unwrap_or_else(|| persona_vault_root(data_root, persona).join("evidence"));
        match blob_custody
            .migrate_legacy_store(&evidence_root, scope, LEGACY_KNOT_LEASE)
            .await?
        {
            LegacyBlobMigration::Copied { blobs } => tracing::info!(
                source = %evidence_root.display(),
                blobs,
                "imported Knot's old evidence store into resident custody"
            ),
            LegacyBlobMigration::AlreadyComplete { blobs } => tracing::debug!(
                source = %evidence_root.display(),
                blobs,
                "Knot evidence migration remains verified"
            ),
            LegacyBlobMigration::SourceAbsent | LegacyBlobMigration::AlreadyShared => {},
        }
        blob_custody.bind_scope(scope).await?;
        let blobs = blob_custody.blobs();
        let retention = KnotContentRetentionPort::borrow_scoped(
            blobs.clone(),
            max_artifact_bytes,
            readers.clone(),
            scope,
        )?;
        source.grant_content_retention(retention);

        // Restore authority from the resident Djot projection. A reference
        // whose bytes are absent stays unserved and may be fetched later.
        for reference in source.retained_evidence_references()? {
            let hash = reference.blob_hash()?;
            if blobs
                .has(hash)
                .await
                .map_err(|error| format!("could not inspect resident Knot evidence: {error}"))?
            {
                readers.retain(scope, hash);
            }
        }

        let sync = match settings.sync.as_ref() {
            Some(sync) => {
                let host_config = sync_host_config(sync)?;
                Some(
                    KnotSyncHost::open_with_scoped_evidence(
                        &store,
                        &signing_seed,
                        host_config,
                        blobs,
                        readers,
                        max_artifact_bytes,
                    )
                    .await
                    .map_err(|error| format!("could not start resident Knot sync: {error}"))?,
                )
            },
            None => None,
        };

        let mut rosette = KnotRosetteConfig::default();
        rosette.max_source_bytes = config.max_source_bytes;
        gate.arm(source, rosette);
        Ok(Self {
            sync,
            settings_file,
            gate,
        })
    }

    /// Register the stable route, once, over the gate. Every admitted open
    /// receives an independent session over the same resident source and
    /// operation store while the lane is open.
    pub fn register(
        &self,
        catalog: &mut ResidentEndpointCatalog,
    ) -> Result<(), ResidentEndpointCatalogError> {
        let gate = self.gate.clone();
        catalog.register_erased(RESIDENT_KNOT_ROUTE, "Knot", move |context| gate.open(context))
    }

    /// The gate the route is served through.
    pub fn gate(&self) -> KnotGate {
        self.gate.clone()
    }

    /// Local route descriptor granted to an installed first-party client.
    pub fn route() -> ResidentEndpointRoute {
        ResidentEndpointRoute::new(RESIDENT_KNOT_ROUTE, RESIDENT_KNOT_NOTICE_POLL)
            .expect("the resident Knot route is valid")
    }

    /// Reconcile pairing, evidence readers, and newly learned dial hints.
    pub async fn refresh_settings(&mut self) -> Result<bool, String> {
        let Some(host) = self.sync.as_mut() else {
            return Ok(false);
        };
        let latest = KnotSettings::load(&self.settings_file)
            .map_err(|error| format!("could not reload resident Knot settings: {error}"))?;
        let next = latest
            .sync
            .as_ref()
            .map(KnotSpaceAuthoritySnapshot::from_personal_settings)
            .transpose()
            .map_err(|error| format!("could not read resident Knot pairing: {error}"))?
            .unwrap_or_default();
        let changed = host
            .apply_authority(next)
            .await
            .map_err(|error| format!("could not apply resident Knot authority: {error}"))?;
        if latest.sync.is_some() {
            host.refresh_dial_hints(&self.settings_file).await;
        }
        Ok(changed)
    }

    /// Whether this resident currently has a bound p2p sync host.
    pub fn sync_enabled(&self) -> bool {
        self.sync.is_some()
    }

    /// Device-distinct Knot network identity, when sync is enabled.
    pub fn node_id(&self) -> Option<[u8; 32]> {
        self.sync.as_ref().map(KnotSyncHost::node_id)
    }

    /// Persona-stable Knot space identity, when sync is enabled.
    pub fn space_id(&self) -> Option<[u8; 32]> {
        self.sync.as_ref().map(KnotSyncHost::space_id)
    }

    /// Stop network activity before the source-owned evidence actor shuts down.
    /// The route refuses from here on, and live sessions are cut.
    pub async fn close(mut self) -> Result<(), String> {
        self.gate.close();
        if let Some(sync) = self.sync.take() {
            sync.close()
                .await
                .map_err(|error| format!("could not close resident Knot sync: {error}"))?;
        }
        Ok(())
    }
}

fn sync_host_config(sync: &knot_editor::KnotSyncSettings) -> Result<KnotSyncHostConfig, String> {
    let relay_urls =
        transport::P2pandaHostPolicy::parse_relay_urls(sync.relay_urls.iter().map(String::as_str))
            .map_err(|error| format!("resident Knot {error}"))?;
    Ok(KnotSyncHostConfig {
        authority: KnotSpaceAuthoritySnapshot::from_personal_settings(sync)
            .map_err(|error| format!("could not read resident Knot pairing: {error}"))?,
        relay_urls,
    })
}

#[cfg(test)]
mod tests {
    use chirograph::{
        AdvertisedAction, EditableTextV1, IntentInvocation, IntentResult, KnotClipArtifactRoleV1,
        KnotClipArtifactV1, PresentationCodec, ResourceRequest, SaveTextV1,
    };
    use graphshell_endpoint::{
        IntentSink, PresentationSource, ProjectionCatalog, ProjectionSource,
    };
    use knot_editor::{KnotSyncEvent, KnotSyncFileStore, KnotVault, VaultDocument};
    use p2panda_core::SigningKey;
    use sceno::InstanceId;
    use transport::BlobReadAuthorizer;

    use graphshell::lifecycle::AdmittedEndpointContext;

    use super::*;

    fn editable_resource(
        endpoint: &mut graphshell::native::endpoint_catalog::ResidentEndpointSession,
        snapshot: &chirograph::ProjectionSnapshot,
        address_suffix: &str,
    ) -> (InstanceId, EditableTextV1, AdvertisedAction) {
        for (instance, _) in snapshot.scene.active_items_in_order() {
            let offers = snapshot.presentation.offers_for(instance).unwrap();
            let Some(offer) = offers
                .iter()
                .find(|offer| offer.codec == PresentationCodec::EditableTextV1)
            else {
                continue;
            };
            let response = endpoint
                .resource(ResourceRequest {
                    session: snapshot.session.clone(),
                    resource: offer.resource,
                })
                .unwrap();
            let editable: EditableTextV1 = serde_json::from_slice(&response.bytes).unwrap();
            if editable.address.ends_with(address_suffix) {
                return (instance, editable, offer.semantics.actions[0].clone());
            }
        }
        panic!("snapshot did not disclose editable {address_suffix}");
    }

    fn save_invocation(
        snapshot: &chirograph::ProjectionSnapshot,
        target: InstanceId,
        action: &AdvertisedAction,
        payload: &SaveTextV1,
    ) -> IntentInvocation {
        IntentInvocation {
            session: snapshot.session.clone(),
            target,
            observed_epoch: snapshot.scene.epoch,
            observed_revision: snapshot.scene.revision,
            intent: action.intent.0.clone(),
            payload: serde_json::to_vec(payload).unwrap(),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn route_reopens_over_joined_sync_and_live_pairing_updates_all_authority() {
        let root = tempfile::tempdir().unwrap();
        let seed = [0x51; 32];
        let writer = *SigningKey::from_bytes(&seed).verifying_key().as_bytes();
        let peer = *SigningKey::from_bytes(&[0x52; 32])
            .verifying_key()
            .as_bytes();
        let space = [0x91; 32];
        let vault = KnotVault::open(root.path().join("vault"), [0xa1; 32]).unwrap();
        let store =
            KnotSyncFileStore::open(root.path().join("sync.redb"), space, [writer]).unwrap();
        store
            .author(
                &seed,
                &vault,
                &KnotSyncEvent::Put(VaultDocument {
                    id: "field-note".into(),
                    title: "Field note".into(),
                    body: b"# Resident\n".to_vec(),
                    media_type: "text/djot".into(),
                }),
            )
            .await
            .unwrap();
        let source = KnotResidentSource::from_synced_vault(vault, store.clone(), &seed).unwrap();
        let scope = BlobScope::new(space);
        let readers = BlobReadAuthorizer::new();
        let retention = KnotContentRetentionPort::open_scoped(
            root.path().join("evidence"),
            4096,
            readers.clone(),
            scope,
        )
        .unwrap();
        let blobs = retention.blob_store();
        let evidence = KnotClipArtifactV1 {
            role: KnotClipArtifactRoleV1::SourceResponse,
            media_type: "text/plain".into(),
            canonical_uri: "https://example.test/evidence".into(),
            bytes: b"resident evidence".to_vec(),
        };
        let reference = retention.retain_async(&evidence).await.unwrap();
        let hash = reference.blob_hash().unwrap();
        source.grant_content_retention(retention);
        let host = KnotSyncHost::open_with_scoped_evidence(
            &store,
            &seed,
            KnotSyncHostConfig::default(),
            blobs.clone(),
            readers.clone(),
            4096,
        )
        .await
        .unwrap();

        let other_seed = [0x61; 32];
        let other_writer = *SigningKey::from_bytes(&other_seed)
            .verifying_key()
            .as_bytes();
        let other_store = KnotSyncFileStore::open(
            root.path().join("other-sync.redb"),
            [0x92; 32],
            [other_writer],
        )
        .unwrap();
        let other_host =
            KnotSyncHost::open(&other_store, &other_seed, KnotSyncHostConfig::default())
                .await
                .unwrap();
        assert_ne!(host.node_id(), other_host.node_id());
        assert_ne!(host.space_id(), other_host.space_id());
        other_host.close().await.unwrap();

        let settings_file = root.path().join("knot-sync.json");
        let rosette = KnotRosetteConfig {
            max_source_bytes: 4096,
            ..KnotRosetteConfig::default()
        };
        let gate = KnotGate::default();
        gate.arm(source.clone(), rosette);
        let mut resident = ResidentKnot {
            sync: Some(host),
            settings_file: settings_file.clone(),
            gate,
        };
        let empty_authority_revision = resident.sync.as_ref().unwrap().authority_revision();
        assert!(!readers.allows(scope, &peer, hash));
        let mut sync_settings = knot_editor::KnotSyncSettings::default();
        assert!(sync_settings.pair(peer));
        KnotSettings {
            sync: Some(sync_settings.clone()),
        }
        .save(&settings_file)
        .unwrap();
        assert!(resident.refresh_settings().await.unwrap());
        let paired_authority_revision = resident.sync.as_ref().unwrap().authority_revision();
        assert_ne!(paired_authority_revision, empty_authority_revision);
        assert!(store.admitted_writers().contains(&peer));
        assert!(readers.allows(scope, &peer, hash));

        assert!(sync_settings.unpair(peer));
        KnotSettings {
            sync: Some(sync_settings),
        }
        .save(&settings_file)
        .unwrap();
        assert!(resident.refresh_settings().await.unwrap());
        assert_eq!(
            resident.sync.as_ref().unwrap().authority_revision(),
            empty_authority_revision
        );
        assert!(!store.admitted_writers().contains(&peer));
        assert!(!readers.allows(scope, &peer, hash));
        assert!(blobs.has(hash).await.unwrap());

        let mut catalog = ResidentEndpointCatalog::new();
        resident.register(&mut catalog).unwrap();
        let context = AdmittedEndpointContext::new(
            chirograph::ProjectionSession("r3:turnstone".into()),
            [0xc1; 32],
        );

        let mut first = catalog.open(RESIDENT_KNOT_ROUTE, &context).unwrap();
        let request = first.describe().projections.remove(0).request;
        let snapshot = first.snapshot(request).unwrap();
        let (target, editable, action) = editable_resource(&mut first, &snapshot, "field-note");
        assert_eq!(
            first
                .invoke(save_invocation(
                    &snapshot,
                    target,
                    &action,
                    &SaveTextV1 {
                        base_token: editable.base_token,
                        source: "# Edited while sync is resident\n".into(),
                    },
                ))
                .unwrap(),
            IntentResult::Accepted
        );
        drop(first);
        assert!(resident.sync_enabled(), "UI close does not leave LogSync");

        let mut reopened = catalog.open(RESIDENT_KNOT_ROUTE, &context).unwrap();
        let request = reopened.describe().projections.remove(0).request;
        let snapshot = reopened.snapshot(request.clone()).unwrap();
        let (_, edited, _) = editable_resource(&mut reopened, &snapshot, "field-note");
        assert_eq!(edited.source, "# Edited while sync is resident\n");

        // Ruling 48: closing the lane cuts the live session and refuses
        // opens; re-arming serves the same route again.
        let gate = resident.gate();
        assert_eq!(gate.close(), 1, "the live session is cut");
        assert!(!gate.is_open());
        assert!(reopened.snapshot(request.clone()).is_err());
        let refused = catalog.open(RESIDENT_KNOT_ROUTE, &context).map(drop);
        assert!(
            refused
                .unwrap_err()
                .to_string()
                .contains("closed while the vault is locked")
        );
        drop(reopened);
        gate.arm(source.clone(), rosette);
        let mut again = catalog.open(RESIDENT_KNOT_ROUTE, &context).unwrap();
        let request = again.describe().projections.remove(0).request;
        assert!(again.snapshot(request).is_ok());
        drop(again);
        drop(catalog);
        resident.close().await.unwrap();
    }
}
