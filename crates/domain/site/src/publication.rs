// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Public native sites selected through existing Moot collection authority.
//!
//! Hosts retain snapshots with Eidetic's `save_schema` and `save_typed`, then
//! explicitly contribute and select them with Gemot. This adapter neither
//! creates a community nor infers a latest revision. Anonymous public readers
//! need no membership. It opens only locally retained PublicPortable bytes.
//! An opened value is a point-in-time read, not an enduring serving grant:
//! hosts must revalidate selection when serving policy requires current authority.
//!
//! Hosts can retain content using the existing Eidetic owners:
//! ```no_run
//! # use eidetic::{Store, ManifestId, PrivacyClass, ProvenanceRecord, Timestamp, TrustEnvelope};
//! # use mere_site::{PublishedSnapshotV1, publication::snapshot_schema};
//! # async fn retain(store: &mut dyn Store, snapshot: &PublishedSnapshotV1) -> eidetic::Result<ManifestId> {
//! eidetic::schema_def::bootstrap_meta_schema(store).await?;
//! eidetic::schema_def::save_schema(
//!     store, &snapshot_schema(), PrivacyClass::PublicPortable,
//!     ProvenanceRecord::self_imported("mere-site schema"),
//!     TrustEnvelope::self_asserted(), Timestamp(1),
//! ).await?;
//! let id = eidetic::typed::save_typed(
//!     store, snapshot, vec![], PrivacyClass::PublicPortable,
//!     ProvenanceRecord::self_imported("explicit saved-site export"),
//!     TrustEnvelope::self_asserted(), Timestamp(2),
//! ).await?;
//! # Ok(id)
//! # }
//! ```

use eidetic::manifest::{load_manifest, resolve_blob};
use eidetic::schema_def::SchemaDefinition;
use eidetic::typed::TypedPayload;
use eidetic::{BlobSource, ManifestId, NoFetcher, PrivacyClass, SchemaRef, Store};
use gemot::moot::{CollectionVersion, ContributionRef, Moot, MootCommandReceipt, MootError};
use muniment::Backend;
use personae::IdentityProvider;
use serde::{Deserialize, Serialize};

use crate::{MAX_ENCODED_SNAPSHOT_BYTES, PublishedPageV1, PublishedSnapshotV1, SiteFormat};

pub const SNAPSHOT_SCHEMA_ID: &str = "mere.site-snapshot.v1";
const SNAPSHOT_SCHEMA_BYTES: &[u8] = include_bytes!("snapshot-schema.json");

/// Retain this schema with Eidetic's `save_schema` alongside the typed snapshot.
/// Rust validation additionally enforces native filenames, totals and metadata.
/// Its body keys are sorted in the bundled definition so enabling serde_json's
/// `preserve_order` cannot change the schema's content identity.
pub fn snapshot_schema() -> SchemaDefinition {
    serde_json::from_slice(SNAPSHOT_SCHEMA_BYTES).expect("the bundled snapshot schema is valid")
}

impl TypedPayload for PublishedSnapshotV1 {
    fn schema_ref() -> SchemaRef {
        SchemaRef::from_id(ManifestId::of_blob(SNAPSHOT_SCHEMA_BYTES.trim_ascii()))
    }

    fn serialize_to_bytes(&self) -> eidetic::Result<Vec<u8>> {
        self.encode().map_err(eidetic::Error::new)
    }

    fn deserialize_from_bytes(bytes: &[u8]) -> eidetic::Result<Self> {
        Self::decode(bytes).map_err(eidetic::Error::new)
    }
}

/// An explicit contribution at an exact observed collection version. This is a
/// reference to Gemot's signed history, not a new signed publication envelope.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiteSelection {
    pub collection: CollectionVersion,
    pub contribution: ContributionRef,
}

#[derive(Debug)]
pub enum PublicationError {
    Content(eidetic::Error),
    Community(Box<MootError>),
    Refused(String),
}

impl std::fmt::Display for PublicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Content(error) => error.fmt(f),
            Self::Community(error) => error.fmt(f),
            Self::Refused(reason) => f.write_str(reason),
        }
    }
}
impl std::error::Error for PublicationError {}
impl From<eidetic::Error> for PublicationError {
    fn from(error: eidetic::Error) -> Self {
        Self::Content(error)
    }
}
impl From<MootError> for PublicationError {
    fn from(error: MootError) -> Self {
        Self::Community(Box::new(error))
    }
}

fn refuse(reason: &str) -> PublicationError {
    PublicationError::Refused(reason.into())
}

async fn local_snapshot(
    store: &mut dyn Store,
    id: ManifestId,
) -> Result<PublishedSnapshotV1, PublicationError> {
    let mut manifest = load_manifest(store, id)
        .await?
        .ok_or_else(|| refuse("The retained site manifest is missing"))?;
    if manifest.manifest_version != eidetic::BlobManifest::CURRENT_VERSION
        || manifest.id != id
        || manifest.content_hash != id.0
        || manifest.schema != PublishedSnapshotV1::schema_ref()
        || manifest.privacy != PrivacyClass::PublicPortable
        || manifest.byte_size > MAX_ENCODED_SNAPSHOT_BYTES as u64
    {
        return Err(refuse(
            "Expected a bounded public site snapshot V1 manifest",
        ));
    }
    // Local sources perform neither acquisition nor cache writes. Remote sources
    // stay in the retained manifest for another owner's explicit delivery work.
    manifest
        .sources
        .retain(|source| matches!(source, BlobSource::Local { .. }));
    let bytes = resolve_blob(store, &mut NoFetcher, &manifest).await?;
    if bytes.len() as u64 != manifest.byte_size {
        return Err(refuse(
            "Site snapshot byte size does not match its manifest",
        ));
    }
    let snapshot = PublishedSnapshotV1::decode(&bytes).map_err(PublicationError::Refused)?;
    if !matches!(
        snapshot.format,
        SiteFormat::Gemini | SiteFormat::Scroll | SiteFormat::Micron
    ) {
        return Err(refuse(
            "This publication adapter supports Gemini, Scroll and Micron",
        ));
    }
    Ok(snapshot)
}

/// Contribute already retained, explicitly public content through the current
/// membership/capability gate. It does not select a collection or write content.
/// Use Gemot's collection command with observed heads to select the returned
/// signed operation. Failed authorization leaves no new contribution.
pub async fn contribute<B: Backend + Clone, P: IdentityProvider + ?Sized>(
    moot: &Moot<B>,
    identity: &P,
    store: &mut dyn Store,
    snapshot: ManifestId,
    title: String,
    at_ms: u64,
) -> Result<MootCommandReceipt, PublicationError> {
    local_snapshot(store, snapshot).await?;
    Ok(moot
        .share_authorized_for_identity(
            identity,
            *snapshot.0.as_bytes(),
            SNAPSHOT_SCHEMA_ID.into(),
            title,
            at_ms,
        )
        .await?)
}

/// A native site read qualified at the returned collection version. Exact
/// metadata remains available to Scroll/Micron adapters instead of being
/// flattened to a Gemini body or a common MIME response.
#[derive(Clone, Debug)]
pub struct OpenedSite {
    pub selection: SiteSelection,
    pub manifest: ManifestId,
    snapshot: PublishedSnapshotV1,
}

impl OpenedSite {
    pub fn snapshot(&self) -> &PublishedSnapshotV1 {
        &self.snapshot
    }

    /// Exact native path lookup; `/` selects the native index. URL decoding,
    /// query policy and protocol status/header construction belong to the host.
    pub fn page(&self, path: &str) -> Option<&PublishedPageV1> {
        if path == "/" {
            self.snapshot
                .pages
                .get(&format!("/{}", self.snapshot.format.index_file()))
        } else {
            self.snapshot.pages.get(path)
        }
    }
}

async fn selected_manifest<B: Backend + Clone>(
    moot: &Moot<B>,
    selection: &SiteSelection,
    at_ms: u64,
) -> Result<ManifestId, PublicationError> {
    if selection.collection.collection.moot_id != moot.moot_id().0
        || selection.contribution.moot_id != moot.moot_id().0
    {
        return Err(refuse("The site selection belongs to another Moot"));
    }
    let view = moot
        .authorized_collection(selection.collection.collection.collection_id, at_ms)
        .await?
        .ok_or_else(|| refuse("The site collection has no current authorized view"))?;
    if view.version != selection.collection
        || !view.effective_selected.contains(&selection.contribution)
    {
        return Err(refuse("The site selection is stale or ineffective"));
    }
    let fauna = moot.authorized_fauna(at_ms).await?;
    let share = fauna
        .iter()
        .find(|share| share.op_hash == selection.contribution.share)
        .ok_or_else(|| refuse("The site contribution is no longer effective"))?;
    if share.schema_id != SNAPSHOT_SCHEMA_ID {
        return Err(refuse("The selected contribution is not a site snapshot"));
    }
    let hash = eidetic::Hash::parse(&blake3::Hash::from_bytes(share.manifest_id).to_hex())
        .expect("a 32-byte BLAKE3 digest is valid");
    Ok(ManifestId::from_hash(hash))
}

/// Open an explicitly selected site without admitting a reader into the Moot.
/// Selection is checked before and after local content resolution; a later
/// authority change requires a fresh read. Transport and hosting remain owned
/// by the caller and their respective policy/actor layers.
pub async fn open<B: Backend + Clone>(
    moot: &Moot<B>,
    store: &mut dyn Store,
    selection: &SiteSelection,
    at_ms: u64,
) -> Result<OpenedSite, PublicationError> {
    let manifest = selected_manifest(moot, selection, at_ms).await?;
    let snapshot = local_snapshot(store, manifest).await?;
    if selected_manifest(moot, selection, at_ms).await? != manifest {
        return Err(refuse("The site contribution changed during the read"));
    }
    Ok(OpenedSite {
        selection: selection.clone(),
        manifest,
        snapshot,
    })
}
