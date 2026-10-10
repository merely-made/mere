// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Installed-product authority: a persisted Personae choice plus private paths.
//!
//! This module deliberately owns neither scheduler nor device policy. A caller
//! supplies the mesh store, its retention policy, [`crate::mesh_host::HostConfig`],
//! and [`crate::ResidentSettings`] when it binds a resident. Those are mesh
//! and device facts, not preferences Distillery may quietly invent.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::mesh_host::{HostConfig, MeshHost};
use insigne::DerivedKeyAttestation;
use mesh::{MESH_AUTHOR_SALT, MeshStore, SyncedMesh};
use muniment::Backend;
use personae::vault::ProfileId;
use personae::{DerivedKeypair, Ed25519Keypair, IdentityError, IdentityProvider};
use serde::{Deserialize, Serialize};
use transport::{P2pandaTransport, TransportError};

use crate::{ResidentAuthority, ResidentError, ResidentSettings, ResidentStorage};

/// Domain separator for the mesh this installed product joins when nobody
/// named another one.
///
/// The mesh id is **derived, not configured**: a personal mesh belongs to the
/// profile that speaks for this installation, so asking an owner to type a
/// 64-hex identifier would only invite them to mistype one and silently join
/// nothing. Deriving it under a product-owned salt gives the same profile the
/// same mesh on every device it unlocks, and gives a different profile a
/// different one, with no stored value to drift. Every device holding one
/// master shares a mesh; today each vault mints its own master, so in practice
/// a mesh is one device's (vault lock plan §6, 2026-10-09).
///
/// The salt is product-owned rather than borrowed from
/// [`MESH_AUTHOR_SALT`]: reusing the author's salt would make the mesh id equal
/// the author key, so the name of the room and the name of the speaker would be
/// the same string.
pub const DISTILLERY_MESH_SALT: &[u8] = b"distillery-personal-mesh";

const DISTILLERY_DIR: &str = "distillery";
const SETTINGS_FILENAME: &str = "settings.json";
const MESHES_DIR: &str = "meshes";
const MESH_STORE_FILENAME: &str = "mesh.redb";
const BLOB_STORE_DIR: &str = "blobs";

/// Product-owned settings path: `<data_root>/distillery/settings.json`.
pub fn distillery_settings_path(data_root: &Path) -> PathBuf {
    data_root.join(DISTILLERY_DIR).join(SETTINGS_FILENAME)
}

/// The one durable choice Distillery makes before a mesh is opened.
///
/// The profile is an explicit Personae profile id, not an environment seed or
/// an application-local replacement identity. Its secret stays in the shared
/// Personae vault; this file carries only the public-facing profile name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstalledSettings {
    /// scope=application; movement=local-only; mutability=restart-required;
    /// security=ordinary. The Personae profile authorized to speak for this
    /// installed Distillery instance.
    pub profile: String,
}

impl InstalledSettings {
    /// Record an explicitly selected profile.
    pub fn new(profile: ProfileId) -> Result<Self, InstalledSettingsError> {
        let settings = Self { profile: profile.0 };
        settings.validate()?;
        Ok(settings)
    }

    /// Recover the typed Personae profile id.
    pub fn profile_id(&self) -> ProfileId {
        ProfileId(self.profile.clone())
    }

    /// Reject a non-choice instead of treating it as a hidden default profile.
    pub fn validate(&self) -> Result<(), InstalledSettingsError> {
        if self.profile.trim().is_empty() {
            return Err(InstalledSettingsError::InvalidProfile);
        }
        Ok(())
    }

    /// Load settings. An absent file means the product has not been configured.
    pub fn load(data_root: &Path) -> Result<Option<Self>, InstalledSettingsError> {
        let path = distillery_settings_path(data_root);
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(InstalledSettingsError::file(&path, error)),
        };
        let settings: Self = serde_json::from_str(&text)
            .map_err(|error| InstalledSettingsError::file(&path, error))?;
        settings.validate()?;
        Ok(Some(settings))
    }

    /// Save through Pandect's recoverable temporary/backup replacement.
    pub fn save(&self, data_root: &Path) -> Result<(), InstalledSettingsError> {
        self.validate()?;
        let path = distillery_settings_path(data_root);
        let mut json = serde_json::to_string_pretty(self)
            .map_err(|error| InstalledSettingsError::file(&path, error))?;
        json.push('\n');
        pandect::write_bytes_with_backup(&path, json.as_bytes())
            .map_err(|error| InstalledSettingsError::file(&path, error))
    }
}

/// A persisted settings failure.
#[derive(Debug, thiserror::Error)]
pub enum InstalledSettingsError {
    /// A file could not be read, parsed, or replaced.
    #[error("Distillery settings at {path}: {message}")]
    File {
        /// The failing file.
        path: String,
        /// The operating-system or serialization error.
        message: String,
    },
    /// The profile field did not name a profile.
    #[error("Distillery settings name an empty Personae profile")]
    InvalidProfile,
}

impl InstalledSettingsError {
    fn file(path: &Path, error: impl std::fmt::Display) -> Self {
        Self::File {
            path: path.display().to_string(),
            message: error.to_string(),
        }
    }
}

/// Private durable paths for one configured mesh.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DistilleryPaths {
    root: PathBuf,
    mesh_id: [u8; 32],
}

impl DistilleryPaths {
    /// Build paths under the installed product root.
    pub fn for_mesh(data_root: &Path, mesh_id: [u8; 32]) -> Self {
        Self {
            root: data_root
                .join(DISTILLERY_DIR)
                .join(MESHES_DIR)
                .join(hex(mesh_id)),
            mesh_id,
        }
    }

    /// The mesh this path set belongs to.
    pub fn mesh_id(&self) -> [u8; 32] {
        self.mesh_id
    }

    /// Parent directory for this mesh's private state.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Create the product-private directory before the mesh owner opens its
    /// caller-configured store there.
    ///
    /// This is filesystem mechanism only. It does not choose a retention or
    /// admission policy and does not open either store.
    pub fn prepare(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.root)
    }

    /// The redb database path a mesh-policy owner may open.
    pub fn mesh_store_path(&self) -> PathBuf {
        self.root.join(MESH_STORE_FILENAME)
    }

    /// The persistent blob-store root mounted by the resident.
    pub fn blob_store_root(&self) -> PathBuf {
        self.root.join(BLOB_STORE_DIR)
    }
}

/// A configured, unlocked installed identity.
///
/// It is intentionally useful before a resident exists: callers can expose
/// the selected profile and its protection, derive the mesh author, or supply
/// the transport identity while their mesh and device owners assemble their
/// respective facts.
pub struct InstalledAuthority {
    data_root: PathBuf,
    settings: InstalledSettings,
    identity: Arc<dyn IdentityProvider>,
    protection: String,
}

impl InstalledAuthority {
    /// Persist the explicit profile selection for this installation.
    pub fn configure(
        data_root: &Path,
        profile: ProfileId,
    ) -> Result<InstalledSettings, InstalledError> {
        let settings = InstalledSettings::new(profile)?;
        settings.save(data_root)?;
        Ok(settings)
    }

    /// The salts the installed lane derives from: the mesh author (also the
    /// transport identity, ruling 92) and the personal mesh's name. These are
    /// what a resident releases to a lane outside it (dramatis repo plan,
    /// D11); the master never comes with them.
    pub fn release_salts() -> Vec<Vec<u8>> {
        vec![MESH_AUTHOR_SALT.to_vec(), DISTILLERY_MESH_SALT.to_vec()]
    }

    /// Open over the configured profile's identity, which the caller supplies.
    ///
    /// Distillery opens no vault (dramatis repo plan, D5): djinn opens the
    /// profile [`InstalledSettings::profile_id`] names and hands in either the
    /// unlocked vault (its own resident lane) or the keys it released for
    /// [`Self::release_salts`]. `protection` is the custodian's account of
    /// what protects that identity at rest, shown and never guessed. An
    /// unconfigured product is refused rather than bound to a guessed face.
    pub fn open(
        data_root: &Path,
        identity: Arc<dyn IdentityProvider>,
        protection: impl Into<String>,
    ) -> Result<Self, InstalledError> {
        let settings = InstalledSettings::load(data_root)?.ok_or(InstalledError::Unconfigured)?;
        Ok(Self {
            data_root: data_root.to_path_buf(),
            settings,
            identity,
            protection: protection.into(),
        })
    }

    /// The private application root supplied by the installed host.
    pub fn data_root(&self) -> &Path {
        &self.data_root
    }

    /// The persisted settings that selected this identity.
    pub fn settings(&self) -> &InstalledSettings {
        &self.settings
    }

    /// The Personae profile this installed port speaks as.
    pub fn profile(&self) -> ProfileId {
        self.settings.profile_id()
    }

    /// The custodian's account of how the selected profile is protected at
    /// rest.
    pub fn protection(&self) -> &str {
        &self.protection
    }

    /// The transport identity: the mesh author key, derived under the
    /// selected profile (vault lock plan, rulings 2, 44 and 92). The master
    /// never reaches the transport, so a locked vault leaves the running lane
    /// nothing of it. Errors while the vault is locked.
    pub fn transport_identity(&self) -> Result<DerivedKeypair, InstalledError> {
        Ok(self.identity.derived_keypair(MESH_AUTHOR_SALT)?)
    }

    /// The mesh author derived under the selected profile.
    pub fn mesh_author(&self) -> Result<Ed25519Keypair, InstalledError> {
        Ok(self.identity.derive_keypair(MESH_AUTHOR_SALT)?)
    }

    /// The mesh this profile's personal ring uses, derived under
    /// [`DISTILLERY_MESH_SALT`].
    ///
    /// This is the public key bytes of a derived keypair, used only as a
    /// 32-byte name. Its private half is never used to sign anything: the mesh
    /// author ([`mesh_author`](Self::mesh_author)) does the signing, under its
    /// own salt.
    pub fn personal_mesh_id(&self) -> Result<[u8; 32], InstalledError> {
        Ok(self
            .identity
            .derive_keypair(DISTILLERY_MESH_SALT)?
            .public_key()
            .to_bytes())
    }

    /// The evidence peers need to connect this device's mesh author, which is
    /// also its transport identity (ruling 92), to the persona that authorized
    /// it.
    pub fn mesh_author_attestation(&self) -> Result<DerivedKeyAttestation, InstalledError> {
        Ok(self.identity.attest_derived_key(MESH_AUTHOR_SALT)?)
    }

    /// Product-owned locations for one mesh. The caller still supplies its
    /// mesh store and retention policy before anything is opened.
    pub fn paths(&self, mesh_id: [u8; 32]) -> DistilleryPaths {
        DistilleryPaths::for_mesh(&self.data_root, mesh_id)
    }

    /// Bind an installed resident from facts owned by the mesh and device hosts.
    ///
    /// `store` carries mesh retention/admission truth. `host_config` is formed
    /// after receiving this resident's blob space, so the device owner supplies
    /// its own scheduler, resources, conditions, and policy without Distillery
    /// fabricating any of them.
    pub async fn bind_resident<B, F>(
        self,
        mesh_id: [u8; 32],
        store: MeshStore<B>,
        settings: ResidentSettings,
        host_config: F,
    ) -> Result<ResidentAuthority<B>, InstalledError>
    where
        B: Backend + Clone + Send + Sync + 'static,
        F: FnOnce(Arc<crate::mesh_host::TransportBlobSpace>) -> HostConfig,
    {
        let paths = self.paths(mesh_id);
        let storage =
            ResidentStorage::open(paths.blob_store_root(), mesh_id, settings.blob_gc_every).await?;
        let blobs = storage.blobs();
        let transport = Arc::new(
            P2pandaTransport::builder_for(&self.transport_identity()?)
                .gossip()
                .blobs(&blobs)
                .bind()
                .await?,
        );
        let (endpoint, gossip) = transport
            .sync_parts()
            .ok_or(InstalledError::MissingGossip)?;
        let synced = SyncedMesh::join(endpoint, gossip, store, mesh_id).await?;
        let host = MeshHost::new(synced, self.mesh_author()?, host_config(storage.space()));
        Ok(ResidentAuthority::new(host, transport, storage, settings)?)
    }
}

/// Failure while opening or binding an installed Distillery authority.
#[derive(Debug, thiserror::Error)]
pub enum InstalledError {
    /// The installed product has not selected a Personae profile yet.
    #[error("Distillery is not configured: select and persist a Personae profile first")]
    Unconfigured,
    /// Product-owned settings failed to load or save.
    #[error(transparent)]
    Settings(#[from] InstalledSettingsError),
    /// Personae could not unlock or load the selected profile.
    #[error(transparent)]
    Personae(#[from] IdentityError),
    /// The p2p transport could not bind.
    #[error(transparent)]
    Transport(#[from] TransportError),
    /// The mesh store or sync lane rejected the join.
    #[error(transparent)]
    Mesh(#[from] mesh::MeshSyncError),
    /// The resident could not open or compose its durable resources.
    #[error(transparent)]
    Resident(#[from] ResidentError),
    /// A resident binding requires a gossip-enabled transport.
    #[error("Distillery installed authority bound without a gossip transport")]
    MissingGossip,
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use mesh::{
        AvailabilityPolicy, ErasurePolicy, KeepBound, LeasePolicy, MeshRetentionPolicy,
        PolicyRevision,
    };
    use personae::{InMemoryProvider, RetainedKeys};

    use super::*;
    use crate::RetentionSettings;

    const MESH: [u8; 32] = [0xD1; 32];
    const PROTECTION: &str = "released by the test custodian";

    /// What djinn hands an installed lane outside it: the released keys for
    /// [`InstalledAuthority::release_salts`], never the master.
    fn released(seed: u8) -> Arc<dyn IdentityProvider> {
        let persona = InMemoryProvider::from_seed([seed; 32]);
        Arc::new(RetainedKeys::capture(&persona, &InstalledAuthority::release_salts()).unwrap())
    }

    #[test]
    fn settings_bind_only_an_explicit_profile_and_reject_unknown_policy() {
        let directory = tempfile::tempdir().unwrap();
        let settings =
            InstalledAuthority::configure(directory.path(), ProfileId("research".into())).unwrap();

        assert_eq!(settings.profile, "research");
        assert_eq!(
            distillery_settings_path(directory.path()),
            directory.path().join("distillery/settings.json")
        );
        assert_eq!(
            InstalledSettings::load(directory.path()).unwrap(),
            Some(settings)
        );
        InstalledAuthority::configure(directory.path(), ProfileId("work".into())).unwrap();
        assert_eq!(
            InstalledSettings::load(directory.path()).unwrap(),
            Some(InstalledSettings::new(ProfileId("work".into())).unwrap()),
            "reconfiguration replaces the prior selection on every platform"
        );

        let path = distillery_settings_path(directory.path());
        let json = fs::read_to_string(&path).unwrap();
        assert!(json.contains("\"profile\": \"work\""));
        assert!(!json.contains("research"));
        assert!(!json.contains("seed"));
        assert!(!path.with_extension("json.tmp").exists());
        assert!(!path.with_extension("json.previous").exists());
        fs::write(&path, r#"{"profile":"research","tick_every_ms":5}"#).unwrap();
        assert!(InstalledSettings::load(directory.path()).is_err());
    }

    #[test]
    fn configured_profile_reopens_stably_and_names_private_mesh_paths() {
        let directory = tempfile::tempdir().unwrap();
        let profile = ProfileId("research".into());
        InstalledAuthority::configure(directory.path(), profile.clone()).unwrap();

        let first = InstalledAuthority::open(directory.path(), released(0x61), PROTECTION).unwrap();
        let first_author = first.mesh_author().unwrap().public_key().to_bytes();
        let first_transport = first.transport_identity().unwrap().public_key().to_bytes();
        let master = first.identity.master_public_key().to_bytes();
        assert_eq!(first.profile(), profile);
        assert_eq!(first.protection(), PROTECTION);
        assert_ne!(first_author, master, "mesh author is profile-derived");
        assert_eq!(
            first_transport, first_author,
            "the mesh author is the transport identity (ruling 92)"
        );
        let paths = first.paths(MESH);
        assert_eq!(paths.mesh_store_path(), paths.root().join("mesh.redb"));
        assert_eq!(paths.blob_store_root(), paths.root().join("blobs"));
        drop(first);

        let reopened =
            InstalledAuthority::open(directory.path(), released(0x61), PROTECTION).unwrap();
        assert_eq!(
            reopened.mesh_author().unwrap().public_key().to_bytes(),
            first_author,
            "the persisted profile, not an environment seed, owns the mesh author"
        );
    }

    #[test]
    fn the_personal_mesh_is_derived_stably_and_is_not_the_author_or_the_transport() {
        let directory = tempfile::tempdir().unwrap();
        let profile = ProfileId("research".into());
        InstalledAuthority::configure(directory.path(), profile).unwrap();

        let first = InstalledAuthority::open(directory.path(), released(0x62), PROTECTION).unwrap();
        let mesh_id = first.personal_mesh_id().unwrap();
        assert_ne!(
            mesh_id,
            first.mesh_author().unwrap().public_key().to_bytes(),
            "the name of the room must not be the name of the speaker"
        );
        assert_ne!(
            mesh_id,
            first.transport_identity().unwrap().public_key().to_bytes()
        );
        drop(first);

        let reopened =
            InstalledAuthority::open(directory.path(), released(0x62), PROTECTION).unwrap();
        assert_eq!(
            reopened.personal_mesh_id().unwrap(),
            mesh_id,
            "a derived mesh id must not drift across opens: nothing stores it"
        );

        // A different profile is a different personal mesh, which is what makes
        // the derivation safe to do without asking.
        let other = ProfileId("burner".into());
        InstalledAuthority::configure(directory.path(), other).unwrap();
        let burner =
            InstalledAuthority::open(directory.path(), released(0x63), PROTECTION).unwrap();
        assert_ne!(burner.personal_mesh_id().unwrap(), mesh_id);
    }

    #[tokio::test]
    async fn binding_requires_mesh_and_device_facts_from_the_caller() {
        let directory = tempfile::tempdir().unwrap();
        let profile = ProfileId("research".into());
        InstalledAuthority::configure(directory.path(), profile).unwrap();
        let authority =
            InstalledAuthority::open(directory.path(), released(0x64), PROTECTION).unwrap();
        let author = authority.mesh_author().unwrap();
        let policy = MeshRetentionPolicy {
            revision: PolicyRevision([0x41; 32]),
            checkpoint_authority: author.public_key().to_bytes(),
            availability: AvailabilityPolicy {
                promised_floor: KeepBound::Forever,
            },
            erasure: ErasurePolicy {
                privacy_ceiling: KeepBound::UntilCheckpoint,
                terminal_job_payload: mesh::PayloadRule::EraseTerminalAtCheckpoint,
            },
            lease: LeasePolicy { max_skew_ms: 0 },
        };
        let paths = authority.paths(MESH);
        paths.prepare().unwrap();
        let store = MeshStore::at_path_with_retention(paths.mesh_store_path(), policy).unwrap();
        let settings = ResidentSettings {
            tick_every: Duration::from_secs(1),
            maintenance_every: None,
            blob_gc_every: Duration::from_secs(1),
            retention: RetentionSettings::default(),
        };

        let resident = authority
            .bind_resident(MESH, store, settings, |space| HostConfig::supervised(space))
            .await
            .unwrap();
        assert_eq!(
            resident.authority().host().me(),
            author.public_key().to_bytes()
        );
        resident.shutdown().await.unwrap();
    }
}
